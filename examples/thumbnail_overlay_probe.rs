//! TH-02 only: synthetic main image + explicitly labelled TEST PLACEHOLDERS.
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "support/render_frame.rs"]
mod render_frame;
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};
static LOADS: AtomicUsize = AtomicUsize::new(0);
mod image_loader {
    use super::*;
    pub struct LoadedFrame;
    impl LoadedFrame {
        pub fn texture(&self) -> gdk::Texture {
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
    pub async fn load(_: &std::path::Path, _: &gio::Cancellable) -> Result<LoadedFrame, String> {
        LOADS.fetch_add(1, Ordering::SeqCst);
        Ok(LoadedFrame)
    }
    pub async fn preload(
        p: &std::path::Path,
        c: &gio::Cancellable,
        _: usize,
    ) -> Result<LoadedFrame, String> {
        load(p, c).await
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
    w.set_content(Some(view.widget()));
    w.present();
    wait(|| w.is_mapped());
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
fn controller<T: IsA<glib::Object> + glib::types::StaticType>(p: &gtk::Picture) -> T {
    let list = p.observe_controllers();
    (0..list.n_items())
        .find_map(|i| list.item(i).unwrap().downcast().ok())
        .expect("controller")
}
fn main() {
    adw::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra.TH02")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    let (w, v, p) = open(&app);
    let (other, ov, op) = open(&app);
    let button: gtk::ToggleButton = find(v.widget().upcast_ref(), "thumbnail-toggle");
    let strip: gtk::ScrolledWindow = find(v.widget().upcast_ref(), "thumbnail-strip");
    let dock: gtk::Box = find(v.widget().upcast_ref(), "thumbnail-controls");
    let other_button: gtk::ToggleButton = find(ov.widget().upcast_ref(), "thumbnail-toggle");
    assert!(
        dock.is_visible(),
        "run in GNOME or explicit XDG_CURRENT_DESKTOP=GNOME test environment"
    );
    assert!(!strip.is_visible());
    assert_eq!(button.icon_name().as_deref(), Some("pan-up-symbolic"));
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
            button.set_active(true);
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
            let center = button.compute_bounds(v.widget()).unwrap();
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
            assert!(!other_button.is_active());
            button.set_active(false);
            assert_eq!(before, image_bytes(&w, &p));
            assert!(!strip.is_mapped());
            assert!(dock.width() < 80, "closed target too wide");
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
    button.set_active(true);
    assert_eq!(zoomed, image_bytes(&w, &p));
    button.set_active(false);
    let row: gtk::Box = find(v.widget().upcast_ref(), "thumbnail-items");
    let cell = row.first_child().unwrap();
    assert!(cell.has_css_class("selected"));
    assert_eq!(cell.opacity(), 1.);
    assert_eq!(strip.opacity(), 1.);
    assert_eq!(LOADS.load(Ordering::SeqCst), loads);
    assert_eq!(v.load_stats().started, 1);
    assert_eq!(ov.load_stats().started, 1);
    let stable = image_bytes(&other, &op);
    w.close();
    assert!(other.is_visible());
    assert_eq!(stable, image_bytes(&other, &op));
    println!(
        "PASS TH-02 synthetic UI: overlay allocation/unchanged Fit-Fill-Actual pixels, zoom/pan preservation, horizontal scroll, icons, selected class/opacity, narrow resize, pick-through when closed, two-window close/isolation, zero additional decoder jobs. scale_factor={}",
        op.scale_factor()
    );
    if std::env::args().any(|s| s == "--manual") {
        other_button.set_active(true);
        println!(
            "MANUAL: TEST MAIN IMAGE + TEST PLACEHOLDERS only. Inspect background alpha, opaque cells, selected border, bottom-center toggle and scroll. No provider/generation/selection binding/T/persistence."
        );
        let (manual, _manual_view, _manual_picture) = open(&app);
        let loop_ = glib::MainLoop::new(None, false);
        let remaining = std::rc::Rc::new(std::cell::Cell::new(2usize));
        for window in [&other, &manual] {
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
    } else {
        other.close();
    }
}
