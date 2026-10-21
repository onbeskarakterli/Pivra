//! Real image-view regression gate; requires secure Glycin and a graphical session.
//! Exit 77 means unavailable decode, never a passing geometry/lifecycle test.
#[path = "../src/image_loader.rs"]
mod image_loader;
#[path = "../src/image_view.rs"]
mod image_view;

use adw::prelude::*;
use gtk::{gdk, glib};
use std::time::{Duration, Instant};

fn wait_until(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !ready() {
        assert!(Instant::now() < deadline, "GUI operation timed out");
        while glib::MainContext::default().pending() {
            glib::MainContext::default().iteration(false);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn settle_frame(picture: &gtk::Picture) {
    // WidgetPaintable reflects the last rendered frame, not merely the new allocation.
    let frames = std::rc::Rc::new(std::cell::Cell::new(0));
    let ticks = frames.clone();
    picture.add_tick_callback(move |_, _| {
        ticks.set(ticks.get() + 1);
        if ticks.get() >= 3 {
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
    picture.queue_draw();
    wait_until(|| frames.get() >= 3);
}

fn geometry(window: &adw::ApplicationWindow, picture: &gtk::Picture) {
    settle_frame(picture);
    let (width, height) = (picture.width(), picture.height());
    assert!(width > 0 && height > 0);
    let image = picture.paintable().expect("decoded image");
    let (iw, ih) = (image.intrinsic_width(), image.intrinsic_height());
    let scale = (width as f64 / iw as f64)
        .min(height as f64 / ih as f64)
        .min(1.0);
    let snapshot = gtk::Snapshot::new();
    gtk::WidgetPaintable::new(Some(picture)).snapshot(&snapshot, width as f64, height as f64);
    let node = snapshot.to_node().expect("image render node");
    let texture = window.renderer().unwrap().render_texture(
        &node,
        Some(&gtk::graphene::Rect::new(
            0.,
            0.,
            width as f32,
            height as f32,
        )),
    );
    assert_eq!((texture.width(), texture.height()), (width, height));
    let mut pixels = vec![0; width as usize * height as usize * 4];
    texture.download(&mut pixels, width as usize * 4);
    let (mut left, mut top, mut right, mut bottom) = (width, height, -1, -1);
    for y in 0..height {
        for x in 0..width {
            // Native ARGB32 alpha byte is fourth on this supported LE target.
            if pixels[((y * width + x) * 4 + 3) as usize] > 127 {
                left = left.min(x);
                top = top.min(y);
                right = right.max(x);
                bottom = bottom.max(y);
            }
        }
    }
    let expected_width = iw as f64 * scale;
    let expected_height = ih as f64 * scale;
    assert!((f64::from(right - left + 1) - expected_width).abs() <= 2.0);
    assert!((f64::from(bottom - top + 1) - expected_height).abs() <= 2.0);
    assert!((f64::from(left + right + 1) / 2. - f64::from(width) / 2.).abs() <= 1.0);
    assert!((f64::from(top + bottom + 1) / 2. - f64::from(height) / 2.).abs() <= 1.0);
    println!("PASS actual image_view snapshot Fit/center: {iw}x{ih} in {width}x{height}");
}

fn main() {
    const {
        assert!(cfg!(target_endian = "little"));
    }
    if let Err(error) = adw::init() {
        eprintln!("BLOCKED graphical session: {error}");
        std::process::exit(77);
    }
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.ImageGuiProbe")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images");
    // Both real production views start loading before the main loop is pumped.
    let views: Vec<_> = ["large.png", "orientation-6.jpg"]
        .into_iter()
        .map(|name| {
            let window = adw::ApplicationWindow::builder()
                .application(&app)
                .default_width(800)
                .default_height(600)
                .build();
            let overlay = image_view::build(&window, root.join(name));
            let picture = overlay.child().unwrap().downcast::<gtk::Picture>().unwrap();
            let status = picture
                .next_sibling()
                .unwrap()
                .downcast::<gtk::Label>()
                .unwrap();
            window.set_content(Some(&overlay));
            window.present();
            (window, picture, status)
        })
        .collect();
    wait_until(|| {
        views.iter().all(|(_, picture, status)| {
            picture.paintable().is_some() || status.text().starts_with("Unable to display image.")
        })
    });
    for (window, picture, status) in &views {
        if picture.paintable().is_none() {
            eprintln!("BLOCKED secure image-view decode: {}", status.text());
            for (window, _, _) in &views {
                window.close();
            }
            std::process::exit(77);
        }
        assert!(!status.is_visible());
        assert!(picture.paintable().unwrap().is::<gdk::Texture>());
        geometry(window, picture);
    }
    println!(
        "PASS two concurrently scheduled image-view load jobs completed independently (decoder time overlap not measured)"
    );
    let (first, picture, _) = &views[0];
    let old = (picture.width(), picture.height());
    first.set_default_size(480, 320);
    wait_until(|| (picture.width(), picture.height()) != old);
    geometry(first, picture);
    first.close();
    wait_until(|| !first.is_visible());
    assert!(views[1].0.is_visible());
    geometry(&views[1].0, &views[1].1);
    views[1].0.close();
    wait_until(|| !views[1].0.is_visible());
    println!(
        "PASS resize and independent normal close after successful decode; active-decode close NOT TESTED"
    );
}
