//! Experimental DNG bridge. The RAW decoder only runs under mandatory Bubblewrap.
//! This development-tree helper is deliberately opt-in; no packaged RAW support.
use gtk::{gdk, gio, glib, prelude::*};
use std::{path::Path, process::Command, sync::atomic::{AtomicU64, Ordering}};
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
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/run_raw_sandbox_probe.sh");
    // Never use a shell to interpolate the source filename. This experiment
    // invokes a known repository script that enforces Bwrap isolation.
    let status = Command::new("bash").arg(script).arg(path).arg(&output)
        .status().map_err(|e| format!("DNG sandbox process: {e}"))?;
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
    let bytes = std::fs::read(&output).map_err(|e| format!("DNG image read: {e}"))?;
    let frame = crate::raw_ppm::parse(&bytes).map_err(str::to_string)?;
    let stride = usize::try_from(frame.width).map_err(|e| e.to_string())? * 3;
    let pixels = glib::Bytes::from_owned(frame.pixels.to_vec());
    Ok(gdk::MemoryTexture::new(frame.width as i32, frame.height as i32,
        gdk::MemoryFormat::R8g8b8, &pixels, stride).upcast())
}
