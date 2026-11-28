//! In-memory media ordering. Folder workers attach metadata already read during discovery.
use crate::{media::MediaItem, natural};
use std::{cmp::Ordering, fs::Metadata, time::SystemTime};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fields {
    pub modified: Option<SystemTime>,
    pub bytes: u64,
}

impl From<&Metadata> for Fields {
    fn from(metadata: &Metadata) -> Self {
        Self {
            modified: metadata.modified().ok(),
            bytes: metadata.len(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Order {
    #[default]
    NameAscending,
    NameDescending,
    ModifiedAscending,
    ModifiedDescending,
    SizeAscending,
    SizeDescending,
    TypeAscending,
    TypeDescending,
}

impl Order {
    pub const OPTIONS: [(&str, &str); 8] = [
        ("name-asc", "Name (A–Z)"),
        ("name-desc", "Name (Z–A)"),
        ("modified-asc", "Date Modified (Oldest First)"),
        ("modified-desc", "Date Modified (Newest First)"),
        ("size-asc", "File Size (Smallest First)"),
        ("size-desc", "File Size (Largest First)"),
        ("type-asc", "File Type (A–Z)"),
        ("type-desc", "File Type (Z–A)"),
    ];

    pub fn id(self) -> &'static str {
        Self::OPTIONS[self as usize].0
    }

    pub fn from_id(id: &str) -> Option<Self> {
        const ORDERS: [Order; 8] = [
            Order::NameAscending,
            Order::NameDescending,
            Order::ModifiedAscending,
            Order::ModifiedDescending,
            Order::SizeAscending,
            Order::SizeDescending,
            Order::TypeAscending,
            Order::TypeDescending,
        ];
        ORDERS.into_iter().find(|order| order.id() == id)
    }

    pub fn sort(self, items: &mut [MediaItem]) {
        items.sort_unstable_by(|left, right| self.compare(left, right));
    }

    pub fn compare(self, left: &MediaItem, right: &MediaItem) -> Ordering {
        let name = || natural::compare_paths(left.path(), right.path());
        let a = left.sort_fields();
        let b = right.sort_fields();
        let primary = match self {
            Self::NameAscending => return name(),
            Self::NameDescending => return name().reverse(),
            Self::ModifiedAscending | Self::ModifiedDescending => compare_optional(
                a.and_then(|field| field.modified),
                b.and_then(|field| field.modified),
                self == Self::ModifiedDescending,
            ),
            Self::SizeAscending | Self::SizeDescending => compare_optional(
                a.map(|field| field.bytes),
                b.map(|field| field.bytes),
                self == Self::SizeDescending,
            ),
            Self::TypeAscending | Self::TypeDescending => {
                let casefold = |byte: u8| byte.to_ascii_lowercase();
                let cmp = extension(left)
                    .bytes()
                    .map(casefold)
                    .cmp(extension(right).bytes().map(casefold));
                if self == Self::TypeDescending {
                    cmp.reverse()
                } else {
                    cmp
                }
            }
        };
        primary.then_with(name)
    }
}

fn extension(item: &MediaItem) -> &str {
    let suffix = item
        .path()
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if suffix.eq_ignore_ascii_case("jpeg") {
        "jpg"
    } else {
        suffix
    }
}

fn compare_optional<T: Ord>(left: Option<T>, right: Option<T>, descending: bool) -> Ordering {
    match (left, right) {
        (Some(a), Some(b)) if descending => b.cmp(&a),
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{catalog::MediaCollection, media::MediaKind};
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn ordering_preserves_active_item_and_resolves_equal_or_missing_keys() {
        let item = |path, bytes, time| {
            MediaItem::new(path, MediaKind::Image).with_sort_fields(Fields {
                modified: Some(UNIX_EPOCH + Duration::from_secs(time)),
                bytes,
            })
        };
        let items = vec![
            item("10.PNG", 7, 1),
            item("2.png", 7, 1),
            item("8.jpeg", 10, 2),
            MediaItem::new("1.png", MediaKind::Image),
        ];
        let mut collection = MediaCollection::new(items);
        for (order, expected) in [
            (Order::NameAscending, ["1.png", "2.png", "8.jpeg", "10.PNG"]),
            (
                Order::NameDescending,
                ["10.PNG", "8.jpeg", "2.png", "1.png"],
            ),
            (Order::SizeAscending, ["2.png", "10.PNG", "8.jpeg", "1.png"]),
            (
                Order::SizeDescending,
                ["8.jpeg", "2.png", "10.PNG", "1.png"],
            ),
            (
                Order::ModifiedDescending,
                ["8.jpeg", "2.png", "10.PNG", "1.png"],
            ),
            (Order::TypeAscending, ["8.jpeg", "1.png", "2.png", "10.PNG"]),
        ] {
            collection.sort_preserving(order);
            let paths: Vec<_> = collection
                .items()
                .iter()
                .map(|item| item.path().to_str().unwrap())
                .collect();
            assert_eq!(paths, expected);
            assert_eq!(
                collection.current().unwrap().path().to_str(),
                Some("10.PNG")
            );
        }
        assert!(
            Order::OPTIONS
                .into_iter()
                .all(|(id, _)| Order::from_id(id).unwrap().id() == id)
        );
    }
}
