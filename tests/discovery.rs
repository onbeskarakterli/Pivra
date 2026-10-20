mod support;

use pivra::{
    discovery::{ScanReport, scan_directory, scan_neighbors},
    media::MediaKind,
};
use std::os::unix::{
    ffi::OsStringExt,
    fs::{PermissionsExt, symlink},
};
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

fn fixture(directory: &Path, name: impl AsRef<Path>) -> PathBuf {
    let path = directory.join(name);
    fs::write(&path, include_bytes!("fixtures/pixels-2x2.png")).unwrap();
    path
}
fn scan(path: &Path) -> ScanReport {
    scan_directory(path, &AtomicBool::new(false)).unwrap()
}
fn neighbors(path: &Path) -> ScanReport {
    scan_neighbors(path, &AtomicBool::new(false)).unwrap()
}
fn entries(report: &ScanReport) -> HashMap<PathBuf, MediaKind> {
    report
        .collection
        .items()
        .iter()
        .map(|item| (item.path().to_owned(), item.kind()))
        .collect()
}

#[test]
fn single_photo_selects_opened_file_and_preserves_bytes() {
    let dir = support::temp_dir();
    let opened = fixture(dir.path(), "only.png");
    let report = neighbors(&opened);
    assert!(report.issues.is_empty());
    assert_eq!(report.collection.items().len(), 1);
    assert_eq!(report.collection.current().unwrap().path(), opened);
    assert_eq!(
        fs::read(opened).unwrap(),
        include_bytes!("fixtures/pixels-2x2.png")
    );
}

#[test]
fn mixed_case_formats_keep_kinds_and_opened_selection_without_order_assumptions() {
    let dir = support::temp_dir();
    let mut expected = HashMap::new();
    let images = [
        "10.JPG",
        "2.jPeG",
        "seçilen fotoğraf 🦀.png",
        "image.WeBp",
        "image.GIF",
        "image.BMP",
        "image.TiF",
        "image.TIFF",
    ];
    for name in images {
        expected.insert(fixture(dir.path(), name), MediaKind::Image);
    }
    for name in ["clip.MP4", "clip.MKV", "clip.WebM", "clip.MOV", "clip.AVI"] {
        let path = dir.path().join(name);
        fs::write(&path, include_bytes!("fixtures/model-video.txt")).unwrap();
        expected.insert(path, MediaKind::Video);
    }
    let opened = dir.path().join(images[2]);
    let report = neighbors(&opened);
    assert!(report.issues.is_empty());
    assert_eq!(entries(&report), expected);
    assert_eq!(report.collection.current().unwrap().path(), opened);
    assert_eq!(
        report.collection.current().unwrap().kind(),
        MediaKind::Image
    );
    for path in expected.keys() {
        assert!(path.exists());
    }
}

#[test]
fn excludes_unsupported_directories_and_nested_media_without_recursion() {
    let dir = support::temp_dir();
    let opened = fixture(dir.path(), "open.png");
    for name in [
        "notes.txt",
        "no-extension",
        "later.raw",
        "later.svg",
        "later.psd",
    ] {
        fixture(dir.path(), name);
    }
    let sub = dir.path().join("folder.jpg");
    fs::create_dir(&sub).unwrap();
    fixture(&sub, "nested.png");
    symlink(&sub, dir.path().join("directory-link.PNG")).unwrap();
    symlink("/dev/null", dir.path().join("special-file.png")).unwrap();
    assert_eq!(
        entries(&neighbors(&opened)),
        HashMap::from([(opened, MediaKind::Image)])
    );
}

#[test]
fn empty_or_no_candidate_directory_has_no_selection() {
    let dir = support::temp_dir();
    for _ in 0..2 {
        let report = scan(dir.path());
        assert!(report.collection.items().is_empty());
        assert_eq!(report.collection.current(), None);
        assert_eq!(report.collection.selected_index(), None);
        assert!(report.issues.is_empty());
        fixture(dir.path(), "not-media.txt");
    }
}

#[test]
fn nonexistent_directory_and_deleted_opened_file_are_errors() {
    let dir = support::temp_dir();
    let opened = fixture(dir.path(), "deleted.png");
    fs::remove_file(&opened).unwrap();
    assert_eq!(
        scan_neighbors(&opened, &AtomicBool::new(false))
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
    assert_eq!(
        scan_directory(&dir.path().join("missing"), &AtomicBool::new(false))
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
}

#[test]
fn symlink_aliases_remain_distinct_and_scan_the_alias_folder() {
    let folder = support::temp_dir();
    let outside = support::temp_dir();
    let target = fixture(outside.path(), "target.png");
    fixture(outside.path(), "not-a-neighbor.jpg");
    let alias = folder.path().join("açılan alias.png");
    let other = folder.path().join("other-alias.JPG");
    symlink(&target, &alias).unwrap();
    symlink(&target, &other).unwrap();
    let report = neighbors(&alias);
    assert!(report.issues.is_empty());
    assert_eq!(
        entries(&report),
        HashMap::from([(alias.clone(), MediaKind::Image), (other, MediaKind::Image)])
    );
    assert_eq!(report.collection.current().unwrap().path(), alias);
    assert_eq!(
        fs::read(target).unwrap(),
        include_bytes!("fixtures/pixels-2x2.png")
    );
}

#[test]
fn broken_links_are_warnings_not_candidates_or_silent_success() {
    let dir = support::temp_dir();
    let opened = fixture(dir.path(), "open.png");
    let broken = dir.path().join("broken.jpg");
    symlink(dir.path().join("missing"), &broken).unwrap();
    let report = neighbors(&opened);
    assert_eq!(
        entries(&report),
        HashMap::from([(opened, MediaKind::Image)])
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].path, broken);
    assert_eq!(report.issues[0].error.kind(), io::ErrorKind::NotFound);
}

#[test]
fn utf8_and_non_utf8_path_bytes_are_retained() {
    let dir = support::temp_dir();
    let raw = std::ffi::OsString::from_vec(b"photo-\xff.PNG".to_vec());
    let opened = fixture(dir.path(), raw);
    let other = fixture(dir.path(), "fotoğraf #?% boşluk.png");
    let report = neighbors(&opened);
    assert_eq!(
        entries(&report),
        HashMap::from([
            (opened.clone(), MediaKind::Image),
            (other, MediaKind::Image)
        ])
    );
    assert_eq!(report.collection.current().unwrap().path(), opened);
}

#[test]
fn relative_path_and_symlinked_parent_keep_requested_path_identity() {
    let dir = tempfile::Builder::new()
        .prefix("pivra-scan-relative-")
        .tempdir_in(".")
        .unwrap();
    let real = dir.path().join("real");
    fs::create_dir(&real).unwrap();
    fixture(&real, "open.png");
    let alias = dir.path().join("parent-alias");
    symlink("real", &alias).unwrap();
    let requested = alias.join("open.png");
    let report = neighbors(&requested);
    assert_eq!(
        report.collection.current().unwrap().path(),
        std::path::absolute(&requested).unwrap()
    );
    assert!(
        report
            .collection
            .current()
            .unwrap()
            .path()
            .ends_with("parent-alias/open.png")
    );
}

#[test]
fn unreadable_file_warns_and_unreadable_folder_fails_for_unprivileged_user() {
    let dir = support::temp_dir();
    let opened = fixture(dir.path(), "open.png");
    let unreadable = fixture(dir.path(), "private.jpg");
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000)).unwrap();
    let report = neighbors(&opened);
    assert_eq!(
        entries(&report),
        HashMap::from([(opened, MediaKind::Image)])
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].error.kind(),
        io::ErrorKind::PermissionDenied
    );
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o600)).unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o111)).unwrap();
    let result = scan_directory(dir.path(), &AtomicBool::new(false));
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
}

#[test]
fn large_directory_returns_exact_count_and_selected_file() {
    let dir = support::temp_dir();
    for index in 0..1200 {
        fixture(dir.path(), format!("{index}.png"));
    }
    for index in 0..100 {
        fixture(dir.path(), format!("{index}.txt"));
    }
    let opened = dir.path().join("917.png");
    let report = neighbors(&opened);
    assert_eq!(report.collection.items().len(), 1200);
    assert!(report.issues.is_empty());
    assert_eq!(report.collection.current().unwrap().path(), opened);
}

#[test]
fn cancellation_is_an_error_not_a_successful_empty_catalog() {
    let dir = support::temp_dir();
    assert_eq!(
        scan_directory(dir.path(), &AtomicBool::new(true))
            .unwrap_err()
            .kind(),
        io::ErrorKind::Interrupted
    );
}
