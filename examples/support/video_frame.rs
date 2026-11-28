//! The production actions, real decoded frames and native GTK Save dialog.
use super::*;

fn named(window: &gtk::Widget, name: &str) -> gtk::Widget {
    widgets(window)
        .into_iter()
        .find(|widget| widget.widget_name() == name)
        .unwrap_or_else(|| panic!("missing widget {name}"))
}

fn dialog(window: &adw::ApplicationWindow, title: &str) -> Option<gtk::Window> {
    gtk::Window::list_toplevels()
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Window>().ok())
        .find(|w| {
            w.is_mapped()
                && w.title().as_deref() == Some(title)
                && w.transient_for().as_ref() == Some(window.upcast_ref())
        })
}

fn button_label(root: &gtk::Widget, label: &str) -> Option<gtk::Button> {
    widgets(root)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Button>().ok())
        .find(|button| {
            button.is_mapped()
                && button
                    .label()
                    .is_some_and(|text| text.trim_start_matches('_') == label)
        })
}

fn click_label(root: &gtk::Widget, label: &str) {
    button_label(root, label)
        .unwrap_or_else(|| panic!("missing dialog button {label}"))
        .emit_clicked();
}

fn filename(window: &gtk::Window) -> gtk::Entry {
    let mut found = None;
    wait("native filename entry populated", || {
        found = widgets(window.upcast_ref())
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::Entry>().ok())
            .find(|entry| entry.is_mapped() && entry.text().ends_with(".png"));
        found.is_some()
    });
    found.unwrap()
}

fn verify(path: &std::path::Path, red: bool) {
    let image = gtk::gdk_pixbuf::Pixbuf::from_file(path).unwrap();
    assert_eq!((image.width(), image.height()), (192, 96));
    let bytes = image.read_pixel_bytes();
    let color = &bytes.as_ref()[0..3];
    let (high, low) = if red { (0, 2) } else { (2, 0) };
    assert!(
        color[high] > 210 && color[low] < 45 && color[1] < 45,
        "unexpected frame color {color:?}"
    );
}

pub(super) fn run_export(app: &adw::Application) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("source.mp4");
    let fixture = std::env::args().find(|s| s.ends_with(".mp4")).unwrap();
    std::fs::copy(fixture, &path).unwrap();
    let photo = dir.path().join("photo.png");
    std::fs::copy("tests/fixtures/images/wide.png", &photo).unwrap();
    window::build(
        app,
        &Default::default(),
        Some(input::OpenedFile::from_file(&gio::File::for_path(&path)).unwrap()),
    );
    let window = app
        .active_window()
        .and_downcast::<adw::ApplicationWindow>()
        .unwrap();
    let action = window.lookup_action("save-video-frame").unwrap();
    let speed = window.lookup_action("video-speed").unwrap();
    let video = named(window.upcast_ref(), "video-content");
    let bar = named(window.upcast_ref(), "video-speed-button")
        .parent()
        .unwrap();
    let play = widgets(&bar)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Button>().ok())
        .find(|button| {
            button.icon_name().as_deref().is_some_and(|icon| {
                matches!(
                    icon,
                    "media-playback-pause-symbolic" | "media-playback-start-symbolic"
                )
            })
        })
        .unwrap();
    let playing = || play.icon_name().as_deref() == Some("media-playback-pause-symbolic");
    let clock = widgets(&bar)
        .into_iter()
        .find_map(|w| w.downcast::<gtk::Label>().ok())
        .unwrap();
    let timeline = widgets(&bar)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Scale>().ok())
        .find(|s| s.adjustment().step_increment() == 0.001)
        .unwrap();
    let chooser = || dialog(&window, "Save Video Frame");
    let key = development_features::native_key;
    let wait_chooser = || {
        wait("native save chooser focused", || {
            chooser().is_some_and(|w| w.is_active())
        });
        chooser().unwrap()
    };
    let stock = std::env::var("PIVRA_VIDEO_BACKEND").as_deref() == Ok("stock");
    wait("playback and export became ready", || {
        action.is_enabled() && playing() && window.is_active()
    });
    if !stock {
        wait("automatically selected rate-capable GPU backend", || {
            speed.is_enabled()
        });
        speed.activate(Some(&"1.5".to_variant()));
        wait("GStreamer applied rate before capture", || {
            speed.is_enabled() && speed.state().unwrap().str() == Some("1.5")
        });
    }
    let rate = speed.state().unwrap();
    key("s");
    let save = wait_chooser();
    let entry = filename(&save);
    let name = entry.text();
    assert!(name.starts_with("source_00-00-0") && name.ends_with(".png"));
    // The requested frame is red; the chooser remains open until the stream
    // has advanced into its blue segment. The output must stay red.
    wait("video advanced behind chooser", || {
        clock.text().starts_with("0:05 /")
    });
    assert!(playing() && speed.state() == Some(rate.clone()));
    entry.set_text("playing.png");
    click_label(save.upcast_ref(), "Save");
    let png = dir.path().join("playing.png");
    wait("PNG saved", || png.is_file() && action.is_enabled());
    verify(&png, true);
    assert!(playing());

    play.emit_clicked();
    wait("video paused on blue frame", || !playing());
    let header = named(window.upcast_ref(), "pivra-header")
        .downcast::<adw::HeaderBar>()
        .unwrap();
    let text = gtk::Entry::new();
    header.pack_start(&text);
    wait("window and editable focused after Save", || {
        text.is_mapped() && window.is_active()
    });
    assert!(text.grab_focus());
    key("s");
    wait("S types into a focused entry", || text.text() == "s");
    assert!(chooser().is_none() && action.is_enabled());
    header.remove(&text);
    let position = timeline.value();
    gio::prelude::ActionGroupExt::activate_action(&window, "fullscreen", None);
    wait("fullscreen reached", || {
        window.is_fullscreen() && window.is_active()
    });
    let context = || {
        let gesture = video
            .observe_controllers()
            .iter::<glib::Object>()
            .filter_map(Result::ok)
            .filter_map(|w| w.downcast::<gtk::GestureClick>().ok())
            .find(|gesture| gesture.button() == 3)
            .unwrap();
        gesture.emit_by_name::<()>("pressed", &[&1i32, &30f64, &30f64]);
        let row = named(&video, "context-action-save-video-frame")
            .downcast::<gtk::Button>()
            .unwrap();
        assert_eq!(row.action_name().as_deref(), Some("win.save-video-frame"));
        row.emit_clicked();
        wait_chooser()
    };
    let save = context();
    let cancelled_name = filename(&save).text();
    save.close();
    wait("native cancel reenabled export", || action.is_enabled());
    assert!(!dir.path().join(cancelled_name.as_str()).exists());

    wait("focus returned from cancel", || window.is_active());
    let save = context();
    filename(&save).set_text("paused.jpg");
    click_label(save.upcast_ref(), "Save");
    let jpeg = dir.path().join("paused.jpg");
    wait("JPEG saved", || jpeg.is_file() && action.is_enabled());
    verify(&jpeg, false);
    assert!(!playing() && speed.state() == Some(rate));
    assert!((position - timeline.value()).abs() < 0.001);
    assert!(!window.lookup_action("undo").unwrap().is_enabled());

    wait("focus returned from JPEG save", || window.is_active());
    let save = context();
    filename(&save).set_text("playing.png");
    click_label(save.upcast_ref(), "Save");
    let replace_dialog = |parent: &gtk::Window| {
        gtk::Window::list_toplevels()
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::Window>().ok())
            .find(|w| {
                w.transient_for().as_ref() == Some(parent)
                    && button_label(w.upcast_ref(), "Replace").is_some()
            })
    };
    wait("GTK asks before overwriting", || {
        replace_dialog(&save).is_some()
    });
    verify(&png, true);
    click_label(replace_dialog(&save).unwrap().upcast_ref(), "Replace");
    wait("Pivra confirms the existing destination", || {
        replace_dialog(window.upcast_ref()).is_some()
    });
    verify(&png, true);
    click_label(
        replace_dialog(window.upcast_ref()).unwrap().upcast_ref(),
        "Cancel",
    );
    wait("replacement cancellation completed", || action.is_enabled());
    verify(&png, true);
    let unavailable = dir.path().join("unavailable.mp4");
    std::fs::write(&unavailable, b"not video").unwrap();
    development_features::open_uri(&window, &unavailable);
    wait("unavailable video reported", || {
        widgets(window.upcast_ref()).into_iter().any(|w| {
            w.downcast_ref::<gtk::Label>()
                .is_some_and(|label| label.text().starts_with("Video:"))
        })
    });
    assert!(!action.is_enabled() && chooser().is_none());
    development_features::open_uri(&window, &photo);
    assert!(!action.is_enabled());
    window.close();
    eprintln!(
        "PASS real {} export: native S/menu, pre-dialog red frame, paused blue JPEG, fullscreen, text focus, cancel/replace, unavailable video, retained playback and rate",
        if stock { "stock" } else { "OpenGL" }
    );
}
