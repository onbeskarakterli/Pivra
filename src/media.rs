//! Descriptions supplied by callers, not format detection or decoded media.

use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaKind {
    Image,
    Video,
}

/// A path and an explicitly supplied kind. Existence/readability is not implied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaItem {
    path: PathBuf,
    kind: MediaKind,
}

impl MediaItem {
    pub fn new(path: impl Into<PathBuf>, kind: MediaKind) -> Self {
        Self {
            path: path.into(),
            kind,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn kind(&self) -> MediaKind {
        self.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_supplied_path_and_kind_without_extension_detection() {
        let path = Path::new("örnek klasör/henüz-yok.unknown");
        for kind in [MediaKind::Image, MediaKind::Video] {
            let item = MediaItem::new(path, kind);
            assert_eq!(item.path(), path);
            assert_eq!(item.kind(), kind);
        }
    }
}
