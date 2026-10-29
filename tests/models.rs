mod support;

use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use pivra::catalog::MediaCollection;
use pivra::media::{MediaItem, MediaKind};

const PNG: &[u8] = include_bytes!("fixtures/pixels-2x2.png");
const MODEL_VIDEO: &[u8] = include_bytes!("fixtures/model-video.txt");

#[test]
fn mixed_catalog_preserves_unicode_paths_kinds_order_and_file_contents() {
    let directory = support::temp_dir();
    let cases = [
        ("10 kırmızı kare.png", MediaKind::Image, PNG),
        ("2 görüntü 🦀.png", MediaKind::Image, PNG),
        ("1 video örneği.fixture", MediaKind::Video, MODEL_VIDEO),
    ];
    let items: Vec<_> = cases
        .iter()
        .map(|(name, kind, bytes)| {
            let path = directory.path().join(name);
            // Only fresh files inside this test's private directory are written.
            let mut file = fs::File::create_new(&path).expect("create fixture copy");
            std::io::Write::write_all(&mut file, bytes).expect("write fixture copy");
            MediaItem::new(path, *kind)
        })
        .collect();
    let mut collection = MediaCollection::new(items.clone());
    assert_eq!(collection.items(), items.as_slice());
    assert_eq!(collection.current(), Some(&items[0]));

    for (index, (name, kind, bytes)) in cases.iter().enumerate() {
        assert!(collection.select(index));
        let item = collection.current().expect("selected item");
        assert_eq!(item.path().file_name(), Some(Path::new(name).as_os_str()));
        assert_eq!(item.kind(), *kind);
        assert_eq!(fs::read(item.path()).expect("read copied fixture"), *bytes);
    }
    for invalid in [cases.len(), usize::MAX] {
        assert!(!collection.select(invalid));
        assert_eq!(collection.selected_index(), Some(2));
        assert_eq!(collection.current(), Some(&items[2]));
    }
    // Selection must not modify files or silently reorder names naturally.
    assert_eq!(collection.items(), items.as_slice());
    for (name, _, bytes) in cases {
        assert_eq!(fs::read(directory.path().join(name)).unwrap(), bytes);
    }
}

#[test]
fn temporary_files_are_removed_after_normal_scope_and_unwinding() {
    let normal_path;
    {
        let directory = support::temp_dir();
        normal_path = directory.path().to_owned();
        fs::write(normal_path.join("normal Türkçe.txt"), b"owned test data").unwrap();
    }
    assert!(!normal_path.exists());

    let mut unwind_path = None;
    let result = catch_unwind(AssertUnwindSafe(|| {
        let directory = support::temp_dir();
        unwind_path = Some(directory.path().to_owned());
        fs::write(directory.path().join("panic 🦀.txt"), b"owned test data").unwrap();
        panic!("intentional panic to verify RAII cleanup");
    }));
    assert!(result.is_err());
    assert!(!unwind_path.expect("created directory").exists());
}

#[test]
fn parallel_tests_can_use_the_same_filename_without_sharing_files() {
    let handles: Vec<_> = (0..8_u8)
        .map(|value| {
            std::thread::spawn(move || {
                let directory = support::temp_dir();
                let path = directory.path().to_owned();
                let file = path.join("aynı dosya 🦀.txt");
                fs::write(&file, [value]).unwrap();
                assert_eq!(fs::read(file).unwrap(), [value]);
                // Keep each directory alive until all paths can be compared.
                directory
            })
        })
        .collect();
    let directories: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let paths: std::collections::HashSet<_> = directories.iter().map(|d| d.path()).collect();
    assert_eq!(paths.len(), 8);
    let paths: Vec<_> = paths.into_iter().map(Path::to_owned).collect();
    drop(directories);
    assert!(paths.iter().all(|path| !path.exists()));
}
