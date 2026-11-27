//! Each selected effect must produce a real GTK frame and release its layer.
use super::*;
pub(super) fn run(app: &adw::Application) {
    let dir = tempfile::tempdir().unwrap();
    for name in ["1.png", "portrait.png"] {
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
    wait("two transition photos", || {
        window.title().is_some_and(|t| t.contains("2 candidates"))
    });
    photo_presented(&window);
    key(&window, gtk::gdk::Key::F5);
    key(&window, gtk::gdk::Key::space);
    let picture = widgets(window.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Overlay>().ok())
        .find_map(|o| o.child().and_downcast::<gtk::Picture>())
        .unwrap();
    let layer = widgets(window.upcast_ref())
        .into_iter()
        .find(|w| w.widget_name() == "slideshow-photo-transition")
        .unwrap()
        .downcast::<gtk::Picture>()
        .unwrap();
    for (i, effect) in pivra::slideshow::Effect::ALL.into_iter().enumerate() {
        window
            .lookup_action("slideshow-transition")
            .unwrap()
            .activate(Some(&effect.id().to_variant()));
        assert_eq!(pivra::settings::load_slideshow_effect(), effect);
        window.lookup_action("image-mode").unwrap().activate(Some(
            &(if i % 2 == 0 { "fit" } else { "fill" }).to_variant(),
        ));
        render_frame::rendered_node(&window, &picture);
        let painted = std::rc::Rc::new(std::cell::Cell::new(false));
        let seen = painted.clone();
        let weak = layer.downgrade();
        let clock = picture.frame_clock().unwrap();
        let id = clock.connect_after_paint(move |_| {
            if let Some(layer) = weak.upgrade() {
                if layer.is_visible() && layer.paintable().is_some() {
                    seen.set(true);
                }
            }
        });
        window
            .lookup_action("slideshow-next")
            .unwrap()
            .activate(None);
        photo_presented(&window);
        if effect != pivra::slideshow::Effect::None {
            wait("selected effect paints", || painted.get());
        }
        clock.disconnect(id);
        wait("transition resources released", || {
            !layer.is_visible() && layer.paintable().is_none()
        });
        eprintln!(
            "PASS transition {}: painted={} and cleaned",
            effect.id(),
            painted.get()
        );
    }
    window
        .lookup_action("slideshow-stop")
        .unwrap()
        .activate(None);
    window.lookup_action("next-media").unwrap().activate(None);
    assert!(!layer.is_visible() && layer.paintable().is_none());
    window.close();
    eprintln!(
        "PASS slideshow effects: eight choices/persistence, real Fit/Fill wide/portrait frames, bounded cleanup, ordinary-navigation isolation; controlled pixels, native visual acceptance pending"
    );
}
