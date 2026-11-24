//! Small metadata-only cache fed by existing decoders/discovery. Never stores pixels.
use crate::image_cache::FileKey;
use std::{
    collections::VecDeque,
    path::Path,
    sync::{Mutex, OnceLock},
};
pub type Fields = Vec<(String, String)>;
fn display_text(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).take(128).collect()
}
fn cache() -> &'static Mutex<VecDeque<(FileKey, Fields)>> {
    static CACHE: OnceLock<Mutex<VecDeque<(FileKey, Fields)>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}
pub fn remember(key: FileKey, fields: Fields) {
    let fields = fields
        .into_iter()
        .take(16)
        .map(|(k, v)| (display_text(&k), display_text(&v)))
        .collect();
    let mut cache = cache().lock().unwrap();
    cache.retain(|(old, _)| old.path != key.path);
    cache.push_back((key, fields));
    while cache.len() > 16 {
        cache.pop_front();
    }
}
/// File identity is checked on the caller's worker, never by a GTK render callback.
pub fn lookup(path: &Path) -> Fields {
    let Ok(key) = FileKey::read(path) else {
        return vec![];
    };
    cache()
        .lock()
        .unwrap()
        .iter()
        .find(|(old, _)| old == &key)
        .map(|(_, fields)| fields.clone())
        .unwrap_or_default()
}
pub fn image_fields(image: &glycin::Image) -> Fields {
    use std::os::fd::AsRawFd;
    let details = image.details();
    let mut fields = vec![("Format".into(), image.mime_type().to_string())];
    if let Some(data) = details.metadata_exif() {
        // Check the sealed metadata memfd before copying; no JPEG decoder on host.
        let bounded = std::fs::metadata(format!("/proc/self/fd/{}", data.as_raw_fd()))
            .is_ok_and(|m| m.len() <= 1024 * 1024);
        if let Some(exif) = bounded
            .then(|| data.get_full().ok())
            .flatten()
            .and_then(|bytes| gufo_exif::Exif::new(bytes).ok())
        {
            if let Some(v) = exif.make() {
                fields.push(("Camera make".into(), v));
            }
            if let Some(v) = exif.model() {
                fields.push(("Camera model".into(), v));
            }
            if let Some(v) = exif.iso_speed_rating() {
                fields.push(("ISO".into(), v.to_string()));
            }
            if let Some(v) = exif.f_number().filter(|v| v.is_finite()) {
                fields.push(("Aperture".into(), format!("f/{v:.1}")));
            }
            if let Some(v) = exif.focal_length().filter(|v| v.is_finite()) {
                fields.push(("Focal length".into(), format!("{v:.1} mm")));
            }
            if let Some((a, b)) = exif.exposure_time().filter(|(_, b)| *b != 0) {
                fields.push(("Exposure".into(), format!("{a}/{b} s")));
            }
            if let Some(v) = exif.orientation() {
                fields.push(("EXIF orientation".into(), format!("{v:?}")));
            }
        }
    }
    if let Some(values) = details.metadata_key_value() {
        fields.extend(values.iter().take(8).map(|(k, v)| (k.clone(), v.clone())));
    }
    fields
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replacement_and_deletion_cannot_reuse_old_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("photo");
        std::fs::write(&path, b"old").unwrap();
        remember(
            FileKey::read(&path).unwrap(),
            vec![("Camera".into(), "Old".into())],
        );
        assert_eq!(lookup(&path)[0].1, "Old");
        remember(
            FileKey::read(&path).unwrap(),
            vec![("Metadata".into(), format!("\0{}", "x".repeat(1000)))],
        );
        assert_eq!(lookup(&path)[0].1, "x".repeat(128));
        std::fs::write(&path, b"replacement content").unwrap();
        assert!(lookup(&path).is_empty());
        std::fs::remove_file(&path).unwrap();
        assert!(lookup(&path).is_empty());
    }
}
