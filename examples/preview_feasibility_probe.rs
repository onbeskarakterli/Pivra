//! Standalone capability experiment, not a production preview provider.
use gtk::{gio, glib, prelude::*};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};

fn target(w: u32, h: u32) -> (u32, u32) {
    let edge = w.max(h).max(1);
    if edge <= 16 {
        return (w.max(1), h.max(1));
    }
    (
        (u64::from(w) * 16 / u64::from(edge)).max(1) as u32,
        (u64::from(h) * 16 / u64::from(edge)).max(1) as u32,
    )
}
fn loader(path: &Path, token: &gio::Cancellable) -> glycin::Loader {
    let mut loader = glycin::Loader::new(gio::File::for_path(path));
    loader.sandbox_selector(glycin::SandboxSelector::Bwrap);
    loader.cancellable(token.clone());
    loader.apply_transformations(true);
    loader.use_expose_base_dir(false);
    loader
}
fn report_frame(stage: &str, frame: &glycin::Frame, start: Instant) {
    let ready = start.elapsed().as_micros();
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!(frame.width() <= 32768 && frame.height() <= 32768);
    assert!(u64::from(frame.width()) * u64::from(frame.height()) <= 32_000_000);
    assert!(frame.buf_slice().len() <= 512 * 1024 * 1024);
    let texture = frame.texture();
    let stride = frame.width() as usize * 4;
    let mut pixels = vec![0; stride * frame.height() as usize];
    texture.download(&mut pixels, stride);
    assert!(pixels.chunks_exact(4).any(|p| p[3] > 0), "empty texture");
    println!(
        "{stage}: frame={}x{} frame_ready_us={ready} texture_download_ready_us={} buffer_bytes={} downloaded_bytes={}; NOT screen presentation or RSS/VRAM",
        frame.width(),
        frame.height(),
        start.elapsed().as_micros(),
        frame.buf_slice().len(),
        pixels.len()
    );
}
async fn experiment(path: &Path) -> Result<(), String> {
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > 64 * 1024 * 1024 {
        return Err("probe admits regular files <=64MiB only".into());
    }
    println!("INPUT {path:?}, client=glycin3.0.8, mandatory=Bwrap, transformations=true");
    let token = gio::Cancellable::new();
    let start = Instant::now();
    let image = loader(path, &token)
        .load()
        .await
        .map_err(|e| e.to_string())?;
    let size = (image.details().width(), image.details().height());
    if size.0 == 0
        || size.1 == 0
        || size.0 > 32768
        || size.1 > 32768
        || u64::from(size.0) * u64::from(size.1) > 32_000_000
    {
        return Err("source exceeds probe 32MP/32768 limits".into());
    }
    let requested = target(size.0, size.1);
    println!(
        "HEADER mime={} source={size:?} orientation={:?} header_us={} request={requested:?}",
        image.mime_type(),
        image.transformation_orientation(),
        start.elapsed().as_micros()
    );
    let first = image
        .specific_frame(glycin::FrameRequest::new().scale(requested.0, requested.1))
        .await;
    match first {
        Ok(frame) => {
            report_frame("SCALE_REQUEST", &frame, start);
            let reduced = u64::from(frame.width()) * u64::from(frame.height())
                < u64::from(size.0) * u64::from(size.1);
            println!(
                "REDUCED_FRAME={reduced}; early-preview=NOT PROVEN (requires decoder cost evidence, full timing and actual presentation)"
            );
            drop(frame); // Do not retain two frame buffers across requests.
        }
        Err(error) => println!("SCALE_REQUEST FAIL: {error}"),
    }
    let second_start = Instant::now();
    match image.next_frame().await {
        Ok(frame) => report_frame("SAME_IMAGE_NEXT_FRAME", &frame, second_start),
        Err(error) => println!("SAME_IMAGE_NEXT_FRAME UNSUPPORTED/ERROR: {error}"),
    }
    drop(image);
    token.cancel();
    // Separate experimental baseline only; never added to production or looped.
    let token = gio::Cancellable::new();
    let baseline = Instant::now();
    let image = loader(path, &token)
        .load()
        .await
        .map_err(|e| e.to_string())?;
    let frame = image.next_frame().await.map_err(|e| e.to_string())?;
    assert_eq!(
        u64::from(frame.width()) * u64::from(frame.height()),
        u64::from(size.0) * u64::from(size.1),
        "fresh full frame area differs from source metadata"
    );
    report_frame("FRESH_FULL_BASELINE", &frame, baseline);
    drop(frame);
    drop(image);
    token.cancel();
    println!(
        "OBSERVATION COMPLETE; no automatic early-preview PASS, no speedup conclusion; pool idle processes may remain"
    );
    Ok(())
}
fn main() {
    if let Err(error) = gtk::init() {
        eprintln!("NOT RUN GTK: {error}");
        std::process::exit(77);
    }
    let mut paths: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if paths.is_empty() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images");
        paths = vec![root.join("large.png"), root.join("no-exif.jpg")];
    }
    assert!(
        paths.len() <= 4,
        "at most four explicit inputs; no benchmark loop"
    );
    for path in paths {
        if let Err(error) = glib::MainContext::default().block_on(experiment(&path)) {
            let blocked = error.contains("Loader process exited early")
                || error.contains("bwrap")
                || error.contains("Unknown image format");
            eprintln!(
                "{} input={path:?}: {error}",
                if blocked {
                    "NOT RUN/UNSUPPORTED"
                } else {
                    "FAIL"
                }
            );
            std::process::exit(if blocked { 77 } else { 1 });
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_aspect_target_uses_wide_arithmetic() {
        assert_eq!(target(1600, 1200), (16, 12));
        assert_eq!(target(32, 64), (8, 16));
        assert_eq!(target(1, u32::MAX), (1, 16));
        assert_eq!(target(4, 2), (4, 2));
        assert_eq!(target(u32::MAX, u32::MAX), (16, 16));
    }
}
