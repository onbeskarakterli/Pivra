//! Native secure-Glycin evidence only. One capability gate; no repeated blocked decode.
use gtk::gio;
use pivra::{
    freedesktop_generator::{GeneratedThumbnail, generate},
    freedesktop_thumbnail::SizeClass,
    image_cache::FileKey,
    thumbnail_provider::{Error, Request},
};
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn run(path: &Path, class: SizeClass) -> Result<GeneratedThumbnail, Error> {
    let key = FileKey::read(path).map_err(|e| Error::SourceUnavailable(e.to_string()))?;
    let cancel = gio::Cancellable::new();
    let out = generate(
        &Request {
            source: &key,
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(30),
        },
        class,
    )?;
    assert_eq!(FileKey::read(path).unwrap(), key, "source changed");
    assert_eq!(out.pixels.len(), out.stride * out.height as usize);
    assert!(out.width <= class.max_edge() && out.height <= class.max_edge());
    println!(
        "PASS secure in-memory {} {:?}: {}x{} output={} decoded={}x{} frame={} download={} decode_us={} resize_us={}",
        path.display(),
        class,
        out.width,
        out.height,
        out.pixels.len(),
        out.metrics.decoded_width,
        out.metrics.decoded_height,
        out.metrics.frame_bytes,
        out.metrics.download_bytes,
        out.metrics.decode_time.as_micros(),
        out.metrics.resize_time.as_micros()
    );
    Ok(out)
}
fn corners(out: &GeneratedThumbnail, order: [usize; 4], tolerance: u8) {
    let rgb = [[255u8, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 0]];
    for (i, expected) in order.into_iter().enumerate() {
        let x = if i % 2 == 0 {
            out.width / 4
        } else {
            3 * out.width / 4
        };
        let y = if i < 2 {
            out.height / 4
        } else {
            3 * out.height / 4
        };
        let at = y as usize * out.stride + x as usize * 4;
        let pixel = &out.pixels[at..at + 4];
        for (a, b) in pixel[..3].iter().zip(rgb[expected]) {
            assert!(
                a.abs_diff(b) <= tolerance,
                "actual={pixel:?}, expected={:?}",
                rgb[expected]
            );
        }
        assert_eq!(pixel[3], 255);
    }
}
fn main() {
    std::thread::spawn(probe)
        .join()
        .expect("native probe worker panicked");
}
fn probe() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images");
    // The real adapter itself must pass before malformed/native checks get credit.
    let first = match run(&root.join("wide.png"), SizeClass::Normal) {
        Ok(out) => out,
        Err(Error::GenerationFailed(message))
            if [
                "bwrap",
                "Loader process exited early",
                "No loaders",
                "No loader",
            ]
            .iter()
            .any(|s| message.contains(s)) =>
        {
            eprintln!("NOT RUN secure Glycin prerequisite: {message}");
            std::process::exit(77);
        }
        Err(error) => panic!("native baseline failed (not a PASS/skip): {error:?}"),
    };
    assert_eq!((first.width, first.height), (64, 32));
    corners(&first, [0, 1, 2, 3], 0);
    for class in [
        SizeClass::Normal,
        SizeClass::Large,
        SizeClass::XLarge,
        SizeClass::XxLarge,
    ] {
        let out = run(&root.join("large.png"), class).expect("large secure decode");
        let edge = class.max_edge();
        assert_eq!((out.width, out.height), (edge, edge * 3 / 4));
        corners(&out, [0, 1, 2, 3], 1);
        assert_eq!(
            (out.metrics.decoded_width, out.metrics.decoded_height),
            (1600, 1200),
            "this is full decode, not reduced resolution"
        );
    }
    for (name, size) in [
        ("tall.png", (32, 64)),
        ("small.png", (4, 2)),
        ("no-exif.jpg", (64, 32)),
        ("png-content.jpg", (64, 32)),
    ] {
        let out = run(&root.join(name), SizeClass::Normal).expect("valid secure image");
        assert_eq!((out.width, out.height), size);
    }
    for (i, order) in [
        [0, 1, 2, 3],
        [1, 0, 3, 2],
        [3, 2, 1, 0],
        [2, 3, 0, 1],
        [0, 2, 1, 3],
        [2, 0, 3, 1],
        [3, 1, 2, 0],
        [1, 3, 0, 2],
    ]
    .into_iter()
    .enumerate()
    {
        let out = run(
            &root.join(format!("orientation-{}.jpg", i + 1)),
            SizeClass::Normal,
        )
        .expect("secure EXIF");
        assert_eq!(
            (out.width, out.height),
            if i < 4 { (64, 32) } else { (32, 64) }
        );
        corners(&out, order, 12);
    }
    // Private source fixture (not a cached thumbnail): RGBA 2x2 encoded beforehand.
    let private = tempfile::tempdir().unwrap();
    let alpha = private.path().join("alpha source ç #?.png");
    std::fs::write(&alpha, ALPHA_PNG).unwrap();
    let out = run(&alpha, SizeClass::Normal).expect("real alpha PNG");
    assert_eq!((out.width, out.height), (2, 2));
    for (pixel, expected) in out.pixels.chunks_exact(4).zip([
        [255u8, 0, 0, 255],
        [0, 255, 0, 128],
        [0, 0, 255, 64],
        [255, 255, 0, 0],
    ]) {
        assert!(pixel[3].abs_diff(expected[3]) <= 1);
        if expected[3] > 0 {
            for (a, b) in pixel[..3].iter().zip(&expected[..3]) {
                assert!(a.abs_diff(*b) <= 1, "alpha/color {pixel:?}");
            }
        }
    }
    // Recognizable headers force actual secure decoding failures after the valid gate.
    for (name, bytes) in [
        ("truncated.png", &ALPHA_PNG[..33]),
        ("truncated.jpg", &[0xff, 0xd8, 0xff, 0xe0, 0, 16, 0][..]),
    ] {
        let path = private.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        assert!(
            matches!(
                run(&path, SizeClass::Normal),
                Err(Error::GenerationFailed(_) | Error::InvalidSource(_))
            ),
            "malformed input must fail without provider promotion"
        );
    }
    let first_pixels = first.pixels.to_vec();
    let other = run(&root.join("tall.png"), SizeClass::Large).unwrap();
    assert_eq!(first.pixels.as_ref(), first_pixels.as_slice());
    assert_ne!(first.source, other.source);
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in status
        .lines()
        .filter(|line| line.starts_with("VmRSS:") || line.starts_with("VmHWM:"))
    {
        println!("MEASURE parent {line} (excludes loader child RSS)");
    }
    println!(
        "PASS FD-04 native JPEG/PNG/EXIF/alpha/in-memory classes; no cache write or production integration; active native cancellation/cleanup not certified by this probe"
    );
}
const ALPHA_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0, 0, 2, 8, 6, 0,
    0, 0, 114, 182, 13, 36, 0, 0, 0, 22, 73, 68, 65, 84, 120, 156, 99, 248, 207, 192, 240, 31, 8,
    27, 24, 128, 180, 195, 127, 32, 15, 0, 62, 26, 6, 187, 128, 69, 235, 195, 0, 0, 0, 0, 73, 69,
    78, 68, 174, 66, 96, 130,
];
