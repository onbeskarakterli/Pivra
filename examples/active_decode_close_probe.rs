//! Run through tests/active_decode_close.py, not directly: its parent certifies
//! a real, partially written secure-loader output before sending CLOSE.
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/image_loader.rs"]
mod secure_loader;

use adw::prelude::*;
use gtk::{gio, glib};
use std::{
    cell::Cell,
    io::{BufRead, Write},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

static TARGET: OnceLock<PathBuf> = OnceLock::new();
static ENTERED: AtomicBool = AtomicBool::new(false);
static RETURNED: AtomicBool = AtomicBool::new(false);
static JOB_JOINED: AtomicBool = AtomicBool::new(false);

// Test-only observation around the unchanged production loader. The real view
// calls this boundary; no mocked frame, decoder or alternate sandbox is used.
mod image_loader {
    pub async fn preload(
        path: &std::path::Path,
        token: &gtk::gio::Cancellable,
        budget: usize,
    ) -> Result<LoadedFrame, String> {
        secure_loader::preload(path, token, budget).await
    }
    pub(crate) type LoadedFrame = glycin::Frame;
    use super::*;
    pub async fn load(path: &Path, cancel: &gio::Cancellable) -> Result<glycin::Frame, String> {
        let target = TARGET.get().is_some_and(|p| p == path);
        if target {
            ENTERED.store(true, Ordering::SeqCst);
            event("TARGET_LOAD_ENTERED");
        }
        let result = secure_loader::load(path, cancel).await;
        if target {
            RETURNED.store(true, Ordering::SeqCst);
            match &result {
                Ok(_) => event("TARGET_LOAD_RETURNED ok"),
                Err(error) => event(&format!(
                    "TARGET_LOAD_RETURNED error={}",
                    error.lines().next().unwrap_or("unknown")
                )),
            }
        }
        result
    }
}

fn event(message: &str) {
    println!("PIV09 {message}");
    std::io::stdout().flush().unwrap();
}

fn pump() {
    // Bound each dispatch batch so deadlines remain enforceable.
    for _ in 0..100 {
        if !glib::MainContext::default().pending() {
            break;
        }
        glib::MainContext::default().iteration(false);
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn wait(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !condition() {
        assert!(Instant::now() < deadline, "GUI/worker deadline exceeded");
        pump();
    }
}

fn command(commands: &mpsc::Receiver<String>, expected: &str) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Ok(command) = commands.try_recv() {
            assert_eq!(command, expected, "parent protocol");
            return;
        }
        assert!(Instant::now() < deadline, "parent command timeout");
        pump();
    }
}

struct View {
    window: adw::ApplicationWindow,
    picture: gtk::Picture,
    status: gtk::Label,
}

fn view(app: &adw::Application, path: PathBuf) -> View {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(640)
        .default_height(480)
        .build();
    let overlay = if TARGET.get().is_some_and(|target| *target == path) {
        image_view::build_with_completion(&window, path, || {
            JOB_JOINED.store(true, Ordering::SeqCst);
        })
    } else {
        image_view::build(&window, path)
    };
    let picture = overlay.child().unwrap().downcast::<gtk::Picture>().unwrap();
    let status = picture
        .next_sibling()
        .unwrap()
        .downcast::<gtk::Label>()
        .unwrap();
    window.set_content(Some(&overlay));
    window.present();
    View {
        window,
        picture,
        status,
    }
}

fn pixels(picture: &gtk::Picture) -> Vec<u8> {
    let texture = picture
        .paintable()
        .unwrap()
        .downcast::<gtk::gdk::Texture>()
        .unwrap();
    let stride = texture.width() as usize * 4;
    let mut pixels = vec![0; stride * texture.height() as usize];
    texture.download(&mut pixels, stride);
    pixels
}

fn main() {
    let path = PathBuf::from(std::env::args_os().nth(1).expect("use Python test driver"));
    TARGET.set(path.clone()).unwrap();
    let (send, commands) = mpsc::channel();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            let Ok(line) = line else { break };
            if send.send(line).is_err() {
                break;
            }
        }
    });
    if let Err(error) = adw::init() {
        event(&format!("BLOCKED GTK: {error}"));
        std::process::exit(77);
    }
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.ActiveDecodeProbe")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    let survivor = view(
        &app,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images/wide.png"),
    );
    wait(|| survivor.picture.paintable().is_some() || survivor.status.text().starts_with("Unable"));
    if survivor.picture.paintable().is_none() {
        event(&format!(
            "BLOCKED baseline secure decode: {}",
            survivor.status.text()
        ));
        survivor.window.close();
        std::process::exit(77);
    }
    let original = pixels(&survivor.picture);
    event("READY survivor_loaded");
    command(&commands, "START");
    event("TARGET_STARTING");
    // Observation must be armed before the real view can spawn its loader.
    command(&commands, "ARMED");
    let target = view(&app, path);
    let closed = Rc::new(Cell::new(false));
    let mutations = Rc::new(Cell::new(0_u32));
    for (object, property) in [
        (target.picture.clone().upcast::<glib::Object>(), "paintable"),
        (target.status.clone().upcast::<glib::Object>(), "label"),
        (target.status.clone().upcast::<glib::Object>(), "visible"),
    ] {
        let closed = closed.clone();
        let mutations = mutations.clone();
        object.connect_notify_local(Some(property), move |_, _| {
            if closed.get() {
                mutations.set(mutations.get() + 1);
            }
        });
    }
    command(&commands, "CLOSE");
    // Parent has stopped a verified incomplete frame. These guards independently
    // reject a queued job or an already returned result at the close checkpoint.
    assert!(
        ENTERED.load(Ordering::SeqCst),
        "worker never entered loader"
    );
    assert!(!RETURNED.load(Ordering::SeqCst), "worker already returned");
    assert!(
        target.picture.paintable().is_none(),
        "target already displayed"
    );
    assert_eq!(target.status.text().as_str(), "Loading image securely…");
    let weak_window = target.window.downgrade();
    let weak_picture = target.picture.downgrade();
    let weak_status = target.status.downgrade();
    closed.set(true);
    target.window.close();
    assert!(!target.window.is_visible());
    drop(target);
    event("WINDOW_CLOSED entered=true returned_at_checkpoint=false");
    // Parent resumes the loader immediately on WINDOW_CLOSED; production
    // cancellation must settle the worker and discard the late result normally.
    wait(|| {
        RETURNED.load(Ordering::SeqCst)
            && JOB_JOINED.load(Ordering::SeqCst)
            && weak_window.upgrade().is_none()
            && weak_picture.upgrade().is_none()
            && weak_status.upgrade().is_none()
    });
    let grace = Instant::now() + Duration::from_millis(300);
    while Instant::now() < grace {
        pump();
    }
    assert_eq!(mutations.get(), 0, "stale result mutated closed widgets");
    assert!(survivor.window.is_visible());
    assert!(!survivor.status.is_visible());
    assert_eq!(pixels(&survivor.picture), original);
    event(
        "SETTLED load_returned=true job_joined=true widgets_released=true stale_mutations=0 survivor_pixels_unchanged=true",
    );
    command(&commands, "FINISH");
    survivor.window.close();
    wait(|| !survivor.window.is_visible());
    event("PASS active_decode_close");
}
