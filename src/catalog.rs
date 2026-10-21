//! In-memory ordered selection, independent of a directory scanner or UI.
//!
//! Callers supply the complete order. No sorting, filtering, filesystem access,
//! wrapping policy or file watching is implemented here.

use crate::media::MediaItem;

/// Selection is either absent for an empty list or a valid index into `items`.
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

    pub fn items(&self) -> &[MediaItem] {
        &self.items
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    pub fn current(&self) -> Option<&MediaItem> {
        self.selected.and_then(|index| self.items.get(index))
    }

    /// Move one step without wrapping. False means selection did not change.
    pub fn select_previous(&mut self) -> bool {
        let Some(index) = self.selected.and_then(|index| index.checked_sub(1)) else {
            return false;
        };
        self.select(index)
    }

    pub fn select_next(&mut self) -> bool {
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
