//! Default: real secure decode + production actions. --synthetic: renderer only.
#[path = "../src/image_loader.rs"]
mod image_loader;
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/input.rs"]
mod input;
#[path = "support/resampling_cases.rs"]
mod resampling_cases;
#[path = "../src/window.rs"]
mod window;
#[path = "support/zoom_pan_cases.rs"]
mod zoom_pan_cases;
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use pivra::viewing::ViewMode;
use std::{
    path::Path,
    time::{Duration, Instant},
};
fn pump() {
    for _ in 0..100 {
        if !glib::MainContext::default().pending() {
            break;
        }
        glib::MainContext::default().iteration(false);
    }
    std::thread::sleep(Duration::from_millis(1));
}
fn wait(mut ready: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(20);
    while !ready() {
        assert!(Instant::now() < end, "mode probe deadline");
        pump();
    }
}
fn settle() {
    let end = Instant::now() + Duration::from_millis(100);
    while Instant::now() < end {
        pump();
    }
}
fn widgets(w: &gtk::Widget) -> Vec<gtk::Widget> {
    let mut v = vec![w.clone()];
    let mut c = w.first_child();
    while let Some(w) = c {
        v.extend(widgets(&w));
        c = w.next_sibling();
    }
    v
}
fn picture(w: &adw::ApplicationWindow) -> gtk::Picture {
    widgets(w.upcast_ref())
        .into_iter()
        .find_map(|w| w.downcast().ok())
        .unwrap()
}
fn select(w: &adw::ApplicationWindow, mode: ViewMode) {
    gio::prelude::ActionGroupExt::activate_action(w, "image-mode", Some(&mode.id().to_variant()));
}
fn state(w: &adw::ApplicationWindow) -> String {
    w.lookup_action("image-mode")
        .unwrap()
        .state()
        .unwrap()
        .str()
        .unwrap()
        .into()
}
// WidgetPaintable reflects GTK's rendered widget cache, not decoder completion.
// Keep it alive across a real frame; a fixed sleep cannot certify layout/paint.
fn rendered_node(w: &adw::ApplicationWindow, p: &gtk::Picture) -> gtk::gsk::RenderNode {
    use std::{cell::Cell, rc::Rc};
    assert!(
        p.paintable().is_some(),
        "snapshot requested without image content"
    );
    let paintable = gtk::WidgetPaintable::new(Some(p));
    w.present(); // In two-window cases the target may be compositor-occluded.
    let clock = p.frame_clock().expect("mapped picture frame clock");
    let painted = Rc::new(Cell::new(false));
    let observed = painted.clone();
    let handler = clock.connect_after_paint(move |_| observed.set(true));
    let tick = p.add_tick_callback(|_, _| glib::ControlFlow::Continue);
    p.queue_draw();
    let end = Instant::now() + Duration::from_secs(20);
    let node = loop {
        if painted.replace(false) && p.is_mapped() && p.width() > 0 && p.height() > 0 {
            let snapshot = gtk::Snapshot::new();
            paintable.snapshot(&snapshot, p.width() as f64, p.height() as f64);
            if let Some(node) = snapshot.to_node() {
                break Some(node);
            }
            p.queue_draw();
        }
        if Instant::now() >= end {
            break None;
        }
        pump();
    };
    tick.remove();
    clock.disconnect(handler);
    node.unwrap_or_else(|| panic!(
        "rendered image missing after GTK after-paint: mapped={} visible={} allocation={}x{} content={} mode={}",
        p.is_mapped(), p.is_visible(), p.width(), p.height(), p.paintable().is_some(), state(w)
    ))
}
fn render(
    w: &adw::ApplicationWindow,
    p: &gtk::Picture,
    size: (i32, i32),
    mode: ViewMode,
    gradient: bool,
) {
    let node = rendered_node(w, p);
    let (width, height) = (p.width(), p.height());
    assert!(width > 0 && height > 0);
    let g = mode
        .geometry(
            (size.0 as f64, size.1 as f64),
            (width as f64, height as f64),
        )
        .unwrap();
    let bounds = node.bounds();
    assert!(
        bounds.x() >= -1.
            && bounds.y() >= -1.
            && bounds.x() + bounds.width() <= width as f32 + 1.
            && bounds.y() + bounds.height() <= height as f32 + 1.,
        "unclipped render node: {bounds:?}"
    );
    let texture = w.renderer().unwrap().render_texture(
        node,
        Some(&gtk::graphene::Rect::new(
            0.,
            0.,
            width as f32,
            height as f32,
        )),
    );
    let mut pixels = vec![0; width as usize * height as usize * 4];
    texture.download(&mut pixels, width as usize * 4);
    let (mut left, mut top, mut right, mut bottom) = (width, height, -1, -1);
    for y in 0..height {
        for x in 0..width {
            if pixels[((y * width + x) * 4 + 3) as usize] > 127 {
                left = left.min(x);
                right = right.max(x);
                top = top.min(y);
                bottom = bottom.max(y);
            }
        }
    }
    let expected = (
        g.x.max(0.),
        g.y.max(0.),
        (g.x + g.width).min(width as f64),
        (g.y + g.height).min(height as f64),
    );
    for (actual, expected) in [
        (left as f64, expected.0),
        (top as f64, expected.1),
        ((right + 1) as f64, expected.2),
        ((bottom + 1) as f64, expected.3),
    ] {
        assert!(
            (actual - expected).abs() <= 2.,
            "{mode:?} rendered bounds {left},{top},{right},{bottom}; geometry {g:?}"
        );
    }
    if gradient {
        for fx in [0.2, 0.5, 0.8] {
            for fy in [0.2, 0.5, 0.8] {
                let x = (expected.0 + (expected.2 - expected.0) * fx).floor() as i32;
                let y = (expected.1 + (expected.3 - expected.1) * fy).floor() as i32;
                let offset = ((y * width + x) * 4) as usize;
                let red = (((x as f64 + 0.5 - g.x) / g.width) * 255.).clamp(0., 255.);
                let green = (((y as f64 + 0.5 - g.y) / g.height) * 255.).clamp(0., 255.);
                assert!(
                    (pixels[offset + 2] as f64 - red).abs() <= 5.
                        && (pixels[offset + 1] as f64 - green).abs() <= 5.,
                    "stretched/miscentered sample {mode:?}: {:?} expected {red}/{green}",
                    &pixels[offset..offset + 4]
                );
            }
        }
    }
    println!(
        "PASS rendered {mode:?} source {}x{} viewport {width}x{height}, centered/aspect/clip",
        size.0, size.1
    );
}
fn synthetic(app: &adw::Application) {
    let w = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(800)
        .default_height(600)
        .build();
    let view = image_view::ImageView::new(&w);
    w.set_content(Some(view.widget()));
    w.present();
    wait(|| w.is_mapped());
    let p = picture(&w);
    for (iw, ih) in [(1600, 1200), (1200, 1600), (32, 64), (64, 32)] {
        let mut data = Vec::with_capacity(iw as usize * ih as usize * 4);
        for y in 0..ih {
            for x in 0..iw {
                data.extend_from_slice(&[
                    ((x as f64 + 0.5) / iw as f64 * 255.) as u8,
                    ((y as f64 + 0.5) / ih as f64 * 255.) as u8,
                    83,
                    255,
                ]);
            }
        }
        let bytes = glib::Bytes::from_owned(data);
        let t: gdk::Texture =
            gdk::MemoryTexture::new(iw, ih, gdk::MemoryFormat::R8g8b8a8, &bytes, iw as usize * 4)
                .upcast();
        for (vw, vh) in [(800, 600), (480, 320), (320, 700)] {
            w.set_default_size(vw, vh);
            settle();
            for mode in [
                ViewMode::Fit,
                ViewMode::Fill,
                ViewMode::Actual,
                ViewMode::Fit,
            ] {
                image_view::present(&p, Some(&t), mode);
                render(&w, &p, (iw, ih), mode, true);
            }
        }
    }
    w.close();
    println!(
        "PASS 48 synthetic-texture production presentation snapshots; NOT TESTED Glycin/EXIF/native decode"
    );
}
fn open_window(app: &adw::Application, path: &Path) -> adw::ApplicationWindow {
    let before = app.windows();
    let input = input::OpenedFile::from_argument(path.as_os_str()).unwrap();
    assert_eq!(input.file.path().as_deref(), Some(path));
    window::build(app, &Default::default(), Some(input));
    let w = app
        .windows()
        .into_iter()
        .find(|w| !before.contains(w))
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    wait(|| w.is_mapped());
    w
}
fn loaded(w: &adw::ApplicationWindow) -> gtk::Picture {
    let p = picture(w);
    wait(|| {
        p.paintable().is_some()
            || p.next_sibling()
                .unwrap()
                .downcast::<gtk::Label>()
                .unwrap()
                .text()
                .starts_with("Unable")
    });
    assert!(
        p.paintable().is_some(),
        "valid image failed after secure baseline"
    );
    p
}
fn native_isolation(app: &adw::Application, fixtures: &Path) {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let cases = [
        ("1.png", "large.png"),
        ("2.png", "tall.png"),
        ("3.png", "broken.png"),
        ("4.png", "small.png"),
    ];
    for (name, source) in cases {
        std::fs::copy(fixtures.join(source), a.path().join(name)).unwrap();
    }
    std::fs::copy(fixtures.join("tall.png"), b.path().join("only.png")).unwrap();
    let first = open_window(app, &a.path().join("1.png"));
    let second = open_window(app, &b.path().join("only.png"));
    wait(|| first.title().is_some_and(|s| s.ends_with("(4 candidates)")));
    let p = loaded(&first);
    let q = loaded(&second);
    if std::env::args().any(|s| s == "--zoom-pan") {
        zoom_pan_cases::native(&first);
        assert_eq!(state(&second), "fit");
        zoom_pan_cases::native(&second);
        assert_eq!(state(&first), "fit");
    }
    select(&first, ViewMode::Actual);
    select(&second, ViewMode::Fill);
    let survivor = q.paintable();
    render(&first, &p, (1600, 1200), ViewMode::Actual, false);
    render(&second, &q, (32, 64), ViewMode::Fill, false);
    first.set_default_size(480, 320);
    render(&first, &p, (1600, 1200), ViewMode::Actual, false);
    gio::prelude::ActionGroupExt::activate_action(&first, "next-media", None);
    assert_eq!(state(&first), "fit");
    loaded(&first);
    render(&first, &p, (32, 64), ViewMode::Fit, false);
    select(&first, ViewMode::Fill);
    gio::prelude::ActionGroupExt::activate_action(&first, "next-media", None);
    wait(|| {
        p.next_sibling()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap()
            .text()
            .starts_with("Unable")
    });
    assert!(p.paintable().is_none());
    assert!(
        !p.next_sibling()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap()
            .text()
            .contains("Loader process exited early"),
        "loader failure is not invalid-media success"
    );
    gio::prelude::ActionGroupExt::activate_action(&first, "next-media", None);
    assert_eq!(state(&first), "fit");
    loaded(&first);
    render(&first, &p, (4, 2), ViewMode::Fit, false);
    assert_eq!(state(&second), "fill");
    assert_eq!(q.paintable(), survivor);
    select(&first, ViewMode::Actual);
    first.close();
    drop(first);
    settle();
    assert!(second.is_visible());
    assert_eq!(state(&second), "fill");
    render(&second, &q, (32, 64), ViewMode::Fill, false);
    second.close();
    for (name, source) in cases {
        assert_eq!(
            std::fs::read(a.path().join(name)).unwrap(),
            std::fs::read(fixtures.join(source)).unwrap()
        );
    }
    assert_eq!(
        std::fs::read(b.path().join("only.png")).unwrap(),
        std::fs::read(fixtures.join("tall.png")).unwrap()
    );
    println!(
        "PASS secure two-window mode isolation, resize, new-media Fit reset, broken-to-valid recovery and mode/close"
    );
}
fn native(app: &adw::Application) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images");
    let mut baseline = false;
    for name in [
        "large.png",
        "small.png",
        "tall.png",
        "orientation-1.jpg",
        "orientation-2.jpg",
        "orientation-3.jpg",
        "orientation-4.jpg",
        "orientation-5.jpg",
        "orientation-6.jpg",
        "orientation-7.jpg",
        "orientation-8.jpg",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        let original = std::fs::read(root.join(name)).unwrap();
        std::fs::write(&path, &original).unwrap();
        let input = input::OpenedFile::from_argument(path.as_os_str()).unwrap();
        assert_eq!(input.file.path().as_deref(), Some(path.as_path()));
        window::build(app, &Default::default(), Some(input));
        let w = app.windows()[0]
            .clone()
            .downcast::<adw::ApplicationWindow>()
            .unwrap();
        let p = picture(&w);
        wait(|| w.is_mapped());
        // Select during the load request; secure decoder timing is not certified.
        select(&w, ViewMode::Fill);
        wait(|| {
            p.paintable().is_some()
                || p.next_sibling()
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap()
                    .text()
                    .starts_with("Unable")
        });
        if p.paintable().is_none() && !baseline {
            eprintln!("NOT RUN secure baseline decode");
            std::process::exit(77);
        }
        assert!(
            p.paintable().is_some(),
            "valid secure image failed after baseline"
        );
        baseline = true;
        let decoded = p.paintable().unwrap().downcast::<gdk::Texture>().unwrap();
        let size = (decoded.width(), decoded.height());
        let expected = match name {
            "large.png" => (1600, 1200),
            "small.png" => (4, 2),
            "tall.png" => (32, 64),
            _ if name.contains("-5.")
                || name.contains("-6.")
                || name.contains("-7.")
                || name.contains("-8.") =>
            {
                (32, 64)
            }
            _ => (64, 32),
        };
        assert_eq!(size, expected, "EXIF-transformed source dimensions");
        if std::env::args().any(|s| s == "--resampling") {
            resampling_cases::native(&w);
            select(&w, ViewMode::Fill);
        }
        if std::env::args().any(|s| s == "--zoom-pan") {
            zoom_pan_cases::native(&w);
            select(&w, ViewMode::Fill);
        }
        assert_eq!(state(&w), "fill");
        for (vw, vh) in [(800, 600), (320, 700)] {
            w.set_default_size(vw, vh);
            settle();
            for mode in [
                ViewMode::Fit,
                ViewMode::Fill,
                ViewMode::Actual,
                ViewMode::Fit,
            ] {
                select(&w, mode);
                assert_eq!(state(&w), mode.id());
                render(&w, &p, size, mode, false);
            }
        }
        assert_eq!(
            p.paintable().unwrap(),
            decoded.upcast::<gdk::Paintable>(),
            "mode switch reloaded/replaced source texture"
        );
        assert_eq!(std::fs::read(path).unwrap(), original);
        w.close();
        drop(w);
        settle();
    }
    if std::env::args().any(|s| s == "--resampling") {
        resampling_cases::native_checker(app);
    }
    native_isolation(app, &root);
    println!(
        "PASS secure production mode/render/resize/EXIF 1–8/source integrity gate; physical GNOME menu/keys remain manual"
    );
}
fn main() {
    adw::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.ModesProbe")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    if std::env::args().any(|s| s == "--synthetic") {
        if std::env::args().any(|s| s == "--resampling") {
            resampling_cases::synthetic(&app);
        } else if std::env::args().any(|s| s == "--zoom-pan") {
            zoom_pan_cases::synthetic(&app);
        } else {
            synthetic(&app);
        }
    } else {
        native(&app);
    }
}
