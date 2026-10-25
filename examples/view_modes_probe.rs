//! Default: secure decode + production actions. --synthetic: renderer only.
//! --controlled-navigation: explicit synthetic loader, real production actions.
#[path = "../src/image_loader.rs"]
#[allow(dead_code)]
mod secure_image_loader;
// The default probe always uses secure Glycin. Controlled navigation is an
// explicit test-only boundary, never a decoder fallback in the application.
mod image_loader {
    use super::*;
    pub enum LoadedFrame {
        Native(secure_image_loader::LoadedFrame),
        Controlled(i32, i32),
    }
    impl LoadedFrame {
        pub fn texture(&self) -> gdk::Texture {
            match self {
                Self::Native(frame) => frame.texture(),
                Self::Controlled(w, h) => gdk::MemoryTexture::new(
                    *w,
                    *h,
                    gdk::MemoryFormat::R8g8b8a8,
                    &glib::Bytes::from_owned([180, 80, 40, 255].repeat((*w * *h) as usize)),
                    *w as usize * 4,
                )
                .upcast(),
            }
        }
    }
    fn controlled() -> bool {
        std::env::args().any(|s| s == "--controlled-navigation")
    }
    pub async fn load(path: &Path, cancel: &gio::Cancellable) -> Result<LoadedFrame, String> {
        if !controlled() {
            return secure_image_loader::load(path, cancel)
                .await
                .map(LoadedFrame::Native);
        }
        if cancel.is_cancelled() {
            return Err("controlled request cancelled".into());
        }
        match path.file_name().and_then(|p| p.to_str()) {
            Some("1.png") => Ok(LoadedFrame::Controlled(1600, 1200)),
            Some("2.png" | "only.png") => Ok(LoadedFrame::Controlled(32, 64)),
            Some("4.png") => Ok(LoadedFrame::Controlled(4, 2)),
            Some("3.png") => Err("controlled broken image (not secure decoder evidence)".into()),
            _ => Err(format!("unexpected controlled input {}", path.display())),
        }
    }
    pub async fn preload(
        path: &Path,
        cancel: &gio::Cancellable,
        budget: usize,
    ) -> Result<LoadedFrame, String> {
        if !controlled() {
            return secure_image_loader::preload(path, cancel, budget)
                .await
                .map(LoadedFrame::Native);
        }
        let frame = load(path, cancel).await?;
        if let LoadedFrame::Controlled(w, h) = frame {
            if pivra::image_cache::texture_cost(w as u32, h as u32).is_none_or(|n| n > budget) {
                return Err("controlled preload budget exceeded".into());
            }
        }
        Ok(frame)
    }
}
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/input.rs"]
mod input;
#[path = "support/render_frame.rs"]
mod render_frame;
#[path = "support/resampling_cases.rs"]
mod resampling_cases;
use render_frame::rendered_node;
#[path = "../src/window.rs"]
mod window;
#[path = "support/zoom_pan_cases.rs"]
mod zoom_pan_cases;
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use pivra::viewing::ViewMode;
use std::{
    path::Path,
    time::{Duration, Instant},
};
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
        assert!(Instant::now() < end, "mode probe deadline");
        pump();
    }
}
fn settle() {
    let end = Instant::now() + Duration::from_millis(100);
    while Instant::now() < end {
        pump();
    }
}
fn widgets(w: &gtk::Widget) -> Vec<gtk::Widget> {
    let mut v = vec![w.clone()];
    let mut c = w.first_child();
    while let Some(w) = c {
        v.extend(widgets(&w));
        c = w.next_sibling();
    }
    v
}
fn picture(w: &adw::ApplicationWindow) -> gtk::Picture {
    widgets(w.upcast_ref())
        .into_iter()
        .find_map(|w| w.downcast().ok())
        .unwrap()
}
fn select(w: &adw::ApplicationWindow, mode: ViewMode) {
    gio::prelude::ActionGroupExt::activate_action(w, "image-mode", Some(&mode.id().to_variant()));
}
fn state(w: &adw::ApplicationWindow) -> String {
    w.lookup_action("image-mode")
        .unwrap()
        .state()
        .unwrap()
        .str()
        .unwrap()
        .into()
}
fn render(
    w: &adw::ApplicationWindow,
    p: &gtk::Picture,
    size: (i32, i32),
    mode: ViewMode,
    gradient: bool,
) {
    let node = rendered_node(w, p);
    let (width, height) = (p.width(), p.height());
    assert!(width > 0 && height > 0);
    let g = mode
        .geometry(
            (size.0 as f64, size.1 as f64),
            (width as f64, height as f64),
        )
        .unwrap();
    let bounds = node.bounds();
    assert!(
        bounds.x() >= -1.
            && bounds.y() >= -1.
            && bounds.x() + bounds.width() <= width as f32 + 1.
            && bounds.y() + bounds.height() <= height as f32 + 1.,
        "unclipped render node: {bounds:?}"
    );
    let texture = w.renderer().unwrap().render_texture(
        node,
        Some(&gtk::graphene::Rect::new(
            0.,
            0.,
            width as f32,
            height as f32,
        )),
    );
    let mut pixels = vec![0; width as usize * height as usize * 4];
    texture.download(&mut pixels, width as usize * 4);
    let (mut left, mut top, mut right, mut bottom) = (width, height, -1, -1);
    for y in 0..height {
        for x in 0..width {
            if pixels[((y * width + x) * 4 + 3) as usize] > 127 {
                left = left.min(x);
                right = right.max(x);
                top = top.min(y);
                bottom = bottom.max(y);
            }
        }
    }
    let expected = (
        g.x.max(0.),
        g.y.max(0.),
        (g.x + g.width).min(width as f64),
        (g.y + g.height).min(height as f64),
    );
    for (actual, expected) in [
        (left as f64, expected.0),
        (top as f64, expected.1),
        ((right + 1) as f64, expected.2),
        ((bottom + 1) as f64, expected.3),
    ] {
        assert!(
            (actual - expected).abs() <= 2.,
            "{mode:?} rendered bounds {left},{top},{right},{bottom}; geometry {g:?}"
        );
    }
    if gradient {
        for fx in [0.2, 0.5, 0.8] {
            for fy in [0.2, 0.5, 0.8] {
                let x = (expected.0 + (expected.2 - expected.0) * fx).floor() as i32;
                let y = (expected.1 + (expected.3 - expected.1) * fy).floor() as i32;
                let offset = ((y * width + x) * 4) as usize;
                let red = (((x as f64 + 0.5 - g.x) / g.width) * 255.).clamp(0., 255.);
                let green = (((y as f64 + 0.5 - g.y) / g.height) * 255.).clamp(0., 255.);
                assert!(
                    (pixels[offset + 2] as f64 - red).abs() <= 5.
                        && (pixels[offset + 1] as f64 - green).abs() <= 5.,
                    "stretched/miscentered sample {mode:?}: {:?} expected {red}/{green}",
                    &pixels[offset..offset + 4]
                );
            }
        }
    }
    println!(
        "PASS rendered {mode:?} source {}x{} viewport {width}x{height}, centered/aspect/clip",
        size.0, size.1
    );
}
fn synthetic(app: &adw::Application) {
    let w = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(800)
        .default_height(600)
        .build();
    let view = image_view::ImageView::new(&w);
    w.set_content(Some(view.widget()));
    w.present();
    wait(|| w.is_mapped());
    let p = picture(&w);
    for (iw, ih) in [(1600, 1200), (1200, 1600), (32, 64), (64, 32)] {
        let mut data = Vec::with_capacity(iw as usize * ih as usize * 4);
        for y in 0..ih {
            for x in 0..iw {
                data.extend_from_slice(&[
                    ((x as f64 + 0.5) / iw as f64 * 255.) as u8,
                    ((y as f64 + 0.5) / ih as f64 * 255.) as u8,
                    83,
                    255,
                ]);
            }
        }
        let bytes = glib::Bytes::from_owned(data);
        let t: gdk::Texture =
            gdk::MemoryTexture::new(iw, ih, gdk::MemoryFormat::R8g8b8a8, &bytes, iw as usize * 4)
                .upcast();
        for (vw, vh) in [(800, 600), (480, 320), (320, 700)] {
            w.set_default_size(vw, vh);
            settle();
            for mode in [
                ViewMode::Fit,
                ViewMode::Fill,
                ViewMode::Actual,
                ViewMode::Fit,
            ] {
                image_view::present(&p, Some(&t), mode);
                render(&w, &p, (iw, ih), mode, true);
            }
        }
    }
    w.close();
    println!(
        "PASS 48 synthetic-texture production presentation snapshots; NOT TESTED Glycin/EXIF/native decode"
    );
}
fn open_window(app: &adw::Application, path: &Path) -> adw::ApplicationWindow {
    let before = app.windows();
    let input = input::OpenedFile::from_argument(path.as_os_str()).unwrap();
    assert_eq!(input.file.path().as_deref(), Some(path));
    window::build(app, &Default::default(), Some(input));
    let w = app
        .windows()
        .into_iter()
        .find(|w| !before.contains(w))
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    wait(|| w.is_mapped());
    w
}
fn status(p: &gtk::Picture) -> gtk::Label {
    p.next_sibling().unwrap().downcast::<gtk::Label>().unwrap()
}
fn selected(w: &adw::ApplicationWindow, name: &str) -> bool {
    w.title()
        .is_some_and(|s| s.starts_with(&format!("{name} —")))
}
fn loaded(
    w: &adw::ApplicationWindow,
    name: &str,
    size: (i32, i32),
    previous: Option<&gdk::Paintable>,
) -> gtk::Picture {
    let p = picture(w);
    let label = status(&p);
    let end = Instant::now() + Duration::from_secs(20);
    loop {
        assert!(
            !(selected(w, name) && label.is_visible() && label.text().starts_with("Unable")),
            "valid selected image {name} failed after baseline: {}",
            label.text()
        );
        // Retained old pixels and the synchronous title change do not certify
        // completion. Hold the old object alive and require a replacement with
        // the selected fixture's dimensions. Sample only after the GTK callback.
        if selected(w, name)
            && p.paintable().is_some_and(|frame| {
                (frame.intrinsic_width(), frame.intrinsic_height()) == size
                    && previous.is_none_or(|old| &frame != old)
            })
        {
            break;
        }
        assert!(
            Instant::now() < end,
            "selected-frame deadline: target={name} size={size:?} title={:?} mode={} current-size={:?} error={}",
            w.title(),
            state(w),
            p.paintable()
                .map(|p| (p.intrinsic_width(), p.intrinsic_height())),
            label.text()
        );
        pump();
    }
    assert!(
        !label.is_visible(),
        "ready selected image still shows an error"
    );
    p
}

fn failed_selection(w: &adw::ApplicationWindow, name: &str, previous: &gdk::Paintable) {
    let p = picture(w);
    let label = status(&p);
    wait(|| {
        selected(w, name)
            && label.is_visible()
            && label.text().starts_with("Unable")
            && label.text().contains(&format!("File: {name}"))
    });
    assert_eq!(
        p.paintable().as_ref(),
        Some(previous),
        "failed image must retain the previous photo"
    );
    assert!(label.text().contains("Previous photo is still displayed."));
    assert!(
        !label.text().contains("Loader process exited early"),
        "loader failure is not invalid-media success"
    );
}

fn native_isolation(app: &adw::Application, fixtures: &Path) {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let cases = [
        ("1.png", "large.png"),
        ("2.png", "tall.png"),
        ("3.png", "broken.png"),
        ("4.png", "small.png"),
    ];
    for (name, source) in cases {
        std::fs::copy(fixtures.join(source), a.path().join(name)).unwrap();
    }
    std::fs::copy(fixtures.join("tall.png"), b.path().join("only.png")).unwrap();
    let first = open_window(app, &a.path().join("1.png"));
    let second = open_window(app, &b.path().join("only.png"));
    wait(|| first.title().is_some_and(|s| s.ends_with("(4 candidates)")));
    let p = loaded(&first, "1.png", (1600, 1200), None);
    let q = loaded(&second, "only.png", (32, 64), None);
    if std::env::args().any(|s| s == "--zoom-pan") {
        zoom_pan_cases::native(&first);
        assert_eq!(state(&second), "fit");
        zoom_pan_cases::native(&second);
        assert_eq!(state(&first), "fit");
    }
    select(&first, ViewMode::Actual);
    select(&second, ViewMode::Fill);
    let survivor = q.paintable();
    render(&first, &p, (1600, 1200), ViewMode::Actual, false);
    render(&second, &q, (32, 64), ViewMode::Fill, false);
    first.set_default_size(480, 320);
    render(&first, &p, (1600, 1200), ViewMode::Actual, false);
    let old = p.paintable().unwrap();
    gio::prelude::ActionGroupExt::activate_action(&first, "next-media", None);
    assert!(selected(&first, "2.png"));
    assert_eq!(
        p.paintable().as_ref(),
        Some(&old),
        "request acceptance must not be confused with publication"
    );
    assert_eq!(
        state(&first),
        "actual",
        "pending navigation must preserve the visible preset"
    );
    loaded(&first, "2.png", (32, 64), Some(&old));
    assert_eq!(
        state(&first),
        "actual",
        "accepted preset must survive new-frame publication"
    );
    render(&first, &p, (32, 64), ViewMode::Actual, false);
    // Explicit user mode changes exercise Fit and expose its backing texture
    // for the cache identity check below; this is not an automatic Fit reset.
    select(&first, ViewMode::Fit);
    gio::prelude::ActionGroupExt::activate_action(
        &first,
        "image-resampling",
        Some(&"smooth".to_variant()),
    );
    let cached_tall = p.paintable().unwrap().downcast::<gdk::Texture>().unwrap();
    render(&first, &p, (32, 64), ViewMode::Fit, false);
    select(&first, ViewMode::Fill);
    let before_error = p.paintable();
    gio::prelude::ActionGroupExt::activate_action(&first, "next-media", None);
    failed_selection(&first, "3.png", before_error.as_ref().unwrap());
    assert_eq!(state(&first), "fill");
    let old = p.paintable().unwrap();
    gio::prelude::ActionGroupExt::activate_action(&first, "next-media", None);
    loaded(&first, "4.png", (4, 2), Some(&old));
    assert_eq!(
        state(&first),
        "fill",
        "error recovery must preserve the chosen preset"
    );
    render(&first, &p, (4, 2), ViewMode::Fill, false);
    select(&first, ViewMode::Actual);
    let scroll = p
        .observe_controllers()
        .iter::<glib::Object>()
        .filter_map(Result::ok)
        .find_map(|c| c.downcast::<gtk::EventControllerScroll>().ok())
        .unwrap();
    assert!(scroll.emit_by_name::<bool>("scroll", &[&0f64, &-1f64]));
    assert_eq!(state(&first), "manual");
    let manual = p.paintable().unwrap();
    // Rapid 4→broken3→cached2: old error may not overwrite the latest selection.
    gio::prelude::ActionGroupExt::activate_action(&first, "previous-media", None);
    gio::prelude::ActionGroupExt::activate_action(&first, "previous-media", None);
    assert_eq!(
        state(&first),
        "manual",
        "manual geometry must remain until the replacement is ready"
    );
    loaded(&first, "2.png", (32, 64), Some(&manual));
    assert_eq!(
        state(&first),
        "fit",
        "manual transform must reset only at new-frame handoff"
    );
    assert_eq!(
        p.paintable(),
        Some(cached_tall.upcast()),
        "warm navigation did not reuse the cached source texture"
    );
    assert_eq!(
        first
            .lookup_action("image-resampling")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("smooth")
    );
    render(&first, &p, (32, 64), ViewMode::Fit, false);
    assert_eq!(state(&second), "fill");
    assert_eq!(q.paintable(), survivor);
    select(&first, ViewMode::Actual);
    let closed = first.downgrade();
    first.close();
    drop(first);
    wait(|| closed.upgrade().is_none());
    assert!(second.is_visible());
    assert_eq!(state(&second), "fill");
    render(&second, &q, (32, 64), ViewMode::Fill, false);
    second.close();
    for (name, source) in cases {
        assert_eq!(
            std::fs::read(a.path().join(name)).unwrap(),
            std::fs::read(fixtures.join(source)).unwrap()
        );
    }
    assert_eq!(
        std::fs::read(b.path().join("only.png")).unwrap(),
        std::fs::read(fixtures.join("tall.png")).unwrap()
    );
    let boundary = if std::env::args().any(|s| s == "--controlled-navigation") {
        "CONTROLLED pixels, not Glycin"
    } else {
        "SECURE Glycin pixels"
    };
    println!(
        "PASS {boundary}: selected-frame handoff, Actual/Fill preset retention, manual-to-Fit reset, warm texture identity, rapid broken-to-cached navigation and two-window close/isolation"
    );
}
fn native(app: &adw::Application) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images");
    let mut baseline = false;
    for name in [
        "large.png",
        "small.png",
        "tall.png",
        "orientation-1.jpg",
        "orientation-2.jpg",
        "orientation-3.jpg",
        "orientation-4.jpg",
        "orientation-5.jpg",
        "orientation-6.jpg",
        "orientation-7.jpg",
        "orientation-8.jpg",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        let original = std::fs::read(root.join(name)).unwrap();
        std::fs::write(&path, &original).unwrap();
        let input = input::OpenedFile::from_argument(path.as_os_str()).unwrap();
        assert_eq!(input.file.path().as_deref(), Some(path.as_path()));
        window::build(app, &Default::default(), Some(input));
        let w = app.windows()[0]
            .clone()
            .downcast::<adw::ApplicationWindow>()
            .unwrap();
        let p = picture(&w);
        wait(|| w.is_mapped());
        // Select during the load request; secure decoder timing is not certified.
        select(&w, ViewMode::Fill);
        wait(|| {
            p.paintable().is_some()
                || p.next_sibling()
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap()
                    .text()
                    .starts_with("Unable")
        });
        if p.paintable().is_none() && !baseline {
            eprintln!("NOT RUN secure baseline decode");
            std::process::exit(77);
        }
        assert!(
            p.paintable().is_some(),
            "valid secure image failed after baseline"
        );
        baseline = true;
        let decoded = p.paintable().unwrap().downcast::<gdk::Texture>().unwrap();
        let size = (decoded.width(), decoded.height());
        let expected = match name {
            "large.png" => (1600, 1200),
            "small.png" => (4, 2),
            "tall.png" => (32, 64),
            _ if name.contains("-5.")
                || name.contains("-6.")
                || name.contains("-7.")
                || name.contains("-8.") =>
            {
                (32, 64)
            }
            _ => (64, 32),
        };
        assert_eq!(size, expected, "EXIF-transformed source dimensions");
        if std::env::args().any(|s| s == "--resampling") {
            resampling_cases::native(&w);
            select(&w, ViewMode::Fill);
        }
        if std::env::args().any(|s| s == "--zoom-pan") {
            zoom_pan_cases::native(&w);
            select(&w, ViewMode::Fill);
        }
        assert_eq!(state(&w), "fill");
        for (vw, vh) in [(800, 600), (320, 700)] {
            w.set_default_size(vw, vh);
            settle();
            for mode in [
                ViewMode::Fit,
                ViewMode::Fill,
                ViewMode::Actual,
                ViewMode::Fit,
            ] {
                select(&w, mode);
                assert_eq!(state(&w), mode.id());
                render(&w, &p, size, mode, false);
            }
        }
        assert_eq!(
            p.paintable().unwrap(),
            decoded.upcast::<gdk::Paintable>(),
            "mode switch reloaded/replaced source texture"
        );
        assert_eq!(std::fs::read(path).unwrap(), original);
        w.close();
        drop(w);
        settle();
    }
    if std::env::args().any(|s| s == "--resampling") {
        resampling_cases::native_checker(app);
    }
    native_isolation(app, &root);
    println!(
        "PASS secure production mode/render/resize/EXIF 1–8/source integrity gate; physical GNOME menu/keys remain manual"
    );
}
fn main() {
    adw::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.ModesProbe")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    if std::env::args().any(|s| s == "--controlled-navigation") {
        assert!(
            !std::env::args().any(|s| s == "--synthetic"),
            "navigation and renderer-only modes must stay separate"
        );
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images");
        native_isolation(&app, &root);
    } else if std::env::args().any(|s| s == "--synthetic") {
        if std::env::args().any(|s| s == "--resampling") {
            resampling_cases::synthetic(&app);
        } else if std::env::args().any(|s| s == "--zoom-pan") {
            zoom_pan_cases::synthetic(&app);
        } else {
            synthetic(&app);
        }
    } else {
        native(&app);
    }
}
