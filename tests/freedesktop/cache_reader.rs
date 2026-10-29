//! Container/metadata tests only. No pixel decoder, GNOME factory or cache writer under test.
use super::*;
use std::{
    io::Cursor,
    os::unix::fs::{PermissionsExt, symlink},
    time::Duration,
};

fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut bytes = (data.len() as u32).to_be_bytes().to_vec();
    bytes.extend_from_slice(kind);
    bytes.extend_from_slice(data);
    bytes.extend_from_slice(&(!crc(crc(!0, kind), data)).to_be_bytes());
    bytes
}
fn text_chunk(key: &str, value: &str) -> Vec<u8> {
    chunk(b"tEXt", &[key.as_bytes(), &[0], value.as_bytes()].concat())
}
fn png(fields: &[(&str, String)]) -> Vec<u8> {
    let base = include_bytes!("../fixtures/images/wide.png");
    let mut out = base[..33].to_vec();
    for (key, value) in fields {
        out.extend(text_chunk(key, value));
    }
    out.extend_from_slice(&base[33..]);
    out
}
struct Fixture {
    root: tempfile::TempDir,
    reader: CacheReader,
    source: FileKey,
    cancel: gio::Cancellable,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("photo ç # % ?.png");
        fs::write(&path, include_bytes!("../fixtures/images/wide.png")).unwrap();
        let source = FileKey::read(&path).unwrap();
        let reader = CacheReader::new(root.path().join("cache")).unwrap();
        Self {
            root,
            reader,
            source,
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
    fn fields(&self) -> Vec<(&'static str, String)> {
        let m = fs::metadata(&self.source.path).unwrap();
        vec![
            (
                "Thumb::URI",
                gio::File::for_path(&self.source.path).uri().to_string(),
            ),
            ("Thumb::MTime", m.mtime().to_string()),
            ("Thumb::Size", m.len().to_string()),
        ]
    }
    fn path(&self, class: SizeClass) -> PathBuf {
        self.reader
            .home
            .join("thumbnails")
            .join(class.directory())
            .join(cache_name(&self.source.path))
    }
    fn write(&self, class: SizeClass, bytes: &[u8]) -> PathBuf {
        let path = self.path(class);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, bytes).unwrap();
        path
    }
    fn query(&self) -> Result<CacheOutcome, Error> {
        self.reader.lookup(&self.request(), SizeClass::Normal)
    }
}
fn hit(result: Result<CacheOutcome, Error>) -> Candidate {
    match result.unwrap() {
        CacheOutcome::Hit(c) => c,
        other => panic!("expected Hit, got {other:?}"),
    }
}
fn invalid(result: Result<CacheOutcome, Error>) {
    assert!(matches!(result.unwrap(), CacheOutcome::Invalid(_)));
}

#[test]
fn xdg_absolute_unset_empty_relative_and_nul_fallback() {
    let home = Path::new("/home/test");
    assert_eq!(
        cache_home(Some(OsStr::new("/cache/custom")), home).unwrap(),
        Path::new("/cache/custom")
    );
    for value in [
        None,
        Some(OsStr::new("")),
        Some(OsStr::new("relative")),
        Some(OsStr::new("/cache/\0bad")),
    ] {
        assert_eq!(cache_home(value, home).unwrap(), home.join(".cache"));
    }
    assert!(cache_home(None, Path::new("relative-home")).is_err());
    assert!(CacheReader::new(PathBuf::from("relative")).is_err());
}
#[test]
fn four_classes_choose_smallest_adequate_without_large_fallback() {
    for (edge, class) in [
        (1, SizeClass::Normal),
        (128, SizeClass::Normal),
        (129, SizeClass::Large),
        (256, SizeClass::Large),
        (257, SizeClass::XLarge),
        (512, SizeClass::XLarge),
        (513, SizeClass::XxLarge),
        (1024, SizeClass::XxLarge),
    ] {
        assert_eq!(SizeClass::for_edge(edge), Some(class));
        let f = Fixture::new();
        f.write(class, &png(&f.fields()));
        hit(f.reader.lookup(&f.request(), class));
        if class != SizeClass::Normal {
            assert!(matches!(f.query().unwrap(), CacheOutcome::Missing));
        }
    }
    assert_eq!(SizeClass::for_edge(0), None);
    assert_eq!(SizeClass::for_edge(1025), None);
}
#[test]
fn gio_uri_and_standard_md5_not_source_content_hash() {
    let path = Path::new("/tmp/pivra test#%?.png");
    assert_eq!(
        gio::File::for_path(path).uri(),
        "file:///tmp/pivra%20test%23%25%3F.png"
    );
    assert_eq!(cache_name(path), "9b25b56f1e46c9e06354b6b814e2bde7.png");
    let uri = gio::File::for_path(path).uri();
    assert_eq!(gio::File::for_uri(&uri).path().as_deref(), Some(path));
    assert_eq!(
        cache_name(Path::new("tests/fixtures/images/./wide.png")),
        cache_name(Path::new("tests/fixtures/images/wide.png"))
    );
    let f = Fixture::new();
    let before = cache_name(&f.source.path);
    fs::write(&f.source.path, b"different source content").unwrap();
    assert_eq!(cache_name(&f.source.path), before);
}
#[test]
fn unicode_special_and_non_utf8_names_match_gio() {
    use std::os::unix::ffi::OsStrExt;
    for name in [
        OsStr::new("ç 日本 # % ?.png"),
        OsStr::from_bytes(b"non-utf8-\xff.png"),
    ] {
        let f = Fixture::new();
        let path = f.root.path().join(name);
        fs::copy(&f.source.path, &path).unwrap();
        let f = Fixture {
            source: FileKey::read(&path).unwrap(),
            ..f
        };
        let cpath = f.write(SizeClass::Normal, &png(&f.fields()));
        let c = hit(f.query());
        assert_eq!(c.uri, gio::File::for_path(&path).uri());
        assert_eq!(c.thumbnail.path, cpath);
    }
}
#[test]
fn metadata_png_hit_preserves_bytes_and_original_identity() {
    let f = Fixture::new();
    let bytes = png(&f.fields());
    let path = f.write(SizeClass::Normal, &bytes);
    let candidate = hit(f.query());
    assert_eq!(candidate.png, bytes);
    assert_eq!(candidate.source, f.source);
    assert_eq!(candidate.thumbnail, FileKey::read(&path).unwrap());
    assert!(candidate.still_current(&f.cancel));
}
#[test]
fn missing_does_not_create_directories_or_change_source() {
    let f = Fixture::new();
    assert!(matches!(f.query().unwrap(), CacheOutcome::Missing));
    assert!(!f.reader.home.exists());
    assert_eq!(FileKey::read(&f.source.path).unwrap(), f.source);
}
#[test]
fn optional_size_and_required_fields() {
    let f = Fixture::new();
    let mut fields = f.fields();
    fields.pop();
    f.write(SizeClass::Normal, &png(&fields));
    hit(f.query());
    for omit in ["Thumb::URI", "Thumb::MTime"] {
        let fields: Vec<_> = f.fields().into_iter().filter(|(k, _)| *k != omit).collect();
        f.write(SizeClass::Normal, &png(&fields));
        invalid(f.query());
    }
}
#[test]
fn wrong_uri_invalid_but_wrong_mtime_or_size_stale() {
    let f = Fixture::new();
    for (i, value) in [(0, "file:///wrong"), (1, "0"), (2, "999999")] {
        let mut fields = f.fields();
        fields[i].1 = value.into();
        f.write(SizeClass::Normal, &png(&fields));
        if i == 0 {
            invalid(f.query());
        } else {
            assert!(matches!(f.query().unwrap(), CacheOutcome::Stale));
        }
    }
}
#[test]
fn duplicate_malformed_and_overflow_metadata_rejected() {
    let f = Fixture::new();
    for key in ["Thumb::URI", "Thumb::MTime", "Thumb::Size"] {
        let mut fields = f.fields();
        let value = fields.iter().find(|(k, _)| *k == key).unwrap().1.clone();
        fields.push((key, value));
        f.write(SizeClass::Normal, &png(&fields));
        invalid(f.query());
    }
    for value in ["", "-1", "+1", " 1", "1x", "18446744073709551616"] {
        for i in [1, 2] {
            let mut fields = f.fields();
            fields[i].1 = value.into();
            f.write(SizeClass::Normal, &png(&fields));
            invalid(f.query());
        }
    }
}
#[test]
fn corrupt_crc_truncation_signature_and_trailing_bytes_rejected() {
    let f = Fixture::new();
    let valid = png(&f.fields());
    let mut crc_bad = valid.clone();
    *crc_bad.last_mut().unwrap() ^= 1;
    let mut trailing = valid.clone();
    trailing.push(0);
    for bytes in [
        b"not PNG".to_vec(),
        valid[..20].to_vec(),
        valid[..valid.len() - 1].to_vec(),
        crc_bad,
        trailing,
    ] {
        f.write(SizeClass::Normal, &bytes);
        invalid(f.query());
    }
}
#[test]
fn dimensions_bit_depth_chunk_order_and_unknown_critical_rejected() {
    let f = Fixture::new();
    let base = png(&f.fields());
    for (at, value) in [(3, 129), (7, 0), (8, 3), (9, 1), (10, 1), (11, 1), (12, 2)] {
        let mut ihdr = base[16..29].to_vec();
        ihdr[at] = value;
        let mut bytes = base[..8].to_vec();
        bytes.extend(chunk(b"IHDR", &ihdr));
        bytes.extend_from_slice(&base[33..]);
        f.write(SizeClass::Normal, &bytes);
        invalid(f.query());
    }
    for kind in [b"IHDR", b"ABCD"] {
        let mut bytes = base[..33].to_vec();
        bytes.extend(chunk(kind, b""));
        bytes.extend_from_slice(&base[33..]);
        f.write(SizeClass::Normal, &bytes);
        invalid(f.query());
    }
}
#[test]
fn metadata_and_file_and_chunk_count_bounds() {
    let f = Fixture::new();
    let mut fields = f.fields();
    fields.push(("Other", "x".repeat(MAX_TEXT)));
    f.write(SizeClass::Normal, &png(&fields));
    invalid(f.query());
    let path = f.write(SizeClass::Normal, b"");
    File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_len(MAX_FILE + 1)
        .unwrap();
    invalid(f.query());
    let base = png(&f.fields());
    let mut bytes = base[..33].to_vec();
    for _ in 0..MAX_CHUNKS {
        bytes.extend(chunk(b"teSt", b""));
    }
    bytes.extend_from_slice(&base[33..]);
    f.write(SizeClass::Normal, &bytes);
    invalid(f.query());
}
#[test]
fn conflicting_nonstandard_text_encodings_are_rejected() {
    let f = Fixture::new();
    let base = png(&f.fields());
    for kind in [b"zTXt", b"iTXt"] {
        let mut bytes = base[..33].to_vec();
        bytes.extend(chunk(kind, b"Thumb::URI\0hidden"));
        bytes.extend_from_slice(&base[33..]);
        f.write(SizeClass::Normal, &bytes);
        invalid(f.query());
    }
}
#[test]
fn source_delete_and_replacement_do_not_hit() {
    let f = Fixture::new();
    f.write(SizeClass::Normal, &png(&f.fields()));
    let replacement = f.root.path().join("replacement");
    fs::write(&replacement, b"replaced").unwrap();
    fs::rename(replacement, &f.source.path).unwrap();
    assert!(matches!(f.query().unwrap(), CacheOutcome::Stale));
    fs::remove_file(&f.source.path).unwrap();
    assert!(matches!(
        f.query().unwrap(),
        CacheOutcome::SourceUnavailable(_)
    ));
}
#[test]
fn source_symlink_uses_link_uri_and_cache_symlinks_are_rejected() {
    let f = Fixture::new();
    let link = f.root.path().join("link.png");
    symlink(&f.source.path, &link).unwrap();
    let f = Fixture {
        source: FileKey::read(&link).unwrap(),
        ..f
    };
    let path = f.write(SizeClass::Normal, &png(&f.fields()));
    assert_ne!(
        cache_name(&link),
        cache_name(&fs::canonicalize(&link).unwrap())
    );
    hit(f.query());
    let target = f.root.path().join("cache-target");
    fs::rename(&path, &target).unwrap();
    symlink(&target, &path).unwrap();
    invalid(f.query());
}
#[test]
fn cache_parent_symlink_and_fifo_do_not_follow_or_block() {
    let f = Fixture::new();
    f.write(SizeClass::Normal, &png(&f.fields()));
    let dir = f.path(SizeClass::Normal).parent().unwrap().to_path_buf();
    let moved = f.root.path().join("outside");
    fs::rename(&dir, &moved).unwrap();
    symlink(&moved, &dir).unwrap();
    invalid(f.query());
    fs::remove_file(&dir).unwrap();
    fs::rename(&moved, &dir).unwrap();
    let path = f.path(SizeClass::Normal);
    fs::remove_file(&path).unwrap();
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&path)
            .status()
            .unwrap()
            .success()
    );
    invalid(f.query());
}
#[test]
fn cache_permission_and_directory_access_errors_are_explicit() {
    let f = Fixture::new();
    let path = f.write(SizeClass::Normal, &png(&f.fields()));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    assert!(matches!(f.query(), Err(Error::Io(_))));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let dir = path.parent().unwrap();
    fs::set_permissions(dir, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(matches!(f.query(), Err(Error::Io(_))));
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).unwrap();
}
#[test]
fn valid_failure_markers_are_not_candidates_and_stale_markers_are_ignored() {
    let f = Fixture::new();
    let path = f
        .reader
        .home
        .join("thumbnails/fail/test-producer")
        .join(cache_name(&f.source.path));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, png(&f.fields())).unwrap();
    assert!(matches!(f.query().unwrap(), CacheOutcome::FailedCached));
    let mut fields = f.fields();
    fields[1].1 = "0".into();
    fs::write(&path, png(&fields)).unwrap();
    assert!(matches!(f.query().unwrap(), CacheOutcome::Missing));
    f.write(SizeClass::Normal, &png(&f.fields()));
    hit(f.query());
}
#[test]
fn cancelled_and_timed_out_requests_are_distinct() {
    let f = Fixture::new();
    let mut request = f.request();
    request.deadline = Instant::now();
    assert!(matches!(
        f.reader.lookup(&request, SizeClass::Normal),
        Err(Error::TimedOut)
    ));
    f.cancel.cancel();
    assert!(matches!(
        f.reader.lookup(&request, SizeClass::Normal),
        Err(Error::Cancelled)
    ));
}
struct CancelReader<'a> {
    inner: Cursor<Vec<u8>>,
    cancel: &'a gio::Cancellable,
}
impl Read for CancelReader<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(bytes)?;
        self.cancel.cancel();
        Ok(n)
    }
}
#[test]
fn parser_checks_cancellation_after_io_not_only_before_start() {
    let f = Fixture::new();
    let bytes = png(&f.fields());
    let mut reader = CancelReader {
        inner: Cursor::new(bytes.clone()),
        cancel: &f.cancel,
    };
    assert!(matches!(
        png_metadata(
            &mut reader,
            bytes.len() as u64,
            SizeClass::Normal,
            &f.request()
        ),
        Err(ParseError::Control(Error::Cancelled))
    ));
}
#[test]
fn post_read_source_and_cache_identity_revalidation_rejects_changes() {
    for change_source in [false, true] {
        let f = Fixture::new();
        let path = f.write(SizeClass::Normal, &png(&f.fields()));
        let file = open_cache(&path).unwrap();
        let before = file.metadata().unwrap();
        let key = FileKey::read(&path).unwrap();
        assert!(current(&path, &file, &before, &key, &f.request()));
        let target = if change_source { &f.source.path } else { &path };
        let replacement = f.root.path().join("new");
        fs::write(&replacement, fs::read(target).unwrap()).unwrap();
        fs::rename(replacement, target).unwrap();
        assert!(!current(&path, &file, &before, &key, &f.request()));
    }
}
#[test]
fn successful_and_invalid_reads_leave_contents_and_write_metadata_unchanged() {
    let f = Fixture::new();
    let path = f.write(SizeClass::Normal, &png(&f.fields()));
    let before = FileKey::read(&path).unwrap();
    let bytes = fs::read(&path).unwrap();
    let source = fs::read(&f.source.path).unwrap();
    hit(f.query());
    hit(f.query());
    assert_eq!(FileKey::read(&path).unwrap(), before);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(FileKey::read(&f.source.path).unwrap(), f.source);
    assert_eq!(fs::read(&f.source.path).unwrap(), source);
    let bad = f.write(SizeClass::Normal, b"bad cache");
    let before = FileKey::read(&bad).unwrap();
    invalid(f.query());
    assert_eq!(FileKey::read(&bad).unwrap(), before);
}
#[test]
fn reader_integration_is_routed_through_manager_not_gnome_algorithm() {
    // FD-06 activates the manager; retain the isolation boundary and verify wiring.
    for source in [
        include_str!("../../src/thumbnail.rs"),
        include_str!("../../src/window.rs"),
        include_str!("../../src/image_loader.rs"),
    ] {
        assert!(!source.contains("CacheReader::new"));
    }
    let strip = include_str!("../../src/thumbnail_strip.rs");
    assert!(!strip.contains("CacheReader::new"));
    assert!(strip.contains("Manager::session()"));
    assert!(include_str!("../../src/thumbnail_manager.rs").contains("CacheReader::new"));
}

#[test]
fn cache_older_than_source_ctime_is_stale_even_when_mtime_and_size_match() {
    let f = Fixture::new();
    let path = f.write(SizeClass::Normal, &png(&f.fields()));
    // An old cache can match seconds and size but predates the new source identity.
    let old = std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1);
    File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(old)
        .unwrap();
    assert!(matches!(f.query().unwrap(), CacheOutcome::Stale));
}
#[test]
fn source_unreadable_and_invalid_failure_markers_never_hit() {
    let f = Fixture::new();
    let path = f
        .reader
        .home
        .join("thumbnails/fail/test-producer")
        .join(cache_name(&f.source.path));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"invalid marker").unwrap();
    invalid(f.query());
    fs::set_permissions(&f.source.path, fs::Permissions::from_mode(0o000)).unwrap();
    assert!(matches!(
        f.query().unwrap(),
        CacheOutcome::SourceUnavailable(_)
    ));
    fs::set_permissions(&f.source.path, fs::Permissions::from_mode(0o600)).unwrap();
}
#[test]
fn failure_app_scan_is_bounded() {
    let f = Fixture::new();
    for n in 0..=MAX_FAILURE_APPS {
        fs::create_dir_all(f.reader.home.join(format!("thumbnails/fail/app{n}"))).unwrap();
    }
    invalid(f.query());
}
#[test]
fn cache_substitution_with_a_symlink_fails_post_read_identity_check() {
    let f = Fixture::new();
    let path = f.write(SizeClass::Normal, &png(&f.fields()));
    let file = open_cache(&path).unwrap();
    let before = file.metadata().unwrap();
    let key = FileKey::read(&path).unwrap();
    let moved = f.root.path().join("original-cache");
    fs::rename(&path, &moved).unwrap();
    symlink(&moved, &path).unwrap();
    assert!(!current(&path, &file, &before, &key, &f.request()));
}
struct ErrorAfterCancel<'a>(&'a gio::Cancellable);
impl Read for ErrorAfterCancel<'_> {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        self.0.cancel();
        Err(io::Error::new(io::ErrorKind::Interrupted, "cancelled read"))
    }
}
#[test]
fn cancellation_during_io_error_is_not_misclassified_as_corrupt_cache() {
    let f = Fixture::new();
    let mut reader = ErrorAfterCancel(&f.cancel);
    assert!(matches!(
        png_metadata(&mut reader, 100, SizeClass::Normal, &f.request()),
        Err(ParseError::Control(Error::Cancelled))
    ));
}
