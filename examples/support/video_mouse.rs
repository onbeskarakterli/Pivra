//! Real X11 input on a production window, using stock GTK video and a fixture photo.
use super::*;
use std::{cell::Cell, process::Command, rc::Rc};

fn click(window: &adw::ApplicationWindow, target: &gtk::Widget, button: &str, count: &str) {
    let bounds = target.compute_bounds(window).expect("input target bounds");
    let x = (bounds.x() + bounds.width() / 2.).round();
    let y = (bounds.y() + bounds.height() / 2.).round();
    assert!(
        window
            .pick(x.into(), y.into(), gtk::PickFlags::DEFAULT)
            .is_some_and(|w| w == *target || w.is_ancestor(target)),
        "input target at {x},{y}: {}",
        target.widget_name()
    );
    let active = Command::new("xdotool")
        .arg("getactivewindow")
        .output()
        .unwrap();
    assert!(active.status.success());
    let xid = std::str::from_utf8(&active.stdout).unwrap().trim();
    assert!(
        Command::new("xdotool")
            .args([
                "mousemove",
                "--window",
                xid,
                &x.to_string(),
                &y.to_string(),
                "click",
                "--repeat",
                count,
                "--delay",
                "60",
                button,
            ])
            .status()
            .unwrap()
            .success()
    );
}

pub(super) fn run(app: &adw::Application) {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("fixture.mp4");
    let photo = folder.path().join("photo.png");
    let video = std::env::args().find(|a| a.ends_with(".mp4")).unwrap();
    std::fs::copy(video, &path).unwrap();
    std::fs::copy("tests/fixtures/images/wide.png", &photo).unwrap();
    window::build(
        app,
        &Default::default(),
        Some(input::OpenedFile::from_file(&gio::File::for_path(path)).unwrap()),
    );
    let window = app
        .active_window()
        .and_downcast::<adw::ApplicationWindow>()
        .unwrap();
    let contents = || widgets(window.upcast_ref());
    let video = contents()
        .into_iter()
        .find(|w| w.widget_name() == "video-content")
        .unwrap();
    wait("video frame and active window", || {
        video.is_mapped()
            && window.is_active()
            && video
                .downcast_ref::<gtk::Picture>()
                .unwrap()
                .paintable()
                .is_some_and(|p| p.intrinsic_width() > 0)
    });
    let play = contents()
        .into_iter()
        .filter(|w| w.is_mapped())
        .filter_map(|w| w.downcast::<gtk::Button>().ok())
        .find(|b| b.icon_name().as_deref() == Some("media-playback-pause-symbolic"))
        .expect("video play control");
    let playing = || play.icon_name().as_deref() == Some("media-playback-pause-symbolic");
    // Exercise a setting longer than the legacy hard-coded 300 ms.
    video.settings().set_gtk_double_click_time(700);
    let settled = || {
        let until = Instant::now() + Duration::from_millis(900);
        wait("multi-click interval elapsed", || Instant::now() >= until);
    };
    click(&window, &video, "1", "1");
    wait("video single click pauses", || !playing());
    click(&window, &video, "1", "1");
    wait("video single click resumes", playing);
    assert!(!window.is_fullscreen());

    let changed = Rc::new(Cell::new(0));
    let samples = changed.clone();
    let previous = std::cell::RefCell::new(play.icon_name());
    let id = play.connect_icon_name_notify(move |button| {
        let before = previous.replace(button.icon_name());
        if before != button.icon_name() {
            samples.set(samples.get() + 1);
        }
    });
    click(&window, &video, "1", "1");
    let second = Instant::now() + Duration::from_millis(390);
    wait("past old timeout before second click", || {
        Instant::now() >= second
    });
    assert!(playing());
    click(&window, &video, "1", "1");
    wait("video double click enters fullscreen", || {
        window.is_fullscreen()
    });
    settled();
    assert!(playing());
    assert_eq!(changed.get(), 0, "double click changed playback");

    click(&window, &video, "1", "1");
    wait("fullscreen single click pauses", || !playing());
    click(&window, &video, "1", "1");
    wait("fullscreen single click resumes", playing);
    assert!(window.is_fullscreen());

    changed.set(0);
    click(&window, &video, "1", "2");
    wait("video double click exits fullscreen", || {
        !window.is_fullscreen()
    });
    settled();
    assert!(playing());
    assert_eq!(changed.get(), 0, "exit double click changed playback");
    play.disconnect(id);

    // A transport button also restores an explicit keyboard focus target.
    click(&window, play.upcast_ref(), "1", "1");
    wait("playback button pauses", || !playing());
    settled();
    assert!(
        !playing(),
        "playback button click also reached the video surface"
    );
    development_features::native_key("space");
    wait("playback keyboard control resumes", playing);

    click(&window, &video, "3", "1");
    wait("video context menu mapped", || {
        widgets(&video)
            .iter()
            .any(|w| w.is::<gtk::Popover>() && w.is_mapped())
    });
    assert!(
        widgets(&video)
            .iter()
            .any(|w| w.widget_name() == "context-action-context-playback")
    );
    development_features::native_key("Escape");
    wait("menu dismissed", || {
        window.is_active()
            && widgets(&video)
                .iter()
                .all(|w| !w.is::<gtk::Popover>() || !w.is_mapped())
    });
    window
        .lookup_action("open-file")
        .unwrap()
        .activate(Some(&gio::File::for_path(photo).uri().to_variant()));
    photo_presented(&window);
    let picture = contents()
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Overlay>().ok())
        .find_map(|o| o.child().and_downcast::<gtk::Picture>())
        .unwrap();
    click(&window, picture.upcast_ref(), "1", "1");
    settled();
    assert!(
        !window.is_fullscreen(),
        "photo single click changed fullscreen"
    );
    click(&window, picture.upcast_ref(), "1", "2");
    wait("photo double click enters fullscreen", || {
        window.is_fullscreen()
    });
    settled();
    click(&window, picture.upcast_ref(), "1", "2");
    wait("photo double click exits fullscreen", || {
        !window.is_fullscreen()
    });
    window.close();
    eprintln!(
        "PASS real X11 video single/double click, no stray playback, context menu, keyboard/transport and photo clicks"
    );
}
