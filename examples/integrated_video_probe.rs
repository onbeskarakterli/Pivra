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
    eprintln!("LIFECYCLE_STAGE {label} begin");
    image_view::gpu_video::trace_binding(c"wait-begin");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        for _ in 0..100 {
            if !glib::MainContext::default().pending() {
                break;
            }
            glib::MainContext::default().iteration(false);
        }
        if ready() {
            let s = stats();
            eprintln!(
                "LIFECYCLE_STAGE {label} ready contexts={} frames={} presentations={}",
                s.contexts, s.frames, s.presentations
            );
            image_view::gpu_video::trace_binding(c"wait-ready");
            return;
        }
        assert!(
            Instant::now() < deadline,
            "FAIL {label}: filters={} workers={} pending={} contexts={} frames={} presentations={}",
            stats().filters,
            stats().workers,
            stats().pending,
            stats().contexts,
            stats().frames,
            stats().presentations
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn frame(w: &gtk::Widget) -> bool {
    if let Some(p) = w.downcast_ref::<gtk::Picture>() {
        if p.widget_name() == "video-content"
            && p.is_visible()
            && p.paintable().is_some_and(|p| p.intrinsic_width() > 0)
        {
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
    eprintln!("LIFECYCLE_ACTION {name}");
    image_view::gpu_video::trace_binding(c"action-before");
    gio::prelude::ActionGroupExt::activate_action(w, name, None);
    image_view::gpu_video::trace_binding(c"action-after");
}
fn runtime_provenance() {
    eprintln!(
        "LIFECYCLE_RUNTIME gtk={}.{}.{}",
        gtk::major_version(),
        gtk::minor_version(),
        gtk::micro_version()
    );
    let maps = std::fs::read_to_string("/proc/self/maps").unwrap_or_default();
    let libraries: std::collections::BTreeSet<_> = maps
        .lines()
        .filter_map(|line| {
            let path = line.get(line.find('/')?..)?;
            [
                "libgtk-4.so",
                "libgstreamer-1.0.so",
                "libgstgl-1.0.so",
                "libEGL",
                "libGLX",
                "libepoxy",
                "libgallium",
            ]
            .iter()
            .any(|name| path.contains(name))
            .then_some(path)
        })
        .collect();
    for path in libraries {
        eprintln!("LIFECYCLE_LIBRARY {path}");
    }
}
fn main() {
    adw::init().unwrap();
    runtime_provenance();
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.GpuTest")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "--error-path") {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("invalid.mp4");
        std::fs::write(&path, b"controlled malformed input, not a valid video").unwrap();
        // Deliberate decoder-error injection in this probe only. Production
        // metadata admission remains unchanged.
        match image_view::gpu_video::open_selected(&path, image_view::gpu_video::Backend::Gpu) {
            Ok(media) => {
                wait("controlled pipeline error", || media.error().is_some());
                media.clear();
            }
            Err(error) => eprintln!("EXPECTED controlled pipeline error: {error}"),
        }
        wait("error resources retired", drained);
        println!(
            "PASS malformed GPU input error, no retained contexts/frames/presentations/workers"
        );
        return;
    }
    if args.first().is_some_and(|arg| arg == "--routing-only") {
        for argument in &args[1..] {
            let argument = argument.to_str().expect("routing fixture UTF-8 path");
            let (expected, path) = argument.split_once('=').expect("gpu=path or stock=path");
            let expected = match expected {
                "gpu" => image_view::gpu_video::Backend::Gpu,
                "stock" => image_view::gpu_video::Backend::Stock,
                _ => panic!("unknown expected route"),
            };
            let actual = glib::MainContext::default()
                .block_on(image_view::gpu_video::choose(path.into(), || true));
            assert_eq!(actual, Some(expected), "metadata route: {path}");
            println!("PASS production metadata route {expected:?}: {path}");
        }
        assert_eq!(stats().filters, 0, "routing does not start a GPU player");
        return;
    }
    if gtk::MediaFile::new().type_().name() == "GtkNoMediaFile" {
        eprintln!(
            "NOT RUN: real stock GTK media backend is missing; mixed lifecycle cannot be certified"
        );
        std::process::exit(77);
    }
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
    runtime_provenance();
    assert_eq!(stats().probes, 1, "same-source discovery deduplicated");
    let stock = stats().stock;
    action(&first, "next-media");
    wait("sRGB stock route", || {
        first.title().is_some_and(|t| t.contains("2.mp4"))
            && stats().stock == stock + 1
            && stats().filters == 1
            && stats().workers == 0
            && frame(first.upcast_ref())
    });
    action(&first, "next-media");
    wait("unknown stock route", || {
        first.title().is_some_and(|t| t.contains("3.mp4"))
            && stats().stock == stock + 2
            && stats().filters == 1
            && stats().workers == 0
            && frame(first.upcast_ref())
    });
    action(&first, "next-media");
    wait("full-range GPU", || {
        first.title().is_some_and(|t| t.contains("4.mp4"))
            && stats().filters == 2
            && frame(first.upcast_ref())
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
    wait("all filters retired", drained);
    for _ in 0..3 {
        let w = open(&app, &dir.path().join("1.mp4"));
        wait("reopen frame", || {
            stats().filters == 1 && frame(w.upcast_ref())
        });
        w.close();
        wait("reopen cleanup", drained);
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
    wait("delete and close cleanup", drained);
    println!(
        "PASS automatic routing, discovery dedup/cache, rapid navigation, two-window isolation, repeated close, delete-during-selection; stock pixels/native hardware NOT certified"
    );
}

fn drained() -> bool {
    let s = stats();
    s.filters == 0
        && s.workers == 0
        && s.pending == 0
        && s.contexts == 0
        && s.frames == 0
        && s.presentations == 0
}
