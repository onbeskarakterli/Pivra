//! Issue #9 diagnostic: color-state conversion and unmodified MediaFile snapshots.
//! Synthetic checks are not evidence of native H.264 playback or visual correctness.
#[path = "../src/actual_paintable.rs"]
#[allow(dead_code)] // Other paintable operations are exercised by the existing viewing probes.
mod actual_paintable;
use gtk::{gdk, glib, prelude::*};
use pivra::viewing::{Transform, ViewMode};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

fn rgb(texture: &gdk::Texture) -> glib::Bytes {
    let mut downloader = gdk::TextureDownloader::new(texture);
    downloader.set_color_state(&gdk::ColorState::srgb());
    downloader.set_format(gdk::MemoryFormat::R8g8b8a8);
    let (bytes, stride) = downloader.download_bytes();
    assert_eq!(stride, texture.width() as usize * 4);
    bytes
}

fn synthetic() {
    let params = gdk::CicpParams::new();
    params.set_color_primaries(1);
    params.set_transfer_function(1);
    params.set_matrix_coefficients(0);
    params.set_range(gdk::CicpRange::Full);
    let state = params.build_color_state().expect("BT.709 RGB color state");
    let values = [0_u8, 16, 32, 64, 128, 235, 255];
    let bytes: Vec<_> = values.iter().flat_map(|v| [*v, *v, *v, 255]).collect();
    let texture = gdk::MemoryTextureBuilder::new()
        .set_width(values.len() as i32)
        .set_height(1)
        .set_format(gdk::MemoryFormat::R8g8b8a8)
        .set_stride(values.len() * 4)
        .set_bytes(Some(&glib::Bytes::from_owned(bytes)))
        .set_color_state(&state)
        .build();
    let output = rgb(&texture);
    for (index, value) in values.into_iter().enumerate() {
        let encoded = f64::from(value) / 255.;
        let linear = if encoded < 0.081 {
            encoded / 4.5
        } else {
            ((encoded + 0.099) / 1.099).powf(1. / 0.45)
        };
        let srgb = if linear <= 0.0031308 {
            linear * 12.92
        } else {
            1.055 * linear.powf(1. / 2.4) - 0.055
        };
        let expected = (srgb * 255.).round() as u8;
        let pixel = &output[index * 4..index * 4 + 4];
        assert!(
            pixel[..3].iter().all(|v| v.abs_diff(expected) <= 1),
            "CICP conversion {value}: {pixel:?}, expected {expected}"
        );
        assert_eq!(pixel[3], 255);
        println!("BT709 RGB input={value} -> sRGB={pixel:?}");
    }
    assert_eq!(&output[..4], &[0, 0, 0, 255], "black endpoint changed");
    assert_eq!(
        &output[24..28],
        &[255, 255, 255, 255],
        "white endpoint changed"
    );
    println!("PASS synthetic GDK BT.709-to-sRGB transfer; this is not a video range test");
}

fn render(
    paintable: &impl IsA<gdk::Paintable>,
    renderer: &gtk::gsk::Renderer,
    w: i32,
    h: i32,
) -> Option<gdk::Texture> {
    let snapshot = gtk::Snapshot::new();
    paintable.snapshot(&snapshot, f64::from(w), f64::from(h));
    snapshot.to_node().map(|node| {
        renderer.render_texture(
            &node,
            Some(&gtk::graphene::Rect::new(0., 0., w as f32, h as f32)),
        )
    })
}

fn native(path: &std::path::Path) -> Result<(), String> {
    let window = gtk::Window::builder()
        .title("Pivra issue #9 color diagnostic")
        .default_width(640)
        .default_height(360)
        .build();
    let media = gtk::MediaFile::for_filename(path);
    media.set_muted(true);
    let wrapped = actual_paintable::ActualPaintable::video(
        &media,
        Rc::new(RefCell::new(Transform::default())),
        ViewMode::Fit,
    );
    let picture = gtk::Picture::for_paintable(&wrapped);
    window.set_child(Some(&picture));
    window.present();
    media.play();
    let deadline = Instant::now() + Duration::from_secs(15);
    let result = loop {
        for _ in 0..100 {
            if !glib::MainContext::default().pending() {
                break;
            }
            glib::MainContext::default().iteration(false);
        }
        if let Some(error) = media.error() {
            break Err(format!("MediaFile: {error}"));
        }
        if let Some(renderer) = window.renderer() {
            let (w, h) = (media.intrinsic_width(), media.intrinsic_height());
            if w > 0 && h > 0 && i64::from(w) * i64::from(h) <= 16_777_216 {
                media.pause(); // Both snapshots must observe the same frame.
                if let (Some(raw), Some(wrapper)) = (
                    render(&media, &renderer, w, h),
                    render(&wrapped, &renderer, w, h),
                ) {
                    let (a, b) = (rgb(&raw), rgb(&wrapper));
                    assert_eq!(a.len(), b.len());
                    let max = a
                        .iter()
                        .zip(b.iter())
                        .map(|(a, b)| a.abs_diff(*b))
                        .max()
                        .unwrap();
                    assert!(
                        max <= 1,
                        "ActualPaintable changes video pixels: max difference={max}"
                    );
                    println!(
                        "PASS raw MediaFile vs ActualPaintable: {w}x{h}, max channel difference={max}"
                    );
                    for x in [w / 6, w / 2, 5 * w / 6] {
                        let offset = ((h / 2 * w + x) * 4) as usize;
                        println!("sample x={x} y={} sRGB={:?}", h / 2, &a[offset..offset + 4]);
                    }
                    break Ok(());
                }
                media.play();
            }
        }
        if Instant::now() >= deadline {
            break Err("no renderable video frame within 15s".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    };
    media.pause();
    picture.set_paintable(None::<&gdk::Paintable>);
    media.clear();
    window.close();
    result
}
fn main() {
    if let Err(error) = gtk::init() {
        eprintln!("NOT RUN GTK: {error}");
        std::process::exit(77);
    }
    synthetic();
    if let Some(path) = std::env::args_os().nth(1) {
        let path = std::path::Path::new(&path);
        if !path.is_file() {
            eprintln!("FAIL input is not a file: {}", path.display());
            std::process::exit(1);
        }
        if let Err(reason) = native(path) {
            // Only the explicit missing-backend condition is an environment skip.
            // Decoder errors and frame deadlines remain real failures.
            if reason.contains("GTK could not find a media module") {
                eprintln!("NOT RUN native video color comparison: {reason}");
                std::process::exit(77);
            }
            eprintln!("FAIL native video color comparison: {reason}");
            std::process::exit(1);
        }
        println!(
            "Native visual comparison with MPV remains required; pixel equality is not a color-correctness verdict"
        );
    }
}
