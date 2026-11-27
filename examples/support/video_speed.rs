//! Real GStreamer playback through the production window and both speed menus.
use super::*;

fn named(root: &gtk::Widget, name: &str) -> gtk::Widget {
    widgets(root)
        .into_iter()
        .find(|widget| widget.widget_name() == name)
        .unwrap_or_else(|| panic!("missing widget {name}"))
}

fn checked(row: &gtk::Widget) -> bool {
    widgets(row)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Image>().ok())
        .any(|image| {
            image.icon_name().as_deref() == Some("object-select-symbolic") && image.opacity() == 1.
        })
}

fn context(window: &adw::ApplicationWindow) -> gtk::Popover {
    wait("context menu owner focused", || window.is_active());
    let video = named(window.upcast_ref(), "video-content");
    let gesture = video
        .observe_controllers()
        .iter::<glib::Object>()
        .filter_map(Result::ok)
        .filter_map(|item| item.downcast::<gtk::GestureClick>().ok())
        .find(|gesture| gesture.button() == 3)
        .unwrap();
    gesture.emit_by_name::<()>("pressed", &[&1i32, &30f64, &30f64]);
    let menu: gtk::Popover = named(&video, "media-context-menu").downcast().unwrap();
    wait("video context menu opened", || menu.is_mapped());
    menu
}

pub(super) fn run(app: &adw::Application) {
    let dir = tempfile::tempdir().unwrap();
    let fixture = std::env::args().find(|arg| arg.ends_with(".mp4")).unwrap();
    for name in ["1.mp4", "2.mp4"] {
        std::fs::copy(&fixture, dir.path().join(name)).unwrap();
    }
    let photo = dir.path().join("3-photo.png");
    std::fs::copy("tests/fixtures/images/wide.png", &photo).unwrap();
    window::build(
        app,
        &Default::default(),
        Some(input::OpenedFile::from_file(&gio::File::for_path(dir.path().join("1.mp4"))).unwrap()),
    );
    let window = app
        .active_window()
        .and_downcast::<adw::ApplicationWindow>()
        .unwrap();
    let action = window.lookup_action("video-speed").unwrap();
    let selected_rate = || action.state().unwrap().get::<String>().unwrap();
    let speed: gtk::MenuButton = named(window.upcast_ref(), "video-speed-button")
        .downcast()
        .unwrap();
    if std::env::var("PIVRA_VIDEO_BACKEND").as_deref() == Ok("stock") {
        wait("stock video transport is visible", || speed.is_mapped());
        assert!(!action.is_enabled() && !speed.is_sensitive());
        action.activate(Some(&"2".to_variant()));
        assert_eq!(selected_rate(), "1");
        assert_eq!(speed.label().as_deref(), Some("1×"));
        window.close();
        eprintln!("PASS stock video keeps its real rate and disables unsupported changes");
        return;
    }
    wait("rate-capable video and scaletempo", || action.is_enabled());
    assert_eq!(selected_rate(), "1");
    assert_eq!(speed.label().as_deref(), Some("1×"));
    let bar = speed.parent().unwrap();
    let controls = widgets(&bar);
    let button = |icon: &str| {
        controls
            .iter()
            .find_map(|w| {
                w.downcast_ref::<gtk::Button>()
                    .filter(|b| b.icon_name().as_deref() == Some(icon))
            })
            .unwrap()
    };
    let play = button("media-playback-pause-symbolic");
    let repeat = controls
        .iter()
        .filter_map(|w| w.downcast_ref::<gtk::ToggleButton>())
        .find(|b| b.icon_name().as_deref() == Some("media-playlist-repeat-symbolic"))
        .unwrap();
    let timeline = controls
        .iter()
        .filter_map(|w| w.downcast_ref::<gtk::Scale>())
        .find(|s| s.adjustment().step_increment() == 0.001)
        .unwrap();
    let clock = controls
        .iter()
        .find_map(|w| w.downcast_ref::<gtk::Label>())
        .unwrap();
    let paused = || play.icon_name().as_deref() == Some("media-playback-start-symbolic");
    play.emit_clicked();
    wait("video paused", paused);
    timeline.set_value(0.35);
    wait("paused seek actually reached four seconds", || {
        clock.text().starts_with("0:04 /")
    });
    wait("finished initial seek", || action.is_enabled());

    let menu = speed.popover().unwrap();
    speed.popup();
    wait("toolbar speed menu", || menu.is_mapped());
    assert!(checked(&named(menu.upcast_ref(), "video-speed-1")));
    let position = timeline.value();
    named(menu.upcast_ref(), "video-speed-0.25")
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    assert_eq!(
        selected_rate(),
        "1",
        "optimistic rate before GStreamer confirmation"
    );
    wait("GStreamer confirmed minimum speed", || {
        selected_rate() == "0.25" && action.is_enabled()
    });
    assert!(paused() && (timeline.value() - position).abs() < 0.02);
    speed.popup();
    wait("toolbar menu reopened", || menu.is_mapped());
    assert!(checked(&named(menu.upcast_ref(), "video-speed-0.25")));
    named(menu.upcast_ref(), "video-speed-1.5")
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    wait("GStreamer confirmed toolbar speed", || {
        selected_rate() == "1.5" && action.is_enabled()
    });
    assert_eq!(speed.label().as_deref(), Some("1.5×"));
    assert!(paused());
    assert!((timeline.value() - position).abs() < 0.02);

    let root = context(&window);
    named(root.upcast_ref(), "playback-speed-submenu-button")
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    let submenu = named(root.upcast_ref(), "playback-speed-submenu");
    wait("speed cascade opened", || submenu.is_mapped());
    assert!(checked(&named(&submenu, "context-action-1.5")));
    named(&submenu, "context-action-2")
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    wait("GStreamer confirmed context speed", || {
        selected_rate() == "2" && action.is_enabled()
    });
    assert_eq!(speed.label().as_deref(), Some("2×"));
    assert!(paused() && !window.is_fullscreen());
    assert!((timeline.value() - position).abs() < 0.02);
    action.change_state(&"invalid".to_variant());
    assert_eq!(selected_rate(), "2");

    gio::prelude::ActionGroupExt::activate_action(&window, "fullscreen", None);
    wait("fullscreen transport", || {
        window.is_fullscreen() && speed.is_mapped()
    });
    speed.popup();
    wait("fullscreen speed menu", || menu.is_mapped());
    assert!(checked(&named(menu.upcast_ref(), "video-speed-2")));
    speed.popdown();
    gio::prelude::ActionGroupExt::activate_action(&window, "fullscreen", None);
    wait("leave fullscreen", || !window.is_fullscreen());

    // Measure media time after a seek and after a repeat, not just action state.
    let measure_rate = || {
        let start = Instant::now();
        let before = timeline.value();
        wait("sample real playback progression", || {
            start.elapsed() >= Duration::from_millis(1150)
        });
        let ratio = (timeline.value() - before) * 12.0 / start.elapsed().as_secs_f64();
        assert!(
            (1.6..=2.4).contains(&ratio),
            "expected 2× media clock, got {ratio:.2}×"
        );
    };
    play.emit_clicked();
    wait("resumed", || !paused());
    action.activate(Some(&"1.25".to_variant()));
    wait("rate change while playing", || {
        selected_rate() == "1.25" && action.is_enabled()
    });
    assert!(!paused());
    action.activate(Some(&"2".to_variant()));
    wait("restore 2× while playing", || {
        selected_rate() == "2" && action.is_enabled()
    });
    assert!(!paused());
    measure_rate();
    repeat.set_active(true);
    timeline.set_value(0.94);
    wait("seek to end applied", || clock.text().starts_with("0:11 /"));
    wait("repeat returned to start", || {
        clock.text().starts_with("0:00 /")
    });
    measure_rate();
    assert_eq!(selected_rate(), "2");

    gio::prelude::ActionGroupExt::activate_action(&window, "next-media", None);
    wait("retained speed on second video", || {
        selected(&window, "2.mp4") && selected_rate() == "2" && action.is_enabled()
    });
    assert_eq!(speed.label().as_deref(), Some("2×"));
    development_features::open_uri(&window, &photo);
    photo_presented(&window);
    assert!(!speed.is_mapped() && !action.is_enabled());
    window.close();
    eprintln!(
        "PASS GPU playback speed, paused position, seek/repeat clock, both menus, fullscreen and navigation"
    );
}
