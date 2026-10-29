//! Real GIO events and production windows. Default requires secure Glycin pixels.
#[path = "../src/image_loader.rs"]
mod image_loader;
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/input.rs"]
mod input;
#[path = "support/media_failure_cases.rs"]
mod media_failure_cases;
#[path = "../src/window.rs"]
mod window;
use adw::prelude::*;
use gtk::{gio, glib};
use std::{
    path::Path,
    time::{Duration, Instant},
};
fn pump() {
    for _ in 0..100 {
        if !glib::MainContext::default().pending() {
            break;
        }
        glib::MainContext::default().iteration(false);
    }
    std::thread::sleep(Duration::from_millis(1));
}
fn wait(mut test: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(15);
    while !test() {
        assert!(Instant::now() < end, "folder watch deadline");
        pump();
    }
}
fn settle() {
    let end = Instant::now() + Duration::from_millis(500);
    while Instant::now() < end {
        pump();
    }
}
fn widgets(w: &gtk::Widget) -> Vec<gtk::Widget> {
    let mut result = vec![w.clone()];
    let mut child = w.first_child();
    while let Some(w) = child {
        result.extend(widgets(&w));
        child = w.next_sibling();
    }
    result
}
fn picture(w: &adw::ApplicationWindow) -> gtk::Picture {
    widgets(w.upcast_ref())
        .into_iter()
        .find_map(|w| w.downcast().ok())
        .unwrap()
}
fn banner(w: &adw::ApplicationWindow) -> gtk::Revealer {
    widgets(w.upcast_ref())
        .into_iter()
        .find_map(|w| w.downcast().ok())
        .unwrap()
}
fn action(w: &adw::ApplicationWindow, name: &str) {
    gio::prelude::ActionGroupExt::activate_action(w, name, None);
}
fn count(w: &adw::ApplicationWindow, n: usize) -> bool {
    w.title().is_some_and(|s| {
        s.ends_with(&format!(
            "({n} {})",
            if n == 1 { "candidate" } else { "candidates" }
        ))
    })
}
fn open(app: &adw::Application, path: &Path) -> adw::ApplicationWindow {
    let old = app.windows();
    let input = input::OpenedFile::from_argument(path.as_os_str()).unwrap();
    assert_eq!(input.file.path().as_deref(), Some(path));
    window::build(app, &Default::default(), Some(input));
    let w = app
        .windows()
        .into_iter()
        .find(|w| !old.contains(w))
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    wait(|| count(&w, 1) && w.is_mapped());
    w
}
fn pixels(w: &adw::ApplicationWindow, required: bool, baseline: bool) {
    if !required {
        settle();
        return;
    }
    let picture = picture(w);
    let status = picture
        .next_sibling()
        .unwrap()
        .downcast::<gtk::Label>()
        .unwrap();
    wait(|| picture.paintable().is_some() || status.text().starts_with("Unable to display image."));
    if picture.paintable().is_none() && baseline {
        eprintln!(
            "NOT RUN: secure baseline decode unavailable: {}",
            status.text()
        );
        std::process::exit(77);
    }
    assert!(
        picture.paintable().is_some(),
        "secure refresh decode failed: {}",
        status.text()
    );
}
fn main() {
    let native = !std::env::args().any(|s| s == "--expect-image-error");
    adw::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.FolderWatchProbe")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    if std::env::args().any(|s| s == "--media-failures") {
        media_failure_cases::run(&app, native);
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images");
    let initial = std::fs::read(fixtures.join("small.png")).unwrap();
    let replacement = std::fs::read(fixtures.join("tall.png")).unwrap();
    let selected = dir.path().join("2.png");
    std::fs::write(&selected, &initial).unwrap();
    std::fs::write(other.path().join("only.png"), &initial).unwrap();
    let w = open(&app, &selected);
    let second = open(&app, &other.path().join("only.png"));
    pixels(&w, native, true);
    pixels(&second, native, false);
    let original = picture(&w).paintable();
    std::fs::write(dir.path().join("1.mp4"), b"candidate only").unwrap();
    std::fs::write(dir.path().join("10.png"), &initial).unwrap();
    std::fs::write(dir.path().join("ignored.txt"), b"ignore").unwrap();
    wait(|| count(&w, 3));
    assert!(w.title().unwrap().starts_with("2.png"));
    assert_eq!(picture(&w).paintable(), original);
    assert!(!banner(&w).reveals_child());
    std::fs::write(dir.path().join("10.png"), &replacement).unwrap();
    settle();
    assert!(!banner(&w).reveals_child());
    // In-place save, independent of atomic replacement.
    std::fs::write(&selected, &initial).unwrap();
    wait(|| banner(&w).reveals_child());
    action(&w, "previous-media");
    assert!(w.title().unwrap().starts_with("1.mp4"));
    action(&w, "next-media");
    pixels(&w, native, false);
    let old = picture(&w).paintable();
    // Atomic save must notify without automatically replacing the current frame.
    let temp = dir.path().join(".save");
    std::fs::write(&temp, &replacement).unwrap();
    std::fs::rename(&temp, &selected).unwrap();
    wait(|| banner(&w).reveals_child());
    settle();
    assert_eq!(picture(&w).paintable(), old);
    action(&w, "reload-media");
    pixels(&w, native, false);
    if native {
        wait(|| !banner(&w).reveals_child());
        let frame = picture(&w).paintable().unwrap();
        assert_eq!(
            (frame.intrinsic_width(), frame.intrinsic_height()),
            (32, 64)
        );
        assert_ne!(Some(frame), old);
    } else {
        assert!(
            banner(&w).reveals_child(),
            "failed refresh must retain notice"
        );
    }
    std::fs::rename(&selected, dir.path().join("20.png")).unwrap();
    wait(|| w.title().is_some_and(|s| s.starts_with("20.png")));
    action(&w, "previous-media");
    assert!(w.title().unwrap().starts_with("10.png"));
    assert!(!banner(&w).reveals_child());
    for i in 30..80 {
        std::fs::write(dir.path().join(format!("{i}.mp4")), b"candidate").unwrap();
    }
    for _ in 0..20 {
        action(&w, "next-media");
        action(&w, "previous-media");
    }
    wait(|| count(&w, 53));
    assert!(w.title().unwrap().starts_with("10.png"));
    std::fs::remove_file(dir.path().join("10.png")).unwrap();
    wait(|| {
        w.title()
            .is_some_and(|s| s.starts_with("No media selected"))
    });
    assert!(picture(&w).paintable().is_none());
    action(&w, "next-media");
    wait(|| count(&w, 52));
    assert!(w.title().unwrap().starts_with("1.mp4"));
    assert!(count(&second, 1));
    assert!(!banner(&second).reveals_child());
    // Empty live catalog, then a new photo: do not auto-jump, arrows recover.
    for entry in std::fs::read_dir(dir.path()).unwrap() {
        std::fs::remove_file(entry.unwrap().path()).unwrap();
    }
    wait(|| {
        w.title()
            .is_some_and(|s| s.starts_with("No media selected"))
    });
    settle();
    std::fs::write(dir.path().join("new.png"), &initial).unwrap();
    settle();
    assert!(w.title().unwrap().starts_with("No media selected"));
    action(&w, "next-media");
    wait(|| count(&w, 1));
    // Preserve lexical alias policy, including content changes outside the folder.
    std::os::unix::fs::symlink(other.path().join("only.png"), dir.path().join("alias.png"))
        .unwrap();
    wait(|| count(&w, 2));
    action(&w, "previous-media");
    assert!(w.title().unwrap().starts_with("alias.png"));
    pixels(&w, native, false);
    let alias_frame = picture(&w).paintable();
    std::fs::write(other.path().join("only.png"), &replacement).unwrap();
    wait(|| banner(&w).reveals_child());
    settle();
    assert_eq!(picture(&w).paintable(), alias_frame);
    assert!(banner(&second).reveals_child());
    std::fs::remove_file(other.path().join("only.png")).unwrap();
    wait(|| {
        w.title()
            .is_some_and(|s| s.starts_with("No media selected"))
    });
    // Restore the independent window by explicit navigation, without auto-jump.
    std::fs::write(other.path().join("only.png"), &initial).unwrap();
    settle();
    action(&second, "next-media");
    wait(|| count(&second, 1));
    action(&w, "next-media");
    wait(|| count(&w, 2));
    // Rename the whole watched folder: controlled unavailable state, no crash.
    let moved = dir.path().with_extension("moved");
    std::fs::rename(dir.path(), &moved).unwrap();
    wait(|| {
        w.title()
            .is_some_and(|s| s.starts_with("No media selected"))
    });
    let weak = w.downgrade();
    w.close();
    drop(w);
    settle();
    assert!(weak.upgrade().is_none(), "monitor/window ownership leak");
    std::fs::write(moved.join("after-close.png"), &initial).unwrap();
    settle();
    assert!(count(&second, 1));
    second.close();
    drop(second);
    std::fs::remove_dir_all(moved).unwrap();
    println!(
        "PASS real filesystem monitor/add/video/delete/rename/order/atomic-save/no-auto-load/rapid-navigation/empty/recovery/folder-loss/independent-window/close cleanup"
    );
    if native {
        println!(
            "PASS secure Glycin refresh, new dimensions, retained old frame until user action"
        );
    } else {
        println!("NOT RUN secure pixel/refresh acceptance; scoped error-path GUI only");
    }
}
