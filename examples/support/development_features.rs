//! Small controlled GTK regressions for the next development cycle.
use super::*;
fn widget(window: &adw::ApplicationWindow, name: &str) -> gtk::Widget {
    widgets(window.upcast_ref())
        .into_iter()
        .find(|w| w.widget_name() == name)
        .unwrap()
}
pub(super) fn open_uri(window: &adw::ApplicationWindow, path: &std::path::Path) {
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
    let button = widget(&window, "open-file-button")
        .downcast::<gtk::Button>()
        .unwrap();
    assert_eq!(button.action_name().as_deref(), Some("win.choose-media"));
    let choose = window.lookup_action("choose-media").unwrap();
    assert!(choose.is_enabled());
    let empty = widget(&window, "empty-media-state");
    assert!(empty.is_visible());
    assert!(
        !widgets(widget(&window, "pivra-header").as_ref())
            .iter()
            .any(
                |w| w.widget_name() == "open-media-button" || w.widget_name() == "open-file-button"
            )
    );
    button.emit_clicked();
    wait("empty Open File opens existing chooser", || {
        chooser(&window).is_some() && !choose.is_enabled()
    });
    chooser(&window).unwrap().close();
    wait("empty Open File cancel", || choose.is_enabled());
    assert!(empty.is_visible());
    assert!(!state(&window, "media-present"));
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
    assert!(!empty.is_visible());
    assert!(state(&window, "media-present"));
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
    let empty = widget(&window, "empty-media-state");
    wait("empty page layout", || {
        empty.is_mapped() && empty.width() > 0
    });
    let viewport = empty.parent().unwrap();
    let page_bounds = empty.compute_bounds(&viewport).unwrap();
    assert!(!row.is_visible());
    assert!((page_bounds.width() - viewport.width() as f32).abs() <= 1.);
    assert!((page_bounds.height() - viewport.height() as f32).abs() <= 1.);
    let button = widget(&window, "open-file-button");
    let status = widgets(empty.upcast_ref())
        .into_iter()
        .find(|w| {
            w.downcast_ref::<gtk::Label>()
                .is_some_and(|l| l.text() == "Open a photo or video to get started")
        })
        .unwrap();
    let button_bounds = button.compute_bounds(&empty).unwrap();
    let title_bounds = status.compute_bounds(&empty).unwrap();
    assert!(button_bounds.y() >= title_bounds.y() + title_bounds.height());
    assert!(
        (button_bounds.x() + button_bounds.width() / 2. - empty.width() as f32 / 2.).abs() <= 2.
    );
    let mut group = status.clone();
    while !button.is_ancestor(&group) {
        group = group.parent().expect("shared empty-state content");
    }
    let group_bounds = group.compute_bounds(&empty).unwrap();
    let empty_error = (
        (group_bounds.x() + group_bounds.width() / 2. - empty.width() as f32 / 2.).abs(),
        (group_bounds.y() + group_bounds.height() / 2. - empty.height() as f32 / 2.).abs(),
    );
    eprintln!(
        "LAYOUT empty page: {}x{} content={group_bounds:?} center_error={empty_error:?}",
        empty.width(),
        empty.height()
    );
    let center_error = |bounds: gtk::graphene::Rect| {
        (
            (bounds.x() + bounds.width() / 2. - row.width() as f32 / 2.).abs(),
            (bounds.y() + bounds.height() / 2. - row.height() as f32 / 2.).abs(),
        )
    };
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
        "empty-state content center: {empty_error:?}"
    );
    assert!(
        photo_error.0 <= 1. && photo_error.1 <= 1.,
        "portrait Fit center: {photo_error:?}"
    );
    assert!(
        map.width() == 220 && map.height() == 440,
        "bounded minimap: {b:?}"
    );
    assert!(
        (b.width() / b.height() - 0.5).abs() < 0.02,
        "portrait minimap aspect ratio: {b:?}"
    );
    assert!(
        (picture.width() as f32 - b.x() - b.width() - 16.).abs() <= 1.
            && (picture.height() as f32 - b.y() - b.height() - 16.).abs() <= 1.,
        "bottom-right minimap margins: {b:?}"
    );
    // Controlled UI obstacles, using real GTK scrollbar allocations. These are
    // not claims that production photo panning creates ScrolledWindow bars.
    let overlay = picture
        .parent()
        .unwrap()
        .downcast::<gtk::Overlay>()
        .unwrap();
    let adjustment = gtk::Adjustment::new(0., 0., 1000., 1., 50., 100.);
    let horizontal = gtk::Scrollbar::new(gtk::Orientation::Horizontal, Some(&adjustment));
    horizontal.set_valign(gtk::Align::End);
    let vertical = gtk::Scrollbar::new(gtk::Orientation::Vertical, Some(&adjustment));
    vertical.set_halign(gtk::Align::End);
    for bar in [&horizontal, &vertical] {
        overlay.add_overlay(bar);
        overlay.set_measure_overlay(bar, false);
    }
    let clearance = || {
        let b = map.compute_bounds(&picture).unwrap();
        let h = horizontal.compute_bounds(&picture).unwrap();
        let v = vertical.compute_bounds(&picture).unwrap();
        b.x() >= 16.
            && b.y() >= 16.
            && b.x() + b.width() <= v.x() - 16.
            && b.y() + b.height() <= h.y() - 16.
            && b.width() <= 560.
            && b.height() <= 440.
    };
    wait("allocated scrollbar clearance", || {
        horizontal.is_mapped() && vertical.is_mapped() && clearance()
    });
    horizontal.set_height_request(34);
    vertical.set_width_request(42);
    wait("changed scrollbar thickness clearance", || {
        horizontal.height() >= 34 && vertical.width() >= 42 && clearance()
    });
    let old_size = (picture.width(), picture.height());
    window.set_default_size(700, 500);
    wait("smaller viewport scrollbar clearance", || {
        picture.width() < old_size.0 && picture.height() < old_size.1 && clearance()
    });
    eprintln!(
        "LAYOUT scrollbar clearance: map={:?} horizontal={:?} vertical={:?}",
        map.compute_bounds(&picture).unwrap(),
        horizontal.compute_bounds(&picture).unwrap(),
        vertical.compute_bounds(&picture).unwrap()
    );
    overlay.remove_overlay(&horizontal);
    overlay.remove_overlay(&vertical);
    let strip = widget(&window, "thumbnail-strip");
    // Minimap and navigation are now mutually exclusive. Changing normal strip
    // visibility/size must not remap it or reduce the map's usable viewport.
    strip.set_visible(true);
    assert!(!strip.is_child_visible() && !strip.is_mapped());
    strip.set_height_request(190);
    photo_presented(&window);
    assert!(!strip.is_child_visible() && !strip.is_mapped());
    strip.set_visible(false);
    wait("suppressed navigation leaves usable area", || {
        let b = map.compute_bounds(&picture).unwrap();
        (picture.height() as f32 - b.y() - b.height() - 16.).abs() <= 1.
            && (picture.width() as f32 - b.x() - b.width() - 16.).abs() <= 1.
    });
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

pub(super) fn navigation(app: &adw::Application) {
    use std::{cell::Cell, rc::Rc};
    let dir = tempfile::tempdir().unwrap();
    for name in ["1.png", "2.png", "4.png"] {
        std::fs::copy("tests/fixtures/images/wide.png", dir.path().join(name)).unwrap();
    }
    window::build(app, &pivra::settings::Settings::default(), None);
    let window = app
        .active_window()
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    open_uri(&window, &dir.path().join("2.png"));
    photo_presented(&window);
    wait("middle item navigation", || {
        ["previous-media", "next-media"]
            .iter()
            .all(|name| window.lookup_action(name).unwrap().is_enabled())
    });
    let map = widget(&window, "image-minimap");
    let strip = widget(&window, "thumbnail-strip");
    let controls = widget(&window, "thumbnail-controls");
    let edge = widget(&window, "thumbnail-resize-edge");
    let previous = widgets(window.upcast_ref())
        .into_iter()
        .find(|w| {
            w.downcast_ref::<gtk::Button>()
                .is_some_and(|b| b.action_name().as_deref() == Some("win.previous-media"))
        })
        .unwrap();
    let next = widgets(window.upcast_ref())
        .into_iter()
        .find(|w| {
            w.downcast_ref::<gtk::Button>()
                .is_some_and(|b| b.action_name().as_deref() == Some("win.next-media"))
        })
        .unwrap();
    let picture = widgets(window.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Overlay>().ok())
        .find_map(|o| o.child().and_downcast::<gtk::Picture>())
        .unwrap();
    let overlay = picture.parent().unwrap();
    let motion = |x: f64, y: f64| {
        let list = overlay.observe_controllers();
        for i in 0..list.n_items() {
            if let Ok(c) = list
                .item(i)
                .unwrap()
                .downcast::<gtk::EventControllerMotion>()
            {
                c.emit_by_name::<()>("motion", &[&x, &y]);
            }
        }
    };
    pivra::settings::thumbnail_preference().set_enabled(true);
    let pin = controls
        .last_child()
        .unwrap()
        .downcast::<gtk::ToggleButton>()
        .unwrap();
    if !pin.is_active() {
        pin.emit_clicked();
    }
    wait("normal pinned navigation", || {
        strip.is_mapped() && controls.is_mapped() && edge.is_mapped()
    });
    motion(0., 100.);
    assert!(previous.is_mapped() && !next.is_mapped());
    motion(overlay.width() as f64 - 1., 100.);
    assert!(next.is_mapped() && !previous.is_mapped());
    let preferences = ["thumbnails.conf", "thumbnail-pinned.conf"]
        .map(|name| glib::user_config_dir().join("pivra").join(name));
    let saved = preferences
        .iter()
        .map(|p| std::fs::read(p).unwrap())
        .collect::<Vec<_>>();
    let nav = vec![strip.clone(), controls.clone(), edge, previous, next];
    let flashes = Rc::new(Cell::new(0));
    let frames = Rc::new(Cell::new(0));
    let map_frames = Rc::new(Cell::new(0));
    let clock = picture.frame_clock().unwrap();
    let weak_map = map.downgrade();
    let watched = nav.iter().map(|w| w.downgrade()).collect::<Vec<_>>();
    let count = flashes.clone();
    let observed = frames.clone();
    let effective = map_frames.clone();
    let handler = clock.connect_after_paint(move |_| {
        observed.set(observed.get() + 1);
        if weak_map.upgrade().is_some_and(|m| m.is_mapped()) {
            effective.set(effective.get() + 1);
            if watched
                .iter()
                .any(|w| w.upgrade().is_some_and(|w| w.is_mapped()))
            {
                count.set(count.get() + 1);
            }
        }
    });
    // Catch transient remapping from hover/pin callbacks, not just final frames.
    for w in &nav {
        let weak = map.downgrade();
        let count = flashes.clone();
        w.connect_map(move |_| {
            if weak.upgrade().is_some_and(|m| m.is_mapped()) {
                count.set(count.get() + 1);
            }
        });
    }
    widget(&window, "wheel-zoom-toggle")
        .downcast::<gtk::ToggleButton>()
        .unwrap()
        .set_active(true);
    let zoom = || {
        controller::<gtk::EventControllerMotion>(picture.upcast_ref())
            .emit_by_name::<()>("enter", &[&200f64, &150f64]);
        controller::<gtk::EventControllerScroll>(picture.upcast_ref())
            .emit_by_name::<bool>("scroll", &[&0f64, &-20f64]);
        wait("map masks navigation", || map.is_mapped());
        let mini = widget(&window, "minimap-image")
            .downcast::<gtk::Picture>()
            .unwrap();
        let _ = render_frame::rendered_node(&window, &mini);
        assert!(nav.iter().all(|w| !w.is_mapped() && !w.is_child_visible()));
    };
    for cycle in 0..3 {
        zoom();
        motion(0., overlay.height() as f64 - 1.);
        motion(overlay.width() as f64 - 1., overlay.height() as f64 - 1.);
        assert!(nav.iter().all(|w| !w.is_mapped()));
        let height = strip.height_request();
        let resize = controller::<gtk::GestureDrag>(&overlay);
        let y = strip.compute_bounds(&overlay).unwrap().y() as f64;
        resize.emit_by_name::<()>("drag-begin", &[&10f64, &y]);
        resize.emit_by_name::<()>("drag-update", &[&0f64, &-30f64]);
        resize.emit_by_name::<()>("drag-end", &[&0f64, &-30f64]);
        assert_eq!(
            strip.height_request(),
            height,
            "hidden strip resize must not intercept minimap drag"
        );
        if cycle == 1 {
            let old = picture.width();
            window.set_default_size(1100, 750);
            wait("resized minimap navigation", || picture.width() != old);
            let _ = render_frame::rendered_node(&window, &picture);
            assert!(map.is_mapped() && nav.iter().all(|w| !w.is_mapped()));
        }
        // Visibility itself is the source of truth, independently of zoom/mode.
        map.set_visible(false);
        assert!(strip.is_mapped() && controls.is_mapped());
        assert!(nav.iter().all(|w| w.is_child_visible()));
        map.set_visible(true);
        assert!(nav.iter().all(|w| !w.is_mapped()));
        window
            .lookup_action("image-mode")
            .unwrap()
            .activate(Some(&"fit".to_variant()));
        photo_presented(&window);
        wait("Fit restores pinned navigation", || {
            !map.is_mapped() && strip.is_mapped() && controls.is_mapped()
        });
    }
    zoom();
    key(&window, gtk::gdk::Key::F5);
    wait("slideshow restores normal navigation", || {
        !map.is_mapped() && strip.is_mapped()
    });
    key(&window, gtk::gdk::Key::Escape);
    zoom();
    key(&window, gtk::gdk::Key::Right);
    photo_presented(&window);
    wait("selection restores navigation", || {
        !map.is_mapped() && strip.is_mapped()
    });
    let video = std::env::args().find(|s| s.ends_with(".mp4")).unwrap();
    open_uri(&window, std::path::Path::new(&video));
    assert!(!map.is_mapped() && nav.iter().all(|w| w.is_child_visible()));
    assert!(
        pivra::settings::thumbnail_preference().enabled()
            && pivra::settings::load_thumbnail_pinned()
    );
    assert_eq!(
        preferences
            .iter()
            .map(|p| std::fs::read(p).unwrap())
            .collect::<Vec<_>>(),
        saved
    );
    // Disabled/unpinned preferences must also survive temporary masking; hiding
    // the map must not force a disabled strip open.
    pin.emit_clicked();
    pivra::settings::thumbnail_preference().set_enabled(false);
    let disabled = preferences
        .iter()
        .map(|p| std::fs::read(p).unwrap())
        .collect::<Vec<_>>();
    open_uri(&window, &dir.path().join("2.png"));
    photo_presented(&window);
    zoom();
    window
        .lookup_action("image-mode")
        .unwrap()
        .activate(Some(&"fit".to_variant()));
    photo_presented(&window);
    assert!(!map.is_mapped() && !strip.is_mapped() && nav.iter().all(|w| w.is_child_visible()));
    assert!(
        !pivra::settings::thumbnail_preference().enabled()
            && !pivra::settings::load_thumbnail_pinned()
    );
    assert_eq!(
        preferences
            .iter()
            .map(|p| std::fs::read(p).unwrap())
            .collect::<Vec<_>>(),
        disabled
    );
    assert!(frames.get() > 0);
    assert!(map_frames.get() > 0);
    assert_eq!(
        flashes.get(),
        0,
        "navigation remapped/painted while minimap effective"
    );
    clock.disconnect(handler);
    window.close();
    eprintln!(
        "PASS navigation/minimap: complete strip/edge/controls/arrows masked, hover cannot override, effective visibility restores, pinned/enabled files unchanged, repeated mode/selection/video/slideshow transitions, zero overlapping frames/remaps"
    );
}

pub(super) fn native_key(keys: &str) {
    assert!(
        std::process::Command::new("xdotool")
            .args(["key", "--clearmodifiers", keys])
            .status()
            .unwrap()
            .success()
    );
}
fn chooser(window: &adw::ApplicationWindow) -> Option<gtk::Window> {
    gtk::Window::list_toplevels()
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Window>().ok())
        .find(|w| w.is_visible() && w.transient_for().as_ref() == Some(window.upcast_ref()))
}
pub(super) fn shortcuts(app: &adw::Application) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("1.png");
    std::fs::copy("tests/fixtures/images/wide.png", &file).unwrap();
    window::build(app, &pivra::settings::Settings::default(), None);
    let window = app
        .active_window()
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    let choose = window.lookup_action("choose-media").unwrap();
    wait("empty window keyboard focus", || window.is_active());
    native_key("ctrl+o");
    wait("Ctrl+O opens existing chooser from empty window", || {
        !choose.is_enabled() && chooser(&window).is_some()
    });
    wait("chooser keyboard focus", || {
        chooser(&window).unwrap().is_active()
    });
    native_key("Escape");
    wait("empty chooser cancel", || choose.is_enabled());
    assert_eq!(app.windows().len(), 1);
    assert!(has_text(&window, "Open a photo or video to get started"));
    window.present();
    wait("keyboard focus after cancel", || window.is_active());
    native_key("ctrl+o");
    wait("empty chooser reopened", || chooser(&window).is_some());
    let dialog = chooser(&window).unwrap();
    wait("chooser input focus", || dialog.is_active());
    native_key("ctrl+l");
    wait("chooser location entry", || {
        gtk::prelude::GtkWindowExt::focus(&dialog).is_some_and(|w| w.is::<gtk::Editable>())
    });
    assert!(
        std::process::Command::new("xdotool")
            .args([
                "type",
                "--clearmodifiers",
                "--delay",
                "0",
                "--",
                file.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success()
    );
    native_key("Return");
    wait("chooser accepted in original window", || {
        choose.is_enabled() && selected(&window, "1.png")
    });
    photo_presented(&window);
    assert_eq!(app.windows().len(), 1);
    let picture = widgets(window.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Overlay>().ok())
        .find_map(|o| o.child().and_downcast::<gtk::Picture>())
        .unwrap();
    let painted = picture.paintable();
    let mode = window.lookup_action("image-mode").unwrap().state();
    let title = window.title();
    window.present();
    wait("active viewer keyboard focus", || window.is_active());
    native_key("ctrl+o");
    wait("Ctrl+O opens chooser from active viewer", || {
        !choose.is_enabled() && chooser(&window).is_some()
    });
    wait("active chooser keyboard focus", || {
        chooser(&window).unwrap().is_active()
    });
    native_key("Escape");
    wait("viewer chooser cancel", || choose.is_enabled());
    assert_eq!(picture.paintable(), painted);
    assert_eq!(window.lookup_action("image-mode").unwrap().state(), mode);
    assert_eq!(window.title(), title);
    assert_eq!(app.windows().len(), 1);
    window.close();
    eprintln!(
        "PASS Ctrl+O real X11 keys and GTK chooser: empty/active, cancel unchanged, selected file reuses window"
    );
}

pub(super) fn zoom_state(app: &adw::Application) {
    let dir = tempfile::tempdir().unwrap();
    for name in ["1.png", "2.png"] {
        std::fs::copy("tests/fixtures/images/wide.png", dir.path().join(name)).unwrap();
    }
    pivra::settings::save_wheel_zoom(false);
    window::build(app, &pivra::settings::Settings::default(), None);
    let window = app
        .active_window()
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    open_uri(&window, &dir.path().join("1.png"));
    photo_presented(&window);
    wait("two-photo catalog", || {
        window.title().is_some_and(|t| t.contains("2 candidates"))
    });
    let button = widget(&window, "wheel-zoom-toggle")
        .downcast::<gtk::ToggleButton>()
        .unwrap();
    let mode = window.lookup_action("image-mode").unwrap();
    let picture = widgets(window.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Overlay>().ok())
        .find_map(|o| o.child().and_downcast::<gtk::Picture>())
        .unwrap();
    let map = widget(&window, "image-minimap");
    let preset = |id: &str| {
        mode.activate(Some(&id.to_variant()));
        photo_presented(&window);
        assert!(
            !button.is_active() && !button.has_css_class("suggested-action"),
            "preset {id} is not manual zoom"
        );
    };
    preset("fit");
    let wheel = |ctrl: bool, number: &str| {
        window.present();
        wait("wheel keyboard focus", || window.is_active());
        let b = picture.compute_bounds(&window).unwrap();
        let native = std::process::Command::new("xdotool")
            .arg("getactivewindow")
            .output()
            .unwrap();
        let xid = std::str::from_utf8(&native.stdout).unwrap().trim();
        assert!(
            std::process::Command::new("xdotool")
                .args([
                    "mousemove",
                    "--window",
                    xid,
                    &(b.x() + b.width() * 0.2).round().to_string(),
                    &(b.y() + b.height() * 0.2).round().to_string()
                ])
                .status()
                .unwrap()
                .success()
        );
        let args = if ctrl {
            vec![
                "keydown",
                "Control_L",
                "click",
                number,
                "keyup",
                "Control_L",
            ]
        } else {
            vec!["click", number]
        };
        assert!(
            std::process::Command::new("xdotool")
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    };
    wheel(true, "4");
    wait("Ctrl wheel synchronizes blue toggle", || {
        button.is_active() && button.has_css_class("suggested-action")
    });
    assert_eq!(mode.state().unwrap().str(), Some("manual"));
    assert!(
        !pivra::settings::load_wheel_zoom(),
        "temporary zoom must not persist wheel mode"
    );
    wait("zoomed photo minimap", || map.is_mapped());
    preset("fit");
    // Releasing Ctrl retains ordinary wheel navigation, not persistent zoom.
    wheel(false, "5");
    wait("unmodified wheel still navigates", || {
        selected(&window, "2.png")
    });
    photo_presented(&window);
    assert!(!button.is_active());
    preset("fill");
    preset("actual");
    wheel(true, "4");
    wait("manual zoom from 100 percent is active", || {
        button.is_active()
    });
    key(&window, gtk::gdk::Key::Left);
    photo_presented(&window);
    assert!(selected(&window, "1.png") && !button.is_active());
    assert!(!pivra::settings::load_wheel_zoom());
    // The explicit wheel-mode preference remains separate and still persists.
    preset("fit");
    button.emit_clicked();
    assert!(button.is_active() && pivra::settings::load_wheel_zoom());
    wheel(false, "4");
    wait("explicit wheel zoom", || {
        mode.state().unwrap().str() == Some("manual")
    });
    key(&window, gtk::gdk::Key::z);
    assert!(!pivra::settings::load_wheel_zoom());
    assert!(
        button.is_active(),
        "actual manual view remains indicated until reset"
    );
    preset("fit");
    wheel(true, "4");
    wait("temporary zoom before video", || button.is_active());
    let video = std::env::args().find(|s| s.ends_with(".mp4")).unwrap();
    open_uri(&window, std::path::Path::new(&video));
    assert!(!button.is_active() && !button.has_css_class("suggested-action"));
    assert!(!pivra::settings::load_wheel_zoom());
    window.close();
    eprintln!(
        "PASS real Ctrl+wheel zoom state: blue immediately, no preference leak, ordinary wheel navigation, photo/video/preset resets, explicit button/Z choice retained, minimap unchanged"
    );
}

pub(super) fn context_menu(app: &adw::Application) {
    fn rows(menu: &gtk::Popover) -> Vec<gtk::Button> {
        let mut result = Vec::new();
        let mut child = menu.child().unwrap().first_child();
        while let Some(widget) = child {
            child = widget.next_sibling();
            if let Ok(button) = widget.downcast::<gtk::Button>() {
                result.push(button);
            }
        }
        result
    }
    fn label(button: &gtk::Button) -> gtk::Label {
        button
            .child()
            .unwrap()
            .first_child()
            .unwrap()
            .downcast()
            .unwrap()
    }
    fn font(text: &gtk::Label) -> gtk::pango::FontDescription {
        let font = text
            .layout()
            .iter()
            .run_readonly()
            .unwrap()
            .item()
            .analysis()
            .font();
        gtk::pango::prelude::FontExt::describe(&font)
    }
    fn labels(menu: &gtk::Popover) -> Vec<String> {
        rows(menu)
            .iter()
            .map(|b| label(b).text().to_string())
            .collect()
    }
    fn motion(button: &gtk::Button) -> gtk::EventControllerMotion {
        button
            .observe_controllers()
            .iter::<glib::Object>()
            .filter_map(Result::ok)
            .find_map(|c| c.downcast::<gtk::EventControllerMotion>().ok())
            .unwrap()
    }
    fn popup(window: &adw::ApplicationWindow, video: bool) -> gtk::Popover {
        wait("owning window active", || {
            window
                .surface()
                .and_downcast::<gtk::gdk::Toplevel>()
                .is_some_and(|s| s.state().contains(gtk::gdk::ToplevelState::FOCUSED))
        });
        let picture = widgets(window.upcast_ref())
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::Picture>().ok())
            .find(|p| (p.widget_name() == "video-content") == video)
            .unwrap();
        let click = picture
            .observe_controllers()
            .iter::<glib::Object>()
            .filter_map(Result::ok)
            .filter_map(|c| c.downcast::<gtk::GestureClick>().ok())
            .find(|c| c.button() == 3)
            .unwrap();
        click.emit_by_name::<()>("pressed", &[&1i32, &30f64, &30f64]);
        widgets(picture.upcast_ref())
            .into_iter()
            .find(|w| w.widget_name() == "media-context-menu")
            .unwrap()
            .downcast()
            .unwrap()
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("1.png");
    std::fs::copy("tests/fixtures/images/wide.png", &path).unwrap();
    window::build(app, &pivra::settings::Settings::default(), None);
    let window = app
        .active_window()
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    let menu = popup(&window, false);
    assert!(
        !menu.is_visible(),
        "empty window must not offer media actions"
    );
    open_uri(&window, &path);
    photo_presented(&window);
    let menu = popup(&window, false);
    wait("context menu mapped/allocated", || {
        menu.is_mapped() && rows(&menu).iter().all(|b| b.width() > 0)
    });
    assert_eq!(
        labels(&menu),
        [
            "Open...",
            "Open With…",
            "Show in Files",
            "Rename",
            "Zoom",
            "Display Mode",
            "Fullscreen",
            "Start Slideshow",
            "Thumbnail Strip",
            "Media Info",
            "Move to Trash"
        ]
    );
    let buttons = rows(&menu);
    let first_font = font(&label(&buttons[0]));
    assert_eq!(first_font.weight(), gtk::pango::Weight::Normal);
    assert!(
        gtk::IconTheme::for_display(&gtk::prelude::WidgetExt::display(&window))
            .has_icon("object-select-symbolic")
    );
    let first_x = label(&buttons[0]).compute_bounds(&buttons[0]).unwrap().x();
    let first_height = buttons[0].height();
    for button in &buttons {
        let text = label(button);
        assert_eq!(font(&text), first_font, "row font: {}", text.text());
        assert!(
            (text.compute_bounds(button).unwrap().x() - first_x).abs() <= 1.,
            "row label alignment: {}",
            text.text()
        );
        assert_eq!(button.height(), first_height, "row height: {}", text.text());
        assert!(first_x < 24., "unnecessary leading indicator space");
    }
    let thumbnail = buttons[8].clone();
    let indicator = thumbnail.child().unwrap().last_child().unwrap();
    let saved = pivra::settings::thumbnail_preference().enabled();
    let dimensions = thumbnail.measure(gtk::Orientation::Horizontal, -1);
    for enabled in [true, false, saved] {
        pivra::settings::thumbnail_preference().set_enabled(enabled);
        assert_eq!(indicator.opacity(), if enabled { 1. } else { 0. });
        assert_eq!(
            thumbnail.measure(gtk::Orientation::Horizontal, -1),
            dimensions,
            "checkmark changed row allocation"
        );
    }
    eprintln!(
        "MENU_ROWS label_x={first_x} height={first_height} font={}; trailing check keeps allocation",
        first_font
    );
    window.set_title(window.title().as_deref());
    assert!(
        menu.is_visible(),
        "unchanged selection/title refresh dismissed context menu"
    );
    let original_content = menu.child();
    let modes = buttons[5].clone();
    let child = widget(&window, "display-mode-submenu")
        .downcast::<gtk::Popover>()
        .unwrap();
    assert_eq!(labels(&child), ["Fit", "Fill", "100%"]);
    for button in rows(&child) {
        assert_eq!(font(&label(&button)), first_font);
    }
    assert_eq!(child.position(), gtk::PositionType::Right);
    let hover = motion(&modes);
    // A cancelled hover must not reopen after the original deadline.
    hover.emit_by_name::<()>("enter", &[&5f64, &5f64]);
    hover.emit_by_name::<()>("leave", &[]);
    let passed = std::rc::Rc::new(std::cell::Cell::new(false));
    let elapsed = passed.clone();
    glib::timeout_add_local_once(Duration::from_millis(250), move || elapsed.set(true));
    wait("cancelled hover deadline", || passed.get());
    assert!(!child.is_visible() && menu.is_visible());
    let started = Instant::now();
    hover.emit_by_name::<()>("enter", &[&5f64, &5f64]);
    assert!(!child.is_visible(), "hover must not open immediately");
    wait("200ms cascading submenu", || child.is_mapped());
    assert!(started.elapsed() >= Duration::from_millis(150));
    assert!(menu.is_mapped());
    assert_ne!(menu.surface(), child.surface());
    assert_eq!(
        menu.root().and_downcast::<adw::ApplicationWindow>(),
        Some(window.clone())
    );
    assert_eq!(
        child.root().and_downcast::<adw::ApplicationWindow>(),
        Some(window.clone())
    );
    assert_eq!(
        child
            .surface()
            .unwrap()
            .dynamic_cast::<gtk::gdk::Popup>()
            .unwrap()
            .parent(),
        menu.surface()
    );
    hover.emit_by_name::<()>("leave", &[]);
    assert!(
        menu.is_visible() && child.is_visible(),
        "parent-to-child gap closed menu"
    );
    hover.emit_by_name::<()>("enter", &[&5f64, &5f64]);
    assert!(
        child.is_visible(),
        "returning to submenu parent closed child"
    );
    motion(&buttons[3]).emit_by_name::<()>("enter", &[&5f64, &5f64]);
    assert!(
        !child.is_visible() && menu.is_visible(),
        "unrelated root row must close only child"
    );
    hover.emit_by_name::<()>("enter", &[&5f64, &5f64]);
    wait("submenu reopens", || child.is_mapped());
    window.set_title(window.title().as_deref());
    assert!(menu.is_visible() && child.is_visible());
    assert_eq!(
        popup(&window, false).child(),
        original_content,
        "repeated popup rebuilt rows"
    );
    native_key("Escape");
    wait("Escape closes only active child", || !child.is_visible());
    assert!(menu.is_mapped());
    modes.grab_focus();
    native_key("Right");
    wait("keyboard Right", || child.is_mapped());
    native_key("Left");
    wait("keyboard Left", || !child.is_visible());
    assert!(menu.is_mapped());
    modes.emit_clicked();
    wait("clicked submenu", || child.is_mapped());
    native_key("Down");
    pump();
    native_key("Return");
    wait("mode activation closes chain", || {
        !menu.is_visible() && !child.is_visible()
    });
    assert_eq!(
        window
            .lookup_action("image-mode")
            .unwrap()
            .state()
            .and_then(|v| v.get::<String>())
            .as_deref(),
        Some("fill")
    );
    let menu = popup(&window, false);
    wait("root reopened", || menu.is_mapped());
    let zoom = widget(&window, "wheel-zoom-toggle")
        .downcast::<gtk::ToggleButton>()
        .unwrap();
    zoom.emit_clicked();
    assert_eq!(state(&window, "context-zoom"), zoom.is_active());
    window.lookup_action("context-zoom").unwrap().activate(None);
    assert_eq!(state(&window, "context-zoom"), zoom.is_active());
    window
        .lookup_action("context-thumbnails")
        .unwrap()
        .activate(None);
    assert_eq!(state(&window, "context-thumbnails"), !saved);
    pivra::settings::thumbnail_preference().set_enabled(saved);
    native_key("Escape");
    wait("Escape closes root", || !menu.is_visible());
    assert!(window.is_visible());
    window.lookup_action("media-info").unwrap().activate(None);
    assert!(state(&window, "media-info"));
    // A separate real GTK window takes focus through the window manager.
    let menu = popup(&window, false);
    wait("popup before focus transfer", || menu.is_mapped());
    let modes = widget(&window, "display-mode-submenu-button")
        .downcast::<gtk::Button>()
        .unwrap();
    let child = widget(&window, "display-mode-submenu")
        .downcast::<gtk::Popover>()
        .unwrap();
    modes.emit_clicked();
    wait("child before focus transfer", || child.is_mapped());
    let other = gtk::Window::builder()
        .title("TEST focus destination")
        .default_width(200)
        .default_height(120)
        .build();
    other.present();
    wait("focus destination mapped", || other.is_mapped());
    let id = std::process::Command::new("xdotool")
        .args([
            "search",
            "--onlyvisible",
            "--name",
            "^TEST focus destination$",
        ])
        .output()
        .unwrap();
    assert!(id.status.success());
    let id = String::from_utf8(id.stdout).unwrap();
    assert!(
        std::process::Command::new("xdotool")
            .args(["windowactivate", id.lines().last().unwrap()])
            .status()
            .unwrap()
            .success()
    );
    wait("window manager transfers focus", || {
        other.is_active() && !window.is_active()
    });
    wait("inactive window dismisses full popup chain", || {
        !menu.is_visible() && !child.is_visible()
    });
    other.close();
    window.present();
    wait("owner active again", || window.is_active());
    let menu = popup(&window, false);
    wait("popup before owner hide", || menu.is_mapped());
    window.set_visible(false);
    assert!(!menu.is_visible());
    window.present();
    wait("owner remapped", || window.is_active());
    let title = window.title();
    let picture = widgets(window.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Overlay>().ok())
        .find_map(|o| o.child().and_downcast::<gtk::Picture>())
        .unwrap();
    let painted = picture.paintable();
    let mode = window.lookup_action("image-mode").unwrap().state();
    let menu = popup(&window, false);
    wait("context Open row allocated", || menu.is_mapped());
    let open = rows(&menu)[0].clone();
    assert_eq!(label(&open).text(), "Open...");
    assert_eq!(open.action_name().as_deref(), Some("win.choose-media"));
    open.emit_clicked();
    let choose = window.lookup_action("choose-media").unwrap();
    wait("context Open uses existing chooser", || {
        !choose.is_enabled() && chooser(&window).is_some()
    });
    assert!(!menu.is_visible(), "menu grab survived chooser dispatch");
    wait("context chooser keyboard focus", || {
        chooser(&window).unwrap().is_active()
    });
    native_key("Escape");
    wait("context chooser cancel", || choose.is_enabled());
    assert_eq!(window.title(), title);
    assert_eq!(picture.paintable(), painted);
    assert_eq!(window.lookup_action("image-mode").unwrap().state(), mode);
    assert_eq!(
        app.windows().len(),
        1,
        "context Open created another viewer"
    );
    window.present();
    wait("owner after chooser cancel", || window.is_active());
    let video = std::env::args().find(|s| s.ends_with(".mp4")).unwrap();
    open_uri(&window, std::path::Path::new(&video));
    assert!(
        !menu.is_visible(),
        "selection change must dismiss obsolete menu"
    );
    let menu = popup(&window, true);
    wait("video menu", || menu.is_mapped());
    assert!(labels(&menu).iter().any(|s| s == "Play / Pause"));
    menu.popdown();
    window.close();
    eprintln!(
        "PASS context menu: uniform rows/check geometry, title/reopen stability, cancelled 200ms hover, sibling/gap lifecycle, owned popup surfaces, native keyboard/action routing, real window focus transfer/hide cleanup, photo/video actions, shared Open chooser/cancel/same viewer; native GNOME pointer/edge behavior NOT TESTED"
    );
}

pub(super) fn media_file_actions(app: &adw::Application) {
    use adw::prelude::MessageDialogExt;

    let folder = tempfile::tempdir().unwrap();
    let original = folder.path().join("1.png");
    let renamed = folder.path().join("20.png");
    let neighbor = folder.path().join("10.png");
    let video = std::env::args().find(|s| s.ends_with(".mp4")).unwrap();
    let pixels = std::fs::read("tests/fixtures/images/wide.png").unwrap();
    for file in [&original, &neighbor] {
        std::fs::write(file, &pixels).unwrap();
    }
    std::fs::copy(video, folder.path().join("2.mp4")).unwrap();
    window::build(app, &pivra::settings::Settings::default(), None);
    let window = app
        .active_window()
        .and_downcast::<adw::ApplicationWindow>()
        .unwrap();
    open_uri(&window, &original);
    photo_presented(&window);
    wait("rename/filter catalog settled", || {
        window
            .title()
            .is_some_and(|title| title.contains("3 candidates"))
    });

    pivra::settings::thumbnail_preference().set_enabled(true);
    let pin = widget(&window, "thumbnail-controls")
        .last_child()
        .and_downcast::<gtk::ToggleButton>()
        .unwrap();
    if !pin.is_active() {
        pin.emit_clicked();
    }
    let filter = widget(&window, "thumbnail-media-filter")
        .downcast::<gtk::DropDown>()
        .unwrap();
    let items = widget(&window, "thumbnail-items");
    let visible = || {
        widgets(&items)
            .into_iter()
            .filter(|w| w.is_visible() && w.widget_name().starts_with("thumbnail-cell-"))
            .map(|w| w.widget_name().to_string())
            .collect::<Vec<_>>()
    };
    wait("all three thumbnail cells allocated", || {
        visible().len() == 3
    });
    filter.set_selected(2);
    assert_eq!(visible(), ["thumbnail-cell-1"]);
    assert!(selected(&window, "1.png"));
    window.lookup_action("next-media").unwrap().activate(None);
    assert!(selected(&window, "2.mp4"));
    filter.set_selected(1);
    assert_eq!(visible(), ["thumbnail-cell-0", "thumbnail-cell-2"]);
    assert!(selected(&window, "2.mp4"));
    filter.set_selected(0);
    assert_eq!(visible().len(), 3);
    window
        .lookup_action("previous-media")
        .unwrap()
        .activate(None);
    photo_presented(&window);

    wait("F2 viewer focused", || window.is_active());
    native_key("F2");
    wait("rename dialog shown", || chooser(&window).is_some());
    let dialog = chooser(&window)
        .unwrap()
        .downcast::<adw::MessageDialog>()
        .unwrap();
    let entry = widgets(dialog.upcast_ref())
        .into_iter()
        .find(|w| w.widget_name() == "rename-filename")
        .and_downcast::<gtk::Entry>()
        .unwrap();
    assert_eq!(entry.text(), "1.png");
    assert_eq!(entry.selection_bounds(), Some((0, 1)));
    entry.set_text("20.png");
    dialog.response("rename");
    wait("renamed file catalog and title updated", || {
        renamed.is_file() && !original.exists() && selected(&window, "20.png")
    });
    assert_eq!(std::fs::read(&renamed).unwrap(), pixels);
    assert_eq!(std::fs::read(&neighbor).unwrap(), pixels);
    window
        .lookup_action("previous-media")
        .unwrap()
        .activate(None);
    assert!(
        selected(&window, "10.png"),
        "natural order stale after rename"
    );
    window.close();
    eprintln!(
        "PASS GTK media filters and F2 rename: both active-media exclusions, preserved content and selected path, reordered catalog"
    );
}
