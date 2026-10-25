//! Controlled mode is synthetic. Default requires the real secure loader.
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/image_loader.rs"]
mod secure_loader;
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::{Duration, Instant},
};
static CONTROLLED: AtomicBool = AtomicBool::new(false);
static HOLD: AtomicBool = AtomicBool::new(false);
static RELEASE: AtomicBool = AtomicBool::new(false);
static HELD: AtomicBool = AtomicBool::new(false);
static CALLS: AtomicUsize = AtomicUsize::new(0);
mod image_loader {
    use super::*;
    pub enum LoadedFrame {
        Secure(glycin::Frame),
        Synthetic(u8),
    }
    impl LoadedFrame {
        pub fn texture(&self) -> gdk::Texture {
            match self {
                Self::Secure(f) => f.texture(),
                Self::Synthetic(byte) => {
                    let rgb = match byte {
                        b'r' => [255, 0, 0],
                        b'g' => [0, 255, 0],
                        _ => [0, 0, 255],
                    };
                    gdk::MemoryTexture::new(
                        16,
                        16,
                        gdk::MemoryFormat::R8g8b8a8,
                        &glib::Bytes::from_owned([rgb[0], rgb[1], rgb[2], 255].repeat(16 * 16)),
                        64,
                    )
                    .upcast()
                }
            }
        }
    }
    pub async fn load(path: &Path, token: &gio::Cancellable) -> Result<LoadedFrame, String> {
        CALLS.fetch_add(1, Ordering::SeqCst);
        if !CONTROLLED.load(Ordering::SeqCst) {
            return secure_loader::load(path, token)
                .await
                .map(LoadedFrame::Secure);
        }
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        if bytes == b"bad" {
            return Err("controlled malformed neighbor".into());
        }
        Ok(LoadedFrame::Synthetic(bytes[0]))
    }
    pub async fn preload(
        path: &Path,
        token: &gio::Cancellable,
        budget: usize,
    ) -> Result<LoadedFrame, String> {
        if HOLD.load(Ordering::SeqCst) && path.file_name().unwrap() == "3.png" {
            HELD.store(true, Ordering::SeqCst);
            let end = Instant::now() + Duration::from_secs(20);
            while !RELEASE.load(Ordering::SeqCst) {
                assert!(Instant::now() < end, "preload barrier deadline");
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        if !CONTROLLED.load(Ordering::SeqCst) {
            CALLS.fetch_add(1, Ordering::SeqCst);
            return secure_loader::preload(path, token, budget)
                .await
                .map(LoadedFrame::Secure);
        }
        if token.is_cancelled() {
            return Err("controlled preload cancelled".into());
        }
        load(path, token).await
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
    let end = Instant::now() + Duration::from_secs(20);
    while !ready() {
        assert!(Instant::now() < end, "preload probe deadline");
        pump();
    }
}
fn open(app: &adw::Application) -> (adw::ApplicationWindow, image_view::ImageView) {
    let w = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(480)
        .default_height(320)
        .build();
    let view = image_view::ImageView::new(&w);
    w.set_content(Some(view.widget()));
    w.present();
    wait(|| w.is_mapped());
    (w, view)
}
fn picture(v: &image_view::ImageView) -> gtk::Picture {
    v.widget().child().unwrap().downcast().unwrap()
}
fn status(v: &image_view::ImageView) -> gtk::Label {
    picture(v).next_sibling().unwrap().downcast().unwrap()
}
fn loaded(v: &image_view::ImageView) {
    wait(|| picture(v).paintable().is_some() || status(v).text().starts_with("Unable"));
    assert!(
        picture(v).paintable().is_some(),
        "valid load failed: {}",
        status(v).text()
    );
    assert_eq!(v.phase(), pivra::image_pipeline::Phase::FullReady);
}
fn idle(v: &image_view::ImageView) {
    wait(|| {
        v.load_stats().running == 0
            && v.load_stats().pending == 0
            && !v.preload_stats().preload_active
    });
}
fn choose(w: &adw::ApplicationWindow, action: &str, id: &str) {
    gio::prelude::ActionGroupExt::activate_action(w, action, Some(&id.to_variant()));
}
fn check_bound(v: &image_view::ImageView) {
    let s = v.preload_stats();
    assert!(s.cache.entries <= 3 && s.cache.bytes <= s.cache.budget);
    assert!(v.load_stats().max_running <= 3);
}
fn setup(controlled: bool) -> (tempfile::TempDir, Vec<PathBuf>) {
    let dir = tempfile::tempdir().unwrap();
    let mut paths = vec![];
    for (i, source) in ["tall.png", "small.png", "wide.png", "no-exif.jpg"]
        .into_iter()
        .enumerate()
    {
        let p = dir.path().join(format!("{}.png", i + 1));
        if controlled {
            std::fs::write(&p, [b'r' + i as u8]).unwrap();
        } else {
            std::fs::copy(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/images")
                    .join(source),
                &p,
            )
            .unwrap();
        }
        paths.push(p);
    }
    (dir, paths)
}
fn run(app: &adw::Application, controlled: bool) {
    let (_dir, paths) = setup(controlled);
    let originals: Vec<_> = paths.iter().map(|p| std::fs::read(p).unwrap()).collect();
    let (w, v) = open(app);
    v.configure_cache(256 * 1024, 96 * 1024);
    v.set_neighbors(vec![paths[2].clone(), paths[0].clone()]);
    v.load(paths[1].clone());
    assert_eq!(v.phase(), pivra::image_pipeline::Phase::FullLoading);
    wait(|| picture(&v).paintable().is_some() || status(&v).text().starts_with("Unable"));
    if !controlled && picture(&v).paintable().is_none() {
        eprintln!("NOT RUN secure preload baseline: {}", status(&v).text());
        std::process::exit(77);
    }
    loaded(&v);
    idle(&v);
    let stats = v.preload_stats();
    assert_eq!(stats.started, 2);
    assert_eq!(stats.completed, 2);
    assert_eq!(stats.cache.entries, 3);
    check_bound(&v);
    let next = v.cached_texture(&paths[2]).unwrap();
    let decodes = stats.foreground_decoded;
    choose(&w, "image-mode", "actual");
    choose(&w, "image-resampling", "nearest");
    v.load(paths[2].clone());
    loaded(&v);
    idle(&v);
    assert_eq!(v.preload_stats().foreground_decoded, decodes);
    assert!(v.preload_stats().cache.hits >= 1);
    assert_eq!(
        w.lookup_action("image-mode")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("actual") // Current accepted behavior preserves the selected preset across navigation.
    );
    assert_eq!(
        w.lookup_action("image-resampling")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("nearest")
    );
    choose(&w, "image-resampling", "smooth");
    // Actual is a presentation wrapper; Fit exposes its same backing texture.
    // Check object identity in that representation without expecting a cache hit
    // to reset the user's preset implicitly.
    choose(&w, "image-mode", "fit");
    assert_eq!(picture(&v).paintable(), Some(next.upcast()));
    assert_eq!(v.preload_stats().foreground_decoded, decodes);
    choose(&w, "image-mode", "actual");
    v.set_neighbors(vec![]);
    v.load(paths[1].clone());
    loaded(&v);
    idle(&v);
    v.load(paths[3].clone());
    loaded(&v);
    idle(&v);
    assert!(v.preload_stats().cache.evictions >= 1);
    let before = v.preload_stats().foreground_decoded;
    v.load(paths[0].clone());
    loaded(&v);
    idle(&v);
    assert_eq!(v.preload_stats().foreground_decoded, before + 1);
    check_bound(&v);
    // Same path, different inode: metadata identity alone must reject old contents.
    let temp = paths[1].with_extension("save");
    std::fs::write(&temp, &originals[0]).unwrap();
    std::fs::rename(temp, &paths[1]).unwrap();
    let before = v.preload_stats().foreground_decoded;
    v.load(paths[1].clone());
    loaded(&v);
    idle(&v);
    assert_eq!(v.preload_stats().foreground_decoded, before + 1);
    assert_eq!(
        picture(&v).paintable().unwrap().intrinsic_height(),
        if controlled { 16 } else { 64 }
    );
    std::fs::remove_file(&paths[3]).unwrap();
    v.load(paths[3].clone());
    wait(|| status(&v).text().starts_with("Unable"));
    assert!(picture(&v).paintable().is_none());
    assert!(v.cached_texture(&paths[3]).is_none());
    v.load(paths[0].clone());
    loaded(&v);
    idle(&v);
    v.memory_pressure();
    assert!(!v.preload_stats().enabled);
    assert!(v.preload_stats().cache.entries <= 1);
    check_bound(&v);
    let (off, disabled) = open(app);
    disabled.disable_preload();
    disabled.set_neighbors(vec![paths[2].clone(), paths[1].clone()]);
    disabled.load(paths[0].clone());
    loaded(&disabled);
    idle(&disabled);
    disabled.load(paths[2].clone());
    loaded(&disabled);
    idle(&disabled);
    assert_eq!(disabled.preload_stats().started, 0);
    assert_eq!(disabled.preload_stats().foreground_decoded, 2);
    for (i, p) in paths.iter().enumerate() {
        if i != 3 {
            assert_eq!(
                std::fs::read(p).unwrap(),
                if i == 1 {
                    originals[0].clone()
                } else {
                    originals[i].clone()
                }
            );
        }
    }
    println!(
        "PASS {} cache hits reuse texture/selected-preset/filter, next-before-previous preload, LRU re-decode, atomic replace/delete identity, fixed budget, injected pressure, off comparison, source integrity; stats {:?}",
        if controlled {
            "controlled synthetic"
        } else {
            "secure"
        },
        v.preload_stats()
    );
    println!(
        "MEASURED enabled initial foreground decodes=1, background=2; disabled two-image foreground decodes=2. No RSS/VRAM or speedup conclusion."
    );
    w.close();
    off.close();
    idle(&v);
    idle(&disabled);
}
fn controlled_priority(app: &adw::Application) {
    let (_dir, paths) = setup(true);
    let (w, v) = open(app);
    v.load(paths[1].clone());
    loaded(&v);
    idle(&v);
    HOLD.store(true, Ordering::SeqCst);
    RELEASE.store(false, Ordering::SeqCst);
    HELD.store(false, Ordering::SeqCst);
    v.set_neighbors(vec![paths[2].clone(), paths[0].clone()]);
    wait(|| HELD.load(Ordering::SeqCst));
    let calls = CALLS.load(Ordering::SeqCst);
    v.load(paths[2].clone());
    assert_eq!(CALLS.load(Ordering::SeqCst), calls);
    RELEASE.store(true, Ordering::SeqCst);
    loaded(&v);
    idle(&v);
    assert_eq!(v.preload_stats().promoted, 1);
    // A blocked, cancelled preload may occupy one slot, never the user's first slot.
    v.invalidate_cache_paths(&paths);
    HELD.store(false, Ordering::SeqCst);
    RELEASE.store(false, Ordering::SeqCst);
    v.set_neighbors(vec![paths[2].clone()]);
    v.load(paths[1].clone());
    loaded(&v);
    wait(|| HELD.load(Ordering::SeqCst));
    v.load(paths[3].clone());
    loaded(&v);
    assert!(v.load_stats().running >= 1);
    assert_eq!(picture(&v).paintable().unwrap().intrinsic_width(), 16);
    RELEASE.store(true, Ordering::SeqCst);
    idle(&v);
    assert!(v.preload_stats().cancelled >= 1);
    check_bound(&v);
    HELD.store(false, Ordering::SeqCst);
    RELEASE.store(false, Ordering::SeqCst);
    v.invalidate_cache_paths(&paths);
    v.set_neighbors(vec![paths[2].clone()]);
    wait(|| HELD.load(Ordering::SeqCst));
    v.load_with_completion(paths[1].clone(), || {});
    loaded(&v);
    v.set_neighbors(vec![]);
    RELEASE.store(true, Ordering::SeqCst);
    idle(&v);
    assert!(
        v.cached_texture(&paths[2]).is_none(),
        "Refresh resurrected obsolete preload"
    );
    // Background errors do not touch foreground labels; continue to the other neighbor.
    HOLD.store(false, Ordering::SeqCst);
    let bad = paths[2].with_file_name("bad.png");
    std::fs::write(&bad, b"bad").unwrap();
    let label = status(&v).text();
    v.set_neighbors(vec![bad, paths[0].clone()]);
    idle(&v);
    assert_eq!(status(&v).text(), label);
    assert!(!status(&v).is_visible());
    // Hundreds of selections remain bounded; cache and workers drain, not a throughput benchmark.
    v.set_neighbors(vec![]);
    for i in 0..300 {
        v.load(paths[i % 2].clone());
    }
    loaded(&v);
    idle(&v);
    check_bound(&v);
    let (other, survivor) = open(app);
    survivor.load(paths[0].clone());
    loaded(&survivor);
    idle(&survivor);
    let stable = picture(&survivor).paintable();
    v.invalidate_cache_paths(&paths);
    HELD.store(false, Ordering::SeqCst);
    RELEASE.store(false, Ordering::SeqCst);
    HOLD.store(true, Ordering::SeqCst);
    v.set_neighbors(vec![paths[2].clone()]);
    wait(|| HELD.load(Ordering::SeqCst));
    w.close();
    RELEASE.store(true, Ordering::SeqCst);
    idle(&v);
    assert_eq!(v.preload_stats().cache.entries, 0);
    assert_eq!(v.phase(), pivra::image_pipeline::Phase::Cancelled);
    assert_eq!(picture(&survivor).paintable(), stable);
    assert!(other.is_visible());
    other.close();
    println!(
        "PASS controlled promotion/no duplicate decoder, reserved foreground admission, cancelled obsolete preload, silent neighbor error, 300 selections, two-window close/drain. NOT real decoder process-stop proof"
    );
}
// Gate only the worker entry, not the production decoder. This establishes
// priority/promotion deterministically; it does not certify an active decoder interval.
fn native_priority(app: &adw::Application) {
    let (_dir, paths) = setup(false);
    let (w, v) = open(app);
    v.load(paths[1].clone());
    loaded(&v);
    idle(&v);
    HOLD.store(true, Ordering::SeqCst);
    RELEASE.store(false, Ordering::SeqCst);
    HELD.store(false, Ordering::SeqCst);
    v.set_neighbors(vec![paths[2].clone()]);
    wait(|| HELD.load(Ordering::SeqCst));
    let before = v.load_stats().started;
    v.load(paths[2].clone());
    assert_eq!(v.load_stats().started, before);
    RELEASE.store(true, Ordering::SeqCst);
    loaded(&v);
    idle(&v);
    assert_eq!(v.preload_stats().promoted, 1);
    v.invalidate_cache_paths(&paths);
    HELD.store(false, Ordering::SeqCst);
    RELEASE.store(false, Ordering::SeqCst);
    v.load(paths[1].clone());
    loaded(&v);
    wait(|| HELD.load(Ordering::SeqCst));
    v.load(paths[3].clone());
    loaded(&v); // Must not wait for the held background worker.
    RELEASE.store(true, Ordering::SeqCst);
    idle(&v);
    check_bound(&v);
    let (other, survivor) = open(app);
    survivor.load(paths[0].clone());
    loaded(&survivor);
    idle(&survivor);
    let stable = picture(&survivor).paintable();
    v.invalidate_cache_paths(&paths);
    HELD.store(false, Ordering::SeqCst);
    RELEASE.store(false, Ordering::SeqCst);
    v.set_neighbors(vec![paths[2].clone()]);
    wait(|| HELD.load(Ordering::SeqCst));
    w.close();
    RELEASE.store(true, Ordering::SeqCst);
    idle(&v);
    assert_eq!(v.preload_stats().cache.entries, 0);
    assert_eq!(v.phase(), pivra::image_pipeline::Phase::Cancelled);
    assert_eq!(picture(&survivor).paintable(), stable);
    assert!(other.is_visible());
    println!(
        "PASS native secure outputs, worker-entry promotion/foreground priority and close/drain: {:?}; active decoder overlap NOT MEASURED",
        v.preload_stats()
    );
    other.close();
}
fn main() {
    let controlled = std::env::args().any(|s| s == "--controlled");
    CONTROLLED.store(controlled, Ordering::SeqCst);
    adw::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.PreloadProbe")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    run(&app, controlled);
    if controlled {
        controlled_priority(&app);
    } else {
        native_priority(&app);
        println!(
            "Native loader pool process/RSS/VRAM behavior requires separate observation; application counters are not those measurements."
        );
    }
}
