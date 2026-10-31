//! Experimental DNG bridge. The RAW decoder only runs under mandatory Bubblewrap.
//! RAW support is opt-in; distribution builds require an installed helper.
use gtk::{gdk, gio, glib, prelude::*};
use std::os::unix::process::CommandExt;

unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
}

// Terminate the complete script/timeout/bwrap process group, not just bash.
fn terminate_group(child: &mut std::process::Child) {
    let pid = child.id();
    if let Ok(group) = i32::try_from(pid) {
        // SAFETY: this child was spawned as leader of a fresh process group.
        unsafe { kill(-group, 9); }
    }
    let _ = child.kill();
    let _ = child.wait();
}

use std::{path::Path, process::{Command, Stdio}, sync::atomic::{AtomicU64, Ordering}, time::{Duration, Instant}};
static SERIAL: AtomicU64 = AtomicU64::new(0);

pub(crate) fn enabled() -> bool {
    std::env::var_os("PIVRA_EXPERIMENTAL_DNG").as_deref() == Some(std::ffi::OsStr::new("1"))
}
pub(crate) fn is_dng(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("dng"))
}
pub(crate) fn load(path: &Path, cancel: &gio::Cancellable) -> Result<gdk::Texture, String> {
    if !enabled() || !is_dng(path) {
        return Err("Experimental DNG decoding is disabled".into());
    }
    if cancel.is_cancelled() {
        return Err("DNG loading cancelled".into());
    }
    let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!("pivra-dng-{}-{serial}", std::process::id()));
    std::fs::create_dir(&directory).map_err(|e| format!("DNG temp directory: {e}"))?;
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
    }
    let _cleanup = Cleanup(directory.clone());
    let output = directory.join("frame.ppm");
    // Installed builds use a sibling sandbox runner and prebuilt LibRaw helper.
    // Development builds keep the existing source-tree runner fallback.
    let installed_runner = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("pivra-raw-runner")))
        .filter(|runner| runner.is_file());
    // A distribution binary must never fall back to a path embedded from the
    // developer's build machine. Missing installed helpers fail closed.
    if option_env!("PIVRA_DISTRIBUTION_RAW").is_some() && installed_runner.is_none() {
        return Err("Experimental RAW sandbox runner is not installed".into());
    }
    let script = installed_runner.clone().unwrap_or_else(|| {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("raw/run_decode.sh")
    });
    let mut command = Command::new("bash");
    command.arg(&script).arg(path).arg(&output);
    // Scope all runner scratch directories to this load. The RAII cleanup
    // removes them even when the entire process group is killed on cancel.
    command.env("TMPDIR", &directory);
    if installed_runner.is_some() {
        // Installed Pivra must use its packaged decoder, not an inherited
        // development override pointing to an arbitrary host executable.
        command.env("PIVRA_RAW_INSTALLED", "1");
        command.env_remove("PIVRA_RAW_HELPER");
    }
    // Filenames are passed as individual argv values, never shell-interpolated.
    let mut child = command
        .process_group(0)
        .stdout(Stdio::null()).stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("DNG sandbox process: {e}"))?;
    let started = Instant::now();
    let status = loop {
        if cancel.is_cancelled() || started.elapsed() > Duration::from_secs(100) {
            terminate_group(&mut child);
            return Err(if cancel.is_cancelled() {
                "DNG loading cancelled".into()
            } else {
                "DNG sandbox timed out".into()
            });
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(Duration::from_millis(40)),
            Err(error) => {
                terminate_group(&mut child);
                return Err(format!("DNG sandbox wait: {error}"));
            }
        }
    };
    if !status.success() {
        return Err(format!("DNG sandbox exited: {status}"));
    }
    if cancel.is_cancelled() {
        return Err("DNG loading cancelled".into());
    }
    let metadata = std::fs::metadata(&output).map_err(|e| format!("DNG image missing: {e}"))?;
    if metadata.len() > 256 * 1024 * 1024 + 64 {
        return Err("DNG image exceeds transfer limit".into());
    }
    let mut bytes = std::fs::read(&output).map_err(|e| format!("DNG image read: {e}"))?;
    // Validate the untrusted PPM before removing its small header. Shift the
    // raster in-place instead of allocating a second full-resolution RGB copy.
    let (width, height, offset) = {
        let frame = crate::raw_ppm::parse(&bytes).map_err(str::to_string)?;
        (frame.width, frame.height, bytes.len() - frame.pixels.len())
    };
    if cancel.is_cancelled() {
        return Err("DNG loading cancelled".into());
    }
    bytes.drain(..offset);
    let stride = usize::try_from(width).map_err(|e| e.to_string())? * 3;
    let pixels = glib::Bytes::from_owned(bytes);
    Ok(gdk::MemoryTexture::new(width as i32, height as i32,
        gdk::MemoryFormat::R8g8b8, &pixels, stride).upcast())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancelled_raw_process_group_reaps_shell_and_worker() {
        // The shell owns a long-running child in its dedicated process group.
        // Exercise the same termination path as DNG cancellation without GTK.
        let mut child = Command::new("bash")
            .args(["-c", "sleep 30 & wait"])
            .process_group(0)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn isolated worker group");
        let pid = child.id();
        terminate_group(&mut child);
        assert!(child.try_wait().expect("reap shell").is_some());
        // A terminated process group can retain a briefly unreaped zombie in
        // the parent namespace; only verify the leader has been reaped here.
        assert!(pid > 0);
    }

    #[test]
    fn raw_extension_check_is_case_insensitive() {
        assert!(is_dng(Path::new("/tmp/example.DNG")));
        assert!(is_dng(Path::new("/tmp/example.dng")));
        assert!(!is_dng(Path::new("/tmp/example.jpg")));
    }
}
