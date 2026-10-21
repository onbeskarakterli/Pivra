//! Real Glycin decode/pixel/orientation gate. Exit 77 means BLOCKED, never PASS.
#[path = "../src/image_loader.rs"]
mod image_loader;

use gtk::{gio, glib, prelude::*};
use std::path::Path;

fn check(
    path: &Path,
    width: u32,
    height: u32,
    corners: [usize; 4],
    tolerance: u8,
) -> Result<(), String> {
    let frame = glib::MainContext::default()
        .block_on(image_loader::load(path, &gio::Cancellable::new()))?;
    assert_eq!((frame.width(), frame.height()), (width, height));
    let texture = frame.texture();
    let stride = width as usize * 4;
    let mut bytes = vec![0; stride * height as usize];
    texture.download(&mut bytes, stride);
    // GdkTexture.download uses native premultiplied ARGB32: BGRA on this LE Linux target.
    const {
        assert!(cfg!(target_endian = "little"));
    }
    let rgb = [[255_u8, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 0]];
    for (index, corner) in corners.into_iter().enumerate() {
        let x = if index % 2 == 0 {
            width / 4
        } else {
            3 * width / 4
        };
        let y = if index < 2 {
            height / 4
        } else {
            3 * height / 4
        };
        let offset = y as usize * stride + x as usize * 4;
        let actual = [bytes[offset + 2], bytes[offset + 1], bytes[offset]];
        for channel in 0..3 {
            assert!(
                actual[channel].abs_diff(rgb[corner][channel]) <= tolerance,
                "{}: {actual:?}, expected {:?}",
                path.display(),
                rgb[corner]
            );
        }
        assert_eq!(bytes[offset + 3], 255);
    }
    println!(
        "PASS real decode + texture pixel orientation: {}",
        path.display()
    );
    Ok(())
}

fn main() {
    if let Err(error) = gtk::init() {
        eprintln!("BLOCKED graphical probe: {error}");
        std::process::exit(77);
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images");
    // A valid image must decode before any malformed-input check can be credited.
    let jpeg_first = std::env::args().any(|arg| arg == "--jpeg-first");
    let baseline = if jpeg_first {
        "no-exif.jpg"
    } else {
        "wide.png"
    };
    if let Err(error) = check(
        &root.join(baseline),
        64,
        32,
        [0, 1, 2, 3],
        if jpeg_first { 12 } else { 0 },
    ) {
        eprintln!("BLOCKED real JPEG/PNG/EXIF/texture gate: {error}");
        std::process::exit(77);
    }
    for (name, width, height) in [
        ("tall.png", 32, 64),
        ("small.png", 4, 2),
        ("large.png", 1600, 1200),
        ("png-content.jpg", 64, 32),
    ] {
        check(&root.join(name), width, height, [0, 1, 2, 3], 0).expect("PNG pixels");
    }
    check(&root.join("no-exif.jpg"), 64, 32, [0, 1, 2, 3], 12).expect("JPEG pixels");
    let corners = [
        [0, 1, 2, 3],
        [1, 0, 3, 2],
        [3, 2, 1, 0],
        [2, 3, 0, 1],
        [0, 2, 1, 3],
        [2, 0, 3, 1],
        [3, 1, 2, 0],
        [1, 3, 0, 2],
    ];
    for (index, corners) in corners.into_iter().enumerate() {
        let (width, height) = if index < 4 { (64, 32) } else { (32, 64) };
        check(
            &root.join(format!("orientation-{}.jpg", index + 1)),
            width,
            height,
            corners,
            12,
        )
        .expect("EXIF pixel orientation");
    }
    let directory = tempfile::tempdir().unwrap();
    let unicode = directory.path().join("fotoğraf # % 🦀.png");
    std::fs::copy(root.join("wide.png"), &unicode).unwrap();
    check(&unicode, 64, 32, [0, 1, 2, 3], 0).expect("Unicode image");
    for name in [
        "broken.jpg",
        "broken.png",
        "text-content.jpg",
        "missing.png",
    ] {
        assert!(
            glib::MainContext::default()
                .block_on(image_loader::load(
                    &root.join(name),
                    &gio::Cancellable::new()
                ))
                .is_err()
        );
        println!("PASS controlled invalid-image error: {name}");
    }
    match check(&root.join("malformed-exif.jpg"), 64, 32, [0, 1, 2, 3], 12) {
        Ok(()) => (),
        Err(error) => println!("Controlled malformed EXIF error: {error}"),
    }
    println!(
        "PASS decoder/texture gate; application Fit/resize/lifecycle still require separate GUI pixel tests"
    );
}
