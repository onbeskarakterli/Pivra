//! Focused production toolbar/actions, with explicitly controlled photo pixels.
use super::*;
fn named(window: &adw::ApplicationWindow, name: &str) -> gtk::Widget {
    widgets(window.upcast_ref())
        .into_iter()
        .find(|w| w.widget_name() == name)
        .unwrap_or_else(|| panic!("missing toolbar widget {name}"))
}
fn interval(window: &adw::ApplicationWindow) -> u32 {
    window
        .lookup_action("slideshow-interval")
        .unwrap()
        .state()
        .unwrap()
        .get()
        .unwrap()
}
fn id(window: &adw::ApplicationWindow, name: &str) -> String {
    window
        .lookup_action(name)
        .unwrap()
        .state()
        .unwrap()
        .str()
        .unwrap()
        .to_owned()
}
fn choose(window: &adw::ApplicationWindow, name: &str, value: &str) {
    window
        .lookup_action(name)
        .unwrap()
        .activate(Some(&value.to_variant()));
}
fn notice(window: &adw::ApplicationWindow, text: &str) -> bool {
    widgets(window.upcast_ref())
        .iter()
        .filter_map(|w| w.downcast_ref::<gtk::Label>())
        .any(|l| l.is_mapped() && l.text() == text)
}
pub(super) fn run(first: &adw::ApplicationWindow, other: &adw::ApplicationWindow, restore: bool) {
    first.present();
    let mode = named(first, "display-controls");
    let zoom = named(first, "zoom-controls");
    let slides = named(first, "slideshow-controls");
    assert_eq!(mode.parent(), zoom.parent());
    assert_eq!(mode.parent(), slides.parent());
    assert_eq!(slides.observe_children().n_items(), 3);
    let zoom = named(first, "wheel-zoom-toggle")
        .downcast::<gtk::ToggleButton>()
        .unwrap();
    if restore {
        assert_eq!(interval(first), 7);
        assert_eq!(id(first, "slideshow-transition"), "none");
        assert_eq!(id(first, "image-mode"), "fill");
        assert_eq!(id(first, "image-resampling"), "nearest");
        assert!(zoom.is_active());
        assert_eq!(pivra::settings::load_slideshow_interval(), 7);
        assert_eq!(
            pivra::settings::load_slideshow_effect(),
            pivra::slideshow::Effect::None
        );
        first.close();
        other.close();
        eprintln!(
            "PASS toolbar restart: interval/effect/display/filter/zoom restored in a fresh process"
        );
        return;
    }
    assert_eq!(interval(first), 5);
    let selector = named(first, "slideshow-interval-selector")
        .downcast::<gtk::MenuButton>()
        .unwrap();
    selector.popup();
    let entry = named(first, "slideshow-custom-interval")
        .downcast::<gtk::Entry>()
        .unwrap();
    let error = named(first, "slideshow-interval-error");
    let apply = named(first, "slideshow-interval-apply")
        .downcast::<gtk::Button>()
        .unwrap();
    for bad in ["", "0", "3601", "36000", "-2", "1.5", "bad"] {
        entry.set_text(bad);
        apply.emit_clicked();
        assert!(error.is_visible(), "missing validation for {bad:?}");
        assert_eq!(interval(first), 5);
        assert_eq!(pivra::settings::load_slideshow_interval(), 5);
    }
    for seconds in pivra::slideshow::INTERVALS {
        named(first, &format!("slideshow-interval-{seconds}"))
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        assert_eq!(interval(first), seconds);
    }
    entry.set_text("7");
    entry.emit_by_name::<()>("activate", &[]);
    assert_eq!(interval(first), 7);
    assert!(!error.is_visible());
    assert_eq!(selector.label().as_deref(), Some("7 s"));
    choose(first, "slideshow-transition", "none");
    let fade = named(first, "slideshow-photo-transition")
        .downcast::<gtk::Picture>()
        .unwrap();
    named(first, "slideshow-play")
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    wait("toolbar starts fullscreen", || first.is_fullscreen());
    wait("start toast", || notice(first, "Slideshow started"));
    key(first, gtk::gdk::Key::F5); // Already running: no duplicate start/queue.
    key(first, gtk::gdk::Key::space);
    wait("pause toast replaces start", || {
        notice(first, "Slideshow paused") && !notice(first, "Slideshow started")
    });
    key(first, gtk::gdk::Key::Right);
    photo_presented(first);
    assert!(selected(first, "2.png") && fade.paintable().is_none() && !fade.is_visible());
    choose(first, "slideshow-transition", "fade");
    // Record a real painted fade frame: a batched main-context pump can run
    // the entire 300 ms effect before a later visibility sample is taken.
    let painted = std::rc::Rc::new(std::cell::Cell::new(false));
    let observed = painted.clone();
    let weak = fade.downgrade();
    let clock = first.frame_clock().unwrap();
    let handler = clock.connect_after_paint(move |_| {
        if weak
            .upgrade()
            .is_some_and(|w| w.is_visible() && w.paintable().is_some())
        {
            observed.set(true);
        }
    });
    key(first, gtk::gdk::Key::Left);
    assert!(selected(first, "1.png"));
    wait("fade option renders", || painted.get());
    clock.disconnect(handler);
    wait("fade resources released", || {
        !fade.is_visible() && fade.paintable().is_none()
    });
    choose(first, "slideshow-transition", "none");
    choose(first, "image-mode", "fill");
    choose(first, "image-resampling", "nearest");
    zoom.set_active(true);
    assert!(
        !named(other, "wheel-zoom-toggle")
            .downcast::<gtk::ToggleButton>()
            .unwrap()
            .is_active()
    );
    key(first, gtk::gdk::Key::space);
    wait(
        "resume dismisses stale pause without another message",
        || !notice(first, "Slideshow paused"),
    );
    key(first, gtk::gdk::Key::F11);
    key(first, gtk::gdk::Key::Escape);
    assert!(!state(first, "slideshow") && state(first, "fullscreen"));
    wait("finish toast", || notice(first, "Slideshow finished"));
    key(first, gtk::gdk::Key::F11);
    wait("ordinary fullscreen restored", || !first.is_fullscreen());
    first.close();
    assert!(other.is_visible());
    other.close();
    eprintln!(
        "PASS toolbar UX: groups, presets/custom validation, fade/none, keys, latest toasts, independent windows"
    );
}
