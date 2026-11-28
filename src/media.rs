//! Descriptions supplied by callers, not format detection or decoded media.

use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaKind {
    Image,
    Video,
}

/// A path and an explicitly supplied kind. Existence/readability is not implied.
#[derive(Clone, Debug)]
pub struct MediaItem {
    path: PathBuf,
    kind: MediaKind,
    sort_fields: Option<crate::media_sort::Fields>,
}

impl PartialEq for MediaItem {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.kind == other.kind
    }
}
impl Eq for MediaItem {}

impl MediaItem {
    pub fn new(path: impl Into<PathBuf>, kind: MediaKind) -> Self {
        Self {
            path: path.into(),
            kind,
            sort_fields: None,
        }
    }

    pub(crate) fn with_sort_fields(mut self, fields: crate::media_sort::Fields) -> Self {
        self.sort_fields = Some(fields);
        self
    }

    pub(crate) fn sort_fields(&self) -> Option<crate::media_sort::Fields> {
        self.sort_fields
    }

    pub fn at_path(&self, path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            kind: self.kind,
            sort_fields: self.sort_fields,
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
