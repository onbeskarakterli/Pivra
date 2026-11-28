//! One window and one chronological Undo history across four operation types.
use super::*;

fn transient(window: &adw::ApplicationWindow) -> Option<gtk::Window> {
    gtk::Window::list_toplevels()
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Window>().ok())
        .find(|w| w.is_mapped() && w.transient_for().as_ref() == Some(window.upcast_ref()))
}

fn button(window: &gtk::Window, label: &str) {
    widgets(window.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Button>().ok())
        .find(|button| {
            button.is_mapped()
                && button
                    .label()
                    .is_some_and(|s| s.trim_start_matches('_') == label)
        })
        .unwrap_or_else(|| panic!("missing dialog button {label}"))
        .emit_clicked();
}

pub(super) fn run(app: &adw::Application) {
    let data = std::env::var_os("XDG_DATA_HOME").expect("isolated fixture data");
    std::fs::create_dir_all(&data).unwrap();
    let from = tempfile::tempdir_in(&data).unwrap();
    let to = tempfile::tempdir_in(&data).unwrap();
    let base = from.path();
    let names = ["1.png", "2.mp4", "3-photo.png", "4.png"];
    let video = std::env::args().find(|s| s.ends_with(".mp4")).unwrap();
    for name in names {
        std::fs::copy(
            if name.ends_with(".mp4") {
                &video
            } else {
                "tests/fixtures/images/wide.png"
            },
            base.join(name),
        )
        .unwrap();
    }
    window::build(
        app,
        &Default::default(),
        Some(input::OpenedFile::from_file(&gio::File::for_path(base.join(names[2]))).unwrap()),
    );
    let window = app
        .active_window()
        .and_downcast::<adw::ApplicationWindow>()
        .unwrap();
    window.set_default_size(1150, 720);
    photo_presented(&window);
    pivra::settings::thumbnail_preference().set_enabled(true);
    let lookup = |name: &str| {
        widgets(window.upcast_ref())
            .into_iter()
            .find(|w| w.widget_name() == name)
    };
    let pin = lookup("thumbnail-controls")
        .unwrap()
        .last_child()
        .and_downcast::<gtk::ToggleButton>()
        .unwrap();
    if !pin.is_active() {
        pin.emit_clicked();
    }
    let key = development_features::native_key;
    let activate = |name| window.lookup_action(name).unwrap().activate(None);
    let focused = || {
        wait("viewer focused", || {
            window.is_active() && transient(&window).is_none()
        })
    };
    let dialog = || {
        wait("native dialog mapped", || transient(&window).is_some());
        transient(&window).unwrap()
    };
    let member = |index, ctrl: bool| {
        let name = format!("thumbnail-cell-{index}");
        wait("thumbnail target ready", || {
            lookup(&name).is_some_and(|w| w.is_mapped() && w.width() > 0)
        });
        if ctrl {
            assert!(
                std::process::Command::new("xdotool")
                    .args(["keydown", "Control_L"])
                    .status()
                    .unwrap()
                    .success()
            );
        }
        video_mouse::click(&window, &lookup(&name).unwrap(), "1", "1");
        if ctrl {
            assert!(
                std::process::Command::new("xdotool")
                    .args(["keyup", "Control_L"])
                    .status()
                    .unwrap()
                    .success()
            );
        }
    };
    let mark_pair = |first, next| {
        member(first, false);
        pump();
        member(next, true);
        wait("the effective group has two files", || {
            lookup("thumbnail-selection-count")
                .and_downcast::<gtk::Label>()
                .is_some_and(|label| label.text() == "2 selected")
        });
    };
    let completed = || window.lookup_action("undo").unwrap().is_enabled();
    let unname = "3-single.png";
    focused();
    key("F2");
    let rename = dialog();
    let entry = widgets(rename.upcast_ref())
        .into_iter()
        .find(|w| w.widget_name() == "rename-filename")
        .and_downcast::<gtk::Entry>()
        .unwrap();
    entry.set_text(unname);
    button(&rename, "Rename");
    wait("single Rename recorded", || {
        base.join(unname).is_file() && completed()
    });
    focused();

    mark_pair(0, 1);
    key("Delete");
    button(&dialog(), "Move to Trash");
    wait("grouped Trash recorded", || {
        !base.join(names[0]).exists() && !base.join(names[1]).exists() && completed()
    });
    focused();

    mark_pair(0, 1);
    key("F2");
    button(&dialog(), "Rename");
    let numbered = ["IMG_001.png", "IMG_002.png"];
    wait("grouped Rename recorded", || {
        numbered.into_iter().all(|name| base.join(name).is_file()) && completed()
    });
    focused();

    mark_pair(0, 1);
    activate("move-to");
    let picker = dialog();
    wait("folder chooser active", || picker.is_active());
    key("ctrl+l");
    wait("folder chooser editable location", || {
        gtk::prelude::GtkWindowExt::focus(&picker).is_some_and(|w| w.is::<gtk::Editable>())
    });
    assert!(
        std::process::Command::new("xdotool")
            .args([
                "type",
                "--clearmodifiers",
                "--",
                to.path().to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success()
    );
    key("Return");
    wait("move conflict policy confirmation", || {
        transient(&window).is_some_and(|w| w != picker)
    });
    button(&dialog(), "Skip Existing");
    wait("grouped Move recorded", || {
        numbered
            .into_iter()
            .all(|name| to.path().join(name).is_file())
            && numbered.into_iter().all(|name| !base.join(name).exists())
            && completed()
    });

    for (label, expected) in [
        ("Move reverses both transfers", &numbered[..]),
        ("Rename restores names", &[unname, names[3]][..]),
        ("Trash restores both media", &[names[0], names[1]][..]),
        ("Original single name restored", &[names[2]][..]),
    ] {
        focused();
        key("ctrl+z");
        wait(label, || {
            expected.iter().all(|name| base.join(name).is_file())
        });
    }
    wait("exactly four inverse steps consumed", || !completed());
    assert!(
        !base.join(unname).exists() && numbered.into_iter().all(|name| !base.join(name).exists())
    );
    assert_eq!(std::fs::read_dir(to.path()).unwrap().count(), 0);
    assert_eq!(
        std::fs::read(base.join("2.mp4")).unwrap(),
        std::fs::read(video).unwrap()
    );
    window.close();
    eprintln!(
        "PASS one GTK history: single Rename → batch Trash → batch Rename → group Move, four Ctrl+Z presses in reverse order"
    );
}
