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

static STARTED: AtomicUsize = AtomicUsize::new(0);
static RELEASE: [AtomicBool; 4] = [const { AtomicBool::new(false) }; 4];
static CANCELLED: [AtomicBool; 4] = [const { AtomicBool::new(false) }; 4];

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
    view.load_with_completion(index.to_string().into(), move || {
        joined.set(joined.get() + 1)
    });
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
    RELEASE[1].store(true, Ordering::SeqCst);
    wait(|| joined.get() == 1);
    assert!(text(&view).contains("controlled error 1"));
    RELEASE[0].store(true, Ordering::SeqCst);
    wait(|| joined.get() == 2);
    assert!(CANCELLED[0].load(Ordering::SeqCst));
    assert!(
        text(&view).contains("controlled error 1"),
        "late old callback replaced latest state"
    );
    load(&view, 2, &joined);
    view.show_message("Video playback is not available yet.");
    RELEASE[2].store(true, Ordering::SeqCst);
    wait(|| joined.get() == 3);
    assert!(CANCELLED[2].load(Ordering::SeqCst));
    assert_eq!(text(&view), "Video playback is not available yet.");
    load(&view, 3, &joined);
    let weak = first.downgrade();
    let widget = view.widget().downgrade();
    first.close();
    drop(view);
    drop(first);
    RELEASE[3].store(true, Ordering::SeqCst);
    wait(|| joined.get() == 4 && weak.upgrade().is_none() && widget.upgrade().is_none());
    assert!(CANCELLED[3].load(Ordering::SeqCst));
    assert!(second.is_visible());
    assert_eq!(text(&other), "Second window unchanged");
    second.close();
    wait(|| !second.is_visible());
    println!(
        "PASS controlled-error GTK/GIO race gate: reversed completion, video replacement, pending close, cancelled tokens, joined jobs, released widgets, independent window"
    );
    println!(
        "NOT TESTED Glycin decode, image pixels or real decoder cancellation; controlled workers only"
    );
}
