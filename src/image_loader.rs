//! Glycin boundary: no GTK widgets, no fallback decoder and no unsandboxed mode.

use gtk::gio::{self, prelude::*};
use std::path::Path;

const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PIXELS: u64 = 32_000_000;
const MAX_FRAME_BYTES: usize = 512 * 1024 * 1024;
const MAX_EDGE: u32 = 32_768;

pub(crate) fn validate_dimensions(width: u32, height: u32) -> Result<(), String> {
    if width == 0
        || height == 0
        || width > MAX_EDGE
        || height > MAX_EDGE
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err(
            "Image exceeds the 32 MP / 32768-pixel edge limit or has invalid dimensions".into(),
        );
    }
    Ok(())
}

/// All load/transform work is awaited on a worker-owned GLib context by the UI.
/// Glycin alone applies EXIF orientation; callers must not rotate again.
pub(crate) async fn load(
    path: &Path,
    cancellable: &gio::Cancellable,
) -> Result<glycin::Frame, String> {
    if cancellable.is_cancelled() {
        return Err("Image loading cancelled".into());
    }
    let metadata = std::fs::metadata(path).map_err(|e| format!("Cannot access image: {e}"))?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err("Image must be a regular file of at most 64 MiB".into());
    }
    let mut loader = glycin::Loader::new(gio::File::for_path(path));
    // Auto may disable isolation in some development environments. Require bwrap.
    loader.sandbox_selector(glycin::SandboxSelector::Bwrap);
    loader.cancellable(cancellable.clone());
    loader.apply_transformations(true);
    loader.use_expose_base_dir(false);
    let image = loader
        .load()
        .await
        .map_err(|e| format!("Secure image loader failed: {e}"))?;
    if !matches!(
        image.mime_type().to_string().as_str(),
        "image/jpeg" | "image/png"
    ) {
        return Err("Decoded content is not a JPEG or PNG image".into());
    }
    validate_dimensions(image.details().width(), image.details().height())?;
    let frame = image
        .next_frame()
        .await
        .map_err(|e| format!("Image decode failed: {e}"))?;
    validate_dimensions(frame.width(), frame.height())?;
    if frame.buf_slice().len() > MAX_FRAME_BYTES {
        return Err("Decoded frame exceeds the 512 MiB buffer limit".into());
    }
    if cancellable.is_cancelled() {
        return Err("Image loading cancelled".into());
    }
    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_limits_reject_zero_extremes_and_excessive_pixel_count() {
        for (width, height) in [
            (0, 1),
            (1, 0),
            (u32::MAX, u32::MAX),
            (32769, 1),
            (8000, 8000),
        ] {
            assert!(validate_dimensions(width, height).is_err());
        }
        for (width, height) in [(1, 1), (4000, 6000), (8000, 4000), (32768, 1)] {
            assert!(validate_dimensions(width, height).is_ok());
        }
    }

    #[test]
    fn missing_oversized_or_cancelled_inputs_do_not_start_a_decoder() {
        let directory = tempfile::tempdir().unwrap();
        let context = gtk::glib::MainContext::new();
        let cancellable = gio::Cancellable::new();
        assert!(
            context
                .block_on(load(&directory.path().join("missing.png"), &cancellable))
                .is_err()
        );
        let path = directory.path().join("huge.jpg");
        std::fs::File::create_new(&path)
            .unwrap()
            .set_len(MAX_FILE_BYTES + 1)
            .unwrap();
        assert!(
            context
                .block_on(load(&path, &cancellable))
                .unwrap_err()
                .contains("64 MiB")
        );
        cancellable.cancel();
        assert!(
            context
                .block_on(load(&path, &cancellable))
                .unwrap_err()
                .contains("cancelled")
        );
    }
}
