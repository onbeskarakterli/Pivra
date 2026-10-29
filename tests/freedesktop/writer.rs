//! Real existing PNG encoder + FD-03 container/metadata reader. No source decoder.
use super::*;
use crate::{
    freedesktop_generator::Metrics, freedesktop_thumbnail::SizeClass,
    thumbnail_provider::CacheOutcome,
};
use std::{os::unix::fs::symlink, time::Duration};
struct Fixture {
    root: tempfile::TempDir,
    source: FileKey,
    cancel: gio::Cancellable,
    writer: CacheWriter,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("photo ç # % ?.png");
        fs::write(&path, include_bytes!("../fixtures/images/wide.png")).unwrap();
        Self {
            source: FileKey::read(&path).unwrap(),
            writer: CacheWriter::new(root.path().join("cache")).unwrap(),
            root,
            cancel: gio::Cancellable::new(),
        }
    }
    fn request(&self) -> Request<'_> {
        Request {
            source: &self.source,
            cancel: &self.cancel,
            deadline: Instant::now() + Duration::from_secs(60),
        }
    }
    fn thumbnail(&self, class: SizeClass) -> GeneratedThumbnail {
        let w = class.max_edge();
        let h = w / 2;
        let mut pixels = Vec::with_capacity(w as usize * h as usize * 4);
        for y in 0..h {
            for x in 0..w {
                pixels.extend_from_slice(&[
                    if x < w / 2 { 255 } else { 0 },
                    0,
                    if y < h / 2 { 0 } else { 255 },
                    if x < w / 2 { 255 } else { 128 },
                ]);
            }
        }
        GeneratedThumbnail {
            source: self.source.clone(),
            class,
            width: w,
            height: h,
            stride: w as usize * 4,
            metrics: Metrics {
                decoded_width: 64,
                decoded_height: 32,
                frame_bytes: 0,
                download_bytes: 0,
                output_bytes: pixels.len(),
                decode_time: Duration::ZERO,
                resize_time: Duration::ZERO,
            },
            pixels: glib::Bytes::from_owned(pixels),
        }
    }
    fn dir(&self, class: SizeClass) -> PathBuf {
        self.writer.home.join("thumbnails").join(class.directory())
    }
    fn target(&self, class: SizeClass) -> PathBuf {
        self.dir(class).join(cache_name(&self.source.path))
    }
    fn query(&self, class: SizeClass) -> CacheOutcome {
        CacheReader::new(self.writer.home.clone())
            .unwrap()
            .lookup(&self.request(), class)
            .unwrap()
    }
    fn no_temp(&self, class: SizeClass) {
        if self.dir(class).exists() {
            assert!(fs::read_dir(self.dir(class)).unwrap().all(|e| {
                !e.unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".pivra-")
            }));
        }
    }
}
#[test]
fn four_classes_writer_to_reader_hit_metadata_identity_alpha_and_modes() {
    let f = Fixture::new();
    let source = fs::read(&f.source.path).unwrap();
    for class in [
        SizeClass::Normal,
        SizeClass::Large,
        SizeClass::XLarge,
        SizeClass::XxLarge,
    ] {
        let t = f.thumbnail(class);
        assert_eq!(
            f.writer.write(&t, &f.request()).unwrap(),
            Publication::Created(f.target(class))
        );
        let CacheOutcome::Hit(c) = f.query(class) else {
            panic!("writer -> reader Hit required");
        };
        assert_eq!(c.source, f.source);
        assert_eq!(c.uri, gio::File::for_path(&f.source.path).uri());
        assert_eq!(c.png[25], 6, "RGBA IHDR (not pixel decode evidence)");
        for field in [
            "Thumb::URI",
            "Thumb::MTime",
            "Thumb::Size",
            "Software",
            "Pivra",
        ] {
            assert!(c.png.windows(field.len()).any(|w| w == field.as_bytes()));
        }
        assert_eq!(fs::metadata(f.target(class)).unwrap().mode() & 0o777, 0o600);
        for path in [
            &f.writer.home,
            &f.writer.home.join("thumbnails"),
            &f.dir(class),
        ] {
            assert_eq!(fs::metadata(path).unwrap().mode() & 0o777, 0o700);
        }
        f.no_temp(class);
    }
    assert_eq!(fs::read(&f.source.path).unwrap(), source);
    assert_eq!(FileKey::read(&f.source.path).unwrap(), f.source);
}
#[test]
fn no_clobber_preserves_valid_and_invalid_foreign_records() {
    let f = Fixture::new();
    let t = f.thumbnail(SizeClass::Normal);
    f.writer.write(&t, &f.request()).unwrap();
    let path = f.target(t.class);
    let key = FileKey::read(&path).unwrap();
    let before = fs::read(&path).unwrap();
    assert!(matches!(
        f.writer.write(&t, &f.request()),
        Ok(Publication::Existing(_))
    ));
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(FileKey::read(&path).unwrap(), key);
    fs::write(&path, b"foreign invalid cache").unwrap();
    let key = FileKey::read(&path).unwrap();
    assert!(matches!(
        f.writer.write(&t, &f.request()),
        Ok(Publication::Existing(_))
    ));
    assert_eq!(FileKey::read(&path).unwrap(), key);
    assert_eq!(fs::read(&path).unwrap(), b"foreign invalid cache");
    f.no_temp(t.class);
}
#[test]
fn source_change_is_stale_to_reader_and_old_pixels_cannot_get_new_metadata() {
    let f = Fixture::new();
    let t = f.thumbnail(SizeClass::Normal);
    f.writer.write(&t, &f.request()).unwrap();
    fs::write(&f.source.path, b"source changed").unwrap();
    assert!(matches!(f.query(t.class), CacheOutcome::Stale));
    let key = FileKey::read(&f.source.path).unwrap();
    let request = Request {
        source: &key,
        ..f.request()
    };
    assert!(matches!(
        f.writer.write(&t, &request),
        Err(Error::SourceChanged)
    ));
}
#[test]
fn missing_source_and_invalid_pixel_geometry_do_not_create_cache() {
    let f = Fixture::new();
    let mut t = f.thumbnail(SizeClass::Normal);
    t.stride = 1;
    assert!(matches!(
        f.writer.write(&t, &f.request()),
        Err(Error::InvalidSource(_))
    ));
    assert!(!f.writer.home.exists());
    t = f.thumbnail(SizeClass::Normal);
    fs::remove_file(&f.source.path).unwrap();
    assert!(matches!(
        f.writer.write(&t, &f.request()),
        Err(Error::SourceUnavailable(_))
    ));
    assert!(!f.writer.home.exists());
}
#[test]
fn cache_parent_and_target_symlinks_never_modify_their_targets() {
    let f = Fixture::new();
    let outside = f.root.path().join("outside");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, &f.writer.home).unwrap();
    let t = f.thumbnail(SizeClass::Normal);
    assert!(f.writer.write(&t, &f.request()).is_err());
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    fs::remove_file(&f.writer.home).unwrap();
    fs::create_dir_all(f.dir(t.class)).unwrap();
    let victim = f.root.path().join("victim");
    fs::write(&victim, b"keep").unwrap();
    symlink(&victim, f.target(t.class)).unwrap();
    assert!(f.writer.write(&t, &f.request()).is_err());
    assert_eq!(fs::read(victim).unwrap(), b"keep");
    f.no_temp(t.class);
}
#[test]
fn temporary_is_private_random_exclusive_and_no_final_visible_until_commit() {
    let f = Fixture::new();
    let t = f.thumbnail(SizeClass::Normal);
    let mut seen = Vec::new();
    f.writer
        .write_with(&t, &f.request(), |stage| {
            if matches!(stage, Stage::Temporary | Stage::Encoded) {
                assert!(!f.target(t.class).exists());
                let files: Vec<_> = fs::read_dir(f.dir(t.class))
                    .unwrap()
                    .map(|e| e.unwrap().path())
                    .collect();
                assert_eq!(files.len(), 1);
                assert_eq!(fs::metadata(&files[0]).unwrap().mode() & 0o777, 0o600);
                seen.push(files[0].clone());
            }
        })
        .unwrap();
    assert_eq!(seen[0], seen[1]);
    assert!(seen[0].file_name().unwrap().to_string_lossy().len() > 40);
    assert!(!seen[0].exists());
    f.no_temp(t.class);
}
#[test]
fn cancellation_after_encoding_cleans_only_own_temporary_file() {
    let f = Fixture::new();
    let t = f.thumbnail(SizeClass::Normal);
    let foreign = f.root.path().join("foreign");
    fs::write(&foreign, b"preserve").unwrap();
    let out = f.writer.write_with(&t, &f.request(), |s| {
        if matches!(s, Stage::Encoded) {
            f.cancel.cancel();
        }
    });
    assert!(matches!(out, Err(Error::Cancelled)));
    assert!(!f.target(t.class).exists());
    f.no_temp(t.class);
    assert_eq!(fs::read(foreign).unwrap(), b"preserve");
}
#[test]
fn expired_and_cancelled_deadlines_are_distinct_and_no_dirs_created() {
    let f = Fixture::new();
    let t = f.thumbnail(SizeClass::Normal);
    let request = Request {
        deadline: Instant::now(),
        ..f.request()
    };
    assert!(matches!(f.writer.write(&t, &request), Err(Error::TimedOut)));
    f.cancel.cancel();
    assert!(matches!(
        f.writer.write(&t, &request),
        Err(Error::Cancelled)
    ));
    assert!(!f.writer.home.exists());
}
#[test]
fn cancellation_after_commit_does_not_remove_or_invalidate_valid_record() {
    let f = Fixture::new();
    let t = f.thumbnail(SizeClass::Normal);
    assert!(matches!(
        f.writer.write_with(&t, &f.request(), |s| {
            if matches!(s, Stage::Published) {
                f.cancel.cancel();
            }
        }),
        Ok(Publication::Created(_))
    ));
    let cancel = gio::Cancellable::new();
    let req = Request {
        cancel: &cancel,
        ..f.request()
    };
    assert!(matches!(
        CacheReader::new(f.writer.home.clone())
            .unwrap()
            .lookup(&req, t.class)
            .unwrap(),
        CacheOutcome::Hit(_)
    ));
    f.no_temp(t.class);
}
#[test]
fn source_replace_or_delete_before_publication_rejects_old_pixels_and_cleans_temp() {
    for delete in [false, true] {
        let f = Fixture::new();
        let t = f.thumbnail(SizeClass::Normal);
        let result = f.writer.write_with(&t, &f.request(), |s| {
            if matches!(s, Stage::BeforePublish) {
                if delete {
                    fs::remove_file(&f.source.path).unwrap();
                } else {
                    let p = f.root.path().join("replacement");
                    fs::write(&p, b"replacement").unwrap();
                    fs::rename(p, &f.source.path).unwrap();
                }
            }
        });
        if delete {
            assert!(matches!(result, Err(Error::SourceUnavailable(_))));
        } else {
            assert!(matches!(result, Err(Error::SourceChanged)));
        }
        assert!(!f.target(t.class).exists());
        f.no_temp(t.class);
    }
}
#[test]
fn directory_replacement_before_commit_fails_without_publishing_in_moved_directory() {
    let f = Fixture::new();
    let t = f.thumbnail(SizeClass::Normal);
    let moved = f.root.path().join("moved");
    let result = f.writer.write_with(&t, &f.request(), |s| {
        if matches!(s, Stage::BeforePublish) {
            fs::rename(f.dir(t.class), &moved).unwrap();
            fs::create_dir(f.dir(t.class)).unwrap();
        }
    });
    assert!(result.is_err());
    assert_eq!(fs::read_dir(moved).unwrap().count(), 0);
    assert!(!f.target(t.class).exists());
}
#[test]
fn target_appearing_at_commit_is_preserved_no_clobber() {
    let f = Fixture::new();
    let t = f.thumbnail(SizeClass::Normal);
    let result = f
        .writer
        .write_with(&t, &f.request(), |s| {
            if matches!(s, Stage::BeforePublish) {
                fs::write(f.target(t.class), b"other producer wins").unwrap();
            }
        })
        .unwrap();
    assert!(matches!(result, Publication::Existing(_)));
    assert_eq!(fs::read(f.target(t.class)).unwrap(), b"other producer wins");
    f.no_temp(t.class);
}
#[test]
fn read_only_directory_is_rejected_without_permission_changes() {
    let f = Fixture::new();
    fs::create_dir(&f.writer.home).unwrap();
    fs::set_permissions(&f.writer.home, fs::Permissions::from_mode(0o500)).unwrap();
    let t = f.thumbnail(SizeClass::Normal);
    assert!(matches!(
        f.writer.write(&t, &f.request()),
        Err(Error::Io(_))
    ));
    assert_eq!(fs::metadata(&f.writer.home).unwrap().mode() & 0o777, 0o500);
    fs::set_permissions(&f.writer.home, fs::Permissions::from_mode(0o700)).unwrap();
}
#[test]
fn simultaneous_writers_publish_one_complete_record_and_leave_no_temps() {
    use std::sync::mpsc::channel;
    let f = Fixture::new();
    let a = f.thumbnail(SizeClass::Normal);
    let b = f.thumbnail(SizeClass::Normal);
    let (started, observed) = channel();
    let (release_a, wait_a) = channel();
    let (release_b, wait_b) = channel();
    let results = std::thread::scope(|scope| {
        let f = &f;
        let start_a = started.clone();
        let one = scope.spawn(move || {
            f.writer.write_with(&a, &f.request(), |s| {
                if matches!(s, Stage::BeforePublish) {
                    start_a.send(()).unwrap();
                    wait_a.recv_timeout(Duration::from_secs(3)).unwrap();
                }
            })
        });
        let two = scope.spawn(move || {
            f.writer.write_with(&b, &f.request(), |s| {
                if matches!(s, Stage::BeforePublish) {
                    started.send(()).unwrap();
                    wait_b.recv_timeout(Duration::from_secs(3)).unwrap();
                }
            })
        });
        observed.recv_timeout(Duration::from_secs(3)).unwrap();
        observed.recv_timeout(Duration::from_secs(3)).unwrap();
        assert!(matches!(f.query(SizeClass::Normal), CacheOutcome::Missing));
        release_a.send(()).unwrap();
        release_b.send(()).unwrap();
        [one.join().unwrap().unwrap(), two.join().unwrap().unwrap()]
    });
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(r, Publication::Created(_)))
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(r, Publication::Existing(_)))
            .count(),
        1
    );
    assert!(matches!(f.query(SizeClass::Normal), CacheOutcome::Hit(_)));
    f.no_temp(SizeClass::Normal);
}
#[test]
fn stream_limit_and_midstream_cancel_do_not_allocate_or_keep_partial_output() {
    let f = Fixture::new();
    let directory =
        create_directory(&f.dir(SizeClass::Normal), &f.writer.home, &f.request()).unwrap();
    let temp = Temporary::new(directory).unwrap();
    let name = fd_path(&temp.directory).join(&temp.name);
    let mut sink = Sink {
        file: temp.file.try_clone().unwrap(),
        source: f.source.clone(),
        cancel: f.cancel.clone(),
        deadline: f.request().deadline,
        written: 0,
        failure: None,
    };
    sink.write_all(b"partial").unwrap();
    f.cancel.cancel();
    assert!(sink.write_all(b"later").is_err());
    assert!(matches!(sink.failure, Some(Error::Cancelled)));
    drop(sink);
    drop(temp);
    assert!(!name.exists());
    f.no_temp(SizeClass::Normal);
}
#[test]
fn oversized_encoded_png_is_bounded_and_not_published() {
    let f = Fixture::new();
    let mut t = f.thumbnail(SizeClass::XxLarge);
    t.height = 1024;
    t.stride = 4096;
    let mut data = vec![0; 4 * 1024 * 1024];
    let mut value = 17u32;
    for byte in &mut data {
        value ^= value << 13;
        value ^= value >> 17;
        value ^= value << 5;
        *byte = value as u8;
    }
    t.pixels = glib::Bytes::from_owned(data);
    assert!(matches!(
        f.writer.write(&t, &f.request()),
        Err(Error::LimitExceeded(_))
    ));
    assert!(!f.target(t.class).exists());
    f.no_temp(t.class);
}
#[test]
fn xdg_invalid_empty_and_traversal_follow_shared_rules_or_fail_closed() {
    let f = Fixture::new();
    for x in [
        None,
        Some(std::ffi::OsStr::new("")),
        Some(std::ffi::OsStr::new("relative")),
    ] {
        let root = cache_home(x, f.root.path()).unwrap();
        assert_eq!(root, f.root.path().join(".cache"));
        CacheWriter::new(root).unwrap();
    }
    assert!(CacheWriter::new(PathBuf::from("relative")).is_err());
    assert!(CacheWriter::new(f.root.path().join("../escape")).is_err());
}
#[test]
fn writer_drops_all_its_file_descriptors() {
    let f = Fixture::new();
    let t = f.thumbnail(SizeClass::Normal);
    f.writer.write(&t, &f.request()).unwrap();
    for entry in fs::read_dir("/proc/self/fd").unwrap() {
        if let Ok(path) = fs::read_link(entry.unwrap().path()) {
            assert!(!path.starts_with(f.root.path()), "leaked fd {path:?}");
        }
    }
}
#[test]
fn reader_rejects_damaged_metadata_and_writer_does_not_silently_overwrite() {
    let f = Fixture::new();
    let t = f.thumbnail(SizeClass::Normal);
    f.writer.write(&t, &f.request()).unwrap();
    let path = f.target(t.class);
    let mut bytes = fs::read(&path).unwrap();
    let at = bytes.windows(10).position(|s| s == b"Thumb::URI").unwrap();
    bytes[at] ^= 1;
    fs::write(&path, &bytes).unwrap();
    assert!(matches!(f.query(t.class), CacheOutcome::Invalid(_)));
    assert!(matches!(
        f.writer.write(&t, &f.request()),
        Ok(Publication::Existing(_))
    ));
    assert_eq!(fs::read(path).unwrap(), bytes);
}

#[test]
fn temporary_replacement_is_not_published_or_deleted_as_ours() {
    let f = Fixture::new();
    let t = f.thumbnail(SizeClass::Normal);
    let mut replaced = None;
    let result = f.writer.write_with(&t, &f.request(), |s| {
        if matches!(s, Stage::BeforePublish) {
            let p = fs::read_dir(f.dir(t.class))
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            fs::rename(&p, f.root.path().join("moved-by-other-actor")).unwrap();
            fs::write(&p, b"foreign replacement").unwrap();
            replaced = Some(p);
        }
    });
    assert!(result.is_err());
    assert_eq!(fs::read(replaced.unwrap()).unwrap(), b"foreign replacement");
    assert!(!f.target(t.class).exists());
}
#[test]
fn modified_temporary_contents_or_permissions_are_rejected() {
    for permissions in [false, true] {
        let f = Fixture::new();
        let t = f.thumbnail(SizeClass::Normal);
        let result = f.writer.write_with(&t, &f.request(), |s| {
            if matches!(s, Stage::BeforePublish) {
                let p = fs::read_dir(f.dir(t.class))
                    .unwrap()
                    .next()
                    .unwrap()
                    .unwrap()
                    .path();
                if permissions {
                    fs::set_permissions(p, fs::Permissions::from_mode(0o666)).unwrap();
                } else {
                    fs::write(p, b"tampered").unwrap();
                }
            }
        });
        assert!(result.is_err());
        assert!(!f.target(t.class).exists());
        f.no_temp(t.class);
    }
}
#[test]
fn midstream_timeout_is_distinct_and_cleans_partial_file() {
    let f = Fixture::new();
    let dir = create_directory(&f.dir(SizeClass::Normal), &f.writer.home, &f.request()).unwrap();
    let temp = Temporary::new(dir).unwrap();
    let mut sink = Sink {
        file: temp.file.try_clone().unwrap(),
        source: f.source.clone(),
        cancel: f.cancel.clone(),
        deadline: f.request().deadline,
        written: 0,
        failure: None,
    };
    sink.write_all(b"first block").unwrap();
    sink.deadline = Instant::now();
    assert!(sink.write_all(b"second block").is_err());
    assert!(matches!(sink.failure, Some(Error::TimedOut)));
    drop(sink);
    drop(temp);
    f.no_temp(SizeClass::Normal);
}
#[test]
fn source_symlink_uses_gio_link_uri_and_unrelated_cache_file_is_preserved() {
    let mut f = Fixture::new();
    let original = f.source.clone();
    let link = f.root.path().join("source-link.png");
    symlink(&original.path, &link).unwrap();
    f.source = FileKey::read(&link).unwrap();
    let t = f.thumbnail(SizeClass::Normal);
    create_directory(&f.dir(t.class), &f.writer.home, &f.request()).unwrap();
    let foreign = f.dir(t.class).join("another-application.png");
    fs::write(&foreign, b"preserved foreign cache").unwrap();
    let key = FileKey::read(&foreign).unwrap();
    f.writer.write(&t, &f.request()).unwrap();
    let CacheOutcome::Hit(c) = f.query(t.class) else {
        panic!("link URI Hit");
    };
    assert_eq!(c.uri, gio::File::for_path(link).uri());
    assert_eq!(FileKey::read(&foreign).unwrap(), key);
    assert_eq!(fs::read(foreign).unwrap(), b"preserved foreign cache");
    assert_eq!(FileKey::read(&original.path).unwrap(), original);
}

#[test]
fn gnome_path_and_generator_remain_unwired_to_writer() {
    for text in [
        include_str!("../../src/thumbnail.rs"),
        include_str!("../../src/thumbnail_strip.rs"),
        include_str!("../../src/window.rs"),
        include_str!("../../src/freedesktop_generator.rs"),
    ] {
        assert!(!text.contains("freedesktop_writer"));
    }
}
