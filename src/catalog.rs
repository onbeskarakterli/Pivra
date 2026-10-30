//! In-memory ordered selection, independent of a directory scanner or UI.
//!
//! Callers supply the complete order. No sorting, filtering, filesystem access,
//! wrapping policy or file watching is implemented here.

use crate::media::MediaItem;

/// Selection is absent or a valid index into `items`. A removed file leaves
/// no selection until the user navigates, even if neighbors remain.
/// Immutable item access prevents callers from invalidating that invariant.
#[derive(Debug)]
pub struct MediaCollection {
    items: Vec<MediaItem>,
    selected: Option<usize>,
}

impl MediaCollection {
    pub fn new(items: Vec<MediaItem>) -> Self {
        let selected = if items.is_empty() { None } else { Some(0) };
        Self { items, selected }
    }

    /// Reconcile a filesystem snapshot without selecting a different file.
    pub fn replace_preserving(&mut self, items: Vec<MediaItem>) -> bool {
        let path = self.current().map(|item| item.path().to_owned());
        self.items = items;
        self.selected = path
            .as_ref()
            .and_then(|path| self.items.iter().position(|item| item.path() == path));
        path.is_some() && self.selected.is_none()
    }

    pub fn rename_selected(&mut self, old: &std::path::Path, new: &std::path::Path) {
        if let Some(index) = self
            .selected
            .filter(|_| self.current().is_some_and(|item| item.path() == old))
        {
            self.items[index] = MediaItem::new(new, self.items[index].kind());
        }
    }

    /// Next, then previous supported JPEG/PNG; video/unsupported candidates are skipped.
    pub fn photo_neighbors(&self) -> Vec<std::path::PathBuf> {
        let Some(index) = self.selected else {
            return vec![];
        };
        let supported = |item: &&MediaItem| {
            item.kind() == crate::media::MediaKind::Image
                && crate::discovery::supported_photo(item.path())
        };
        self.items[index + 1..]
            .iter()
            .find(supported)
            .into_iter()
            .chain(self.items[..index].iter().rev().find(supported))
            .map(|i| i.path().to_owned())
            .collect()
    }

    pub fn items(&self) -> &[MediaItem] {
        &self.items
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    pub fn current(&self) -> Option<&MediaItem> {
        self.selected.and_then(|index| self.items.get(index))
    }

    /// Find the next displayable v0.1 photo without changing catalog order or
    /// selection. A bounded scan stops at the edge; navigation never wraps.
    pub fn photo_target(&self, forward: bool) -> Option<usize> {
        let supported = |index: &usize| {
            let item = &self.items[*index];
            item.kind() == crate::media::MediaKind::Image
                && crate::discovery::supported_photo(item.path())
        };
        if forward {
            let start = self.selected.map_or(0, |index| index + 1);
            (start..self.items.len()).find(supported)
        } else {
            (0..self.selected.unwrap_or(self.items.len()))
                .rev()
                .find(supported)
        }
    }

    pub fn select_photo(&mut self, forward: bool) -> bool {
        self.photo_target(forward)
            .is_some_and(|index| self.select(index))
    }

    /// Move one step without wrapping. False means selection did not change.
    pub fn select_previous(&mut self) -> bool {
        if self.selected.is_none() {
            return self.select(self.items.len().saturating_sub(1));
        }
        let Some(index) = self.selected.and_then(|index| index.checked_sub(1)) else {
            return false;
        };
        self.select(index)
    }

    pub fn select_next(&mut self) -> bool {
        if self.selected.is_none() {
            return self.select(0);
        }
        let Some(index) = self.selected.and_then(|index| index.checked_add(1)) else {
            return false;
        };
        self.select(index)
    }

    /// Select an existing item. Invalid indices return false and keep selection.
    pub fn select(&mut self, index: usize) -> bool {
        if index >= self.items.len() {
            return false;
        }
        self.selected = Some(index);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::MediaKind;

    fn item(path: &str) -> MediaItem {
        MediaItem::new(path, MediaKind::Image)
    }

    #[test]
    fn empty_list_has_no_selection_and_rejects_indices() {
        let mut collection = MediaCollection::new(vec![]);
        assert!(collection.items().is_empty());
        assert_eq!(collection.selected_index(), None);
        assert_eq!(collection.current(), None);
        assert!(!collection.select(0));
        assert!(!collection.select(usize::MAX));
        assert_eq!(collection.selected_index(), None);
        assert_eq!(collection.current(), None);
    }

    #[test]
    fn selects_first_and_last_without_reordering() {
        let items = vec![item("10.jpg"), item("2.jpg"), item("1.jpg")];
        let mut collection = MediaCollection::new(items.clone());
        assert_eq!(collection.current(), Some(&items[0]));
        assert!(collection.select(2));
        assert_eq!(collection.selected_index(), Some(2));
        assert_eq!(collection.current(), Some(&items[2]));
        assert!(collection.select(0));
        assert_eq!(collection.current(), Some(&items[0]));
        assert_eq!(collection.items(), items.as_slice());
    }

    #[test]
    fn invalid_selection_preserves_current_item() {
        let items = vec![item("a.jpg"), item("b.jpg")];
        let mut collection = MediaCollection::new(items.clone());
        assert!(collection.select(1));
        for invalid in [items.len(), usize::MAX] {
            assert!(!collection.select(invalid));
            assert_eq!(collection.selected_index(), Some(1));
            assert_eq!(collection.current(), Some(&items[1]));
        }
    }

    #[test]
    fn single_item_accepts_only_index_zero() {
        let only = item("only.jpg");
        let mut collection = MediaCollection::new(vec![only.clone()]);
        assert!(collection.select(0));
        assert!(!collection.select(1));
        assert_eq!(collection.selected_index(), Some(0));
        assert_eq!(collection.current(), Some(&only));
    }
}

#[cfg(test)]
mod navigation_tests {
    use super::*;
    use crate::{media::MediaKind, natural};

    #[test]
    fn empty_and_single_catalogs_do_not_move_or_wrap() {
        for items in [vec![], vec![MediaItem::new("only.png", MediaKind::Image)]] {
            let mut catalog = MediaCollection::new(items);
            let original = catalog.current().cloned();
            for _ in 0..10 {
                assert!(!catalog.select_previous());
                assert!(!catalog.select_next());
                assert_eq!(catalog.current(), original.as_ref());
            }
        }
    }

    #[test]
    fn mixed_natural_order_preserves_initial_selection_and_stops_at_bounds() {
        let mut items = vec![
            MediaItem::new("10.jpg", MediaKind::Image),
            MediaItem::new("2.mp4", MediaKind::Video),
            MediaItem::new("1.png", MediaKind::Image),
        ];
        items.sort_by(|a, b| natural::compare_paths(a.path(), b.path()));
        let mut catalog = MediaCollection::new(items.clone());
        assert!(catalog.select(2));
        assert!(!catalog.select_next());
        assert_eq!(catalog.current(), Some(&items[2]));
        assert!(catalog.select_previous());
        assert_eq!(catalog.current().unwrap().kind(), MediaKind::Video);
        assert!(catalog.select_previous());
        assert!(!catalog.select_previous());
        assert_eq!(catalog.current(), Some(&items[0]));
        assert!(catalog.select_next());
        assert!(catalog.select_next());
        assert!(!catalog.select_next());
        assert_eq!(catalog.items(), items);
    }

    #[test]
    fn rapid_alternation_and_two_catalogs_are_independent_without_filesystem_io() {
        let items = vec![
            MediaItem::new("deleted.png", MediaKind::Image),
            MediaItem::new("broken.jpg", MediaKind::Image),
        ];
        let mut a = MediaCollection::new(items.clone());
        let mut b = MediaCollection::new(items);
        assert!(b.select(1));
        for _ in 0..1000 {
            assert!(a.select_next());
            assert!(a.select_previous());
            assert_eq!(b.selected_index(), Some(1));
        }
        assert_eq!(a.selected_index(), Some(0));
    }
}

#[cfg(test)]
mod reconciliation_tests {
    use super::*;
    use crate::media::MediaKind;
    fn items(names: &[&str]) -> Vec<MediaItem> {
        names
            .iter()
            .map(|name| MediaItem::new(name, MediaKind::Image))
            .collect()
    }
    #[test]
    fn insertion_and_unselected_deletion_preserve_selected_path() {
        let mut c = MediaCollection::new(items(&["2.png", "10.png"]));
        c.select(1);
        assert!(!c.replace_preserving(items(&["1.png", "2.png", "10.png"])));
        assert_eq!(c.selected_index(), Some(2));
        assert!(!c.replace_preserving(items(&["10.png"])));
        assert_eq!(c.selected_index(), Some(0));
    }
    #[test]
    fn selected_removal_does_not_jump_and_navigation_can_resume() {
        let mut c = MediaCollection::new(items(&["1.png", "2.png"]));
        assert!(c.replace_preserving(items(&["2.png"])));
        assert!(c.current().is_none());
        assert!(c.select_next());
        assert_eq!(c.selected_index(), Some(0));
        c.replace_preserving(vec![]);
        assert!(!c.select_previous());
        assert!(!c.select_next());
        c.replace_preserving(items(&["10.png"]));
        assert!(c.current().is_none());
        assert!(c.select_previous());
    }
    #[test]
    fn rename_and_snapshot_use_latest_selection_not_scan_start_selection() {
        let mut c = MediaCollection::new(items(&["1.png", "2.png"]));
        c.rename_selected("1.png".as_ref(), "10.png".as_ref());
        assert!(!c.replace_preserving(items(&["2.png", "10.png"])));
        assert_eq!(c.selected_index(), Some(1));
        c.select_previous();
        c.rename_selected("10.png".as_ref(), "20.png".as_ref());
        assert!(!c.replace_preserving(items(&["2.png", "20.png"])));
        assert_eq!(c.selected_index(), Some(0));
    }
}

#[cfg(test)]
mod neighbor_tests {
    use super::*;
    use crate::media::MediaKind;
    #[test]
    fn next_previous_boundaries_and_mixed_candidates() {
        let mut c = MediaCollection::new(vec![
            MediaItem::new("1.png", MediaKind::Image),
            MediaItem::new("2.mp4", MediaKind::Video),
            MediaItem::new("3.webp", MediaKind::Image),
            MediaItem::new("4.jpg", MediaKind::Image),
            MediaItem::new("10.PNG", MediaKind::Image),
        ]);
        assert_eq!(c.photo_neighbors(), vec![std::path::PathBuf::from("3.webp")]);
        c.select(3);
        assert_eq!(
            c.photo_neighbors(),
            vec![
                std::path::PathBuf::from("10.PNG"),
                std::path::PathBuf::from("1.png")
            ]
        );
        c.select(4);
        assert_eq!(c.photo_neighbors(), vec![std::path::PathBuf::from("4.jpg")]);
        assert!(MediaCollection::new(vec![]).photo_neighbors().is_empty());
        assert!(
            MediaCollection::new(vec![MediaItem::new("only.png", MediaKind::Image)])
                .photo_neighbors()
                .is_empty()
        );
    }
}

#[cfg(test)]
mod photo_navigation_tests {
    use super::*;
    use crate::media::MediaKind::{Image, Video};

    #[test]
    fn skips_consecutive_videos_and_unsupported_images_without_removing_entries() {
        let items = vec![
            MediaItem::new("1.png", Image),
            MediaItem::new("2.mp4", Video),
            MediaItem::new("3.webm", Video),
            MediaItem::new("4.webp", Image),
            MediaItem::new("10.JPEG", Image),
            MediaItem::new("11.mp4", Video),
        ];
        let mut c = MediaCollection::new(items.clone());
        assert!(c.select_photo(true));
        assert_eq!(c.selected_index(), Some(3));
        assert!(c.select_photo(true));
        assert_eq!(c.selected_index(), Some(4));
        assert!(!c.select_photo(true));
        assert_eq!(c.selected_index(), Some(4));
        assert!(c.select_photo(false));
        assert_eq!(c.selected_index(), Some(3));
        assert!(c.select_photo(false));
        assert_eq!(c.selected_index(), Some(0));
        assert!(!c.select_photo(false));
        assert_eq!(c.items(), items);
        // Direct opening/selection is unchanged, including unsupported video.
        assert!(c.select(2));
        assert_eq!(c.current().unwrap().kind(), Video);
        assert!(c.select_photo(false));
        assert_eq!(c.selected_index(), Some(0));
    }

    #[test]
    fn no_supported_neighbor_preserves_selection_and_finishes() {
        for items in [
            vec![],
            vec![MediaItem::new("only.png", Image)],
            vec![
                MediaItem::new("1.mp4", Video),
                MediaItem::new("2.mp4", Video),
            ],
        ] {
            let mut c = MediaCollection::new(items);
            let initial = c.selected_index();
            for _ in 0..100 {
                assert!(!c.select_photo(true));
                assert!(!c.select_photo(false));
                assert_eq!(c.selected_index(), initial);
            }
        }
    }
}
