//! Small controlled GTK regressions for the next development cycle.
use super::*;
fn widget(window: &adw::ApplicationWindow, name: &str) -> gtk::Widget {
    widgets(window.upcast_ref())
        .into_iter()
        .find(|w| w.widget_name() == name)
        .unwrap()
}
fn open_uri(window: &adw::ApplicationWindow, path: &std::path::Path) {
    window
        .lookup_action("open-file")
        .unwrap()
        .activate(Some(&gio::File::for_path(path).uri().to_variant()));
}
pub(super) fn open(app: &adw::Application) {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("first");
    let b = dir.path().join("second");
    std::fs::create_dir(&a).unwrap();
    std::fs::create_dir(&b).unwrap();
    std::fs::copy("tests/fixtures/images/wide.png", a.join("1.png")).unwrap();
    std::fs::copy("tests/fixtures/images/wide.png", b.join("2.png")).unwrap();
    window::build(app, &pivra::settings::Settings::default(), None);
    let window = app
        .active_window()
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    assert_eq!(app.windows().len(), 1);
    let button = widget(&window, "open-media-button")
        .downcast::<gtk::Button>()
        .unwrap();
    assert_eq!(button.action_name().as_deref(), Some("win.choose-media"));
    assert!(window.lookup_action("choose-media").unwrap().is_enabled());
    let controls = widget(&window, "viewer-controls");
    assert!(!controls.is_visible());
    open_uri(&window, &a.join("1.png"));
    wait("empty-window Open initializes catalog", || {
        window
            .title()
            .is_some_and(|s| s.starts_with("1.png") && s.contains("1 candidate"))
    });
    photo_presented(&window);
    assert!(controls.is_visible());
    open_uri(&window, &b.join("2.png"));
    wait("existing Open reuses same window", || {
        window
            .title()
            .is_some_and(|s| s.starts_with("2.png") && s.contains("1 candidate"))
    });
    photo_presented(&window);
    assert_eq!(widget(&window, "viewer-controls"), controls);
    assert_eq!(app.windows().len(), 1);
    std::fs::copy("tests/fixtures/images/wide.png", a.join("5.png")).unwrap();
    std::fs::copy("tests/fixtures/images/wide.png", b.join("4.png")).unwrap();
    wait("replacement folder monitor updates catalog", || {
        window
            .title()
            .is_some_and(|s| s.starts_with("2.png") && s.contains("2 candidates"))
    });
    key(&window, gtk::gdk::Key::Right);
    assert!(selected(&window, "4.png"));
    photo_presented(&window);
    window.close();
    eprintln!(
        "PASS development Open: empty/viewer URI action, same window/state, replacement monitor/navigation"
    );
}
fn has_text(window: &adw::ApplicationWindow, text: &str) -> bool {
    widgets(window.upcast_ref())
        .iter()
        .filter_map(|w| w.downcast_ref::<gtk::Label>())
        .any(|l| l.text() == text)
}
pub(super) fn info(app: &adw::Application) {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("1.png");
    let b = dir.path().join("2.png");
    std::fs::copy("tests/fixtures/images/wide.png", &a).unwrap();
    // Keep one file admitted while certifying that opening the panel does not decode.
    window::build(app, &pivra::settings::Settings::default(), None);
    let window = app
        .active_window()
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    open_uri(&window, &a);
    wait("information image", || {
        window.lookup_action("image-mode").unwrap().is_enabled()
    });
    photo_presented(&window);
    let panel = widget(&window, "media-information-panel")
        .downcast::<gtk::Revealer>()
        .unwrap();
    assert!(!panel.reveals_child());
    let loads = CONTROLLED_LOADS.load(std::sync::atomic::Ordering::SeqCst);
    key(&window, gtk::gdk::Key::i);
    wait("file information and current dimensions", || {
        has_text(&window, &a.to_string_lossy())
            && has_text(&window, "64 × 32")
            && has_text(&window, "Modified")
    });
    assert!(panel.reveals_child());
    assert_eq!(
        CONTROLLED_LOADS.load(std::sync::atomic::Ordering::SeqCst),
        loads
    );
    std::fs::copy("tests/fixtures/images/wide.png", &b).unwrap();
    open_uri(&window, &b);
    open_uri(&window, &a);
    open_uri(&window, &b);
    wait("metadata retargeted after rapid opens", || {
        has_text(&window, &b.to_string_lossy()) && !has_text(&window, &a.to_string_lossy())
    });
    let video = std::env::args().find(|s| s.ends_with(".mp4")).unwrap();
    open_uri(&window, std::path::Path::new(&video));
    wait(
        "live video dimensions/duration and reused discoverer metadata",
        || {
            has_text(&window, "Duration")
                && has_text(&window, "192 × 96")
                && has_text(&window, "Codec")
        },
    );
    let probes = image_view::gpu_video::stats().probes;
    key(&window, gtk::gdk::Key::i);
    key(&window, gtk::gdk::Key::i);
    assert_eq!(image_view::gpu_video::stats().probes, probes);
    window.close();
    eprintln!(
        "PASS development info: closed default, I toggle, file/dimensions/date, stale suppression, video metadata, no extra decode/discovery"
    );
}
fn controller<T: glib::object::IsA<glib::Object> + glib::types::StaticType>(w: &gtk::Widget) -> T {
    let list = w.observe_controllers();
    (0..list.n_items())
        .find_map(|i| list.item(i).unwrap().downcast::<T>().ok())
        .expect("production controller")
}
pub(super) fn minimap(app: &adw::Application) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("1.png");
    std::fs::copy("tests/fixtures/images/wide.png", &file).unwrap();
    window::build(app, &pivra::settings::Settings::default(), None);
    let window = app
        .active_window()
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    open_uri(&window, &file);
    photo_presented(&window);
    let map = widget(&window, "image-minimap");
    let mini = widget(&window, "minimap-image")
        .downcast::<gtk::Picture>()
        .unwrap();
    let picture = widgets(window.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Overlay>().ok())
        .find_map(|o| o.child().and_downcast::<gtk::Picture>())
        .unwrap();
    window
        .lookup_action("image-mode")
        .unwrap()
        .activate(Some(&"fit".to_variant()));
    photo_presented(&window);
    assert!(!map.is_visible());
    let loads = CONTROLLED_LOADS.load(std::sync::atomic::Ordering::SeqCst);
    widget(&window, "wheel-zoom-toggle")
        .downcast::<gtk::ToggleButton>()
        .unwrap()
        .set_active(true);
    let zoom = || {
        controller::<gtk::EventControllerMotion>(picture.upcast_ref())
            .emit_by_name::<()>("enter", &[&200f64, &150f64]);
        assert!(
            controller::<gtk::EventControllerScroll>(picture.upcast_ref())
                .emit_by_name::<bool>("scroll", &[&0f64, &-20f64])
        );
        wait("cropped photo minimap", || {
            map.is_visible() && map.is_mapped()
        });
    };
    zoom();
    let source = mini
        .paintable()
        .unwrap()
        .downcast::<gtk::gdk::Texture>()
        .unwrap();
    assert_eq!((source.width(), source.height()), (64, 32));
    let invalidated = std::rc::Rc::new(std::cell::Cell::new(0));
    let count = invalidated.clone();
    let painted = picture.paintable().unwrap();
    painted.connect_invalidate_contents(move |_| count.set(count.get() + 1));
    let drag = controller::<gtk::GestureDrag>(&map);
    drag.emit_by_name::<()>("drag-begin", &[&10f64, &40f64]);
    drag.emit_by_name::<()>("drag-update", &[&120f64, &35f64]);
    drag.emit_by_name::<()>("drag-end", &[&120f64, &35f64]);
    assert!(
        invalidated.get() > 0,
        "minimap must pan the existing paintable"
    );
    assert_eq!(picture.paintable().as_ref(), Some(&painted));
    assert_eq!(
        mini.paintable()
            .unwrap()
            .downcast::<gtk::gdk::Texture>()
            .unwrap(),
        source
    );
    let old_width = picture.width();
    window.set_default_size(1000, 700);
    wait("resized viewport", || picture.width() != old_width);
    photo_presented(&window);
    assert!(map.is_visible());
    assert_eq!(
        CONTROLLED_LOADS.load(std::sync::atomic::Ordering::SeqCst),
        loads
    );
    key(&window, gtk::gdk::Key::F5);
    wait("slideshow hides minimap", || !map.is_visible());
    assert!(
        mini.paintable().is_none(),
        "hidden map releases its texture reference"
    );
    key(&window, gtk::gdk::Key::Escape);
    window
        .lookup_action("image-mode")
        .unwrap()
        .activate(Some(&"fit".to_variant()));
    photo_presented(&window);
    wait("Fit hides minimap", || !map.is_visible());
    zoom();
    let second = dir.path().join("2.png");
    std::fs::copy("tests/fixtures/images/wide.png", &second).unwrap();
    open_uri(&window, &second);
    photo_presented(&window);
    wait("new photo resets minimap", || !map.is_visible());
    let video = std::env::args().find(|s| s.ends_with(".mp4")).unwrap();
    open_uri(&window, std::path::Path::new(&video));
    assert!(!map.is_visible());
    assert!(mini.paintable().is_none());
    window.close();
    eprintln!(
        "PASS development minimap: Fit hidden, cropped region, shared texture/paintable, interactive pan, resize, no decode, slideshow/new-photo/video suppression"
    );
}

pub(super) fn layout(app: &adw::Application) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("portrait.png");
    std::fs::copy("tests/fixtures/images/wide.png", &file).unwrap();
    window::build(app, &pivra::settings::Settings::default(), None);
    let window = app
        .active_window()
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    let panel = widget(&window, "media-information-panel");
    let row = panel.parent().unwrap();
    let picture = widgets(window.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Overlay>().ok())
        .find_map(|o| o.child().and_downcast::<gtk::Picture>())
        .unwrap();
    wait("empty viewer layout", || {
        row.is_mapped() && picture.width() > 0
    });
    let status = widgets(window.upcast_ref())
        .into_iter()
        .find(|w| {
            w.downcast_ref::<gtk::Label>()
                .is_some_and(|l| l.text() == "Open a photo or video to start.")
        })
        .unwrap();
    let center_error = |bounds: gtk::graphene::Rect| {
        (
            (bounds.x() + bounds.width() / 2. - row.width() as f32 / 2.).abs(),
            (bounds.y() + bounds.height() / 2. - row.height() as f32 / 2.).abs(),
        )
    };
    let empty_error = center_error(status.compute_bounds(&row).unwrap());
    eprintln!(
        "LAYOUT empty: content={}x{} picture={}x{} panel={} hexpand={} center_error={empty_error:?}",
        row.width(),
        row.height(),
        picture.width(),
        picture.height(),
        panel.width(),
        panel.compute_expand(gtk::Orientation::Horizontal)
    );
    open_uri(&window, &file);
    window
        .lookup_action("image-mode")
        .unwrap()
        .activate(Some(&"fit".to_variant()));
    photo_presented(&window);
    let node = render_frame::rendered_node(&window, &picture);
    let bounds = node.bounds();
    let pos = picture.compute_bounds(&row).unwrap();
    let photo_error = center_error(gtk::graphene::Rect::new(
        pos.x() + bounds.x(),
        pos.y() + bounds.y(),
        bounds.width(),
        bounds.height(),
    ));
    eprintln!(
        "LAYOUT portrait Fit: content={}x{} picture={}x{} panel={} rendered={bounds:?} center_error={photo_error:?}",
        row.width(),
        row.height(),
        picture.width(),
        picture.height(),
        panel.width()
    );
    widget(&window, "wheel-zoom-toggle")
        .downcast::<gtk::ToggleButton>()
        .unwrap()
        .set_active(true);
    controller::<gtk::EventControllerMotion>(picture.upcast_ref())
        .emit_by_name::<()>("enter", &[&200f64, &150f64]);
    controller::<gtk::EventControllerScroll>(picture.upcast_ref())
        .emit_by_name::<bool>("scroll", &[&0f64, &-8f64]);
    let map = widget(&window, "image-minimap");
    wait("minimap layout", || map.is_mapped() && map.width() > 0);
    let b = map.compute_bounds(&picture).unwrap();
    eprintln!(
        "LAYOUT minimap: bounds={b:?} picture={}x{}",
        picture.width(),
        picture.height()
    );
    assert!(
        empty_error.0 <= 1. && empty_error.1 <= 1.,
        "empty placeholder center: {empty_error:?}"
    );
    assert!(
        photo_error.0 <= 1. && photo_error.1 <= 1.,
        "portrait Fit center: {photo_error:?}"
    );
    assert!(
        map.width() <= 180 && map.height() <= 140,
        "bounded minimap: {b:?}"
    );
    assert!(
        (b.width() / b.height() - 0.5).abs() < 0.02,
        "portrait minimap aspect ratio: {b:?}"
    );
    assert!(
        (picture.width() as f32 - b.x() - b.width() - 12.).abs() <= 1.
            && (picture.height() as f32 - b.y() - b.height() - 12.).abs() <= 1.,
        "bottom-right minimap margins: {b:?}"
    );
    // Closing the sidebar must give all space back, including after resize.
    key(&window, gtk::gdk::Key::i);
    let revealer = panel.downcast_ref::<gtk::Revealer>().unwrap();
    wait("sidebar fully open", || revealer.is_child_revealed());
    photo_presented(&window);
    assert_eq!(picture.width() + panel.width(), row.width());
    key(&window, gtk::gdk::Key::i);
    wait("closed sidebar occupies no width", || panel.width() == 0);
    let old_size = (row.width(), row.height());
    window.set_default_size(1100, 750);
    // The compositor may subtract decorations from the requested window size.
    // Observe the actual content allocation, not an assumed client width.
    wait("resized layout", || {
        row.width() > old_size.0 && row.height() > old_size.1
    });
    eprintln!("LAYOUT resized: content={}x{}", row.width(), row.height());
    for mode in ["fill", "actual", "fit"] {
        window
            .lookup_action("image-mode")
            .unwrap()
            .activate(Some(&mode.to_variant()));
        let node = render_frame::rendered_node(&window, &picture);
        assert_eq!(picture.width(), row.width());
        let bounds = node.bounds();
        assert!(
            (bounds.x() + bounds.width() / 2. - picture.width() as f32 / 2.).abs() <= 1.,
            "{mode} centered after resize: {bounds:?}"
        );
    }
    key(&window, gtk::gdk::Key::F11);
    wait("fullscreen layout", || window.is_fullscreen());
    let node = render_frame::rendered_node(&window, &picture);
    assert_eq!(picture.width(), row.width());
    let b = node.bounds();
    let error = center_error(gtk::graphene::Rect::new(
        b.x(),
        b.y(),
        b.width(),
        b.height(),
    ));
    assert!(
        error.0 <= 1. && error.1 <= 1.,
        "fullscreen Fit center: {error:?}"
    );
    key(&window, gtk::gdk::Key::Escape);
    wait("normal window restored", || !window.is_fullscreen());
    window.close();
    eprintln!(
        "PASS development layout: centered empty/portrait Fit, bounded bottom-right portrait minimap, sidebar open/close, Fill/Actual/resize/fullscreen"
    );
}
