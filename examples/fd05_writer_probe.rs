//! Real secure PNG roundtrip, isolated sources/cache. Exit77 is NOT RUN, not PASS.
use gtk::{gio, glib};
use pivra::{
    freedesktop_generator::{GeneratedThumbnail, generate},
    freedesktop_thumbnail::{CacheReader, SizeClass},
    freedesktop_writer::{CacheWriter, Publication},
    image_cache::FileKey,
    thumbnail_provider::{CacheOutcome, Error, Request},
};
use std::{
    path::Path,
    time::{Duration, Instant},
};
fn request<'a>(key: &'a FileKey, cancel: &'a gio::Cancellable) -> Request<'a> {
    Request {
        source: key,
        cancel,
        deadline: Instant::now() + Duration::from_secs(30),
    }
}
fn decoded(path: &Path, class: SizeClass) -> Result<GeneratedThumbnail, Error> {
    let key = FileKey::read(path).map_err(|e| Error::SourceUnavailable(e.to_string()))?;
    generate(&request(&key, &gio::Cancellable::new()), class)
}
fn roundtrip(writer: &CacheWriter, reader: &CacheReader, t: &GeneratedThumbnail) {
    let cancel = gio::Cancellable::new();
    let req = request(&t.source, &cancel);
    let Publication::Created(path) = writer.write(t, &req).expect("new standard PNG") else {
        panic!("private fixture should create a new record");
    };
    let CacheOutcome::Hit(candidate) = reader.lookup(&req, t.class).expect("metadata reader")
    else {
        panic!("Writer -> Reader must Hit");
    };
    assert_eq!(candidate.source, t.source);
    assert_eq!(candidate.thumbnail.path, path);
    let result = decoded(&path, t.class).expect("actual secure Glycin decode of WRITTEN PNG");
    assert_eq!((result.width, result.height), (t.width, t.height));
    for (actual, expected) in result.pixels.chunks_exact(4).zip(t.pixels.chunks_exact(4)) {
        assert!(actual[3].abs_diff(expected[3]) <= 1);
        if expected[3] > 0 {
            for (a, b) in actual[..3].iter().zip(&expected[..3]) {
                assert!(
                    a.abs_diff(*b) <= 1,
                    "real PNG RGBA mismatch {actual:?} vs {expected:?}"
                );
            }
        }
    }
    assert_eq!(FileKey::read(&t.source.path).unwrap(), t.source);
    println!(
        "PASS native writer->reader Hit->secure Glycin RGBA {:?} {}x{} {} bytes",
        t.class,
        t.width,
        t.height,
        t.pixels.len()
    );
}
fn main() {
    std::thread::spawn(probe).join().expect("native worker");
}
fn probe() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images");
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source ç # % ?.png");
    std::fs::copy(fixtures.join("large.png"), &source).unwrap();
    let first = match decoded(&source, SizeClass::Normal) {
        Ok(t) => t,
        Err(Error::GenerationFailed(text))
            if ["bwrap", "Loader process exited early", "No loader"]
                .iter()
                .any(|s| text.contains(s)) =>
        {
            eprintln!("NOT RUN secure prerequisite: {text}");
            std::process::exit(77);
        }
        Err(e) => panic!("native prerequisite failed: {e:?}"),
    };
    let cache = root.path().join("cache");
    let writer = CacheWriter::new(cache.clone()).unwrap();
    let reader = CacheReader::new(cache.clone()).unwrap();
    roundtrip(&writer, &reader, &first);
    for class in [SizeClass::Large, SizeClass::XLarge, SizeClass::XxLarge] {
        let t = decoded(&source, class).expect("secure source decode");
        roundtrip(&writer, &reader, &t);
    }
    let jpeg = root.path().join("source.jpg");
    std::fs::copy(fixtures.join("no-exif.jpg"), &jpeg).unwrap();
    let t = decoded(&jpeg, SizeClass::Normal).expect("secure JPEG source");
    roundtrip(&writer, &reader, &t);
    // Controlled RGBA for encoder/alpha roundtrip, not a claim of source-image generation.
    let alpha_source = root.path().join("controlled-alpha-source.png");
    std::fs::copy(fixtures.join("wide.png"), &alpha_source).unwrap();
    let mut alpha = decoded(&alpha_source, SizeClass::Normal).unwrap();
    alpha.width = 8;
    alpha.height = 4;
    alpha.stride = 32;
    let mut pixels = Vec::new();
    for _ in 0..8 {
        for pixel in [
            [255, 0, 0, 255],
            [0, 255, 0, 128],
            [0, 0, 255, 64],
            [255, 255, 0, 0],
        ] {
            pixels.extend_from_slice(&pixel);
        }
    }
    alpha.pixels = glib::Bytes::from_owned(pixels);
    alpha.metrics.output_bytes = alpha.pixels.len();
    roundtrip(&writer, &reader, &alpha);
    let before = FileKey::read(&source).unwrap();
    std::fs::write(&source, b"changed private source").unwrap();
    let current = FileKey::read(&source).unwrap();
    assert_ne!(before, current);
    assert!(matches!(
        reader
            .lookup(
                &request(&current, &gio::Cancellable::new()),
                SizeClass::Normal
            )
            .unwrap(),
        CacheOutcome::Stale
    ));
    println!(
        "PASS FD-05 native PNG/JPEG source, four classes, real written PNG alpha/RGBA, source invalidation; private cache only; large-photo support and active native cancellation NOT certified"
    );
}
