//! Locale-independent natural ordering of Linux filename bytes; no filesystem I/O.

use std::{cmp::Ordering, path::Path};

/// Natural filename first, then exact full-path bytes for identical filenames.
/// ASCII letters fold only in the primary comparison. Non-ASCII bytes are opaque.
/// Numeric values use significant length/digits, then (only on primary equality)
/// the first differing numeric run width. Raw bytes finally break case ties.
pub fn compare_paths(left: &Path, right: &Path) -> Ordering {
    let a = left
        .file_name()
        .unwrap_or(left.as_os_str())
        .as_encoded_bytes();
    let b = right
        .file_name()
        .unwrap_or(right.as_os_str())
        .as_encoded_bytes();
    compare_names(a, b).then_with(|| {
        left.as_os_str()
            .as_encoded_bytes()
            .cmp(right.as_os_str().as_encoded_bytes())
    })
}

fn compare_names(a: &[u8], b: &[u8]) -> Ordering {
    let (mut i, mut j) = (0, 0);
    let mut width_tie = Ordering::Equal;
    while i < a.len() && j < b.len() {
        if a[i].is_ascii_digit() && b[j].is_ascii_digit() {
            let end_a = digit_end(a, i);
            let end_b = digit_end(b, j);
            let mut significant_a = i;
            let mut significant_b = j;
            while significant_a < end_a && a[significant_a] == b'0' {
                significant_a += 1;
            }
            while significant_b < end_b && b[significant_b] == b'0' {
                significant_b += 1;
            }
            let order = (end_a - significant_a)
                .cmp(&(end_b - significant_b))
                .then_with(|| a[significant_a..end_a].cmp(&b[significant_b..end_b]));
            if order != Ordering::Equal {
                return order;
            }
            width_tie = width_tie.then_with(|| (end_a - i).cmp(&(end_b - j)));
            i = end_a;
            j = end_b;
        } else {
            let order = a[i].to_ascii_lowercase().cmp(&b[j].to_ascii_lowercase());
            if order != Ordering::Equal {
                return order;
            }
            i += 1;
            j += 1;
        }
    }
    (a.len() - i)
        .cmp(&(b.len() - j))
        .then(width_tie)
        .then_with(|| a.cmp(b))
}

fn digit_end(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    fn sorted(names: &[&str]) -> Vec<String> {
        let mut names: Vec<_> = names.iter().map(|s| s.to_string()).collect();
        names.sort_by(|a, b| compare_paths(Path::new(a), Path::new(b)));
        names
    }

    #[test]
    fn numbers_mixed_extensions_and_multiple_numeric_blocks() {
        assert_eq!(
            sorted(&["foto10.jpg", "foto2.mp4", "foto1.png"]),
            ["foto1.png", "foto2.mp4", "foto10.jpg"]
        );
        assert_eq!(
            sorted(&["a10b1", "a2b10", "a2b3"]),
            ["a2b3", "a2b10", "a10b1"]
        );
        assert_eq!(
            sorted(&["same.png", "same.jpg", "same.mp4"]),
            ["same.jpg", "same.mp4", "same.png"]
        );
    }

    #[test]
    fn numeric_equality_uses_width_after_all_primary_tokens() {
        assert_eq!(
            sorted(&["foto002", "foto02", "foto2", "foto000", "foto0"]),
            ["foto0", "foto000", "foto2", "foto02", "foto002"]
        );
        assert_eq!(
            sorted(&["a2b10", "a02b3", "a002b3", "a2b3"]),
            ["a2b3", "a02b3", "a002b3", "a2b10"]
        );
    }

    #[test]
    fn long_numbers_do_not_parse_integers() {
        let low = format!("photo{}", "9".repeat(10_000));
        let high = format!("photo1{}", "0".repeat(10_000));
        assert_eq!(
            compare_paths(Path::new(&low), Path::new(&high)),
            Ordering::Less
        );
        let padded = format!("photo{}{}", "0".repeat(10_000), "9".repeat(10_000));
        assert_eq!(
            compare_paths(Path::new(&low), Path::new(&padded)),
            Ordering::Less
        );
    }

    #[test]
    fn ascii_folding_unicode_and_raw_linux_bytes_have_explicit_order() {
        assert_eq!(
            sorted(&["foto10", "FOTO2", "Foto1", "foto2"]),
            ["Foto1", "FOTO2", "foto2", "foto10"]
        );
        assert_eq!(
            sorted(&["ı2", "İ2", "I2", "i2", "ş2", "ç2", "🦀2"]),
            ["I2", "i2", "ç2", "İ2", "ı2", "ş2", "🦀2"]
        );
        let raw = OsString::from_vec(b"foto\xff2.png".to_vec());
        let higher = OsString::from_vec(b"foto\xff10.png".to_vec());
        assert_eq!(
            compare_paths(Path::new(&raw), Path::new(&higher)),
            Ordering::Less
        );
        assert_eq!(raw.as_encoded_bytes(), b"foto\xff2.png");
        assert_eq!(
            compare_paths(Path::new("b/same.png"), Path::new("a/same.png")),
            Ordering::Greater
        );
    }

    #[test]
    fn total_order_laws_and_permutations() {
        let mut names: Vec<OsString> = [
            "", "a", "A", "a0", "a00", "a2", "a02", "A002", "a2b3", "a02b10", "a10", "a-2", "a.2",
            "İ2", "ı2", "🦀2",
        ]
        .into_iter()
        .map(OsString::from)
        .collect();
        names.push(OsString::from_vec(b"a\xff2".to_vec()));
        names.push(OsString::from_vec(b"a\xfe10".to_vec()));
        let cmp = |a: &OsString, b: &OsString| compare_paths(Path::new(a), Path::new(b));
        for a in &names {
            for b in &names {
                assert_eq!(cmp(a, b), cmp(b, a).reverse());
                assert_eq!(cmp(a, b) == Ordering::Equal, a == b);
                for c in &names {
                    if cmp(a, b) != Ordering::Greater && cmp(b, c) != Ordering::Greater {
                        assert_ne!(cmp(a, c), Ordering::Greater);
                    }
                }
            }
        }
        let mut expected = names.clone();
        expected.sort_by(cmp);
        for rotation in 0..names.len() {
            let mut permuted = names.clone();
            permuted.rotate_left(rotation);
            if rotation % 2 == 0 {
                permuted.reverse();
            }
            permuted.sort_by(cmp);
            assert_eq!(permuted, expected);
        }
    }
}
