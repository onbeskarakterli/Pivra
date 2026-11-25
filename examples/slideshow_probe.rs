//! Real production windows/catalog/GTK/GIO. Controlled photo pixels are explicitly
//! labelled; --native uses secure Glycin and requires physical acceptance separately.
#[path = "../src/raw_loader.rs"]
mod raw_loader;
#[path = "../src/raw_ppm.rs"]
mod raw_ppm;
#[path = "../src/image_loader.rs"]
#[allow(dead_code)]
mod secure_image_loader;
mod image_loader {
    use gtk::{gdk, gio, glib, prelude::*};
    use std::path::Path;
    pub enum LoadedFrame {
        Native(secure_image_loader::LoadedFrame),
        Controlled(bool, (i32, i32)),
    }
    impl LoadedFrame {
        pub fn texture(&self) -> gdk::Texture {
            match self {
                Self::Native(f) => f.texture(),
                Self::Controlled(second, (width, height)) => gdk::MemoryTexture::new(
                    *width,
                    *height,
                    gdk::MemoryFormat::R8g8b8a8,
                    &glib::Bytes::from_owned(
                        if *second {
                            [180, 60, 30, 255]
                        } else {
                            [40, 100, 180, 255]
                        }
                        .repeat((*width * *height) as usize),
                    ),
                    *width as usize * 4,
                )
                .upcast(),
            }
        }
    }
    use super::secure_image_loader;
    pub async fn load(path: &Path, cancel: &gio::Cancellable) -> Result<LoadedFrame, String> {
        if std::env::args().any(|a| a == "--native") {
            return secure_image_loader::load(path, cancel)
                .await
                .map(LoadedFrame::Native);
        }
        if cancel.is_cancelled() {
            return Err("controlled cancellation".into());
        }
        super::CONTROLLED_LOADS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if !path.is_file() || path.file_name().is_some_and(|s| s == "3.png") {
            return Err("controlled bad/missing photo, not decoder evidence".into());
        }
        Ok(LoadedFrame::Controlled(
            path.file_name().is_some_and(|s| s == "2.png"),
            if path.file_name().is_some_and(|s| s == "portrait.png") {
                (640, 1280)
            } else {
                (64, 32)
            },
        ))
    }
    pub async fn preload(
        path: &Path,
        cancel: &gio::Cancellable,
        _budget: usize,
    ) -> Result<LoadedFrame, String> {
        load(path, cancel).await
    }
}
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/input.rs"]
mod input;
#[path = "../src/window.rs"]
mod window;
use gtk::{gio, glib, prelude::*};
use std::time::{Duration, Instant};
static CONTROLLED_LOADS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[path = "support/development_features.rs"]
mod development_features;
#[path = "support/render_frame.rs"]
mod render_frame;
#[path = "support/toolbar_ux.rs"]
mod toolbar_ux;
fn pump() {
    // Reuse the existing frame barrier; no fixed wait stands in for rendering.
    for _ in 0..100 {
        if !glib::MainContext::default().pending() {
            break;
        }
        glib::MainContext::default().iteration(false);
    }
    std::thread::yield_now();
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
fn wait(label: &str, mut ready: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(15);
    while !ready() {
        assert!(Instant::now() < end, "slideshow deadline: {label}");
        for _ in 0..100 {
            if !glib::MainContext::default().pending() {
                break;
            }
            glib::MainContext::default().iteration(false);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn state(w: &adw::ApplicationWindow, name: &str) -> bool {
    w.lookup_action(name)
        .unwrap()
        .state()
        .unwrap()
        .get()
        .unwrap()
}
fn selected(w: &adw::ApplicationWindow, name: &str) -> bool {
    w.title()
        .is_some_and(|t| t.starts_with(&format!("{name} —")))
}
fn runtime_provenance(w: &adw::ApplicationWindow) {
    eprintln!(
        "SLIDESHOW_RUNTIME gtk={}.{}.{} display={} renderer={:?}",
        gtk::major_version(),
        gtk::minor_version(),
        gtk::micro_version(),
        gtk::prelude::WidgetExt::display(w).type_(),
        w.renderer().map(|r| r.type_())
    );
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        let libraries: std::collections::BTreeSet<_> = maps
            .lines()
            .filter_map(|line| line.split_whitespace().last())
            .filter(|path| {
                [
                    "libgtk-4.so",
                    "libgstgl-1.0.so",
                    "libEGL",
                    "libgallium",
                    "libgstgtk",
                ]
                .iter()
                .any(|name| path.contains(name))
            })
            .collect();
        for path in libraries {
            eprintln!("SLIDESHOW_LIBRARY {path}");
        }
    }
}
fn photo_presented(w: &adw::ApplicationWindow) {
    let picture = widgets(w.upcast_ref())
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Overlay>().ok())
        .find_map(|overlay| overlay.child().and_downcast::<gtk::Picture>())
        .expect("production photo picture");
    wait("EOS photo content (not only selection title)", || {
        picture.paintable().is_some()
            && widgets(w.upcast_ref()).iter().all(|widget| {
                !((widget.widget_name() == "video-content" && widget.is_visible())
                    || (widget.widget_name() == "photo-loading-spinner" && widget.is_visible()))
            })
    });
    let node = render_frame::rendered_node(w, &picture);
    // Exercise real GTK renderer submission, not a fabricated/empty paintable.
    let texture = w
        .renderer()
        .expect("window renderer")
        .render_texture(&node, None);
    assert!(texture.width() > 0 && texture.height() > 0);
}
fn eos_cycles(first: &adw::ApplicationWindow, other: &adw::ApplicationWindow) {
    let repeat = widgets(first.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::ToggleButton>().ok())
        .find(|b| {
            b.tooltip_text()
                .is_some_and(|s| s == "Repeat current video")
        })
        .expect("video repeat preference");
    repeat.set_active(true);
    for cycle in 1..=3 {
        eprintln!("SLIDESHOW_STAGE cycle={cycle} corrupt-select");
        // A fresh session exercises real corrupt-media rejection each cycle;
        // the normal failed-item set otherwise deliberately skips it thereafter.
        if state(first, "slideshow") {
            key(first, gtk::gdk::Key::Escape);
        }
        key(first, gtk::gdk::Key::Right);
        assert!(selected(first, "2.png"));
        key(first, gtk::gdk::Key::Right);
        assert!(selected(first, "3.png"));
        key(first, gtk::gdk::Key::F5);
        wait("corrupt photo skipped to video", || {
            selected(first, "4.mp4")
        });
        eprintln!("SLIDESHOW_STAGE cycle={cycle} video-selected");
        if cycle == 1 {
            runtime_provenance(first);
        }
        wait("video EOS returns to first photo", || {
            selected(first, "1.png")
        });
        // Selection changes synchronously; decoding/GTK presentation does not.
        // Keep the original rapid-navigation test below, but certify this path
        // independently after an actual photo frame has been submitted.
        key(first, gtk::gdk::Key::space);
        photo_presented(first);
        assert!(state(first, "slideshow-paused"));
        assert!(selected(first, "1.png") && selected(other, "1.png"));
        assert!(other.is_visible() && !state(other, "slideshow"));
        let stats = image_view::gpu_video::stats();
        wait("EOS owned GPU counters released", || {
            let s = image_view::gpu_video::stats();
            s.filters == 0
                && s.contexts == 0
                && s.frames == 0
                && s.presentations == 0
                && s.workers == 0
                && s.pending == 0
        });
        eprintln!(
            "SLIDESHOW_STAGE cycle={cycle} photo-presented stock_opened={} gpu_opened={}",
            stats.stock, stats.gpu
        );
    }
    key(first, gtk::gdk::Key::Escape);
    assert!(!state(first, "slideshow"));
    assert!(repeat.is_active() && repeat.is_sensitive());
    first.close();
    assert!(other.is_visible());
    photo_presented(other);
    other.close();
    eprintln!("PASS slideshow EOS regression: 3 corrupt/video/photo cycles and independent close");
}
fn video_interval(
    first: &adw::ApplicationWindow,
    other: &adw::ApplicationWindow,
    dir: &std::path::Path,
) {
    let position = widgets(first.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Button>().ok())
        .find(|b| b.tooltip_text().is_some_and(|s| s == "Forward 5 seconds"))
        .expect("video seek control")
        .parent()
        .unwrap()
        .observe_children()
        .iter::<glib::Object>()
        .filter_map(Result::ok)
        .find_map(|o| o.downcast::<gtk::Scale>().ok())
        .expect("video position scale");
    key(first, gtk::gdk::Key::F5);
    key(first, gtk::gdk::Key::Left); // Wrap photo -> video; photo interval is 3 s.
    assert!(selected(first, "4.mp4"));
    wait("prepared playing video", || {
        assert!(selected(first, "4.mp4"), "video skipped before preparation");
        position.is_sensitive() && position.value() > 0.0
    });
    let gpu = std::env::args().any(|s| s == "--bt709");
    let stats = image_view::gpu_video::stats();
    if gpu {
        assert!(
            stats.gpu > 0 && stats.filters > 0,
            "GPU route required, stock fallback is not a GPU PASS"
        );
    } else {
        assert!(
            stats.stock > 0 && stats.gpu == 0,
            "unknown metadata must use stock"
        );
    }
    let began = Instant::now();
    wait("playing video retained beyond photo interval", || {
        assert!(
            selected(first, "4.mp4") && !state(first, "slideshow-paused"),
            "playing video advanced or paused before EOS"
        );
        began.elapsed() >= Duration::from_millis(3300) && position.value() >= 0.4
    });
    eprintln!(
        "SLIDESHOW_INTERVAL held-playing elapsed_ms={} position={} backend={}",
        began.elapsed().as_millis(),
        position.value(),
        if gpu { "gpu" } else { "stock" }
    );
    key(first, gtk::gdk::Key::space);
    let paused = Instant::now();
    wait("paused video retained beyond photo interval", || {
        assert!(selected(first, "4.mp4") && state(first, "slideshow-paused"));
        paused.elapsed() >= Duration::from_millis(3300)
    });
    eprintln!("SLIDESHOW_INTERVAL held-paused");
    let time = position
        .parent()
        .unwrap()
        .observe_children()
        .iter::<glib::Object>()
        .filter_map(Result::ok)
        .find_map(|o| o.downcast::<gtk::Label>().ok())
        .expect("video timestamp label");
    position.set_value(0.125); // Real seek while paused, then continue to genuine EOS.
    wait("seek applied while paused", || {
        time.text().starts_with("0:01 /") && position.value() < 0.2
    });
    assert!(selected(first, "4.mp4"));
    key(first, gtk::gdk::Key::space);
    wait("successful video EOS advances", || selected(first, "1.png"));
    key(first, gtk::gdk::Key::space);
    photo_presented(first);
    assert!(selected(other, "1.png") && other.is_visible());
    // A decoder error must remain explicit, not masquerade as successful EOS.
    std::fs::write(dir.join("4.mp4"), b"intentionally invalid video").unwrap();
    key(first, gtk::gdk::Key::space);
    key(first, gtk::gdk::Key::Left);
    wait("failed video pauses slideshow", || {
        state(first, "slideshow-paused")
    });
    assert!(
        widgets(first.upcast_ref())
            .iter()
            .filter_map(|w| w.downcast_ref::<gtk::Label>())
            .any(|l| l.is_visible() && l.text().starts_with("Video:"))
    );
    let failed = Instant::now();
    wait("failed video stays selected", || {
        assert!(selected(first, "4.mp4") && state(first, "slideshow-paused"));
        failed.elapsed() >= Duration::from_millis(3300)
    });
    eprintln!("SLIDESHOW_INTERVAL failed-video-held");
    key(first, gtk::gdk::Key::Right);
    assert!(selected(first, "1.png") && state(first, "slideshow-paused"));
    key(first, gtk::gdk::Key::Escape);
    first.close();
    assert!(other.is_visible());
    other.close();
    eprintln!(
        "PASS slideshow video interval: held past photo deadline, pause/seek/EOS/error/manual/close"
    );
}
fn key(w: &adw::ApplicationWindow, key: gtk::gdk::Key) {
    for controller in w
        .observe_controllers()
        .iter::<glib::Object>()
        .filter_map(Result::ok)
        .filter_map(|o| o.downcast::<gtk::EventControllerKey>().ok())
    {
        if controller.emit_by_name::<bool>(
            "key-pressed",
            &[&key, &0u32, &gtk::gdk::ModifierType::empty()],
        ) {
            return;
        }
    }
    panic!("key was not handled: {key:?}");
}
fn main() {
    adw::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.SlideshowProbe")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    if std::env::args().any(|s| s == "--development-open") {
        development_features::open(&app);
        return;
    }
    if std::env::args().any(|s| s == "--development-info") {
        development_features::info(&app);
        return;
    }
    if std::env::args().any(|s| s == "--development-minimap") {
        development_features::minimap(&app);
        return;
    }
    if std::env::args().any(|s| s == "--development-layout") {
        development_features::layout(&app);
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    for (name, source) in [
        ("1.png", "tests/fixtures/images/wide.png"),
        ("2.png", "tests/fixtures/images/large.png"),
    ] {
        std::fs::copy(source, dir.path().join(name)).unwrap();
    }
    std::fs::write(dir.path().join("3.png"), b"invalid photo").unwrap();
    let video = std::env::args()
        .find(|s| s.ends_with(".mp4"))
        .expect("private video fixture path");
    std::fs::copy(video, dir.path().join("4.mp4")).unwrap();
    let open = || {
        let opened =
            input::OpenedFile::from_argument(dir.path().join("1.png").as_os_str()).unwrap();
        assert_eq!(opened.file.path(), Some(dir.path().join("1.png")));
        window::build(&app, &pivra::settings::Settings::default(), Some(opened));
        app.active_window()
            .unwrap()
            .downcast::<adw::ApplicationWindow>()
            .unwrap()
    };
    let first = open();
    let other = open();
    let native_error = || {
        widgets(first.upcast_ref())
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::Label>().ok())
            .any(|l| l.text().contains("Secure image loader failed"))
    };
    wait("catalog/photo ready or explicit secure error", || {
        first.title().is_some_and(|s| s.contains("4 candidates"))
            && (first.lookup_action("image-mode").unwrap().is_enabled() || native_error())
    });
    if std::env::args().any(|a| a == "--native") && native_error() {
        first.close();
        other.close();
        eprintln!("NOT RUN native secure slideshow prerequisite; sandbox unchanged");
        std::process::exit(77);
    }
    if std::env::args().any(|s| s == "--toolbar-ux" || s == "--toolbar-ux-restore") {
        toolbar_ux::run(
            &first,
            &other,
            std::env::args().any(|s| s == "--toolbar-ux-restore"),
        );
        return;
    }
    first
        .lookup_action("slideshow-interval")
        .unwrap()
        .activate(Some(&3u32.to_variant()));
    assert_eq!(pivra::settings::load_slideshow_interval(), 3);
    first.present();
    runtime_provenance(&first);
    if std::env::args().any(|s| s == "--video-interval-regression") {
        video_interval(&first, &other, dir.path());
        return;
    }
    if std::env::args().any(|s| s == "--eos-regression") {
        eos_cycles(&first, &other);
        return;
    }
    let repeat = widgets(first.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::ToggleButton>().ok())
        .find(|b| {
            b.tooltip_text()
                .is_some_and(|s| s == "Repeat current video")
        })
        .expect("existing video repeat control");
    repeat.set_active(true);
    key(&first, gtk::gdk::Key::F5);
    wait("F5 fullscreen from normal window", || first.is_fullscreen());
    assert!(!state(&first, "fullscreen"));
    key(&first, gtk::gdk::Key::Escape);
    wait("slideshow returns to normal window", || {
        !first.is_fullscreen()
    });
    key(&first, gtk::gdk::Key::F11);
    assert!(state(&first, "fullscreen"));
    key(&first, gtk::gdk::Key::F5);
    assert!(state(&first, "slideshow"));
    assert!(state(&first, "slideshow-fullscreen"));
    wait("actual compositor fullscreen", || first.is_fullscreen());
    assert!(!state(&other, "slideshow"));
    assert!(selected(&first, "1.png"));
    key(&first, gtk::gdk::Key::Escape);
    assert!(!state(&first, "slideshow"));
    assert!(state(&first, "fullscreen"));
    key(&first, gtk::gdk::Key::F11);
    assert!(!state(&first, "fullscreen"));
    key(&first, gtk::gdk::Key::F5);
    key(&first, gtk::gdk::Key::F11);
    assert!(state(&first, "fullscreen") && state(&first, "slideshow-fullscreen"));
    key(&first, gtk::gdk::Key::F11);
    assert!(!state(&first, "fullscreen") && state(&first, "slideshow-fullscreen"));
    wait("automatic photo advance", || selected(&first, "2.png"));
    wait("real crossfade overlay", || {
        widgets(first.upcast_ref())
            .iter()
            .any(|w| w.widget_name() == "slideshow-photo-transition" && w.is_visible())
    });
    key(&first, gtk::gdk::Key::space);
    assert!(state(&first, "slideshow-paused"));
    wait("first fade released", || {
        !widgets(first.upcast_ref())
            .iter()
            .any(|w| w.widget_name() == "slideshow-photo-transition" && w.is_visible())
    });
    std::fs::copy("tests/fixtures/images/wide.png", dir.path().join("2.png")).unwrap();
    wait(
        "changed current photo reloaded through normal image pipeline",
        || {
            widgets(first.upcast_ref())
                .iter()
                .any(|w| w.widget_name() == "slideshow-photo-transition" && w.is_visible())
        },
    );
    assert!(selected(&first, "2.png") && state(&first, "slideshow-paused"));
    key(&first, gtk::gdk::Key::Left);
    assert!(selected(&first, "1.png"));
    key(&first, gtk::gdk::Key::Right);
    assert!(selected(&first, "2.png"));
    key(&first, gtk::gdk::Key::Right);
    assert!(selected(&first, "3.png"));
    assert!(state(&first, "slideshow-paused"));
    key(&first, gtk::gdk::Key::space);
    wait("failed photo skipped to video", || {
        selected(&first, "4.mp4")
    });
    wait("video EOS loops to first photo", || {
        selected(&first, "1.png")
    });
    eprintln!("SLIDESHOW_STAGE ordinary-probe EOS photo selected");
    photo_presented(&first);
    eprintln!("SLIDESHOW_STAGE ordinary-probe EOS photo presented; rapid navigation next");
    assert!(selected(&other, "1.png"));
    key(&first, gtk::gdk::Key::space);
    key(&first, gtk::gdk::Key::Left);
    assert!(selected(&first, "4.mp4"));
    assert!(state(&first, "slideshow-paused"));
    key(&first, gtk::gdk::Key::Right);
    assert!(selected(&first, "1.png"));
    key(&first, gtk::gdk::Key::Left);
    assert!(selected(&first, "4.mp4"));
    key(&first, gtk::gdk::Key::Escape);
    assert!(!state(&first, "slideshow"));
    assert!(!state(&first, "fullscreen"));
    wait("actual compositor fullscreen restored", || {
        !first.is_fullscreen()
    });
    assert!(
        repeat.is_active() && repeat.is_sensitive(),
        "ordinary repeat preference restored"
    );
    assert!(
        !widgets(first.upcast_ref())
            .iter()
            .any(|w| w.widget_name() == "slideshow-photo-transition" && w.is_visible())
    );
    // Stop must remove the real GTK timer: retain the window past its interval.
    let deadline = Instant::now() + Duration::from_millis(3300);
    wait("stopped timer observation", || Instant::now() >= deadline);
    assert!(selected(&first, "4.mp4"));
    key(&other, gtk::gdk::Key::F5);
    key(&other, gtk::gdk::Key::space);
    let late_start = first.lookup_action("slideshow").unwrap();
    first.close();
    late_start.activate(None);
    assert!(
        !state(&first, "slideshow"),
        "late action restarted closed window"
    );
    assert!(other.is_visible() && state(&other, "slideshow"));
    std::fs::remove_file(dir.path().join("1.png")).unwrap();
    wait("live catalog deletion", || !selected(&other, "1.png"));
    assert!(state(&other, "slideshow-paused"));
    other.close();
    wait("window close", || !other.is_visible());
    let bad = tempfile::tempdir().unwrap();
    std::fs::write(bad.path().join("3.png"), b"invalid image").unwrap();
    window::build(
        &app,
        &pivra::settings::Settings::default(),
        Some(input::OpenedFile::from_argument(bad.path().join("3.png").as_os_str()).unwrap()),
    );
    let invalid = app
        .active_window()
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    wait("bad-only catalog", || {
        invalid.title().is_some_and(|t| t.contains("1 candidate"))
    });
    key(&invalid, gtk::gdk::Key::F5);
    wait("all failed media stops without retries", || {
        !state(&invalid, "slideshow")
    });
    assert!(invalid.is_visible());
    invalid.close();
    println!(
        "PASS slideshow production actions/GTK: F5, independent fullscreen, timer, pause/manual, real crossfade, error skip, stock video route, loop, stop/close, two windows, live deletion"
    );
    println!(
        "Photos are {}. Physical GNOME/AMD and display-sleep inhibition require native acceptance.",
        if std::env::args().any(|a| a == "--native") {
            "secure Glycin"
        } else {
            "explicit controlled pixels; NOT decoder evidence"
        }
    );
}
