//! Real manager/writer/reader, controlled already-decoded RGBA only. No native decode PASS.
use super::*;
use crate::{freedesktop_generator::Metrics, thumbnail_provider::Capability};
use gtk::glib;
use std::{fs, os::unix::fs::PermissionsExt};
struct Fixture {
    _lock: std::sync::MutexGuard<'static, ()>,
    _root: tempfile::TempDir,
    source: FileKey,
    cancel: gio::Cancellable,
    manager: Manager,
}
impl Fixture {
    fn new() -> Self {
        let lock = TEST_LOCK.lock().unwrap();
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("source.png");
        fs::write(&path, include_bytes!("../fixtures/images/wide.png")).unwrap();
        Self {
            _lock: lock,
            source: FileKey::read(&path).unwrap(),
            cancel: gio::Cancellable::new(),
            manager: Manager::new(false, root.path().join("cache")),
            _root: root,
        }
    }
    fn request(&self) -> Request<'_> {
        Request {
            source: &self.source,
            cancel: &self.cancel,
            deadline: Instant::now() + Duration::from_secs(60),
        }
    }
    fn reader(&self) -> CacheReader {
        CacheReader::new(self.manager.home.clone()).unwrap()
    }
    fn pixels(&self) -> GeneratedThumbnail {
        GeneratedThumbnail {
            source: self.source.clone(),
            class: SizeClass::Normal,
            width: 4,
            height: 2,
            stride: 16,
            pixels: glib::Bytes::from_owned(vec![128u8; 32]),
            metrics: Metrics {
                decoded_width: 4,
                decoded_height: 2,
                frame_bytes: 32,
                download_bytes: 32,
                output_bytes: 32,
                decode_time: Duration::ZERO,
                resize_time: Duration::ZERO,
            },
        }
    }
    fn target(&self) -> PathBuf {
        self.manager
            .home
            .join("thumbnails/normal")
            .join(crate::freedesktop_thumbnail::cache_name(&self.source.path))
    }
}
#[test]
fn session_gnome_priority_includes_cache_only_and_transient_generation_loss() {
    let _lock = TEST_LOCK.lock().unwrap();
    for cap in [
        Capability::CacheOnly,
        Capability::CacheAndGenerate,
        Capability::TemporarilyFailed,
    ] {
        assert_eq!(
            Manager::from_gnome_snapshot(true, cap, PathBuf::from("/tmp")).id(),
            ProviderId::Gnome
        );
    }
    assert_eq!(
        Manager::from_gnome_snapshot(true, Capability::Unavailable, PathBuf::from("/tmp")).id(),
        ProviderId::Freedesktop
    );
    assert_eq!(
        Manager::new(false, PathBuf::from("/tmp")).id(),
        ProviderId::Freedesktop
    );
}
#[test]
fn created_record_is_revalidated_and_keeps_memory_backup() {
    let f = Fixture::new();
    let resolved = f
        .manager
        .finish(f.pixels(), &f.request(), &f.reader(), true)
        .unwrap();
    let Resolved::Published(candidate, memory) = resolved else {
        panic!("verified publication with backup");
    };
    assert!(candidate.still_current(&f.cancel));
    assert_eq!(memory.source, f.source);
    assert_eq!(memory.pixels.len(), 32);
}
#[test]
fn existing_invalid_record_cannot_hide_fresh_memory() {
    let f = Fixture::new();
    fs::create_dir_all(f.target().parent().unwrap()).unwrap();
    fs::write(f.target(), b"foreign invalid").unwrap();
    let result = f
        .manager
        .finish(f.pixels(), &f.request(), &f.reader(), true)
        .unwrap();
    assert!(matches!(result, Resolved::Memory(_)));
    assert_eq!(fs::read(f.target()).unwrap(), b"foreign invalid");
}
#[test]
fn stale_and_invalid_records_are_never_written_by_memory_resolution() {
    let f = Fixture::new();
    for data in [b"stale cache".as_slice(), b"invalid PNG"] {
        fs::create_dir_all(f.target().parent().unwrap()).unwrap();
        fs::write(f.target(), data).unwrap();
        let before = FileKey::read(&f.target()).unwrap();
        assert!(matches!(
            f.manager
                .finish(f.pixels(), &f.request(), &f.reader(), false)
                .unwrap(),
            Resolved::Memory(_)
        ));
        assert_eq!(FileKey::read(&f.target()).unwrap(), before);
        assert_eq!(fs::read(f.target()).unwrap(), data);
    }
}
#[test]
fn read_only_writer_failure_preserves_good_memory() {
    let f = Fixture::new();
    fs::create_dir(&f.manager.home).unwrap();
    fs::set_permissions(&f.manager.home, fs::Permissions::from_mode(0o500)).unwrap();
    assert!(matches!(
        f.manager
            .finish(f.pixels(), &f.request(), &f.reader(), true)
            .unwrap(),
        Resolved::Memory(_)
    ));
    fs::set_permissions(&f.manager.home, fs::Permissions::from_mode(0o700)).unwrap();
}
#[test]
fn source_change_cancel_and_deadline_prevent_memory_publication() {
    let f = Fixture::new();
    let mut r = f.request();
    r.deadline = Instant::now();
    assert!(matches!(
        f.manager.finish(f.pixels(), &r, &f.reader(), false),
        Err(Error::TimedOut)
    ));
    f.cancel.cancel();
    assert!(matches!(
        f.manager.finish(f.pixels(), &r, &f.reader(), false),
        Err(Error::Cancelled)
    ));
    let fresh = gio::Cancellable::new();
    let r = Request {
        cancel: &fresh,
        ..f.request()
    };
    fs::write(&f.source.path, b"replacement").unwrap();
    assert!(matches!(
        f.manager.finish(f.pixels(), &r, &f.reader(), false),
        Err(Error::SourceChanged)
    ));
}
#[test]
fn shared_permit_lives_through_cache_decode_response_scope() {
    let f = Fixture::new();
    f.manager
        .finish(f.pixels(), &f.request(), &f.reader(), true)
        .unwrap();
    let response = f.manager.resolve(&f.source.path, &f.cancel).unwrap();
    assert!(response.candidate().is_some());
    assert_eq!(Permit::active_count(), 1);
    let other = f.source.path.with_extension("other");
    let second = Permit::take(&other).unwrap();
    assert!(Permit::take(&other.with_extension("third")).is_none());
    drop(second);
    drop(response);
    assert_eq!(Permit::active_count(), 0);
}
#[test]
fn file_generation_failure_never_switches_the_session() {
    let f = Fixture::new();
    fs::write(&f.source.path, b"not PNG").unwrap();
    assert!(matches!(
        f.manager.resolve(&f.source.path, &f.cancel),
        Err(Error::InvalidSource(_))
    ));
    assert_eq!(f.manager.id(), ProviderId::Freedesktop);
    assert_eq!(Permit::active_count(), 0);
}
#[test]
fn large_job_backoff_does_not_repeat_decoder_discovery_or_change_provider() {
    let f = Fixture::new();
    f.manager.deferred.lock().unwrap().push(f.source.clone());
    assert!(f.manager.is_deferred(&f.source, true));
    assert_eq!(f.manager.id(), ProviderId::Freedesktop);
    std::fs::write(&f.source.path, b"new identity").unwrap();
    let key = FileKey::read(&f.source.path).unwrap();
    assert!(!f.manager.is_deferred(&key, true));
    assert!(!f.manager.is_deferred(&f.source, false));
    assert!(f.manager.deferred.lock().unwrap().is_empty());
}
