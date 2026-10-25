//! Production window + GIO monitors with explicitly controlled ERROR workers.
//! No decoder success or secure decoder activity is simulated or certified.
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/input.rs"]
mod input;
#[path = "../src/window.rs"]
mod window;
use adw::prelude::*;
use gtk::{gio, glib};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::{Duration, Instant},
};
static PATHS: std::sync::Mutex<Vec<(usize, std::path::PathBuf)>> =
    std::sync::Mutex::new(Vec::new());
static STARTED: AtomicUsize = AtomicUsize::new(0);
static RELEASE: [AtomicBool; 7] = [const { AtomicBool::new(false) }; 7];
static FINISHED: [AtomicBool; 7] = [const { AtomicBool::new(false) }; 7];
static CANCELLED: [AtomicBool; 7] = [const { AtomicBool::new(false) }; 7];
mod image_loader {
    pub async fn preload(
        path: &std::path::Path,
        token: &gtk::gio::Cancellable,
        _budget: usize,
    ) -> Result<LoadedFrame, String> {
        load(path, token).await
    }
    pub(crate) type LoadedFrame = glycin::Frame;
    use super::*;
    pub async fn load(path: &Path, token: &gio::Cancellable) -> Result<glycin::Frame, String> {
        let id = STARTED.fetch_add(1, Ordering::SeqCst);
        PATHS.lock().unwrap().push((id, path.to_owned()));
        assert!(id < 7, "unexpected automatic retry/extra load");
        let end = Instant::now() + Duration::from_secs(30);
        while !RELEASE[id].load(Ordering::SeqCst) {
            assert!(Instant::now() < end, "controlled worker deadline");
            std::thread::sleep(Duration::from_millis(1));
        }
        CANCELLED[id].store(token.is_cancelled(), Ordering::SeqCst);
        FINISHED[id].store(true, Ordering::SeqCst);
        Err(format!("controlled window error {id}"))
    }
}
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
        assert!(Instant::now() < end, "window race deadline");
        pump();
    }
}
fn settle() {
    let end = Instant::now() + Duration::from_millis(400);
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
fn status(w: &adw::ApplicationWindow) -> String {
    let picture = widgets(w.upcast_ref())
        .into_iter()
        .find_map(|w| w.downcast::<gtk::Picture>().ok())
        .unwrap();
    picture
        .next_sibling()
        .unwrap()
        .downcast::<gtk::Label>()
        .unwrap()
        .text()
        .to_string()
}
fn banner(w: &adw::ApplicationWindow) -> bool {
    widgets(w.upcast_ref())
        .into_iter()
        .find_map(|w| w.downcast::<gtk::Revealer>().ok())
        .unwrap()
        .reveals_child()
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
    wait(|| w.is_mapped() && w.title().is_some_and(|s| s.ends_with("(1 candidate)")));
    w
}
fn action(w: &adw::ApplicationWindow, name: &str, count: usize) {
    gio::prelude::ActionGroupExt::activate_action(w, name, None);
    wait(|| STARTED.load(Ordering::SeqCst) == count);
}
fn mode_key(w: &adw::ApplicationWindow, key: gtk::gdk::Key, expected: &str) {
    let controllers = w.observe_controllers();
    let controller = (0..controllers.n_items())
        .find_map(|i| {
            controllers
                .item(i)
                .unwrap()
                .downcast::<gtk::EventControllerKey>()
                .ok()
        })
        .unwrap();
    assert!(controller.emit_by_name::<bool>(
        "key-pressed",
        &[&key, &0_u32, &gtk::gdk::ModifierType::empty()]
    ));
    let label = match expected {
        "fit" => "Fit",
        "fill" => "Fill",
        _ => "100%",
    };
    assert!(
        widgets(w.upcast_ref())
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::MenuButton>().ok())
            .any(|button| button.label().as_deref() == Some(label)),
        "mode menu label did not track action"
    );
    assert_eq!(
        w.lookup_action("image-mode")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some(expected)
    );
}

fn input_while_pending(w: &adw::ApplicationWindow) {
    let p = widgets(w.upcast_ref())
        .into_iter()
        .find_map(|w| w.downcast::<gtk::Picture>().ok())
        .unwrap();
    let list = p.observe_controllers();
    let scroll = (0..list.n_items())
        .find_map(|i| {
            list.item(i)
                .unwrap()
                .downcast::<gtk::EventControllerScroll>()
                .ok()
        })
        .unwrap();
    assert!(
        !scroll.emit_by_name::<bool>("scroll", &[&0_f64, &-1_f64]),
        "pending input must not fabricate a texture"
    );
    let drag = (0..list.n_items())
        .find_map(|i| list.item(i).unwrap().downcast::<gtk::GestureDrag>().ok())
        .unwrap();
    drag.emit_by_name::<()>("drag-begin", &[&100_f64, &100_f64]);
    drag.emit_by_name::<()>("drag-update", &[&200_f64, &200_f64]);
    drag.emit_by_name::<()>("drag-end", &[&200_f64, &200_f64]);
}

fn rename_pending(app: &adw::Application) {
    let dir = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let old = dir.path().join("old.png");
    let new = dir.path().join("renamed.png");
    std::fs::write(&old, b"controlled source").unwrap();
    std::fs::write(other.path().join("only.png"), b"controlled source").unwrap();
    let first = open(app, &old);
    wait(|| STARTED.load(Ordering::SeqCst) == 1);
    let second = open(app, &other.path().join("only.png"));
    wait(|| STARTED.load(Ordering::SeqCst) == 2);
    mode_key(&first, gtk::gdk::Key::F7, "fill");
    gio::prelude::ActionGroupExt::activate_action(
        &first,
        "image-resampling",
        Some(&"nearest".to_variant()),
    );
    std::fs::rename(&old, &new).unwrap();
    wait(|| STARTED.load(Ordering::SeqCst) == 3 && PATHS.lock().unwrap().len() == 3);
    assert!(
        PATHS
            .lock()
            .unwrap()
            .iter()
            .any(|(id, path)| *id == 2 && path == &new)
    );
    wait(|| first.title().is_some_and(|s| s.starts_with("renamed.png")));
    assert_eq!(
        first
            .lookup_action("image-mode")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("fill")
    );
    RELEASE[2].store(true, Ordering::SeqCst);
    wait(|| status(&first).contains("controlled window error 2"));
    RELEASE[0].store(true, Ordering::SeqCst);
    wait(|| FINISHED[0].load(Ordering::SeqCst));
    settle();
    assert!(CANCELLED[0].load(Ordering::SeqCst));
    assert!(status(&first).contains("controlled window error 2"));
    assert_eq!(
        first
            .lookup_action("image-resampling")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("nearest")
    );
    let completed = dir.path().join("completed.png");
    std::fs::rename(&new, &completed).unwrap();
    wait(|| {
        first
            .title()
            .is_some_and(|s| s.starts_with("completed.png"))
    });
    assert_eq!(
        STARTED.load(Ordering::SeqCst),
        3,
        "completed rename automatically decoded again"
    );
    first.close();
    RELEASE[1].store(true, Ordering::SeqCst);
    wait(|| status(&second).contains("controlled window error 1"));
    assert!(!CANCELLED[1].load(Ordering::SeqCst));
    second.close();
    assert_eq!(std::fs::read(completed).unwrap(), b"controlled source");
    println!(
        "PASS real GIO paired rename retargets pending JPEG/PNG request, preserves mode/filter, suppresses old path error and leaves completed-image rename decode-free; controlled worker, NOT real decoder"
    );
}

fn main() {
    adw::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.WindowRaceProbe")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    if std::env::args().any(|s| s == "--rename-pending") {
        rename_pending(&app);
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("0.png"), b"controlled fixture").unwrap();
    std::fs::write(other.path().join("9.png"), b"controlled fixture").unwrap();
    let first = open(&app, &dir.path().join("0.png"));
    wait(|| STARTED.load(Ordering::SeqCst) == 1);
    let second = open(&app, &other.path().join("9.png"));
    wait(|| STARTED.load(Ordering::SeqCst) == 2);
    assert_eq!(
        first
            .lookup_action("image-resampling")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("smooth")
    );
    gio::prelude::ActionGroupExt::activate_action(
        &first,
        "image-resampling",
        Some(&"nearest".to_variant()),
    );
    assert_eq!(
        second
            .lookup_action("image-resampling")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("smooth")
    );
    mode_key(&first, gtk::gdk::Key::F7, "fill");
    mode_key(&second, gtk::gdk::Key::F8, "actual");
    input_while_pending(&first);
    input_while_pending(&second);
    assert_eq!(
        first
            .lookup_action("image-mode")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("fill")
    );
    assert_eq!(STARTED.load(Ordering::SeqCst), 2);
    for name in ["1.png", "2.png"] {
        std::fs::write(dir.path().join(name), b"controlled fixture").unwrap();
    }
    wait(|| first.title().is_some_and(|s| s.ends_with("(3 candidates)")));
    action(&first, "next-media", 3);
    action(&first, "next-media", 4);
    assert_eq!(
        first
            .lookup_action("image-resampling")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("nearest")
    );
    assert_eq!(
        first
            .lookup_action("image-mode")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("fill") // Pending requests do not reset presentation preferences.
    );
    assert_eq!(
        second
            .lookup_action("image-mode")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("actual")
    );
    std::fs::write(
        dir.path().join("2.png"),
        b"external change while controlled load pending",
    )
    .unwrap();
    wait(|| banner(&first));
    settle();
    assert_eq!(
        STARTED.load(Ordering::SeqCst),
        4,
        "monitor automatically decoded"
    );
    assert!(!banner(&second));
    // Admission is bounded: release cancelled A before admitting refresh.
    RELEASE[0].store(true, Ordering::SeqCst);
    wait(|| FINISHED[0].load(Ordering::SeqCst));
    settle();
    action(&first, "reload-media", 5);
    RELEASE[2].store(true, Ordering::SeqCst);
    wait(|| FINISHED[2].load(Ordering::SeqCst));
    settle();
    action(&first, "reload-media", 6);
    std::fs::remove_file(dir.path().join("2.png")).unwrap();
    wait(|| {
        first
            .title()
            .is_some_and(|s| s.starts_with("No media selected"))
    });
    let removed_status = status(&first);
    for id in [5, 4, 3, 2, 0] {
        RELEASE[id].store(true, Ordering::SeqCst);
        wait(|| FINISHED[id].load(Ordering::SeqCst));
    }
    settle();
    assert_eq!(status(&first), removed_status);
    assert_eq!(
        first
            .lookup_action("image-resampling")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("nearest"),
        "late callback reset filter"
    );
    for id in [0, 2, 3, 4, 5] {
        assert!(CANCELLED[id].load(Ordering::SeqCst));
    }
    assert_eq!(
        status(&second),
        "",
        "pending load must not show loading text"
    );
    action(&first, "next-media", 7);
    // Close while a new worker and a monitor timer can both be pending.
    std::fs::write(dir.path().join("3.png"), b"closing event").unwrap();
    mode_key(&first, gtk::gdk::Key::F7, "fill");
    let weak = first.downgrade();
    first.close();
    drop(first);
    RELEASE[6].store(true, Ordering::SeqCst);
    wait(|| FINISHED[6].load(Ordering::SeqCst) && weak.upgrade().is_none());
    settle();
    assert!(CANCELLED[6].load(Ordering::SeqCst));
    assert!(second.is_visible());
    assert_eq!(
        status(&second),
        "",
        "pending load must not show loading text"
    );
    assert!(!banner(&second));
    RELEASE[1].store(true, Ordering::SeqCst);
    wait(|| status(&second).contains("controlled window error 1"));
    assert!(!CANCELLED[1].load(Ordering::SeqCst));
    mode_key(&second, gtk::gdk::Key::F6, "fit");
    assert_eq!(
        STARTED.load(Ordering::SeqCst),
        7,
        "unbounded/automatic retry"
    );
    assert!(second.title().unwrap().starts_with("9.png"));
    assert_eq!(
        second
            .lookup_action("image-resampling")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("smooth")
    );
    gio::prelude::ActionGroupExt::activate_action(
        &second,
        "image-resampling",
        Some(&"nearest".to_variant()),
    );
    second.close();
    println!(
        "PASS production-window/GIO controlled-worker races: monitor while loading, no auto retry, repeated refresh, selected deletion, stale callbacks, close and independent pending survivor"
    );
    println!(
        "NOT TESTED real Glycin decode/pixels or secure decoder activity/overlap; controlled error workers only"
    );
}
