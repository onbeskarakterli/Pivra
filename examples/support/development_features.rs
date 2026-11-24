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
