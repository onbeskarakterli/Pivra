//! Real production windows and GPU controller; software GL only when explicitly allowed.
#[path = "../src/image_loader.rs"]
mod image_loader;
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/input.rs"]
mod input;
#[path = "../src/raw_loader.rs"]
mod raw_loader;
#[path = "../src/raw_ppm.rs"]
mod raw_ppm;
#[path = "../src/window.rs"]
mod window;
use adw::prelude::*;
use gtk::{gio, glib};
use image_view::gpu_video::stats;
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
        assert!(
            Instant::now() < deadline,
            "{label}: filters={} workers={} pending={}",
            stats().filters,
            stats().workers,
            stats().pending
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn frame(w: &gtk::Widget) -> bool {
    if let Some(p) = w.downcast_ref::<gtk::Picture>() {
        if p.is_visible() && p.paintable().is_some_and(|p| p.intrinsic_width() > 0) {
            return true;
        }
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        if frame(&c) {
            return true;
        }
        child = c.next_sibling();
    }
    false
}
fn open(app: &adw::Application, path: &Path) -> adw::ApplicationWindow {
    let file = input::OpenedFile::from_argument(path.as_os_str()).unwrap();
    assert_eq!(file.file.path().as_deref(), Some(path));
    window::build(app, &pivra::settings::Settings::default(), Some(file));
    app.active_window().unwrap().downcast().unwrap()
}
fn action(w: &adw::ApplicationWindow, name: &str) {
    gio::prelude::ActionGroupExt::activate_action(w, name, None);
}
fn main() {
    adw::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.GpuTest")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    assert_eq!(
        args.len(),
        4,
        "BT709, sRGB, unknown, full-range fixtures required"
    );
    let dir = tempfile::tempdir().unwrap();
    for (i, src) in args.iter().enumerate() {
        std::fs::copy(src, dir.path().join(format!("{}.mp4", i + 1))).unwrap();
    }
    let first = open(&app, &dir.path().join("1.mp4"));
    let second = open(&app, &dir.path().join("1.mp4"));
    wait("two independent GPU windows", || {
        stats().filters == 2 && frame(first.upcast_ref()) && frame(second.upcast_ref())
    });
    assert_eq!(stats().probes, 1, "same-source discovery deduplicated");
    let stock = stats().stock;
    action(&first, "next-media");
    wait("sRGB stock route", || {
        first.title().is_some_and(|t| t.contains("2.mp4"))
            && stats().stock == stock + 1
            && stats().filters == 1
            && stats().workers == 0
    });
    action(&first, "next-media");
    wait("unknown stock route", || {
        first.title().is_some_and(|t| t.contains("3.mp4"))
            && stats().stock == stock + 2
            && stats().filters == 1
            && stats().workers == 0
    });
    action(&first, "next-media");
    wait("full-range GPU", || {
        stats().filters == 2 && frame(first.upcast_ref())
    });
    for _ in 0..4 {
        for name in [
            "previous-media",
            "previous-media",
            "next-media",
            "next-media",
        ] {
            action(&first, name);
        }
    }
    wait("rapid navigation final frame", || {
        stats().filters == 2 && stats().workers == 0 && frame(first.upcast_ref())
    });
    assert!(stats().probes <= 4);
    assert!(stats().cached <= 64);
    first.close();
    wait("first closed other alive", || {
        stats().filters == 1 && frame(second.upcast_ref())
    });
    second.close();
    wait("all filters retired", || {
        stats().filters == 0 && stats().workers == 0 && stats().pending == 0
    });
    for _ in 0..3 {
        let w = open(&app, &dir.path().join("1.mp4"));
        wait("reopen frame", || {
            stats().filters == 1 && frame(w.upcast_ref())
        });
        w.close();
        wait("reopen cleanup", || stats().filters == 0);
    }
    let fresh = dir.path().join("5.mp4");
    std::fs::copy(&args[0], &fresh).unwrap();
    let w = open(&app, &fresh);
    let deadline = Instant::now() + Duration::from_secs(10);
    while stats().workers == 0 && stats().filters == 0 {
        glib::MainContext::default().iteration(false);
        assert!(Instant::now() < deadline, "selection did not start");
    }
    println!(
        "CLOSE_STAGE workers={} filters={}",
        stats().workers,
        stats().filters
    );
    std::fs::remove_file(fresh).unwrap();
    w.close();
    wait("delete and close cleanup", || {
        stats().filters == 0 && stats().workers == 0 && stats().pending == 0
    });
    println!(
        "PASS automatic routing, discovery dedup/cache, rapid navigation, two-window isolation, repeated close, delete-during-selection; stock pixels/native hardware NOT certified"
    );
}
