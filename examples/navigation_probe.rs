//! Production window/controller/view integration. Default requires real secure
//! pixels; --expect-image-error is scoped GUI/error coverage, not image PASS.
#[path = "../src/image_loader.rs"]
mod image_loader;
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/input.rs"]
mod input;
#[path = "../src/window.rs"]
mod window;

use adw::prelude::*;
use gtk::{gdk, glib};
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn wait(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "navigation GUI deadline exceeded"
        );
        for _ in 0..100 {
            if !glib::MainContext::default().pending() {
                break;
            }
            glib::MainContext::default().iteration(false);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn widgets(widget: &gtk::Widget) -> Vec<gtk::Widget> {
    let mut result = vec![widget.clone()];
    let mut child = widget.first_child();
    while let Some(current) = child {
        result.extend(widgets(&current));
        child = current.next_sibling();
    }
    result
}

fn picture(window: &adw::ApplicationWindow) -> gtk::Picture {
    widgets(window.upcast_ref())
        .into_iter()
        .find_map(|w| w.downcast::<gtk::Picture>().ok())
        .unwrap()
}

fn status(window: &adw::ApplicationWindow) -> gtk::Label {
    picture(window)
        .next_sibling()
        .unwrap()
        .downcast::<gtk::Label>()
        .unwrap()
}

fn title(window: &adw::ApplicationWindow, name: &str, count: usize) {
    assert_eq!(
        window.title().unwrap().as_str(),
        format!("{name} — Pivra ({count} candidates)")
    );
}

fn key(window: &adw::ApplicationWindow, right: bool) {
    let controllers = window.observe_controllers();
    let keys = (0..controllers.n_items())
        .find_map(|i| {
            controllers
                .item(i)
                .unwrap()
                .downcast::<gtk::EventControllerKey>()
                .ok()
        })
        .unwrap();
    assert_eq!(keys.propagation_phase(), gtk::PropagationPhase::Capture);
    let consumed = keys.emit_by_name::<bool>(
        "key-pressed",
        &[
            &if right {
                gdk::Key::Right
            } else {
                gdk::Key::Left
            },
            &0_u32,
            &gdk::ModifierType::empty(),
        ],
    );
    assert!(consumed, "arrow handler must consume navigation keys");
}

fn open(app: &adw::Application, path: &Path, count: usize) -> adw::ApplicationWindow {
    let previous = app.windows();
    let input = input::OpenedFile::from_argument(path.as_os_str()).unwrap();
    assert_eq!(input.file.path().as_deref(), Some(path));
    window::build(app, &pivra::settings::Settings::default(), Some(input));
    let result = app
        .windows()
        .into_iter()
        .find(|w| !previous.contains(w))
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    wait(|| {
        result.title().is_some_and(|s| {
            s.ends_with(&format!(
                "({count} {})",
                if count == 1 {
                    "candidate"
                } else {
                    "candidates"
                }
            ))
        })
    });
    wait(|| result.is_mapped() && result.surface().is_some_and(|surface| surface.is_mapped()));
    result
}

fn image(window: &adw::ApplicationWindow, blocked: bool, size: (i32, i32)) {
    let picture = picture(window);
    let label = status(window);
    wait(|| picture.paintable().is_some() || label.text().starts_with("Unable to display image."));
    if blocked {
        assert!(picture.paintable().is_none());
        assert!(label.text().starts_with("Unable to display image."));
        println!("SCOPED PASS current controlled image error; pixels BLOCKED");
    } else {
        if picture.paintable().is_none() {
            eprintln!("BLOCKED real navigation pixels: {}", label.text());
            std::process::exit(77);
        }
        let texture = picture
            .paintable()
            .unwrap()
            .downcast::<gdk::Texture>()
            .unwrap();
        assert_eq!((texture.width(), texture.height()), size);
        assert!(!label.is_visible());
        let stride = texture.width() as usize * 4;
        let mut pixels = vec![0; stride * texture.height() as usize];
        texture.download(&mut pixels, stride);
        for (index, rgb) in [[255_u8, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 0]]
            .into_iter()
            .enumerate()
        {
            let x = if index % 2 == 0 {
                size.0 / 4
            } else {
                3 * size.0 / 4
            };
            let y = if index < 2 {
                size.1 / 4
            } else {
                3 * size.1 / 4
            };
            let offset = y as usize * stride + x as usize * 4;
            let actual = [pixels[offset + 2], pixels[offset + 1], pixels[offset]];
            assert!(actual.iter().zip(rgb).all(|(a, b)| a.abs_diff(b) <= 12));
            assert_eq!(pixels[offset + 3], 255);
        }
        assert_eq!(picture.content_fit(), gtk::ContentFit::ScaleDown);
        geometry(window, &picture);
    }
}

fn geometry(window: &adw::ApplicationWindow, picture: &gtk::Picture) {
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
    wait(|| frames.get() >= 3);
    let (w, h) = (picture.width(), picture.height());
    let paintable = picture.paintable().unwrap();
    let scale = (w as f64 / paintable.intrinsic_width() as f64)
        .min(h as f64 / paintable.intrinsic_height() as f64)
        .min(1.);
    let snapshot = gtk::Snapshot::new();
    gtk::WidgetPaintable::new(Some(picture)).snapshot(&snapshot, w as f64, h as f64);
    let texture = window.renderer().unwrap().render_texture(
        snapshot.to_node().unwrap(),
        Some(&gtk::graphene::Rect::new(0., 0., w as f32, h as f32)),
    );
    let mut data = vec![0; w as usize * h as usize * 4];
    texture.download(&mut data, w as usize * 4);
    let (mut left, mut top, mut right, mut bottom) = (w, h, -1, -1);
    for y in 0..h {
        for x in 0..w {
            if data[((y * w + x) * 4 + 3) as usize] > 127 {
                left = left.min(x);
                right = right.max(x);
                top = top.min(y);
                bottom = bottom.max(y);
            }
        }
    }
    assert!((f64::from(right - left + 1) - paintable.intrinsic_width() as f64 * scale).abs() <= 2.);
    assert!(
        (f64::from(bottom - top + 1) - paintable.intrinsic_height() as f64 * scale).abs() <= 2.
    );
    assert!((f64::from(left + right + 1 - w) / 2.).abs() <= 1.);
    assert!((f64::from(top + bottom + 1 - h) / 2.).abs() <= 1.);
    println!("PASS real selected-image snapshot Fit/center {w}x{h}");
}

fn video(window: &adw::ApplicationWindow) {
    assert!(picture(window).paintable().is_none());
    assert_eq!(
        status(window).text().as_str(),
        "Video playback is not available yet."
    );
    assert!(status(window).is_visible());
}

fn invalid(window: &adw::ApplicationWindow, missing: bool) {
    wait(|| {
        status(window)
            .text()
            .starts_with("Unable to display image.")
    });
    assert!(picture(window).paintable().is_none());
    if missing {
        assert!(status(window).text().contains("Cannot access image"));
    }
}

fn main() {
    const {
        assert!(cfg!(target_endian = "little"));
    }
    if let Err(error) = adw::init() {
        eprintln!("BLOCKED GUI: {error}");
        std::process::exit(77);
    }
    let blocked = std::env::args().any(|a| a == "--expect-image-error");
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.NavigationProbe")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("mixed");
    std::fs::create_dir(&root).unwrap();
    let single = temporary.path().join("single");
    std::fs::create_dir(&single).unwrap();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut original = Vec::new();
    for (name, source) in [
        ("1.png", "images/large.png"),
        ("2.mp4", "model-video.txt"),
        ("10.jpg", "images/no-exif.jpg"),
        ("11.png", "images/broken.png"),
        ("12.png", "images/wide.png"),
        ("20.png", "images/tall.png"),
    ] {
        let bytes = std::fs::read(fixtures.join(source)).unwrap();
        std::fs::write(root.join(name), &bytes).unwrap();
        original.push((root.join(name), bytes));
    }
    std::fs::copy(fixtures.join("images/small.png"), single.join("only.png")).unwrap();
    let first = open(&app, &root.join("10.jpg"), 6);
    let second = open(&app, &single.join("only.png"), 1);
    title(&first, "10.jpg", 6);
    image(&first, blocked, (64, 32));
    image(&second, blocked, (4, 2));
    let survivor = picture(&second).paintable();
    let survivor_status = status(&second).text();
    key(&second, false);
    key(&second, true);
    assert_eq!(picture(&second).paintable(), survivor);
    assert_eq!(status(&second).text(), survivor_status);
    // Scanner snapshot intentionally retains this now-deleted private file.
    std::fs::remove_file(root.join("12.png")).unwrap();
    key(&first, false);
    title(&first, "2.mp4", 6);
    video(&first);
    key(&first, false);
    title(&first, "1.png", 6);
    image(&first, blocked, (1600, 1200));
    let before = picture(&first).paintable();
    key(&first, false);
    title(&first, "1.png", 6);
    assert_eq!(picture(&first).paintable(), before);
    for _ in 0..8 {
        key(&first, true);
        key(&first, true);
        key(&first, false);
        key(&first, false);
    }
    title(&first, "1.png", 6);
    image(&first, blocked, (1600, 1200));
    key(&first, true);
    video(&first);
    key(&first, true);
    title(&first, "10.jpg", 6);
    image(&first, blocked, (64, 32));
    key(&first, true);
    title(&first, "11.png", 6);
    invalid(&first, false);
    key(&first, true);
    title(&first, "12.png", 6);
    invalid(&first, true);
    key(&first, true);
    title(&first, "20.png", 6);
    image(&first, blocked, (32, 64));
    let before = picture(&first).paintable();
    key(&first, true);
    title(&first, "20.png", 6);
    assert_eq!(picture(&first).paintable(), before);
    // Drive old work to completion after the final selection; latest stays current.
    let grace = Instant::now() + Duration::from_millis(300);
    while Instant::now() < grace {
        for _ in 0..100 {
            if !glib::MainContext::default().pending() {
                break;
            }
            glib::MainContext::default().iteration(false);
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    title(&first, "20.png", 6);
    image(&first, blocked, (32, 64));
    assert_eq!(picture(&second).paintable(), survivor);
    assert!(second.is_visible());
    first.close();
    wait(|| !first.is_visible());
    assert!(second.is_visible());
    second.close();
    wait(|| !second.is_visible());
    for (path, bytes) in original {
        if path.file_name().unwrap() != "12.png" {
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
    }
    println!(
        "PASS navigation controller/catalog: selected opening, natural mixed order, boundaries, rapid arrows, error recovery, independent windows, normal close, source bytes unchanged"
    );
    if blocked {
        println!(
            "BLOCKED real navigation pixels/Fit/stale successful-frame presentation; error-path scope only"
        );
    } else {
        println!(
            "PASS real secure navigation pixels/Fit; physical Wayland key/focus interaction still requires manual confirmation"
        );
    }
}
