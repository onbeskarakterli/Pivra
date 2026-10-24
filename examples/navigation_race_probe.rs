//! Deterministic GTK/GIO late-result guard test, with controlled ERROR workers.
//! Never decodes or simulates image pixels; not a Glycin/sandbox success gate.
#[path = "../src/image_view.rs"]
mod image_view;

use adw::prelude::*;
use gtk::{gio, glib};
use std::{
    cell::Cell,
    path::Path,
    rc::Rc,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::{Duration, Instant},
};

static RESULTS: AtomicUsize = AtomicUsize::new(0);
static STARTED: AtomicUsize = AtomicUsize::new(0);
static RELEASE: [AtomicBool; 8] = [const { AtomicBool::new(false) }; 8];
static CANCELLED: [AtomicBool; 8] = [const { AtomicBool::new(false) }; 8];

// Test-only controlled error boundary. The application still uses image_loader.rs
// and mandatory Bwrap; this module provides no decoder or production fallback.
mod image_loader {
    use super::*;
    pub async fn load(path: &Path, token: &gio::Cancellable) -> Result<glycin::Frame, String> {
        let index: usize = path.to_str().unwrap().parse().unwrap();
        STARTED.fetch_add(1, Ordering::SeqCst);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !RELEASE[index].load(Ordering::SeqCst) {
            assert!(Instant::now() < deadline, "controlled worker timeout");
            std::thread::sleep(Duration::from_millis(1));
        }
        CANCELLED[index].store(token.is_cancelled(), Ordering::SeqCst);
        Err(format!("controlled error {index}"))
    }
}

fn wait(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !ready() {
        assert!(Instant::now() < deadline, "race probe deadline exceeded");
        for _ in 0..100 {
            if !glib::MainContext::default().pending() {
                break;
            }
            glib::MainContext::default().iteration(false);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn text(view: &image_view::ImageView) -> String {
    view.widget()
        .child()
        .unwrap()
        .next_sibling()
        .unwrap()
        .downcast::<gtk::Label>()
        .unwrap()
        .text()
        .to_string()
}

fn load(view: &image_view::ImageView, index: usize, joined: &Rc<Cell<usize>>) {
    let joined = joined.clone();
    view.load_observed(
        index.to_string().into(),
        move || joined.set(joined.get() + 1),
        move |success| {
            assert!(!success, "controlled boundary never returns pixels");
            RESULTS.fetch_or(1 << index, Ordering::SeqCst);
        },
    );
    wait(|| STARTED.load(Ordering::SeqCst) == index + 1);
}

fn main() {
    if let Err(error) = adw::init() {
        eprintln!("BLOCKED GTK race test: {error}");
        std::process::exit(77);
    }
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.RaceProbe")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    let first = adw::ApplicationWindow::builder().application(&app).build();
    let view = image_view::ImageView::new(&first);
    first.set_content(Some(view.widget()));
    first.present();
    let second = adw::ApplicationWindow::builder().application(&app).build();
    let other = image_view::ImageView::new(&second);
    other.show_message("Second window unchanged");
    second.set_content(Some(other.widget()));
    second.present();
    wait(|| first.is_mapped() && second.is_mapped());
    let joined = Rc::new(Cell::new(0));
    load(&view, 0, &joined);
    load(&view, 1, &joined);
    load(&view, 2, &joined);
    gio::prelude::ActionGroupExt::activate_action(&first, "image-mode", Some(&"fill".to_variant()));
    RELEASE[2].store(true, Ordering::SeqCst);
    wait(|| joined.get() == 1);
    assert!(text(&view).contains("controlled error 2"));
    for index in [1, 0] {
        RELEASE[index].store(true, Ordering::SeqCst);
        wait(|| joined.get() == 3 - index);
        assert!(CANCELLED[index].load(Ordering::SeqCst));
        assert!(text(&view).contains("controlled error 2"));
    }
    assert_eq!(
        first
            .lookup_action("image-mode")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("fill")
    );
    load(&view, 3, &joined);
    assert_eq!(
        first
            .lookup_action("image-mode")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("fit")
    );
    view.show_message("Video playback is not available yet.");
    RELEASE[3].store(true, Ordering::SeqCst);
    wait(|| joined.get() == 4);
    assert!(CANCELLED[3].load(Ordering::SeqCst));
    assert_eq!(text(&view), "Video playback is not available yet.");
    load(&view, 4, &joined);
    load(&other, 5, &joined); // survivor has a genuinely pending controlled worker
    let weak = first.downgrade();
    let widget = view.widget().downgrade();
    first.close();
    drop(view);
    drop(first);
    RELEASE[4].store(true, Ordering::SeqCst);
    wait(|| joined.get() == 5 && weak.upgrade().is_none() && widget.upgrade().is_none());
    assert!(CANCELLED[4].load(Ordering::SeqCst));
    assert!(second.is_visible());
    assert_eq!(text(&other), "Loading image securely…");
    RELEASE[5].store(true, Ordering::SeqCst);
    wait(|| joined.get() == 6);
    assert!(!CANCELLED[5].load(Ordering::SeqCst));
    assert!(text(&other).contains("controlled error 5"));
    // Repeated load/refresh invalidates the earlier result notification too.
    load(&other, 6, &joined);
    load(&other, 7, &joined);
    RELEASE[6].store(true, Ordering::SeqCst);
    wait(|| joined.get() == 7);
    assert!(CANCELLED[6].load(Ordering::SeqCst));
    assert_eq!(text(&other), "Loading image securely…");
    RELEASE[7].store(true, Ordering::SeqCst);
    wait(|| joined.get() == 8);
    assert!(text(&other).contains("controlled error 7"));
    assert_eq!(
        RESULTS.load(Ordering::SeqCst),
        (1 << 2) | (1 << 5) | (1 << 7)
    );
    gio::prelude::ActionGroupExt::activate_action(
        &second,
        "image-mode",
        Some(&"actual".to_variant()),
    );
    second.close();
    wait(|| !second.is_visible());
    println!(
        "PASS controlled-error GTK/GIO race gate: A/B/C reversed completion, video replacement, pending close with active survivor worker, repeated loads, stale result notifications, cancelled tokens, joined jobs, released widgets"
    );
    println!(
        "NOT TESTED Glycin decode, image pixels or real decoder cancellation; controlled workers only"
    );
}
