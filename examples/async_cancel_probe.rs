//! Default secure native gate. --controlled uses explicit synthetic successes/errors.
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "support/render_frame.rs"]
mod render_frame;
#[path = "../src/image_loader.rs"]
mod secure_loader;
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use std::{
    cell::Cell,
    path::Path,
    rc::Rc,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::{Duration, Instant},
};
static CONTROLLED: AtomicBool = AtomicBool::new(false);
static STARTED: AtomicUsize = AtomicUsize::new(0);
static TEXTURES: AtomicUsize = AtomicUsize::new(0);
static RELEASE: [AtomicBool; 16] = [const { AtomicBool::new(false) }; 16];
static CANCELLED: [AtomicBool; 16] = [const { AtomicBool::new(false) }; 16];
static REQUESTS: std::sync::Mutex<Vec<(std::path::PathBuf, gio::Cancellable)>> =
    std::sync::Mutex::new(Vec::new());
mod image_loader {
    pub async fn preload(
        path: &std::path::Path,
        token: &gtk::gio::Cancellable,
        budget: usize,
    ) -> Result<LoadedFrame, String> {
        if CONTROLLED.load(Ordering::SeqCst) {
            load(path, token).await
        } else {
            secure_loader::preload(path, token, budget)
                .await
                .map(LoadedFrame::Secure)
        }
    }
    use super::*;
    pub enum LoadedFrame {
        Secure(glycin::Frame),
        Synthetic(usize),
    }
    impl LoadedFrame {
        pub fn texture(&self) -> gdk::Texture {
            match self {
                Self::Secure(frame) => frame.texture(),
                Self::Synthetic(id) => {
                    TEXTURES.fetch_add(1, Ordering::SeqCst);
                    let rgb = match id {
                        2 => [0, 0, 255],
                        5 => [0, 255, 0],
                        9 => [255, 0, 0],
                        11 => [255, 0, 255],
                        13 => [0, 255, 255],
                        15 => [30, 200, 40],
                        _ => [255, 255, 0],
                    };
                    let mut data = Vec::with_capacity(800 * 600 * 4);
                    for _ in 0..600 {
                        for x in 0..800 {
                            let color = if !(180..620).contains(&x) {
                                [80, 80, 80]
                            } else {
                                rgb
                            };
                            data.extend_from_slice(&[color[0], color[1], color[2], 255]);
                        }
                    }
                    gdk::MemoryTexture::new(
                        800,
                        600,
                        gdk::MemoryFormat::R8g8b8a8,
                        &glib::Bytes::from_owned(data),
                        800 * 4,
                    )
                    .upcast()
                }
            }
        }
    }
    pub async fn load(path: &Path, token: &gio::Cancellable) -> Result<LoadedFrame, String> {
        if !CONTROLLED.load(Ordering::SeqCst) {
            return secure_loader::load(path, token)
                .await
                .map(LoadedFrame::Secure);
        }
        let id = STARTED.fetch_add(1, Ordering::SeqCst);
        assert!(id < 16, "unexpected decode/retry");
        REQUESTS
            .lock()
            .unwrap()
            .push((path.to_owned(), token.clone()));
        let end = Instant::now() + Duration::from_secs(40);
        while !RELEASE[id].load(Ordering::SeqCst) {
            assert!(Instant::now() < end, "controlled barrier deadline");
            std::thread::sleep(Duration::from_millis(1));
        }
        CANCELLED[id].store(token.is_cancelled(), Ordering::SeqCst);
        if [1, 4, 7, 8, 14].contains(&id) {
            Err(format!("controlled late error {id}"))
        } else {
            Ok(LoadedFrame::Synthetic(id))
        }
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
fn wait(mut ready: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(25);
    while !ready() {
        assert!(Instant::now() < end, "async probe deadline");
        pump();
    }
}
fn open(app: &adw::Application) -> (adw::ApplicationWindow, image_view::ImageView) {
    let w = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(480)
        .default_height(320)
        .build();
    let v = image_view::ImageView::new(&w);
    w.set_content(Some(v.widget()));
    w.present();
    wait(|| w.is_mapped());
    (w, v)
}
fn picture(v: &image_view::ImageView) -> gtk::Picture {
    v.widget().child().unwrap().downcast().unwrap()
}
fn status(v: &image_view::ImageView) -> gtk::Label {
    picture(v).next_sibling().unwrap().downcast().unwrap()
}
fn choose(w: &adw::ApplicationWindow, action: &str, id: &str) {
    gio::prelude::ActionGroupExt::activate_action(w, action, Some(&id.to_variant()));
}
fn state(w: &adw::ApplicationWindow, action: &str) -> String {
    w.lookup_action(action)
        .unwrap()
        .state()
        .unwrap()
        .str()
        .unwrap()
        .into()
}
fn pixels(w: &adw::ApplicationWindow, v: &image_view::ImageView) -> (usize, usize, Vec<u8>) {
    let p = picture(v);
    let node = render_frame::rendered_node(w, &p);
    let t = w.renderer().unwrap().render_texture(
        node,
        Some(&gtk::graphene::Rect::new(
            0.,
            0.,
            p.width() as f32,
            p.height() as f32,
        )),
    );
    let width = t.width() as usize;
    let height = t.height() as usize;
    let mut bytes = vec![0; width * height * 4];
    t.download(&mut bytes, width * 4);
    (width, height, bytes)
}
fn center(w: &adw::ApplicationWindow, v: &image_view::ImageView) -> [u8; 3] {
    let (width, height, bytes) = pixels(w, v);
    let i = (height / 2 * width + width / 2) * 4;
    [bytes[i + 2], bytes[i + 1], bytes[i]]
}

fn request(v: &image_view::ImageView, path: &str, joined: &Rc<Cell<usize>>) {
    let count = joined.clone();
    v.load_with_completion(path.into(), move || count.set(count.get() + 1));
}
fn release(id: usize, joined: &Rc<Cell<usize>>, expected: usize) {
    RELEASE[id].store(true, Ordering::SeqCst);
    wait(|| joined.get() == expected);
}
fn controlled(app: &adw::Application) {
    let (w, v) = open(app);
    let joined = Rc::new(Cell::new(0));
    for (i, name) in ["A", "B", "C"].into_iter().enumerate() {
        request(&v, name, &joined);
        wait(|| STARTED.load(Ordering::SeqCst) == i + 1);
    }
    choose(&w, "image-mode", "fill");
    choose(&w, "image-resampling", "nearest");
    release(2, &joined, 1);
    assert_eq!(center(&w, &v), [0, 0, 255]);
    assert!(!status(&v).is_visible());
    let settled_status = status(&v).text();
    release(0, &joined, 2);
    release(1, &joined, 3);
    assert_eq!(center(&w, &v), [0, 0, 255]);
    assert_eq!(TEXTURES.load(Ordering::SeqCst), 1);
    assert_eq!(status(&v).text(), settled_status);
    assert!(!status(&v).is_visible());
    assert!(v.load_stats().suppressed >= 2);
    assert_eq!(state(&w, "image-mode"), "fill");
    assert_eq!(state(&w, "image-resampling"), "nearest");
    let retained = picture(&v).paintable();
    let retained_pixels = pixels(&w, &v);
    for (i, name) in ["A", "B", "A"].into_iter().enumerate() {
        request(&v, name, &joined);
        wait(|| STARTED.load(Ordering::SeqCst) == i + 4);
        assert_eq!(
            picture(&v).paintable(),
            retained,
            "pending selection cleared the last ready image"
        );
        assert_eq!(
            pixels(&w, &v),
            retained_pixels,
            "pending repaint changed last ready pixels"
        );
    }
    {
        let records = REQUESTS.lock().unwrap();
        assert_eq!(records[3].0, records[5].0);
        assert_ne!(records[3].1, records[5].1);
    }
    choose(&w, "image-mode", "actual");
    release(5, &joined, 4);
    let p = picture(&v);
    let list = p.observe_controllers();
    let scroll = (0..list.n_items())
        .find_map(|i| {
            list.item(i)
                .unwrap()
                .downcast::<gtk::EventControllerScroll>()
                .ok()
        })
        .unwrap();
    scroll.emit_by_name::<bool>("scroll", &[&0_f64, &-1_f64]);
    assert_eq!(state(&w, "image-mode"), "manual");
    choose(&w, "image-resampling", "smooth");
    release(3, &joined, 5);
    release(4, &joined, 6);
    assert_eq!(center(&w, &v), [0, 255, 0]);
    assert_eq!(state(&w, "image-mode"), "manual");
    assert_eq!(state(&w, "image-resampling"), "smooth");
    let drag = (0..list.n_items())
        .find_map(|i| list.item(i).unwrap().downcast::<gtk::GestureDrag>().ok())
        .unwrap();
    drag.emit_by_name::<()>("drag-begin", &[&100f64, &100f64]);
    drag.emit_by_name::<()>("drag-update", &[&20f64, &-10f64]);
    drag.emit_by_name::<()>("drag-end", &[&20f64, &-10f64]);
    let posed = pixels(&w, &v);
    for i in 0..3 {
        request(&v, "held", &joined);
        wait(|| STARTED.load(Ordering::SeqCst) == i + 7);
        assert_eq!(
            pixels(&w, &v),
            posed,
            "pending load reset zoom/pan or cleared image"
        );
        assert_eq!(state(&w, "image-mode"), "manual");
    }
    for i in 0..1000 {
        request(&v, &format!("queued-{i}"), &joined);
    }
    let stats = v.load_stats();
    assert_eq!((stats.running, stats.pending, stats.max_running), (3, 1, 3));
    assert_eq!(STARTED.load(Ordering::SeqCst), 9);
    let responsive = Rc::new(Cell::new(false));
    let tick = responsive.clone();
    glib::idle_add_local_once(move || tick.set(true));
    wait(|| responsive.get());
    release(6, &joined, 7);
    wait(|| STARTED.load(Ordering::SeqCst) == 10);
    assert_eq!(REQUESTS.lock().unwrap()[9].0, Path::new("queued-999"));
    release(9, &joined, 8);
    release(7, &joined, 9);
    release(8, &joined, 10);
    assert_eq!(center(&w, &v), [255, 0, 0]);
    assert!(v.load_stats().dropped_pending >= 999);
    assert_eq!((v.load_stats().running, v.load_stats().pending), (0, 0));
    // Reentrant result observer starts a new request: old completion must stop mutating UI.
    let next = v.clone();
    let count = joined.clone();
    let joined_next = joined.clone();
    v.load_observed(
        "trigger".into(),
        move || count.set(count.get() + 1),
        move |success| {
            assert!(success);
            request(&next, "reentrant latest", &joined_next);
        },
    );
    wait(|| STARTED.load(Ordering::SeqCst) == 11);
    release(10, &joined, 11);
    wait(|| STARTED.load(Ordering::SeqCst) == 12);
    assert!(picture(&v).paintable().is_some());
    assert_eq!(
        center(&w, &v),
        [255, 255, 0],
        "reentrant next request must retain just-published frame"
    );
    // c9e16cc deliberately removed transient loading text. Verify the latest
    // request still owns FullLoading without reviving an old callback's UI.
    assert!(!status(&v).is_visible());
    assert_eq!(status(&v).text(), "");
    assert_eq!(v.phase(), pivra::image_pipeline::Phase::FullLoading);
    release(11, &joined, 12);
    assert_eq!(center(&w, &v), [255, 0, 255]);
    let (other, survivor) = open(app);
    request(&v, "same-file", &joined);
    wait(|| STARTED.load(Ordering::SeqCst) == 13);
    v.load("same-file".into());
    v.load("same-file".into());
    assert_eq!(
        STARTED.load(Ordering::SeqCst),
        13,
        "duplicate active load decoded again"
    );
    request(&survivor, "same-file", &joined);
    wait(|| STARTED.load(Ordering::SeqCst) == 14);
    {
        let r = REQUESTS.lock().unwrap();
        assert_eq!(r[12].0, r[13].0);
        assert_ne!(r[12].1, r[13].1);
    }
    let weak = w.downgrade();
    let widget = v.widget().downgrade();
    w.close();
    drop(p);
    drop(v);
    drop(w);
    release(12, &joined, 13);
    wait(|| weak.upgrade().is_none() && widget.upgrade().is_none());
    assert_eq!(status(&survivor).text(), "");
    assert!(!status(&survivor).is_visible());
    assert_eq!(survivor.phase(), pivra::image_pipeline::Phase::FullLoading);
    release(13, &joined, 14);
    assert!(!CANCELLED[13].load(Ordering::SeqCst));
    assert!(CANCELLED[12].load(Ordering::SeqCst));
    assert_eq!(center(&other, &survivor), [0, 255, 255]);
    let stats = survivor.load_stats();
    assert_eq!((stats.running, stats.pending), (0, 0));
    assert_eq!(TEXTURES.load(Ordering::SeqCst), 6);
    let last = picture(&survivor).paintable();
    let last_pixels = pixels(&other, &survivor);
    request(&survivor, "broken-latest.png", &joined);
    wait(|| STARTED.load(Ordering::SeqCst) == 15);
    assert_eq!(picture(&survivor).paintable(), last);
    assert_eq!(pixels(&other, &survivor), last_pixels);
    release(14, &joined, 15);
    assert_eq!(survivor.phase(), pivra::image_pipeline::Phase::Failed);
    assert_eq!(picture(&survivor).paintable(), last);
    assert_eq!(pixels(&other, &survivor), last_pixels);
    assert!(status(&survivor).is_visible());
    assert!(status(&survivor).text().contains("broken-latest.png"));
    assert!(status(&survivor).text().contains("Previous photo"));
    request(&survivor, "recovery.png", &joined);
    wait(|| STARTED.load(Ordering::SeqCst) == 16);
    assert_eq!(picture(&survivor).paintable(), last);
    assert!(!status(&survivor).is_visible());
    release(15, &joined, 16);
    assert_eq!(
        center(&other, &survivor),
        [30, 200, 40],
        "Actual paintable kept previous texture"
    );
    assert_eq!(survivor.phase(), pivra::image_pipeline::Phase::FullReady);
    assert!(!status(&survivor).is_visible());
    assert_eq!(TEXTURES.load(Ordering::SeqCst), 7);

    other.close();
    println!(
        "PASS controlled synthetic success/error GTK/GIO: C-before-A/B pixels, A/B/A identity, 1000 latest-only pending requests, max3 workers, idle responsiveness, retained full-frame pixels/zoom/pan through slow load, named failure keeps previous photo/recovery, reentrant handoff, same-file two-window cancellation, joined jobs and weak cleanup"
    );
    println!(
        "NOT TESTED real Glycin decode, active secure decoder cancellation or process cleanup; synthetic frames and controlled noncooperative barriers only"
    );
}
fn timeout_gate(app: &adw::Application) {
    let (w, v) = open(app);
    let joined = Rc::new(Cell::new(0));
    for i in 0..3 {
        request(&v, "held", &joined);
        wait(|| STARTED.load(Ordering::SeqCst) == i + 1);
    }
    request(&v, "pending", &joined);
    wait(|| status(&v).text().contains("Image loading is busy"));
    assert_eq!(
        joined.get(),
        0,
        "admission timeout falsely reported worker join"
    );
    assert_eq!(
        (
            v.load_stats().running,
            v.load_stats().pending,
            v.load_stats().timed_out
        ),
        (3, 0, 1)
    );
    assert_eq!(STARTED.load(Ordering::SeqCst), 3);
    assert!(picture(&v).paintable().is_none());
    request(&v, "close pending", &joined);
    assert_eq!(v.load_stats().pending, 1);
    w.close();
    for release in RELEASE.iter().take(3) {
        release.store(true, Ordering::SeqCst);
    }
    wait(|| joined.get() == 3);
    assert_eq!((v.load_stats().running, v.load_stats().pending), (0, 0));
    assert_eq!(STARTED.load(Ordering::SeqCst), 3);
    assert_eq!(TEXTURES.load(Ordering::SeqCst), 0);
    println!(
        "PASS controlled admission timeout: five-second UI busy error, no false worker join, no retry, close drops pending, cancelled jobs joined; NOT TESTED decoder stop"
    );
}

fn native(app: &adw::Application) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images");
    let (w, v) = open(app);
    let (other, survivor) = open(app);
    let a = root.join("large.png");
    let b = root.join("tall.png");
    let c = root.join("wide.png");
    let originals = [&a, &b, &c].map(|p| std::fs::read(p).unwrap());
    v.load(a.clone());
    survivor.load(a.clone());
    wait(|| {
        matches!(
            v.phase(),
            pivra::image_pipeline::Phase::FullReady | pivra::image_pipeline::Phase::Failed
        )
    });
    if picture(&v).paintable().is_none() {
        eprintln!("NOT RUN secure async baseline: {}", status(&v).text());
        std::process::exit(77);
    }
    wait(|| {
        matches!(
            survivor.phase(),
            pivra::image_pipeline::Phase::FullReady | pivra::image_pipeline::Phase::Failed
        )
    });
    assert!(picture(&survivor).paintable().is_some());
    let preserved = picture(&survivor).paintable();
    let photo = picture(&v);
    let gap = Rc::new(Cell::new(false));
    let seen = gap.clone();
    let monitor = photo.connect_paintable_notify(move |p| {
        if p.paintable().is_none() {
            seen.set(true);
        }
    });

    for _ in 0..50 {
        v.load(a.clone());
        v.load(b.clone());
    }
    v.load(c.clone());
    choose(&w, "image-mode", "fill");
    choose(&w, "image-resampling", "nearest");
    wait(|| {
        matches!(
            v.phase(),
            pivra::image_pipeline::Phase::FullReady | pivra::image_pipeline::Phase::Failed
        )
    });
    assert!(
        picture(&v).paintable().is_some(),
        "latest valid decode failed after baseline"
    );
    wait(|| v.load_stats().running == 0 && v.load_stats().pending == 0);
    assert!(v.load_stats().max_running <= 3);
    assert_eq!(state(&w, "image-mode"), "fill");
    assert_eq!(state(&w, "image-resampling"), "nearest");
    choose(&w, "image-resampling", "smooth");
    choose(&w, "image-mode", "fit");
    let decoded = picture(&v)
        .paintable()
        .unwrap()
        .downcast::<gdk::Texture>()
        .unwrap();
    assert_eq!((decoded.width(), decoded.height()), (64, 32));
    let mut pixels = vec![0; 64 * 32 * 4];
    decoded.download(&mut pixels, 64 * 4);
    for (x, y, rgb) in [
        (16, 8, [255, 0, 0]),
        (48, 8, [0, 255, 0]),
        (16, 24, [0, 0, 255]),
        (48, 24, [255, 255, 0]),
    ] {
        let i = (y * 64 + x) * 4;
        assert_eq!([pixels[i + 2], pixels[i + 1], pixels[i]], rgb);
    }
    v.load(a.clone());
    v.load(b.clone());
    v.load(a.clone());
    wait(|| {
        matches!(
            v.phase(),
            pivra::image_pipeline::Phase::FullReady | pivra::image_pipeline::Phase::Failed
        )
    });
    assert!(picture(&v).paintable().is_some());
    wait(|| v.load_stats().running == 0);
    assert_eq!(picture(&v).paintable().unwrap().intrinsic_width(), 1600);
    assert_eq!(picture(&survivor).paintable(), preserved);
    v.load(root.join("broken.png"));
    wait(|| status(&v).text().starts_with("Unable"));
    v.load(c.clone());
    wait(|| {
        matches!(
            v.phase(),
            pivra::image_pipeline::Phase::FullReady | pivra::image_pipeline::Phase::Failed
        )
    });
    assert!(picture(&v).paintable().is_some());
    assert!(!gap.get(), "secure transition emitted an empty paintable");
    photo.disconnect(monitor);
    v.load(a.clone());
    w.close();
    survivor.load(c.clone());
    wait(|| {
        matches!(
            survivor.phase(),
            pivra::image_pipeline::Phase::FullReady | pivra::image_pipeline::Phase::Failed
        )
    });
    assert!(picture(&survivor).paintable().is_some());
    wait(|| v.load_stats().running == 0 && survivor.load_stats().running == 0);
    other.close();
    for (path, bytes) in [&a, &b, &c].into_iter().zip(originals) {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    println!(
        "PASS secure async baseline/latest pixels/A-B-A/error recovery/cancellable burst/two-window isolation/source integrity; stats {:?}",
        v.load_stats()
    );
    println!(
        "NOT MEASURED decoder active interval/overlap in burst; separate PIV-09 checkpoint certifies active secure close/process cleanup"
    );
}
fn main() {
    CONTROLLED.store(
        std::env::args().any(|s| s == "--controlled"),
        Ordering::SeqCst,
    );
    adw::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.AsyncCancelProbe")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    if CONTROLLED.load(Ordering::SeqCst) {
        if std::env::args().any(|s| s == "--queue-timeout") {
            timeout_gate(&app);
        } else {
            controlled(&app);
        }
    } else {
        native(&app);
    }
}
