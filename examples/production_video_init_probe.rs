//! Production ImageView initialization/recovery. Error injection is probe-only.
#[path = "../src/image_loader.rs"]
mod image_loader;
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/raw_loader.rs"]
mod raw_loader;
#[path = "../src/raw_ppm.rs"]
mod raw_ppm;
use adw::prelude::*;
use gtk::{gio, glib};
use image_view::{ImageView, gpu_video};
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn wait(label: &str, ready: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        for _ in 0..100 {
            if !glib::MainContext::default().pending() {
                break;
            }
            glib::MainContext::default().iteration(false);
        }
        if ready() {
            return;
        }
        assert!(Instant::now() < deadline, "FAIL {label}");
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn drained() -> bool {
    let s = gpu_video::stats();
    s.filters + s.contexts + s.frames + s.presentations + s.workers + s.pending == 0
}
fn ready(view: &ImageView) -> bool {
    view.current_video()
        .is_some_and(|s| s.error().is_none() && s.is_prepared() && s.intrinsic_width() > 0)
}
fn visible_error(widget: &gtk::Widget) -> bool {
    if let Some(label) = widget.downcast_ref::<gtk::Label>() {
        if label.is_visible() && label.text().starts_with("Video:") {
            return true;
        }
    }
    let mut child = widget.first_child();
    while let Some(c) = child {
        if visible_error(&c) {
            return true;
        }
        child = c.next_sibling();
    }
    false
}
fn open(app: &adw::Application, path: &Path) -> (adw::ApplicationWindow, ImageView) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(640)
        .default_height(400)
        .build();
    let view = ImageView::new(&window);
    window.set_content(Some(view.widget()));
    view.load_video(path);
    // Intentionally dispatch discovery before presentation. No GPU context may
    // be created before the native window exists, even on a discovery cache hit.
    wait("metadata before window realization", || {
        gpu_video::stats().probes > 0 && gpu_video::stats().workers == 0
    });
    assert!(!window.is_realized());
    assert!(view.current_video().is_none());
    window.present();
    wait("production first frame", || ready(&view));
    let frame = view
        .capture_frame(path)
        .expect("production rendered video snapshot");
    assert!(frame.width() > 0 && frame.height() > 0);
    (window, view)
}
fn inject_error(media: &gtk::MediaFile) {
    use glib::translate::{IntoGlibPtr, ToGlibPtr};
    let error = glib::Error::new(gio::IOErrorEnum::Failed, "controlled GPU pipeline error");
    unsafe {
        gtk::ffi::gtk_media_stream_gerror(
            media.upcast_ref::<gtk::MediaStream>().to_glib_none().0,
            error.into_glib_ptr(),
        );
    }
    // Repeated notifications must not enqueue multiple replacements.
    media.notify("error");
}
fn controls(view: &ImageView) {
    let media = view.current_video().unwrap();
    assert!(media.is_playing());
    assert!(view.video_key(gtk::gdk::Key::space, false));
    assert!(!media.is_playing());
    assert!(view.video_key(gtk::gdk::Key::period, false));
    wait("frame step", || !media.is_seeking());
    assert!(view.video_key(gtk::gdk::Key::Right, true));
    wait("seek control", || !media.is_seeking());
    let muted = media.is_muted();
    assert!(view.video_key(gtk::gdk::Key::m, false));
    assert_eq!(media.is_muted(), !muted);
    media.set_volume(0.25);
    assert_eq!(media.volume(), 0.25);
    media.seek(0);
    wait("seek beginning", || !media.is_seeking());
    assert!(view.video_key(gtk::gdk::Key::space, false));
    assert!(media.is_playing());
}
fn main() {
    adw::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.InitProbe")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mode = args[0].to_str().unwrap();
    // All destructive scenarios operate on our copies, never caller media.
    let files = tempfile::tempdir().unwrap();
    let first_path = files.path().join("first.mp4");
    let other_path = files.path().join("second.mp4");
    std::fs::copy(&args[1], &first_path).unwrap();
    std::fs::copy(&args[2], &other_path).unwrap();
    let path = first_path.as_path();
    let other = other_path.as_path();
    let (window, view) = open(&app, path);
    let first = view.current_video().unwrap();
    let weak = first.downgrade();
    if mode == "init-failure" {
        assert!(
            !gpu_video::is_owned(&first),
            "must recover to real stock GTK"
        );
        assert_eq!(gpu_video::stats().gpu, 1);
        assert_eq!(gpu_video::stats().stock, 1);
        assert!(
            drained(),
            "failed GPU resources must be retired before stock playback"
        );
        controls(&view);
        view.load_video(other);
        wait("navigate after initialization failure", || {
            ready(&view) && view.current_video().as_ref() != Some(&first)
        });
        assert!(!gpu_video::is_owned(&view.current_video().unwrap()));
        assert_eq!(gpu_video::stats().stock, 2);
    } else {
        assert!(
            gpu_video::is_owned(&first),
            "GPU success must not be masked by stock recovery"
        );
        assert_eq!(gpu_video::stats().stock, 0);
        if mode == "deleted-source" {
            std::fs::remove_file(path).unwrap();
            view.load_video(path);
            wait("deleted source has visible controlled error", || {
                visible_error(view.widget().upcast_ref())
            });
            assert_eq!(gpu_video::stats().stock, 1);
            view.load_video(other);
            wait("navigate away from deleted source", || ready(&view));
            assert!(gpu_video::is_owned(&view.current_video().unwrap()));
            assert!(view.capture_frame(other).is_ok());
        } else if mode == "normal" {
            controls(&view);
            view.load_video(other);
            wait("GPU navigation", || {
                ready(&view) && view.current_video().as_ref() != Some(&first)
            });
            assert!(gpu_video::is_owned(&view.current_video().unwrap()));
            assert_eq!(gpu_video::stats().stock, 0);
        } else if mode == "close-pending" {
            inject_error(&first);
            window.close();
            wait("closed recovery cannot reopen stock", drained);
            assert_eq!(gpu_video::stats().stock, 0);
        } else if mode == "stale-recovery" {
            inject_error(&first);
            view.load_video(other);
            wait("stale recovery cannot replace new media", || ready(&view));
            assert!(gpu_video::is_owned(&view.current_video().unwrap()));
            assert_eq!(gpu_video::stats().stock, 0);
        } else {
            assert!(matches!(mode, "async-recovery" | "async-playing"));
            let (second_window, second_view) = open(&app, other);
            let second = second_view.current_video().unwrap();
            if mode == "async-recovery" {
                first.pause();
            }
            first.set_muted(true);
            first.set_volume(0.375);
            first.seek(500_000);
            wait("pre-error seek", || !first.is_seeking());
            inject_error(&first);
            wait("real stock recovery frame", || {
                ready(&view)
                    && view
                        .current_video()
                        .is_some_and(|s| !gpu_video::is_owned(&s))
            });
            let stock = view.current_video().unwrap();
            assert_eq!(
                stock.is_playing(),
                mode == "async-playing",
                "play/pause intent must survive error-induced pause"
            );
            assert!(stock.is_muted());
            assert_eq!(stock.volume(), 0.375);
            wait("restored timestamp", || {
                !stock.is_seeking() && stock.timestamp() > 0
            });
            assert_eq!(gpu_video::stats().stock, 1, "exactly one replacement");
            assert_eq!(
                gpu_video::stats().contexts,
                1,
                "only independent GPU window remains"
            );
            assert_eq!(second_view.current_video(), Some(second.clone()));
            assert!(second.error().is_none() && second.is_playing());
            stock.play();
            assert!(stock.is_playing());
            second_window.close();
            drop(second);
            drop(second_view);
        }
    }
    window.close();
    drop(first);
    drop(view);
    wait("all owned GPU resources drained", drained);
    assert!(weak.upgrade().is_none(), "retired stream/callback cycle");
    println!("PASS production-init {mode}: actual frames, bounded recovery, controls and cleanup");
}
