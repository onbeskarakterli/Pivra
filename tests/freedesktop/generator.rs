//! Controlled lifecycle/resampling only. Synthetic textures are NOT real Glycin evidence.
use super::*;
use std::{
    cell::Cell,
    fs,
    os::unix::fs::symlink,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
static SERIAL: Mutex<()> = Mutex::new(());
struct Fixture {
    _serial: std::sync::MutexGuard<'static, ()>,
    root: tempfile::TempDir,
    source: FileKey,
    cancel: gio::Cancellable,
}
impl Fixture {
    fn new() -> Self {
        let serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("test ç #?.png");
        fs::write(&path, include_bytes!("../fixtures/images/wide.png")).unwrap();
        Self {
            _serial: serial,
            source: FileKey::read(&path).unwrap(),
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
}
#[derive(Clone, Copy)]
enum Action {
    Ready,
    Failed,
    Delete,
    Replace,
    Cancel,
    Pending,
}
struct Fake {
    width: u32,
    height: u32,
    action: Action,
    parent: gio::Cancellable,
    calls: Cell<usize>,
    stopped: Arc<AtomicBool>,
    frame_bytes: usize,
}
impl Fake {
    fn new(f: &Fixture, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            action: Action::Ready,
            parent: f.cancel.clone(),
            calls: Cell::new(0),
            stopped: Arc::new(AtomicBool::new(false)),
            frame_bytes: width as usize * height as usize * 4,
        }
    }
}
struct Stopped(Arc<AtomicBool>);
impl Drop for Stopped {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}
impl Decoder for Fake {
    fn decode<'a>(&'a self, path: &'a Path, cancel: &'a gio::Cancellable) -> DecodeFuture<'a> {
        Box::pin(async move {
            self.calls.set(self.calls.get() + 1);
            let _stopped = Stopped(self.stopped.clone());
            match self.action {
                Action::Ready => {}
                Action::Failed => {
                    return Err(Error::GenerationFailed(
                        "controlled decoder failure; no Glycin".into(),
                    ));
                }
                Action::Delete => fs::remove_file(path).unwrap(),
                Action::Replace => {
                    let replacement = path.with_extension("replacement");
                    fs::write(&replacement, fs::read(path).unwrap()).unwrap();
                    fs::rename(replacement, path).unwrap();
                }
                Action::Cancel => self.parent.cancel(),
                Action::Pending => {
                    cancel.future().await;
                    return Err(Error::GenerationFailed(
                        "controlled decoder acknowledged cancellation".into(),
                    ));
                }
            }
            let mut pixels = Vec::new();
            for _ in 0..self.width as usize * self.height as usize {
                pixels.extend_from_slice(&[20, 80, 160, 128]);
            }
            let bytes = glib::Bytes::from_owned(pixels);
            let texture = gdk::MemoryTexture::new(
                self.width as i32,
                self.height as i32,
                gdk::MemoryFormat::R8g8b8a8,
                &bytes,
                self.width as usize * 4,
            )
            .upcast();
            Ok(Decoded {
                texture,
                frame_bytes: self.frame_bytes,
            })
        })
    }
}
#[test]
fn all_classes_landscape_portrait_square_tiny_no_upscale() {
    for class in [
        SizeClass::Normal,
        SizeClass::Large,
        SizeClass::XLarge,
        SizeClass::XxLarge,
    ] {
        let edge = class.max_edge();
        assert_eq!(
            fitted_size(1600, 1200, class).unwrap(),
            (edge, edge * 3 / 4)
        );
        assert_eq!(
            fitted_size(1200, 1600, class).unwrap(),
            (edge * 3 / 4, edge)
        );
        assert_eq!(fitted_size(1600, 1600, class).unwrap(), (edge, edge));
        assert_eq!(fitted_size(4, 2, class).unwrap(), (4, 2));
        assert_eq!(fitted_size(1, 1, class).unwrap(), (1, 1));
    }
}
#[test]
fn zero_overflow_edges_and_large_decode_budget_are_rejected() {
    for (w, h) in [
        (0, 1),
        (1, 0),
        (u32::MAX, u32::MAX),
        (32769, 1),
        (8000, 8000),
        (10000, 10800),
    ] {
        assert!(matches!(
            fitted_size(w, h, SizeClass::Normal),
            Err(Error::LimitExceeded(_))
        ));
    }
    assert_eq!(fitted_size(32768, 1, SizeClass::Normal).unwrap(), (128, 1));
}
#[test]
fn integer_rounding_preserves_ratio_within_one_pixel_and_never_crops() {
    for (w, h) in [
        (333, 777),
        (777, 333),
        (32768, 1),
        (1, 32768),
        (128, 127),
        (17, 19),
    ] {
        let (x, y) = fitted_size(w, h, SizeClass::Normal).unwrap();
        assert!(x > 0 && y > 0 && x <= 128 && y <= 128 && x <= w && y <= h);
        let ratio = (128f64 / w as f64).min(128f64 / h as f64).min(1.);
        assert!((x as f64 - w as f64 * ratio).abs() <= 1.);
        assert!((y as f64 - h as f64 * ratio).abs() <= 1.);
    }
}
#[test]
fn controlled_real_gdk_resampling_rgba_alpha_and_tight_output() {
    let f = Fixture::new();
    for (w, h) in [(1600, 1200), (1200, 1600), (400, 400), (4, 2)] {
        let fake = Fake::new(&f, w, h);
        for class in [
            SizeClass::Normal,
            SizeClass::Large,
            SizeClass::XLarge,
            SizeClass::XxLarge,
        ] {
            let out = generate_with(&f.request(), class, &fake).unwrap();
            assert_eq!((out.width, out.height), fitted_size(w, h, class).unwrap());
            assert_eq!(out.stride, out.width as usize * 4);
            assert_eq!(out.pixels.len(), out.stride * out.height as usize);
            assert!(out.pixels.len() <= class.max_edge() as usize * class.max_edge() as usize * 4);
            assert_eq!(out.source, f.source);
            assert_eq!(out.class, class);
            for actual in out.pixels.chunks_exact(4) {
                for (a, b) in actual.iter().zip([20u8, 80, 160, 128]) {
                    assert!(a.abs_diff(b) <= 1, "alpha/RGBA normalization: {actual:?}");
                }
            }
            assert_eq!(out.metrics.decoded_width, w);
            assert_eq!(out.metrics.decoded_height, h);
            assert_eq!(out.metrics.output_bytes, out.pixels.len());
        }
    }
}
#[test]
fn missing_invalid_and_sparse_oversized_sources_do_not_invoke_decoder() {
    let f = Fixture::new();
    let fake = Fake::new(&f, 4, 2);
    fs::remove_file(&f.source.path).unwrap();
    assert!(matches!(
        generate_with(&f.request(), SizeClass::Normal, &fake),
        Err(Error::SourceUnavailable(_))
    ));
    for bytes in [b"plain text".as_slice(), b"\x89PNG", b"\xff\xd8"] {
        fs::write(&f.source.path, bytes).unwrap();
        let key = FileKey::read(&f.source.path).unwrap();
        let request = Request {
            source: &key,
            ..f.request()
        };
        assert!(matches!(
            generate_with(&request, SizeClass::Normal, &fake),
            Err(Error::InvalidSource(_))
        ));
    }
    FileForTest::oversized(&f.source.path);
    let key = FileKey::read(&f.source.path).unwrap();
    let request = Request {
        source: &key,
        ..f.request()
    };
    assert!(matches!(
        generate_with(&request, SizeClass::Normal, &fake),
        Err(Error::LimitExceeded(_))
    ));
    assert_eq!(fake.calls.get(), 0);
}
struct FileForTest;
impl FileForTest {
    fn oversized(path: &Path) {
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_len(MAX_FILE + 1)
            .unwrap();
    }
}
#[test]
fn replace_delete_or_cancel_during_controlled_decode_never_publishes() {
    let f = Fixture::new();
    for action in [Action::Replace, Action::Delete, Action::Cancel] {
        fs::write(
            &f.source.path,
            include_bytes!("../fixtures/images/wide.png"),
        )
        .unwrap();
        let key = FileKey::read(&f.source.path).unwrap();
        let request = Request {
            source: &key,
            ..f.request()
        };
        let mut fake = Fake::new(&f, 4, 2);
        fake.action = action;
        let result = generate_with(&request, SizeClass::Normal, &fake);
        match action {
            Action::Replace => assert!(matches!(result, Err(Error::SourceChanged))),
            Action::Delete => assert!(matches!(result, Err(Error::SourceUnavailable(_)))),
            Action::Cancel => assert!(matches!(result, Err(Error::Cancelled))),
            _ => unreachable!(),
        }
        assert!(fake.stopped.load(Ordering::SeqCst));
    }
}
#[test]
fn errors_release_admission_without_changing_provider_or_source() {
    let f = Fixture::new();
    let mut fake = Fake::new(&f, 4, 2);
    fake.action = Action::Failed;
    for _ in 0..3 {
        assert!(matches!(
            generate_with(&f.request(), SizeClass::Normal, &fake),
            Err(Error::GenerationFailed(_))
        ));
    }
    fake.action = Action::Ready;
    generate_with(&f.request(), SizeClass::Normal, &fake).unwrap();
    assert_eq!(FileKey::read(&f.source.path).unwrap(), f.source);
}
#[test]
fn deadline_cancels_pending_child_and_cleanup_releases_admission() {
    let f = Fixture::new();
    let mut fake = Fake::new(&f, 4, 2);
    fake.action = Action::Pending;
    let request = Request {
        deadline: Instant::now() + Duration::from_millis(100),
        ..f.request()
    };
    assert!(matches!(
        generate_with(&request, SizeClass::Normal, &fake),
        Err(Error::TimedOut)
    ));
    assert_eq!(fake.calls.get(), 1);
    assert!(fake.stopped.load(Ordering::SeqCst));
    assert!(!f.cancel.is_cancelled());
    fake.action = Action::Ready;
    generate_with(&f.request(), SizeClass::Normal, &fake).unwrap();
}
#[test]
fn cancelled_request_wins_over_expired_deadline_before_admission() {
    let f = Fixture::new();
    let fake = Fake::new(&f, 4, 2);
    let request = Request {
        deadline: Instant::now(),
        ..f.request()
    };
    assert!(matches!(
        generate_with(&request, SizeClass::Normal, &fake),
        Err(Error::TimedOut)
    ));
    f.cancel.cancel();
    assert!(matches!(
        generate_with(&request, SizeClass::Normal, &fake),
        Err(Error::Cancelled)
    ));
    assert_eq!(fake.calls.get(), 0);
}
#[test]
fn duplicate_hardlink_and_two_active_admission_bounds_are_fail_fast() {
    let f = Fixture::new();
    let m = fs::metadata(&f.source.path).unwrap();
    let identity = (m.dev(), m.ino());
    let a = Admission::acquire(identity).unwrap();
    assert!(matches!(Admission::acquire(identity), Err(Error::Busy)));
    let link = f.root.path().join("hardlink.png");
    fs::hard_link(&f.source.path, &link).unwrap();
    let m = fs::metadata(link).unwrap();
    assert!(matches!(
        Admission::acquire((m.dev(), m.ino())),
        Err(Error::Busy)
    ));
    let b = Admission::acquire((identity.0, identity.1 + 1)).unwrap();
    assert!(matches!(
        Admission::acquire((identity.0, identity.1 + 2)),
        Err(Error::Busy)
    ));
    drop(a);
    drop(b);
    Admission::acquire(identity).unwrap();
}
#[test]
fn symlink_uri_identity_and_target_remain_intact() {
    let f = Fixture::new();
    let link = f.root.path().join("link.png");
    symlink(&f.source.path, &link).unwrap();
    let key = FileKey::read(&link).unwrap();
    let fake = Fake::new(&f, 4, 2);
    let out = generate_with(
        &Request {
            source: &key,
            ..f.request()
        },
        SizeClass::Normal,
        &fake,
    )
    .unwrap();
    assert_eq!(out.source, key);
    assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink());
    assert_eq!(FileKey::read(&f.source.path).unwrap(), f.source);
}
#[test]
fn independent_results_do_not_share_pixels_or_cancellation() {
    let f = Fixture::new();
    let other = f.root.path().join("other.png");
    fs::copy(&f.source.path, &other).unwrap();
    let key = FileKey::read(&other).unwrap();
    let first = generate_with(&f.request(), SizeClass::Normal, &Fake::new(&f, 4, 2)).unwrap();
    let cancel = gio::Cancellable::new();
    let request = Request {
        source: &key,
        cancel: &cancel,
        ..f.request()
    };
    let second = generate_with(&request, SizeClass::Large, &Fake::new(&f, 8, 8)).unwrap();
    f.cancel.cancel();
    assert!(!cancel.is_cancelled());
    assert_eq!((first.width, first.height), (4, 2));
    assert_eq!((second.width, second.height), (8, 8));
    assert_eq!(first.pixels.len(), 32);
    assert_eq!(second.pixels.len(), 256);
    assert_ne!(first.source, second.source);
}
#[test]
fn decode_error_classification_never_promotes_file_errors_to_provider_loss() {
    assert!(matches!(
        loader_error("Preload exceeds cache entry budget".into()),
        Error::LimitExceeded(_)
    ));
    assert!(matches!(
        loader_error("Decoded content is not a JPEG or PNG image".into()),
        Error::InvalidSource(_)
    ));
    for message in [
        "bwrap denied",
        "corrupt JPEG",
        "filename Preload exceeds cache entry budget",
    ] {
        assert!(matches!(
            loader_error(message.into()),
            Error::GenerationFailed(_)
        ));
    }
}
#[test]
fn generation_routes_only_through_manager_and_fd03_stays_independent() {
    // FD-06 replaces the old pre-integration guard with a stricter routing boundary.
    for code in [
        include_str!("../../src/thumbnail.rs"),
        include_str!("../../src/window.rs"),
        include_str!("../../src/freedesktop_thumbnail.rs"),
    ] {
        assert!(!code.contains("freedesktop_generator::generate"));
    }
    let strip = include_str!("../../src/thumbnail_strip.rs");
    assert!(strip.contains("Manager::session()"));
    assert!(!strip.contains("freedesktop_generator::generate"));
    assert!(
        include_str!("../../src/thumbnail_manager.rs")
            .contains("freedesktop_generator::generate_admitted")
    );
}

#[test]
fn actual_frame_buffer_limit_is_checked_before_rgba_download() {
    let f = Fixture::new();
    let mut fake = Fake::new(&f, 4, 2);
    fake.frame_bytes = DECODE_BUDGET + 1;
    assert!(matches!(
        generate_with(&f.request(), SizeClass::Normal, &fake),
        Err(Error::LimitExceeded(_))
    ));
    fake.frame_bytes = 32;
    generate_with(&f.request(), SizeClass::Normal, &fake).unwrap();
}

struct Coordinated {
    started: std::sync::mpsc::Sender<()>,
    release: std::sync::mpsc::Receiver<()>,
}
impl Decoder for Coordinated {
    fn decode<'a>(&'a self, _: &'a Path, cancel: &'a gio::Cancellable) -> DecodeFuture<'a> {
        Box::pin(async move {
            self.started.send(()).unwrap();
            self.release
                .recv_timeout(Duration::from_secs(3))
                .map_err(|e| Error::GenerationFailed(e.to_string()))?;
            if cancel.is_cancelled() {
                return Err(Error::GenerationFailed("controlled cancelled child".into()));
            }
            let bytes = glib::Bytes::from_owned(vec![100u8; 32]);
            Ok(Decoded {
                texture: gdk::MemoryTexture::new(4, 2, gdk::MemoryFormat::R8g8b8a8, &bytes, 16)
                    .upcast(),
                frame_bytes: 32,
            })
        })
    }
}
#[test]
fn two_active_controlled_workers_cancel_independently_and_release_both_permits() {
    use std::sync::mpsc::channel;
    let f = Fixture::new();
    let other = f.root.path().join("other.png");
    fs::copy(&f.source.path, &other).unwrap();
    let first_key = f.source.clone();
    let second_key = FileKey::read(&other).unwrap();
    let first_cancel = f.cancel.clone();
    let second_cancel = gio::Cancellable::new();
    let second_observer = second_cancel.clone();
    let (started, observed) = channel();
    let (release_first, first_rx) = channel();
    let (release_second, second_rx) = channel();
    let a = Coordinated {
        started: started.clone(),
        release: first_rx,
    };
    let b = Coordinated {
        started,
        release: second_rx,
    };
    let (first, second) = std::thread::scope(|scope| {
        let first = scope.spawn(move || {
            generate_with(
                &Request {
                    source: &first_key,
                    cancel: &first_cancel,
                    deadline: Instant::now() + Duration::from_secs(10),
                },
                SizeClass::Normal,
                &a,
            )
        });
        let second = scope.spawn(move || {
            generate_with(
                &Request {
                    source: &second_key,
                    cancel: &second_cancel,
                    deadline: Instant::now() + Duration::from_secs(10),
                },
                SizeClass::Normal,
                &b,
            )
        });
        observed.recv_timeout(Duration::from_secs(3)).unwrap();
        observed.recv_timeout(Duration::from_secs(3)).unwrap();
        f.cancel.cancel();
        release_first.send(()).unwrap();
        release_second.send(()).unwrap();
        (first.join().unwrap(), second.join().unwrap())
    });
    assert!(matches!(first, Err(Error::Cancelled)));
    let out = second.unwrap();
    assert_eq!((out.width, out.height), (4, 2));
    assert!(!second_observer.is_cancelled());
    assert!(ACTIVE.lock().unwrap().is_empty());
}

#[test]
fn alpha_quadrants_survive_real_postdecode_downsampling() {
    let f = Fixture::new();
    let colors = [
        [255u8, 0, 0, 255],
        [0, 255, 0, 128],
        [0, 0, 255, 64],
        [255, 255, 0, 0],
    ];
    let mut pixels = Vec::with_capacity(256 * 256 * 4);
    for y in 0..256 {
        for x in 0..256 {
            pixels.extend_from_slice(&colors[usize::from(y >= 128) * 2 + usize::from(x >= 128)]);
        }
    }
    let bytes = glib::Bytes::from_owned(pixels);
    let texture = gdk::MemoryTexture::new(256, 256, gdk::MemoryFormat::R8g8b8a8, &bytes, 1024);
    let (w, h, small, _) = resample(texture.upcast_ref(), SizeClass::Normal, &f.request()).unwrap();
    assert_eq!((w, h), (128, 128));
    for (i, expected) in colors.iter().enumerate() {
        let x = if i % 2 == 0 { 32 } else { 96 };
        let y = if i < 2 { 32 } else { 96 };
        let at = (y * 128 + x) * 4;
        let pixel = &small[at..at + 4];
        assert!(pixel[3].abs_diff(expected[3]) <= 1);
        if expected[3] > 0 {
            for (a, b) in pixel[..3].iter().zip(&expected[..3]) {
                assert!(a.abs_diff(*b) <= 1, "RGBA {pixel:?}");
            }
        }
    }
}

#[test]
fn high_resolution_geometry_and_weighted_admission_are_bounded() {
    let _f = Fixture::new();
    for (w, h) in [(4000, 3000), (6000, 4000), (8000, 6000), (10000, 5000)] {
        let (x, y) = fitted_size(w, h, SizeClass::Normal).unwrap();
        assert!(x <= 128 && y <= 128);
        let job =
            Reservation::acquire(u64::from(w) * u64::from(h), 8 * 1024 * 1024 * 1024).unwrap();
        assert!(job.large);
        assert!(matches!(
            Reservation::acquire(12_000_000, 8 * 1024 * 1024 * 1024),
            Err(Error::Busy)
        ));
        drop(job);
    }
    assert!(matches!(
        Reservation::acquire(24_000_000, 512 * 1024 * 1024),
        Err(Error::LimitExceeded(_))
    ));
    assert!(matches!(
        Reservation::acquire(108_000_000, usize::MAX),
        Err(Error::LimitExceeded(_))
    ));
    let large = Reservation::acquire(50_000_000, 8 * 1024 * 1024 * 1024).unwrap();
    let small = Reservation::acquire(1_000_000, 8 * 1024 * 1024 * 1024).unwrap();
    assert!(matches!(
        Reservation::acquire(8_000_000, 8 * 1024 * 1024 * 1024),
        Err(Error::LimitExceeded(_))
    ));
    drop(small);
    drop(large);
    assert_eq!(MEMORY.lock().unwrap().0, 0);
    assert!(
        secure_loader::validate_dimensions(8000, 6000).is_ok(),
        "48 MP images are supported by the 64 MP photo policy"
    );
    assert!(
        secure_loader::validate_dimensions(8001, 8000).is_err(),
        "images exceeding 64 MP remain rejected"
    );
}
