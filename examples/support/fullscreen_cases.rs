//! Real GTK/GDK fullscreen acknowledgement; synthetic pixels unless native flag.
use super::*;
use pivra::image_pipeline::Phase;
use std::path::Path;

fn key_event(w: &adw::ApplicationWindow, key: gdk::Key) -> bool {
    w.observe_controllers()
        .iter::<glib::Object>()
        .filter_map(Result::ok)
        .filter_map(|c| c.downcast::<gtk::EventControllerKey>().ok())
        .any(|c| c.emit_by_name::<bool>("key-pressed", &[&key, &0u32, &gdk::ModifierType::empty()]))
}
fn actual(w: &adw::ApplicationWindow) -> bool {
    w.surface()
        .unwrap()
        .downcast::<gdk::Toplevel>()
        .unwrap()
        .state()
        .contains(gdk::ToplevelState::FULLSCREEN)
}
fn fullscreen(w: &adw::ApplicationWindow, p: &gtk::Picture, header: &adw::HeaderBar, yes: bool) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !(w.is_fullscreen() == yes && actual(w) == yes && header.is_visible() != yes) {
        assert!(
            Instant::now() < deadline,
            "fullscreen ack target={yes} title={:?} gtk={} gdk={} header={} active={}",
            w.title(),
            w.is_fullscreen(),
            actual(w),
            header.is_visible(),
            w.is_active()
        );
        pump();
    }
    if p.paintable().is_some() {
        image_bytes(w, p);
    }
    if yes {
        let surface = w.surface().unwrap();
        let monitor = surface.display().monitor_at_surface(&surface).unwrap();
        let area = monitor.geometry();
        // GDK_SCALE may force a surface scale independently of the output scale.
        // Compare the same device coordinate space, not unlike logical sizes.
        let expected = (
            area.width() * monitor.scale_factor(),
            area.height() * monitor.scale_factor(),
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        while (
            surface.width() * surface.scale_factor(),
            surface.height() * surface.scale_factor(),
        ) != expected
        {
            assert!(
                Instant::now() < deadline,
                "fullscreen surface={}x{} scale={} monitor={area:?} scale={}",
                surface.width(),
                surface.height(),
                surface.scale_factor(),
                monitor.scale_factor()
            );
            pump();
        }
        assert_eq!(
            (p.width(), p.height()),
            (w.width(), w.height()),
            "chrome still allocates photo space"
        );
    }
    assert_eq!(
        w.lookup_action("fullscreen")
            .unwrap()
            .state()
            .unwrap()
            .get::<bool>(),
        Some(yes)
    );
}
fn double(p: &gtk::Picture) {
    let gesture = controller::<gtk::GestureClick>(p);
    gesture.emit_by_name::<()>("pressed", &[&2i32, &20f64, &20f64]);
}
fn mode(w: &adw::ApplicationWindow) -> String {
    w.lookup_action("image-mode")
        .unwrap()
        .state()
        .unwrap()
        .str()
        .unwrap()
        .to_owned()
}
fn find_picture(root: &gtk::Widget) -> gtk::Picture {
    if let Ok(p) = root.clone().downcast::<gtk::Picture>() {
        return p;
    }
    let mut c = root.first_child();
    while let Some(w) = c {
        if let Some(p) = find_picture_optional(&w) {
            return p;
        }
        c = w.next_sibling();
    }
    panic!("production picture missing");
}
fn find_picture_optional(root: &gtk::Widget) -> Option<gtk::Picture> {
    if let Ok(p) = root.clone().downcast::<gtk::Picture>() {
        return Some(p);
    }
    let mut c = root.first_child();
    while let Some(w) = c {
        if let Some(p) = find_picture_optional(&w) {
            return Some(p);
        }
        c = w.next_sibling();
    }
    None
}
pub(super) fn run(app: &adw::Application) {
    let native = std::env::args().any(|a| a == "--native-thumbnails");
    let (w, v, p) = open(app);
    let source = Path::new("tests/fixtures/images/large.png")
        .canonicalize()
        .unwrap();
    v.load(source);
    wait(|| matches!(v.phase(), Phase::FullReady | Phase::Failed));
    assert_eq!(
        v.phase(),
        Phase::FullReady,
        "fullscreen image failed; native script checks the secure prerequisite separately"
    );
    let (other, ov, op) = open(app);
    if native {
        ov.load(
            Path::new("tests/fixtures/images/tall.png")
                .canonicalize()
                .unwrap(),
        );
        wait(|| matches!(ov.phase(), Phase::FullReady | Phase::Failed));
        assert_eq!(
            ov.phase(),
            Phase::FullReady,
            "second real image failed after secure baseline"
        );
    }
    image_bytes(&w, &p);
    let header: adw::HeaderBar = find(w.upcast_ref(), "th02-test-header");
    let other_header: adw::HeaderBar = find(other.upcast_ref(), "th02-test-header");
    let scale = p.scale_factor();
    if std::env::args().any(|a| a == "--expect-scale=2") {
        assert_eq!(scale, 2, "real output scaling was not applied");
    }
    let normal = (w.width(), w.height());
    let content = w.content();
    let tree = v.widget().clone();
    let loads = LOADS.load(Ordering::SeqCst);
    let preloads = v.preload_stats().started;
    assert!(
        !key_event(&w, gdk::Key::Escape),
        "normal Esc must propagate"
    );
    for name in ["fit", "fill", "actual"] {
        gio::prelude::ActionGroupExt::activate_action(&w, "image-mode", Some(&name.to_variant()));
        image_bytes(&w, &p);
        let paintable = p.paintable();
        assert!(key_event(&w, gdk::Key::f));
        fullscreen(&w, &p, &header, true);
        assert!(!actual(&other) && other_header.is_visible());
        assert_eq!(p.paintable(), paintable);
        assert_eq!(mode(&w), name);
        assert_eq!(w.content(), content);
        assert_eq!(v.widget(), &tree);
        assert!(key_event(&w, gdk::Key::Escape));
        fullscreen(&w, &p, &header, false);
        wait(|| (w.width(), w.height()) == normal);
        assert_eq!(p.paintable(), paintable);
        assert_eq!(mode(&w), name);
    }
    // An absolute manual scale large enough for both viewports avoids clamping,
    // so a round trip must preserve the complete original rendered pixels.
    gio::prelude::ActionGroupExt::activate_action(
        &w,
        "image-resampling",
        Some(&"nearest".to_variant()),
    );
    controller::<gtk::EventControllerMotion>(&p).emit_by_name::<()>("motion", &[&200f64, &200f64]);
    controller::<gtk::EventControllerScroll>(&p).emit_by_name::<bool>("scroll", &[&0f64, &-5f64]);
    let drag = controller::<gtk::GestureDrag>(&p);
    drag.emit_by_name::<()>("drag-begin", &[&200f64, &200f64]);
    drag.emit_by_name::<()>("drag-update", &[&-30f64, &-20f64]);
    drag.emit_by_name::<()>("drag-end", &[&-30f64, &-20f64]);
    let before = image_bytes(&w, &p);
    let paintable = p.paintable();
    assert_eq!(mode(&w), "manual");
    double(&p);
    fullscreen(&w, &p, &header, true);
    assert_eq!(mode(&w), "manual");
    assert_eq!(p.paintable(), paintable);
    double(&p);
    fullscreen(&w, &p, &header, false);
    wait(|| (w.width(), w.height()) == normal);
    assert_eq!(
        before,
        image_bytes(&w, &p),
        "zoom/pan pixels changed on fullscreen round trip"
    );
    assert_eq!(
        w.lookup_action("image-resampling")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("nearest")
    );
    assert_eq!(LOADS.load(Ordering::SeqCst), loads);
    assert_eq!(v.preload_stats().started, preloads);

    // Popup/text input must keep its typing; fullscreen Esc still wins.
    let entry = gtk::Entry::new();
    v.widget().add_overlay(&entry);
    entry.grab_focus();
    assert!(!key_event(&w, gdk::Key::f));
    v.widget().remove_overlay(&entry);
    p.set_focusable(true);
    p.grab_focus();
    for _ in 0..9 {
        assert!(key_event(&w, gdk::Key::f));
    }
    fullscreen(&w, &p, &header, true);
    for _ in 0..10 {
        assert!(key_event(&w, gdk::Key::f));
    }
    assert!(key_event(&w, gdk::Key::Escape));
    fullscreen(&w, &p, &header, false);
    if std::env::args().any(|a| a == "--x11-input") {
        x11_input(&w, &v, &p, &header);
    }
    assert!(key_event(&other, gdk::Key::f));
    fullscreen(&other, &op, &other_header, true);
    assert!(!actual(&w));
    other.close();
    assert!(w.is_visible());
    w.close();
    drop(ov);
    drop(v);
    production(app, native);
    println!(
        "PASS TH-07 real compositor/GDK fullscreen, F/Esc/double-click, header/viewport/normal size, Fit-Fill-Actual/manual zoom-pan/resampling pixels, same paintable/tree, no added foreground/preload jobs, rapid requests and independent windows; native-pixels={native}, scale_factor={scale}"
    );
}

fn production(app: &adw::Application, native: bool) {
    let root = tempfile::tempdir().unwrap();
    for (name, data) in [
        (
            "0.png",
            include_bytes!("../../tests/fixtures/images/wide.png").as_slice(),
        ),
        (
            "1.jpg",
            include_bytes!("../../tests/fixtures/images/no-exif.jpg").as_slice(),
        ),
        (
            "2.png",
            include_bytes!("../../tests/fixtures/images/tall.png").as_slice(),
        ),
        ("broken.png", b"broken PNG".as_slice()),
    ] {
        let path = root.path().join(name);
        std::fs::write(&path, data).unwrap();
        catalog_probe::cache(&path);
    }
    for n in 3..24 {
        let path = root.path().join(format!("{n}.png"));
        std::fs::write(
            &path,
            include_bytes!("../../tests/fixtures/images/wide.png"),
        )
        .unwrap();
        catalog_probe::cache(&path);
    }
    let old = app.windows();
    window::build(
        app,
        &pivra::settings::Settings::default(),
        Some(input::OpenedFile::from_argument(root.path().join("0.png").as_os_str()).unwrap()),
    );
    let w = app
        .windows()
        .into_iter()
        .find(|w| !old.contains(w))
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    let header: adw::HeaderBar = find(w.upcast_ref(), "pivra-header");
    let p = find_picture(w.upcast_ref());
    wait(|| p.paintable().is_some() && w.title().is_some_and(|t| t.ends_with("(25 candidates)")));
    let strip: gtk::ScrolledWindow = find(w.upcast_ref(), "thumbnail-strip");
    let button: gtk::ToggleButton = find(w.upcast_ref(), "thumbnail-toggle");
    // Decoder/scan completion is not the initial mapped WM configure/frame.
    image_bytes(&w, &p);
    wait(|| w.surface().is_some_and(|s| s.is_mapped()) && header.is_mapped());
    assert!(key_event(&w, gdk::Key::f));
    fullscreen(&w, &p, &header, true);
    // Hide the strip by the real T controller, then navigate without revealing it.
    if pivra::settings::thumbnail_preference().enabled() {
        assert!(key_event(&w, gdk::Key::t));
    }
    assert!(!strip.is_visible());
    assert!(key_event(&w, gdk::Key::Right));
    wait(|| w.title().unwrap().starts_with("1.jpg"));
    assert!(!strip.is_visible() && actual(&w));
    assert!(key_event(&w, gdk::Key::t));
    // Use the actual overlay controller (production windows do not detach it).
    let overlay = p.parent().unwrap();
    controller::<gtk::EventControllerMotion>(&overlay)
        .emit_by_name::<()>("motion", &[&20f64, &(overlay.height() as f64 - 4.)]);
    image_bytes(&w, &p);
    wait(|| strip.is_visible() && strip.hadjustment().upper() > 0.);
    let bounds = strip.compute_bounds(&overlay).unwrap();
    let toggle = button.compute_bounds(&overlay).unwrap();
    assert!((bounds.y() + bounds.height() - overlay.height() as f32).abs() <= 1.);
    assert!(
        (bounds.y() - toggle.y() - toggle.height()).abs() <= 1.,
        "TH-06 flush toggle regressed"
    );
    let row: gtk::Box = find(w.upcast_ref(), "thumbnail-items");
    wait(|| row.first_child().is_some());
    for _ in 0..18 {
        assert!(key_event(&w, gdk::Key::Right));
    }
    wait(|| w.title().unwrap().starts_with("19.png"));
    image_bytes(&w, &p);
    let selected: gtk::Button = find(w.upcast_ref(), "thumbnail-cell-19");
    wait(|| {
        let b = selected.compute_bounds(&row).unwrap();
        f64::from(b.x()) >= strip.hadjustment().value() - 1.
            && f64::from(b.x() + b.width())
                <= strip.hadjustment().value() + strip.hadjustment().page_size() + 1.
    });
    let selected_overlay: gtk::Overlay = selected
        .child()
        .expect("selected thumbnail content")
        .downcast()
        .expect("thumbnail content overlay");
    let selection = selected_overlay
        .first_child()
        .and_then(|child| {
            let mut widget = Some(child);
            while let Some(current) = widget {
                if current.has_css_class("pivra-thumbnail-selection") {
                    return Some(current);
                }
                widget = current.next_sibling();
            }
            None
        })
        .expect("thumbnail selection frame");
    assert!(selection.has_css_class("selected"));
    let cell: gtk::Button = find(w.upcast_ref(), "thumbnail-cell-18");
    let cb = cell.compute_bounds(&overlay).unwrap();
    let target = overlay
        .pick(
            f64::from(cb.x() + cb.width() / 2.),
            f64::from(cb.y() + cb.height() / 2.),
            gtk::PickFlags::DEFAULT,
        )
        .unwrap();
    assert!(
        target != p && !target.is_ancestor(&p),
        "thumbnail click can reach photo gesture"
    );
    cell.emit_clicked();
    cell.emit_clicked();
    assert!(w.title().unwrap().starts_with("18.png"));
    assert!(actual(&w));
    button.emit_clicked();
    button.emit_clicked();
    assert!(actual(&w));
    // Dispatch fullscreen while the new request is still pending in this context.
    gio::prelude::ActionGroupExt::activate_action(&w, "next-media", None);
    key_event(&w, gdk::Key::Escape);
    fullscreen(&w, &p, &header, false);
    assert!(key_event(&w, gdk::Key::f));
    fullscreen(&w, &p, &header, true);
    for _ in 0..30 {
        gio::prelude::ActionGroupExt::activate_action(&w, "next-media", None);
    }
    let label = p.next_sibling().unwrap().downcast::<gtk::Label>().unwrap();
    wait(|| label.is_visible() && label.text().starts_with("Unable to display image."));
    assert!(actual(&w));
    assert!(key_event(&w, gdk::Key::Escape));
    fullscreen(&w, &p, &header, false);
    std::fs::remove_file(root.path().join("broken.png")).unwrap();
    wait(|| w.title().unwrap().starts_with("No media selected"));
    assert!(p.paintable().is_none());
    assert!(key_event(&w, gdk::Key::f));
    fullscreen(&w, &p, &header, true);
    assert!(key_event(&w, gdk::Key::Escape));
    fullscreen(&w, &p, &header, false);
    gio::prelude::ActionGroupExt::activate_action(&w, "next-media", None);
    if std::env::args().any(|a| a == "--manual") {
        println!(
            "MANUAL GNOME TH-07: PRIVATE TEST media. F/Esc/photo double-click; thumbnail/toggle/header double-click must not toggle fullscreen. T, arrows, scroll, zoom/pan, two-window independence and restored normal size. No physical input is certified automatically."
        );
        // A second real production window for physical independence acceptance.
        window::build(
            app,
            &pivra::settings::Settings::default(),
            Some(input::OpenedFile::from_argument(root.path().join("2.png").as_os_str()).unwrap()),
        );
        let loop_ = glib::MainLoop::new(None, false);
        let windows: Vec<_> = app
            .windows()
            .into_iter()
            .filter(|w| w.is_visible())
            .collect();
        let count = std::rc::Rc::new(std::cell::Cell::new(windows.len()));
        for window in windows {
            let loop_ = loop_.clone();
            let count = count.clone();
            window.connect_close_request(move |_| {
                count.set(count.get().saturating_sub(1));
                if count.get() == 0 {
                    loop_.quit();
                }
                glib::Propagation::Proceed
            });
        }
        loop_.run();
    } else {
        assert!(key_event(&w, gdk::Key::f));
        fullscreen(&w, &p, &header, true);
        gio::prelude::ActionGroupExt::activate_action(&w, "next-media", None);
        w.close();
        wait(|| !w.is_visible());
    }
    println!(
        "PASS production window fullscreen + T/hidden-arrow navigation/overlay picks, thumbnail click and controlled/real image errors; native={native}. Active secure decoder timing and physical pointer routing NOT TESTED."
    );
}

// Optional XTest event routing test in the existing private Xvfb/Xfwm session.
// Never required by the Wayland/native entry point; no new dependency installed.
fn x11_input(
    w: &adw::ApplicationWindow,
    v: &image_view::ImageView,
    p: &gtk::Picture,
    header: &adw::HeaderBar,
) {
    fn xdo(args: &[&str]) -> String {
        let result = std::process::Command::new("xdotool")
            .args(args)
            .output()
            .expect("existing xdotool required for --x11-input");
        assert!(
            result.status.success(),
            "XTest command failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        String::from_utf8(result.stdout).unwrap()
    }
    fn click(w: &adw::ApplicationWindow, widget: &gtk::Widget, id: &str) {
        let b = widget.compute_bounds(w).unwrap();
        let scale = w.scale_factor() as f32;
        let x = ((b.x() + b.width() / 2.) * scale).round().to_string();
        let y = ((b.y() + b.height() / 2.) * scale).round().to_string();
        xdo(&[
            "mousemove",
            "--window",
            id,
            &x,
            &y,
            "click",
            "--repeat",
            "2",
            "--delay",
            "80",
            "1",
        ]);
    }
    controller::<gtk::GestureClick>(p)
        .connect_pressed(|_, n, x, y| eprintln!("XTEST photo pressed count={n} at={x},{y}"));
    w.set_title(Some("Pivra TH07 input TEST"));
    image_bytes(w, p);
    let pid = std::process::id().to_string();
    let windows = xdo(&[
        "search",
        "--all",
        "--onlyvisible",
        "--pid",
        &pid,
        "--name",
        "^Pivra TH07 input TEST$",
    ]);
    let id = windows.lines().next().unwrap();
    xdo(&["windowactivate", "--sync", id]);
    // WM_TAKE_FOCUS still needs GTK main-context delivery after EWMH activation.
    wait(|| w.is_active());
    println!(
        "XTEST keyboard F id={id} focus={} active={}",
        xdo(&["getwindowfocus"]),
        xdo(&["getactivewindow"])
    );
    xdo(&["key", "f"]);
    fullscreen(w, p, header, true);
    xdo(&["key", "Escape"]);
    fullscreen(w, p, header, false);
    println!("XTEST photo double-click");
    click(w, p.upcast_ref(), id);
    fullscreen(w, p, header, true);
    set_enabled(v, true);
    image_bytes(w, p);
    let row: gtk::Box = find(w.upcast_ref(), "thumbnail-items");
    click(w, &row.first_child().unwrap(), id);
    image_bytes(w, p);
    assert!(actual(w), "XTest thumbnail double-click toggled fullscreen");
    let button: gtk::ToggleButton = find(w.upcast_ref(), "thumbnail-toggle");
    click(w, button.upcast_ref(), id);
    image_bytes(w, p);
    assert!(actual(w), "XTest toggle double-click toggled fullscreen");
    xdo(&["key", "Escape"]);
    fullscreen(w, p, header, false);
    click(w, header.upcast_ref(), id);
    image_bytes(w, p);
    assert!(!actual(w), "header double-click toggled fullscreen");
    println!(
        "PASS real XTest keyboard F/Esc and photo/thumbnail/toggle/header double-click routing under Xfwm; not physical GNOME input"
    );
}
