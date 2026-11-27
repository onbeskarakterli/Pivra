//! Two fresh processes, one private folder. Controlled pixels, not decoder timing.
use super::*;
fn action(window: &adw::ApplicationWindow, name: &str) {
    window.lookup_action(name).unwrap().activate(None);
}
fn rendered(window: &adw::ApplicationWindow) {
    let picture = widgets(window.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Overlay>().ok())
        .find_map(|o| o.child().and_downcast::<gtk::Picture>())
        .unwrap();
    wait("requested photo paintable", || {
        picture.paintable().is_some()
    });
    let _ = render_frame::rendered_node(window, &picture);
}
pub(super) fn run(app: &adw::Application, direct: bool) {
    let folder = std::path::PathBuf::from(
        std::env::var_os("PIVRA_OPEN_LATENCY_DIR").expect("private benchmark folder"),
    );
    let path = folder.join("1.png");
    let case = if direct { "direct" } else { "chooser" };
    let start = Instant::now();
    window::build(
        app,
        &pivra::settings::Settings::default(),
        direct.then(|| input::OpenedFile::from_file(&gio::File::for_path(&path)).unwrap()),
    );
    let window = app
        .active_window()
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    if !direct {
        let button = widgets(window.upcast_ref())
            .into_iter()
            .find(|w| w.widget_name() == "open-file-button")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();
        wait("empty Open button mapped", || {
            button.is_mapped() && window.is_active()
        });
        let invocation = Instant::now();
        button.emit_clicked();
        let choose = window.lookup_action("choose-media").unwrap();
        wait("native local chooser mapped", || {
            gtk::Window::list_toplevels()
                .into_iter()
                .filter_map(|w| w.downcast::<gtk::Window>().ok())
                .any(|w| {
                    w.is_mapped()
                        && w.is_active()
                        && w.transient_for().as_ref() == Some(window.upcast_ref())
                })
        });
        eprintln!(
            "OPEN_SEQUENCE case={case} stage=chooser-mapped elapsed_us={}",
            invocation.elapsed().as_micros()
        );
        super::development_features::native_key("ctrl+l");
        assert!(
            std::process::Command::new("xdotool")
                .args([
                    "type",
                    "--clearmodifiers",
                    "--delay",
                    "0",
                    "--",
                    path.to_str().unwrap()
                ])
                .status()
                .unwrap()
                .success()
        );
        super::development_features::native_key("Return");
        wait("same-window file selection complete", || {
            choose.is_enabled() && selected(&window, "1.png")
        });
    }
    rendered(&window);
    eprintln!(
        "OPEN_SEQUENCE case={case} stage=initial-render-from-window-build elapsed_us={} includes_chooser_interaction={}",
        start.elapsed().as_micros(),
        !direct
    );
    assert_eq!(app.windows().len(), 1);
    wait("shared folder populated", || {
        window.title().is_some_and(|t| t.contains("3 candidates"))
    });
    for (name, target, phase) in [
        ("next-media", "2.png", "next-first"),
        ("previous-media", "1.png", "back"),
        ("next-media", "2.png", "next-warm"),
    ] {
        let picture = widgets(window.upcast_ref())
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::Overlay>().ok())
            .find_map(|o| o.child().and_downcast::<gtk::Picture>())
            .unwrap();
        let old = picture.paintable();
        let start = Instant::now();
        action(&window, name);
        wait("next selection and replacement paintable", || {
            selected(&window, target) && picture.paintable() != old
        });
        rendered(&window);
        eprintln!(
            "OPEN_SEQUENCE case={case} stage={phase} elapsed_us={}",
            start.elapsed().as_micros()
        );
    }
    window.close();
    pump();
    eprintln!(
        "PASS Open sequence {case}: same folder, real chooser/action/catalog, initial and next GTK rendered nodes, cold/warm navigation, one window; controlled pixels, not native Glycin/performance acceptance"
    );
}
