//! Real secure ImageView delivery timings, not snapshot/first-visible-frame timing.
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
use std::{
    cell::Cell,
    path::Path,
    rc::Rc,
    time::{Duration, Instant},
};
fn pump(ready: impl Fn() -> bool) {
    let start = Instant::now();
    while !ready() {
        for _ in 0..30 {
            if !glib::MainContext::default().pending() {
                break;
            }
            glib::MainContext::default().iteration(false);
        }
        assert!(
            start.elapsed() < Duration::from_secs(45),
            "image delivery deadline"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn open(app: &adw::Application) -> (adw::ApplicationWindow, image_view::ImageView) {
    let w = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(800)
        .default_height(600)
        .build();
    let v = image_view::ImageView::new(&w);
    w.set_content(Some(v.widget()));
    w.present();
    (w, v)
}
fn measured(v: &image_view::ImageView, path: &Path, label: &str) -> bool {
    let done = Rc::new(Cell::new(None));
    let observed = done.clone();
    let start = Instant::now();
    v.load_observed(path.to_owned(), || {}, move |ok| observed.set(Some(ok)));
    pump(|| done.get().is_some());
    let ok = done.get().unwrap();
    println!(
        "{{\"scenario\":\"{label}\",\"success\":{ok},\"elapsed_us\":{}}}",
        start.elapsed().as_micros()
    );
    let _ = v.preload_stats();
    ok
}
fn main() {
    if adw::init().is_err() {
        eprintln!("NOT RUN graphical initialization");
        std::process::exit(77);
    }
    let root = std::path::PathBuf::from(
        std::env::args_os()
            .nth(1)
            .expect("private fixture directory"),
    );
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.Performance")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    let (w, v) = open(&app);
    if !measured(&v, &root.join("1.png"), "first_secure_open") {
        w.close();
        eprintln!(
            "NOT RUN secure image benchmark: decoder unavailable; no further decode attempts"
        );
        std::process::exit(1);
    }
    for _ in 0..9 {
        assert!(measured(
            &v,
            &root.join("1.png"),
            "warm_os_cache_forced_decode"
        ));
    }
    for _ in 0..10 {
        let before = v.preload_stats().cache.hits;
        let started = Instant::now();
        v.load(root.join("1.png"));
        pump(|| {
            let s = v.load_stats();
            s.running == 0 && s.pending == 0
        });
        assert_eq!(
            v.preload_stats().cache.hits,
            before + 1,
            "RAM-hit path really executed"
        );
        println!(
            "{{\"scenario\":\"warm_ram_cache_delivery\",\"success\":true,\"elapsed_us\":{}}}",
            started.elapsed().as_micros()
        );
    }
    // Measure real catalog/widget construction and an idle-turn barrier. Empty
    // MP4 scan fixtures are excluded; only genuine PNGs enter the photo strip.
    for count in [100, 1000, 5000] {
        let catalog = pivra::discovery::scan_directory(
            &root.join(count.to_string()),
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap();
        let photos: Vec<_> = catalog
            .collection
            .items()
            .iter()
            .enumerate()
            .filter(|(_, item)| item.path().extension().is_some_and(|e| e == "png"))
            .map(|(i, item)| (i, item.path().to_owned()))
            .collect();
        let started = Instant::now();
        v.set_thumbnail_catalog(&photos, photos.first().map(|p| p.0));
        println!(
            "{{\"scenario\":\"catalog_widgets_{count}\",\"success\":true,\"elapsed_us\":{}}}",
            started.elapsed().as_micros()
        );
        let idle = Rc::new(Cell::new(false));
        let flag = idle.clone();
        glib::idle_add_local_once(move || flag.set(true));
        pump(|| idle.get());
        println!(
            "{{\"scenario\":\"catalog_idle_turn_{count}\",\"success\":true,\"elapsed_us\":{}}}",
            started.elapsed().as_micros()
        );
    }
    v.set_thumbnail_catalog(&[(0, root.join("1.png")), (1, root.join("2.png"))], Some(0));
    fn show(w: &gtk::Widget) {
        if w.widget_name() == "thumbnail-toggle" {
            if let Some(t) = w.downcast_ref::<gtk::ToggleButton>() {
                t.set_active(true);
            }
        }
        let mut child = w.first_child();
        while let Some(c) = child {
            show(&c);
            child = c.next_sibling();
        }
    }
    show(w.upcast_ref());
    pump(|| {
        let s = v.thumbnail_stats();
        s.completed == s.total && !s.running
    });
    let thumbs = v.thumbnail_stats();
    println!(
        "{{\"scenario\":\"real_thumbnail_pipeline\",\"completed\":{},\"failed\":{},\"estimated_texture_bytes\":{}}}",
        thumbs.completed, thumbs.failed, thumbs.bytes
    );
    let thumbnail_failed = thumbs.total != 2 || thumbs.failed != 0 || thumbs.completed != 2;
    v.set_neighbors(vec![root.join("2.png")]);
    for _ in 0..10 {
        v.load(root.join("2.png"));
        v.load(root.join("1.png"));
    }
    pump(|| {
        let s = v.load_stats();
        s.running == 0 && s.pending == 0
    });
    let _ = v.preload_stats();
    let (other, second) = open(&app);
    assert!(measured(&second, &root.join("2.png"), "second_window"));
    w.close();
    assert!(measured(
        &second,
        &root.join("1.png"),
        "survivor_after_close"
    ));
    other.close();
    for _ in 0..20 {
        let (w, v) = open(&app);
        assert!(measured(&v, &root.join("1.png"), "reopen_cycle"));
        w.close();
    }
    let (w, v) = open(&app);
    assert!(measured(&v, &root.join("64mp.png"), "64mp_secure_open"));
    w.close();
    pump(|| !glib::MainContext::default().pending());
    if thumbnail_failed {
        eprintln!("FAIL: real thumbnail pipeline did not complete both fixtures");
        std::process::exit(1);
    }
}
