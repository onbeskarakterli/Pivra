//! Negative evidence for a proposed protocol, NOT a production replacement implementation.
use pivra::image_cache::FileKey;
use std::fs;
#[test]
fn stat_then_rename_overwrites_a_newer_producer_even_with_an_old_identity_snapshot() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("cache.png");
    let pending = root.path().join("pending.png");
    fs::write(&target, b"stale").unwrap();
    fs::write(&pending, b"our replacement").unwrap();
    let observed = FileKey::read(&target).unwrap();
    let competitor = root.path().join("other.png");
    fs::write(&competitor, b"new valid record").unwrap();
    fs::rename(&competitor, &target).unwrap();
    assert_ne!(FileKey::read(&target).unwrap(), observed);
    // The last validation could have occurred immediately before the competitor.
    // rename has no expected-inode argument: it succeeds and destroys that new record.
    fs::rename(&pending, &target).unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"our replacement");
}
#[test]
fn a_cooperating_lock_cannot_protect_against_an_uncooperative_standard_cache_producer() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("cache.png");
    let _lock = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.path().join("lock"))
        .unwrap();
    fs::write(&target, b"old").unwrap();
    let competitor = root.path().join("other.png");
    fs::write(&competitor, b"new valid record").unwrap();
    fs::rename(&competitor, &target).unwrap();
    assert_eq!(fs::read(target).unwrap(), b"new valid record");
}
