//! Glycin boundary: no GTK widgets, no fallback decoder and no unsandboxed mode.

use gtk::gio::{self, prelude::*};
use std::path::Path;

pub(crate) type LoadedFrame = glycin::Frame;

const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PIXELS: u64 = 32_000_000;
const MAX_FRAME_BYTES: usize = 512 * 1024 * 1024;
const MAX_EDGE: u32 = 32_768;

// Error-only evidence from the same GFile backend used by Glycin, never a MIME override.
fn input_diagnostic(path: &Path, cancel: &gio::Cancellable) -> String {
    let file = gio::File::for_path(path);
    let result = (|| -> Result<String, gtk::glib::Error> {
        let stream = file.read(Some(cancel))?;
        let bytes = stream.read_bytes(512, Some(cancel))?;
        let (kind, uncertain) = gio::content_type_guess(None::<&Path>, bytes.as_ref());
        let mime = gio::content_type_get_mime_type(&kind);
        let signature = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            "PNG"
        } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
            "JPEG"
        } else {
            "other/truncated"
        };
        Ok(format!(
            "content-only MIME={mime:?}, uncertain={uncertain}, signature={signature}, sampled={}",
            bytes.len()
        ))
    })();
    format!(
        "input={path:?}, uri={}, {}",
        file.uri(),
        result.unwrap_or_else(|e| format!("diagnostic read failed: {e}"))
    )
}

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
) -> Result<LoadedFrame, String> {
    load_limited(path, cancellable, None, None).await
}

#[allow(dead_code)] // Standalone image_probe includes only the foreground path.
pub(crate) async fn preload(
    path: &Path,
    cancellable: &gio::Cancellable,
    budget: usize,
) -> Result<LoadedFrame, String> {
    load_limited(path, cancellable, Some(budget), None).await
}
/// Opt-in thumbnail policy. Foreground/preload defaults remain unchanged.
#[allow(dead_code)]
pub(crate) async fn load_thumbnail(
    path: &Path,
    cancel: &gio::Cancellable,
    admit: &dyn Fn(u32, u32) -> Result<(), String>,
) -> Result<LoadedFrame, String> {
    load_limited(path, cancel, None, Some(admit)).await
}
fn dimensions_for(width: u32, height: u32, thumbnail: bool) -> Result<(), String> {
    if !thumbnail {
        return validate_dimensions(width, height);
    }
    if width == 0
        || height == 0
        || width > MAX_EDGE
        || height > MAX_EDGE
        || u64::from(width) * u64::from(height) > 50_000_000
    {
        return Err("Thumbnail exceeds experimental 50 MP / 32768 edge ceiling".into());
    }
    Ok(())
}
async fn load_limited(
    path: &Path,
    cancellable: &gio::Cancellable,
    budget: Option<usize>,
    admit: Option<&dyn Fn(u32, u32) -> Result<(), String>>,
) -> Result<LoadedFrame, String> {
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
    let image = loader.load().await.map_err(|e| {
        format!(
            "Secure image loader failed: {e}\n{}",
            input_diagnostic(path, cancellable)
        )
    })?;
    if !matches!(
        image.mime_type().to_string().as_str(),
        "image/jpeg"
            | "image/png"
            | "image/webp"
            | "image/gif"
            | "image/bmp"
            | "image/tiff"
            | "image/avif"
            | "image/heic"
            | "image/heif"
    ) {
        return Err("Decoded image format is not enabled".into());
    }
    dimensions_for(
        image.details().width(),
        image.details().height(),
        admit.is_some(),
    )?;
    if let Some(admit) = admit {
        admit(image.details().width(), image.details().height())?;
    }
    if let Some(budget) = budget {
        if pivra::image_cache::texture_cost(image.details().width(), image.details().height())
            .is_none_or(|cost| cost > budget)
        {
            return Err("Preload exceeds cache entry budget".into());
        }
    }
    let frame = image
        .next_frame()
        .await
        .map_err(|e| format!("Image decode failed: {e}"))?;
    dimensions_for(frame.width(), frame.height(), admit.is_some())?;
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
    fn thumbnail_opt_in_does_not_raise_foreground_pixel_ceiling() {
        assert!(dimensions_for(6000, 4000, false).is_ok());
        assert!(dimensions_for(8000, 6000, false).is_err());
        assert!(dimensions_for(8000, 6000, true).is_ok());
        assert!(dimensions_for(10000, 5000, true).is_ok());
        assert!(dimensions_for(12000, 9000, true).is_err());
        assert!(pivra::image_cache::texture_cost(6000, 4000).unwrap() > 256 * 1024 * 1024);
    }
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
    #[test]
    fn content_diagnostic_uses_bytes_not_extension_or_uri_spelling() {
        let dir = tempfile::tempdir().unwrap();
        for (name, bytes, expected) in [
            (
                "wrong.jpg",
                include_bytes!("../tests/fixtures/images/wide.png").as_slice(),
                "signature=PNG",
            ),
            (
                "wrong.png",
                include_bytes!("../tests/fixtures/images/no-exif.jpg").as_slice(),
                "signature=JPEG",
            ),
            (
                "fake.png",
                b"plain text, not an image".as_slice(),
                "signature=other/truncated",
            ),
            ("empty.png", b"".as_slice(), "sampled=0"),
        ] {
            let path = dir.path().join(name);
            std::fs::write(&path, bytes).unwrap();
            let diagnostic = input_diagnostic(&path, &gio::Cancellable::new());
            assert!(diagnostic.contains(expected), "{diagnostic}");
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
    }
}
