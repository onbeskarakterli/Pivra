//! Focused click/drag and manual deadline regression with controlled photo pixels.
use super::*;
fn after(duration: Duration) {
    let passed = std::rc::Rc::new(std::cell::Cell::new(false));
    let done = passed.clone();
    glib::timeout_add_local_once(duration, move || done.set(true));
    wait("bounded slideshow observation", || passed.get());
}
pub(super) fn run(app: &adw::Application) {
    let dir = tempfile::tempdir().unwrap();
    for name in ["1.png", "2.png", "4.png"] {
        std::fs::copy("tests/fixtures/images/wide.png", dir.path().join(name)).unwrap();
    }
    window::build(
        app,
        &pivra::settings::Settings::default(),
        Some(input::OpenedFile::from_file(&gio::File::for_path(dir.path().join("1.png"))).unwrap()),
    );
    let window = app
        .active_window()
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    wait("three slideshow photos", || {
        window.title().is_some_and(|t| t.contains("3 candidates"))
    });
    photo_presented(&window);
    let picture = widgets(window.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Overlay>().ok())
        .find_map(|o| o.child().and_downcast::<gtk::Picture>())
        .unwrap();
    let clicks: Vec<_> = picture
        .observe_controllers()
        .iter::<glib::Object>()
        .filter_map(Result::ok)
        .filter_map(|o| o.downcast::<gtk::GestureClick>().ok())
        .filter(|c| c.button() == 1)
        .collect();
    let click = |release: bool| {
        for c in &clicks {
            c.emit_by_name::<()>("pressed", &[&1i32, &40f64, &40f64]);
        }
        if release {
            for c in &clicks {
                c.emit_by_name::<()>("released", &[&1i32, &40f64, &40f64]);
            }
        }
    };
    click(true);
    assert!(
        selected(&window, "1.png"),
        "ordinary photo click must not navigate"
    );
    window
        .lookup_action("slideshow-interval")
        .unwrap()
        .activate(Some(&1u32.to_variant()));
    key(&window, gtk::gdk::Key::F5);
    photo_presented(&window);
    after(Duration::from_millis(650));
    assert!(selected(&window, "1.png"));
    click(true);
    assert!(selected(&window, "2.png"));
    photo_presented(&window);
    after(Duration::from_millis(500));
    assert!(
        selected(&window, "2.png"),
        "old photo deadline survived manual advance"
    );
    key(&window, gtk::gdk::Key::space);
    click(false);
    let drag = picture
        .observe_controllers()
        .iter::<glib::Object>()
        .filter_map(Result::ok)
        .find_map(|o| o.downcast::<gtk::GestureDrag>().ok())
        .unwrap();
    drag.emit_by_name::<()>("drag-update", &[&50f64, &0f64]);
    for c in &clicks {
        c.emit_by_name::<()>("released", &[&1i32, &90f64, &40f64]);
    }
    assert!(
        selected(&window, "2.png"),
        "drag was interpreted as a click"
    );
    click(true);
    assert!(selected(&window, "4.png"));
    photo_presented(&window);
    click(true);
    assert!(
        selected(&window, "1.png"),
        "last photo must wrap using slideshow navigation"
    );
    assert!(state(&window, "slideshow-paused"));
    window.close();
    eprintln!(
        "PASS slideshow photo click: ordinary-click isolation, immediate manual advance, interval reset, drag exclusion, paused navigation and looping; controlled pixels, not physical GNOME"
    );
}
