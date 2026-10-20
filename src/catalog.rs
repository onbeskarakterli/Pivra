//! In-memory ordered selection, independent of a directory scanner or UI.
//!
//! Callers supply the complete order. No sorting, filtering, filesystem access,
//! next/previous commands, wrapping policy or file watching is implemented here.

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
