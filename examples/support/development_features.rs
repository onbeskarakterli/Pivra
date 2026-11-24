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
