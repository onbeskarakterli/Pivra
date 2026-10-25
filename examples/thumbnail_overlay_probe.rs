//! TH-02 only: synthetic main image + explicitly labelled TEST PLACEHOLDERS.
#[path = "support/thumbnail_catalog.rs"]
mod catalog_probe;
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/input.rs"]
#[allow(dead_code)] // The application uses the GFile field too; this probe uses window input.
mod input;
#[path = "../src/image_loader.rs"]
#[allow(dead_code)]
mod native_image_loader;
#[path = "support/render_frame.rs"]
mod render_frame;
#[path = "../src/window.rs"]
mod window;
static CORRUPT_DURING_LOAD: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static THUMB_LOADS: AtomicUsize = AtomicUsize::new(0);
static THUMB_ORDER: std::sync::Mutex<Vec<std::path::PathBuf>> = std::sync::Mutex::new(Vec::new());
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};
static LOADS: AtomicUsize = AtomicUsize::new(0);
thread_local! {
    // Detached only in automated mode: actual GDK crossings/reset must not be
    // mixed with the explicit controller signal sequence under test.
    static TEST_MOTIONS: std::cell::RefCell<Vec<(glib::WeakRef<gtk::Widget>, gtk::EventControllerMotion)>> = const { std::cell::RefCell::new(Vec::new()) };
}
mod image_loader {
    use super::*;
    pub enum LoadedFrame {
        Synthetic(bool),
        Native(native_image_loader::LoadedFrame),
    }
    impl LoadedFrame {
        pub fn texture(&self) -> gdk::Texture {
            match self {
                Self::Native(frame) => return frame.texture(),
                Self::Synthetic(true)
                    if std::env::args().any(|a| a == "--retention-regression") =>
                {
                    return gdk::MemoryTexture::new(
                        512,
                        512,
                        gdk::MemoryFormat::R8g8b8a8,
                        &glib::Bytes::from_owned([180, 80, 40, 255].repeat(512 * 512)),
                        2048,
                    )
                    .upcast();
                }
                Self::Synthetic(true) => {
                    return gdk::MemoryTexture::new(
                        64,
                        32,
                        gdk::MemoryFormat::R8g8b8a8,
                        &glib::Bytes::from_owned([180, 80, 40, 255].repeat(64 * 32)),
                        256,
                    )
                    .upcast();
                }
                Self::Synthetic(false) => {}
            }
            let mut bytes = Vec::with_capacity(1600 * 1200 * 4);
            for y in 0..1200 {
                for x in 0..1600 {
                    bytes.extend_from_slice(&[
                        (x * 255 / 1600) as u8,
                        (y * 255 / 1200) as u8,
                        80,
                        255,
                    ]);
                }
            }
            gdk::MemoryTexture::new(
                1600,
                1200,
                gdk::MemoryFormat::R8g8b8a8,
                &glib::Bytes::from_owned(bytes),
                6400,
            )
            .upcast()
        }
    }
    pub async fn load(
        path: &std::path::Path,
        cancel: &gio::Cancellable,
    ) -> Result<LoadedFrame, String> {
        LOADS.fetch_add(1, Ordering::SeqCst);
        if path.is_absolute() && std::env::args().any(|s| s == "--native-thumbnails") {
            return native_image_loader::load(path, cancel)
                .await
                .map(LoadedFrame::Native);
        }
        Ok(LoadedFrame::Synthetic(
            path.starts_with(glib::user_cache_dir().join("thumbnails")),
        ))
    }
    pub async fn preload(
        p: &std::path::Path,
        c: &gio::Cancellable,
        budget: usize,
    ) -> Result<LoadedFrame, String> {
        THUMB_LOADS.fetch_add(1, Ordering::SeqCst);
        THUMB_ORDER.lock().unwrap().push(p.to_owned());
        if std::env::args().any(|s| s == "--native-thumbnails") {
            native_image_loader::preload(p, c, budget)
                .await
                .map(LoadedFrame::Native)
        } else {
            let result = load(p, c).await;
            if CORRUPT_DURING_LOAD.swap(false, Ordering::SeqCst) {
                std::fs::write(p, b"controlled cache race").unwrap();
            }
            result
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
#[track_caller]
fn wait(mut predicate: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(20);
    while !predicate() {
        assert!(Instant::now() < end, "overlay probe deadline");
        pump();
    }
}
fn find<T: IsA<gtk::Widget> + glib::types::StaticType>(root: &gtk::Widget, name: &str) -> T {
    if root.widget_name() == name {
        return root.clone().downcast().expect("widget type");
    }
    let mut child = root.first_child();
    while let Some(w) = child {
        if contains(&w, name) {
            return find(&w, name);
        }
        child = w.next_sibling();
    }
    panic!("missing {name}");
}
fn contains(w: &gtk::Widget, name: &str) -> bool {
    if w.widget_name() == name {
        return true;
    }
    let mut c = w.first_child();
    while let Some(w) = c {
        if contains(&w, name) {
            return true;
        }
        c = w.next_sibling();
    }
    false
}
fn open(app: &adw::Application) -> (adw::ApplicationWindow, image_view::ImageView, gtk::Picture) {
    let w = adw::ApplicationWindow::builder()
        .application(app)
        .title("TH-02 TEST MAIN IMAGE / TEST PLACEHOLDERS — no real thumbnails")
        .default_width(800)
        .default_height(600)
        .build();
    let view = image_view::ImageView::new(&w);
    if !std::env::args().any(|s| s == "--manual") {
        let motion = controller::<gtk::EventControllerMotion>(view.widget());
        assert_eq!(motion.propagation_phase(), gtk::PropagationPhase::Capture);
        view.widget().remove_controller(&motion);
        TEST_MOTIONS.with(|motions| {
            motions.borrow_mut().push((
                view.widget().upcast_ref::<gtk::Widget>().downgrade(),
                motion,
            ))
        });
    }
    let header = adw::HeaderBar::new();
    header.set_widget_name("th02-test-header");
    header.set_title_widget(Some(&adw::WindowTitle::new(
        "Pivra — TEST",
        "Synthetic image / TEST placeholders",
    )));
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&header);
    content.append(view.widget());
    w.set_content(Some(&content));
    w.present();
    wait(|| w.is_mapped());
    assert!(header.shows_end_title_buttons() && header.shows_start_title_buttons());
    assert!(header.title_widget().is_some());
    assert!(header.is_mapped());
    view.load(std::path::PathBuf::from("TH02-SYNTHETIC"));
    let p = view
        .widget()
        .child()
        .unwrap()
        .downcast::<gtk::Picture>()
        .unwrap();
    wait(|| p.paintable().is_some());
    let row: gtk::Box = find(view.widget().upcast_ref(), "thumbnail-items");
    while let Some(child) = row.first_child() {
        row.remove(&child);
    }
    for n in 0..32 {
        let content = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let sample = gtk::Picture::for_paintable(&gdk::MemoryTexture::new(
            16,
            12,
            gdk::MemoryFormat::R8g8b8a8,
            &glib::Bytes::from_owned([180, n * 6, 80, 255].repeat(16 * 12)),
            64,
        ));
        sample.set_size_request(64, 48);
        sample.set_halign(gtk::Align::Center);
        sample.set_can_shrink(true);
        content.append(&sample);
        let caption = gtk::Label::new(Some(&format!("TEST {}", n + 1)));
        caption.add_css_class("caption");
        content.append(&caption);
        row.append(&image_view::thumbnail_overlay::cell(&content, n == 0));
    }
    (w, view, p)
}
fn image_bytes(w: &adw::ApplicationWindow, p: &gtk::Picture) -> Vec<u8> {
    let node = render_frame::rendered_node(w, p);
    let t = w.renderer().unwrap().render_texture(
        node,
        Some(&gtk::graphene::Rect::new(
            0.,
            0.,
            p.width() as f32,
            p.height() as f32,
        )),
    );
    let mut bytes = vec![0; t.width() as usize * t.height() as usize * 4];
    let stride = t.width() as usize * 4;
    t.download(&mut bytes, stride);
    bytes
}
fn widget_pixels(
    w: &adw::ApplicationWindow,
    p: &gtk::Picture,
    widget: &impl IsA<gtk::Widget>,
) -> (usize, usize, Vec<u8>) {
    let paintable = gtk::WidgetPaintable::new(Some(widget));
    // Keep the paintable alive while a real frame/layout is completed.
    render_frame::rendered_node(w, p);
    let snapshot = gtk::Snapshot::new();
    let width = widget.width();
    let height = widget.height();
    paintable.snapshot(&snapshot, width as f64, height as f64);
    let node = snapshot.to_node().expect("styled widget render node");
    let texture = w.renderer().unwrap().render_texture(
        node,
        Some(&gtk::graphene::Rect::new(
            0.,
            0.,
            width as f32,
            height as f32,
        )),
    );
    let width = texture.width() as usize;
    let height = texture.height() as usize;
    let mut bytes = vec![0; width * height * 4];
    texture.download(&mut bytes, width * 4);
    (width, height, bytes)
}
fn controller<T: IsA<glib::Object> + glib::types::StaticType>(p: &impl IsA<gtk::Widget>) -> T {
    let list = p.observe_controllers();
    (0..list.n_items())
        .find_map(|i| list.item(i).unwrap().downcast().ok())
        .or_else(|| {
            TEST_MOTIONS.with(|motions| {
                motions.borrow().iter().find_map(|(widget, motion)| {
                    if widget.upgrade().as_ref() == Some(p.as_ref()) {
                        motion.clone().upcast::<glib::Object>().downcast::<T>().ok()
                    } else {
                        None
                    }
                })
            })
        })
        .expect("controller")
}
fn point(v: &image_view::ImageView, bottom: bool) {
    controller::<gtk::EventControllerMotion>(v.widget()).emit_by_name::<()>(
        "motion",
        &[
            &20f64,
            &(if bottom {
                v.widget().height() as f64 - 4.
            } else {
                20.
            }),
        ],
    );
}
fn set_enabled(v: &image_view::ImageView, enabled: bool) {
    pivra::settings::thumbnail_preference().set_enabled(enabled);
    point(v, true);
}
fn pause(milliseconds: u64) {
    let until = Instant::now() + Duration::from_millis(milliseconds);
    while Instant::now() < until {
        pump();
    }
}
fn resize_until(
    w: &adw::ApplicationWindow,
    p: &gtk::Picture,
    width: i32,
    height: i32,
    predicate: impl Fn() -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        // Configure replies may update GTK's default-size after our request.
        // Reassert the requested test size after each real frame/roundtrip until
        // the allocation AND visibility criterion hold; never accept a size hint.
        w.set_default_size(width, height);
        image_bytes(w, p);
        p.display().sync();
        pump();
        if predicate() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "resize not confirmed: requested={width}x{height} actual={}x{}",
            w.width(),
            w.height()
        );
    }
}
fn key(w: &adw::ApplicationWindow, modifiers: gdk::ModifierType) -> bool {
    controller::<gtk::EventControllerKey>(w)
        .emit_by_name::<bool>("key-pressed", &[&gdk::Key::t, &0u32, &modifiers])
}
fn main() {
    if std::env::args().any(|s| s == "--require-generation-helper") {
        if let Some(path) = pivra::thumbnail::generation_helper().filter(|p| p.is_file()) {
            println!("GNOME factory sidecar: {}", path.display());
            return;
        }
        eprintln!(
            "NOT RUN: no API-checked GNOME generation sidecar; install native GNOME development packages and rebuild"
        );
        std::process::exit(77);
    }
    // Separate-process persistence checks, without GTK initialization or a display.
    for arg in std::env::args() {
        if let Some(value) = arg.strip_prefix("--persist-enabled=") {
            pivra::settings::thumbnail_preference().set_enabled(value == "true");
            return;
        }
        if let Some(value) = arg.strip_prefix("--expect-enabled=") {
            assert_eq!(
                pivra::settings::thumbnail_preference().enabled(),
                value == "true"
            );
            println!("PASS restarted preference={value}");
            return;
        }
    }

    adw::init().unwrap();
    if std::env::args().any(|arg| arg == "--dark") {
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
    }
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.TH02")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    if std::env::args().any(|s| s == "--retention-regression") {
        catalog_probe::retention(&app);
        return;
    }
    if std::env::args().any(|s| s == "--catalog-regression") {
        catalog_probe::run(&app);
        return;
    }
    if std::env::args().any(|s| s == "--manual") {
        let (first, _first_view, _first_picture) = open(&app);
        let (second, _second_view, _second_picture) = open(&app);
        println!(
            "MANUAL TH-03: TEST images/placeholders only; saved Enabled={}. Move to bottom; click or T to toggle. Re-run to check persistence. No real thumbnails/decoder/generation/selection binding.",
            pivra::settings::thumbnail_preference().enabled()
        );
        let loop_ = glib::MainLoop::new(None, false);
        let remaining = std::rc::Rc::new(std::cell::Cell::new(2usize));
        for window in [&first, &second] {
            let quit = loop_.clone();
            let count = remaining.clone();
            window.connect_close_request(move |_| {
                count.set(count.get().saturating_sub(1));
                if count.get() == 0 {
                    quit.quit();
                }
                glib::Propagation::Proceed
            });
        }
        loop_.run();
        return;
    }
    pivra::settings::thumbnail_preference().set_enabled(false);
    let (w, v, p) = open(&app);
    let (other, ov, op) = open(&app);
    let button: gtk::ToggleButton = find(v.widget().upcast_ref(), "thumbnail-toggle");
    let strip: gtk::ScrolledWindow = find(v.widget().upcast_ref(), "thumbnail-strip");
    let other_button: gtk::ToggleButton = find(ov.widget().upcast_ref(), "thumbnail-toggle");
    assert!(
        !button.is_visible() && !strip.is_visible(),
        "initial visibility is not Enabled"
    );
    point(&v, true);
    assert!(button.is_visible(), "run with XDG_CURRENT_DESKTOP=GNOME");
    assert!(!strip.is_visible());
    assert_eq!(button.icon_name().as_deref(), Some("pan-up-symbolic"));
    // TH-03: real controllers and GLib timers; no decoder success is implied.
    button.emit_clicked();
    assert!(strip.is_visible() && other_button.is_active());
    point(&v, false);
    pause(200);
    assert!(strip.is_visible(), "hide delay prevents flicker");
    point(&v, true);
    pause(400);
    assert!(strip.is_visible(), "re-entry cancels old timeout");
    point(&v, false);
    wait(|| !strip.is_visible() && !button.is_visible());
    // A Wayland compositor may withhold frame callbacks from an occluded
    // window. Present the window whose real frame/layout we are observing.
    image_bytes(&w, &p);
    // Layout-driven re-evaluation, without a new motion signal/polling timer.
    controller::<gtk::EventControllerMotion>(v.widget())
        .emit_by_name::<()>("motion", &[&20f64, &100f64]);
    wait(|| !strip.is_visible());
    resize_until(&w, &p, 800, 240, || {
        v.widget().height() < 250 && strip.is_visible()
    });
    resize_until(&w, &p, 800, 600, || {
        v.widget().height() > 400 && !strip.is_visible()
    });
    for _ in 0..8 {
        point(&v, true);
        point(&v, false);
    }
    point(&v, true);
    pause(550);
    assert!(strip.is_visible());
    let bounds = strip.compute_bounds(v.widget()).unwrap();
    controller::<gtk::EventControllerMotion>(v.widget())
        .emit_by_name::<()>("motion", &[&20f64, &(bounds.y() as f64 + 20.)]);
    pause(550);
    assert!(strip.is_visible(), "panel interaction keeps it open");
    point(&v, false);
    wait(|| !strip.is_visible());
    assert!(key(&w, gdk::ModifierType::empty())); // disable
    assert!(!pivra::settings::thumbnail_preference().enabled());
    for modifier in [
        gdk::ModifierType::CONTROL_MASK,
        gdk::ModifierType::ALT_MASK,
        gdk::ModifierType::SUPER_MASK,
    ] {
        assert!(!key(&w, modifier));
        assert!(!pivra::settings::thumbnail_preference().enabled());
    }
    let entry = gtk::Entry::new();
    v.widget().add_overlay(&entry);
    entry.grab_focus();
    assert!(
        !key(&w, gdk::ModifierType::empty()),
        "typing t is not intercepted"
    );
    v.widget().remove_overlay(&entry);
    p.set_focusable(true);
    p.grab_focus();
    assert!(key(&w, gdk::ModifierType::empty()));
    assert!(strip.is_visible(), "T flashes enabled strip");
    wait(|| !strip.is_visible() && !button.is_visible());
    assert!(pivra::settings::thumbnail_preference().enabled());
    // Other windows can be visible independently; global preference changes sync.
    // Complete the other window's real frame before using its height for motion.
    image_bytes(&other, &op);
    point(&ov, true);
    assert!(other_button.is_visible() && !button.is_visible());
    point(&ov, false);
    wait(|| !other_button.is_visible());
    let unavailable_window = adw::ApplicationWindow::builder().application(&app).build();
    let unavailable = gtk::Overlay::new();
    unavailable.set_child(Some(&gtk::Label::new(Some("TEST unavailable"))));
    image_view::thumbnail_overlay::install(&unavailable_window, &unavailable, false);
    unavailable_window.set_content(Some(&unavailable));
    unavailable_window.present();
    wait(|| unavailable_window.is_mapped());
    let unavailable_button: gtk::ToggleButton = find(unavailable.upcast_ref(), "thumbnail-toggle");
    assert!(!unavailable_button.is_visible());
    assert!(!key(&unavailable_window, gdk::ModifierType::empty()));
    assert!(
        pivra::settings::thumbnail_preference().enabled(),
        "availability does not erase preference"
    );
    unavailable_window.close();
    set_enabled(&v, false);
    let loads = LOADS.load(Ordering::SeqCst);
    for (width, height) in [(800, 600), (320, 300)] {
        w.set_default_size(width, height);
        for mode in ["fit", "fill", "actual"] {
            gio::prelude::ActionGroupExt::activate_action(
                &w,
                "image-mode",
                Some(&mode.to_variant()),
            );
            let before = image_bytes(&w, &p);
            let allocated = (p.width(), p.height());
            let paintable = p.paintable();
            set_enabled(&v, true);
            let after = image_bytes(&w, &p);
            assert_eq!(allocated, (p.width(), p.height()));
            assert_eq!(before, after);
            assert_eq!(p.paintable(), paintable);
            assert_eq!((v.widget().width(), v.widget().height()), allocated);
            assert!(strip.is_visible());
            assert_eq!(button.icon_name().as_deref(), Some("pan-down-symbolic"));
            let bounds = strip.compute_bounds(v.widget()).unwrap();
            assert!(
                bounds.x() >= 0.
                    && bounds.y() >= 0.
                    && bounds.x() + bounds.width() <= p.width() as f32 + 1.
                    && bounds.y() + bounds.height() <= p.height() as f32 + 1.
            );
            assert_eq!(strip.parent(), Some(v.widget().clone().upcast()));
            assert_eq!(button.parent(), Some(v.widget().clone().upcast()));
            assert!(!contains(v.widget().upcast_ref(), "thumbnail-controls"));
            let center = button.compute_bounds(v.widget()).unwrap();
            assert!(
                center.y() + center.height() <= bounds.y(),
                "toggle must float above panel"
            );
            assert!(
                (bounds.y() - center.y() - center.height() - 4.).abs() <= 1.,
                "toggle gap must follow actual allocated panel edge: panel={bounds:?} button={center:?}"
            );
            assert!(strip.is_overlay_scrolling());
            let bar = strip.hscrollbar();
            if bar.is_mapped() {
                let b = bar.compute_bounds(v.widget()).unwrap();
                assert!(
                    b.y() >= bounds.y() - 1.
                        && b.y() + b.height() <= bounds.y() + bounds.height() + 1.,
                    "scrollbar escaped panel allocation"
                );
            }
            assert!(
                (bounds.y() + bounds.height() - p.height() as f32).abs() <= 1.,
                "panel must touch bottom"
            );
            assert!(
                bounds.x().abs() <= 1. && (bounds.width() - p.width() as f32).abs() <= 1.,
                "panel must span viewport width"
            );
            // Real composed overlay pixels alongside the button must equal the
            // picture outside the button, both visually and for pointer picks.
            let composed = widget_pixels(&w, &p, v.widget());
            let picture = image_bytes(&w, &p);
            for y in center.y().ceil() as usize..(center.y() + center.height()).floor() as usize {
                for x in 0..composed.0 {
                    if x as f32 >= center.x() && (x as f32) < center.x() + center.width() {
                        continue;
                    }
                    let offset = (y * composed.0 + x) * 4;
                    assert_eq!(
                        &composed.2[offset..offset + 4],
                        &picture[offset..offset + 4],
                        "unexpected dock/background pixel at {x},{y}"
                    );
                }
            }
            for x in [8., p.width() as f64 - 8.] {
                let hit = v
                    .widget()
                    .pick(
                        x,
                        f64::from(center.y() + center.height() / 2.),
                        gtk::PickFlags::DEFAULT,
                    )
                    .unwrap();
                assert_eq!(
                    hit,
                    p.clone().upcast::<gtk::Widget>(),
                    "open strip leaves button sides interactive"
                );
            }

            assert!((center.x() + center.width() / 2. - p.width() as f32 / 2.).abs() <= 1.);
            let row: gtk::Box = find(v.widget().upcast_ref(), "thumbnail-items");
            let first = row.first_child().unwrap();
            assert_eq!(
                {
                    let bounds = first.compute_bounds(&row).unwrap();
                    (bounds.width() as i32, bounds.height() as i32)
                },
                (96, 80),
                "fixed logical cell allocation"
            );
            assert!(strip.hadjustment().upper() > strip.hadjustment().page_size());
            strip.hadjustment().set_value(120.);
            assert!(strip.hadjustment().value() > 0.);
            point(&ov, false);
            wait(|| !other_button.is_visible());
            assert!(other_button.is_active(), "Enabled is shared");
            let other_strip: gtk::ScrolledWindow =
                find(ov.widget().upcast_ref(), "thumbnail-strip");
            assert!(!other_strip.is_visible(), "Visible is per-window");
            set_enabled(&v, false);
            assert_eq!(before, image_bytes(&w, &p));
            assert!(!strip.is_mapped());
            assert!(button.width() < 80, "toggle target too wide");
            let hit = v
                .widget()
                .pick(8., p.height() as f64 - 8., gtk::PickFlags::DEFAULT)
                .unwrap();
            assert_eq!(hit, p.clone().upcast::<gtk::Widget>());
        }
    }
    controller::<gtk::EventControllerMotion>(&p).emit_by_name::<()>("enter", &[&120f64, &100f64]);
    controller::<gtk::EventControllerScroll>(&p).emit_by_name::<bool>("scroll", &[&0f64, &-3f64]);
    let drag = controller::<gtk::GestureDrag>(&p);
    drag.emit_by_name::<()>("drag-begin", &[&100f64, &100f64]);
    drag.emit_by_name::<()>("drag-update", &[&30f64, &-20f64]);
    drag.emit_by_name::<()>("drag-end", &[&30f64, &-20f64]);
    let zoomed = image_bytes(&w, &p);
    set_enabled(&v, true);
    assert_eq!(zoomed, image_bytes(&w, &p));
    set_enabled(&v, false);
    let row: gtk::Box = find(v.widget().upcast_ref(), "thumbnail-items");
    let cell = row.first_child().unwrap();
    assert!(cell.has_css_class("selected"));
    assert_eq!(cell.opacity(), 1.);
    assert_eq!(strip.opacity(), 1.);
    set_enabled(&v, true);
    let selected = widget_pixels(&w, &p, &cell);
    assert_eq!(
        selected.2[(selected.0 * 24 + selected.0 / 2) * 4 + 3],
        255,
        "thumbnail pixels remain opaque"
    );
    assert!(
        selected.2[(selected.0 * 24 + 1) * 4 + 3] > 0,
        "TH-04 selected outline"
    );
    assert!(
        selected.2[(selected.0 + 1) * 4 + 3] > 0,
        "selected top outline"
    );
    assert!(
        selected.2[((selected.1 - 1) * selected.0 + 1) * 4 + 3] > 0,
        "selected underline"
    );
    cell.remove_css_class("selected");
    let normal = widget_pixels(&w, &p, &cell);
    assert_eq!((normal.0, normal.1), (selected.0, selected.1));
    assert_eq!(
        normal.2[((normal.1 - 1) * normal.0 + 1) * 4 + 3],
        0,
        "no normal-cell frame"
    );
    cell.add_css_class("selected");
    let panel = widget_pixels(&w, &p, &strip);
    let alpha = panel.2[(panel.0 * 2 + panel.0 / 2) * 4 + 3];
    assert!(
        (60..=68).contains(&alpha),
        "panel alpha expected about25%, got {alpha}"
    );
    set_enabled(&v, false);
    assert_eq!(LOADS.load(Ordering::SeqCst), loads);
    assert_eq!(v.load_stats().started, 1);
    assert_eq!(ov.load_stats().started, 1);
    point(&v, false);
    wait(|| !button.is_visible());
    v.load(std::path::PathBuf::from("TH03-NAVIGATION-SYNTHETIC"));
    wait(|| v.load_stats().completed == 2);
    assert!(
        !button.is_visible() && !strip.is_visible(),
        "new media does not reveal controls"
    );
    point(&v, true);
    point(&v, false); // close with a pending hide; callback must not affect survivor
    let stable = image_bytes(&other, &op);
    w.close();
    assert!(other.is_visible());
    pause(550);
    assert_eq!(stable, image_bytes(&other, &op));
    println!(
        "PASS TH-02 synthetic UI: overlay allocation/unchanged Fit-Fill-Actual pixels, zoom/pan preservation, horizontal scroll, icons, 25% panel alpha/opaque cells/borderless underline/header controls, TH-03 motion/T/shared preference/timer cancellation/unavailable/text focus, narrow resize, bottom-attached panel/upper floating toggle/no button-row band/open-and-closed pick-through, two-window close/isolation, zero toggle-induced decoder jobs. scale_factor={}",
        op.scale_factor()
    );
    other.close();
}
