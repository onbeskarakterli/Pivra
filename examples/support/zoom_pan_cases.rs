//! Real GTK controllers + production paintable, explicitly synthetic textures.
use super::*;
use pivra::viewing::Transform;
use std::{cell::RefCell, rc::Rc};
fn controller<T: glib::object::IsA<glib::Object> + glib::types::StaticType>(p: &gtk::Picture) -> T {
    let list = p.observe_controllers();
    (0..list.n_items())
        .find_map(|i| list.item(i).unwrap().downcast::<T>().ok())
        .expect("photo controller")
}
fn wheel(p: &gtk::Picture, x: f64, y: f64, dy: f64) {
    controller::<gtk::EventControllerMotion>(p).emit_by_name::<()>("enter", &[&x, &y]);
    assert!(
        controller::<gtk::EventControllerScroll>(p).emit_by_name::<bool>("scroll", &[&0_f64, &dy])
    );
}
fn drag(p: &gtk::Picture, x: f64, y: f64) {
    let d = controller::<gtk::GestureDrag>(p);
    d.emit_by_name::<()>("drag-begin", &[&100_f64, &100_f64]);
    d.emit_by_name::<()>("drag-update", &[&x, &y]);
    d.emit_by_name::<()>("drag-end", &[&x, &y]);
}
fn snapshot(w: &adw::ApplicationWindow, p: &gtk::Picture, g: pivra::viewing::Geometry) {
    let node = rendered_node(w, p);
    let (width, height) = (p.width(), p.height());
    let bounds = node.bounds();
    assert!(
        bounds.x() >= -1.
            && bounds.y() >= -1.
            && bounds.x() + bounds.width() <= width as f32 + 1.
            && bounds.y() + bounds.height() <= height as f32 + 1.
    );
    let t = w.renderer().unwrap().render_texture(
        node,
        Some(&gtk::graphene::Rect::new(
            0.,
            0.,
            width as f32,
            height as f32,
        )),
    );
    let mut pixels = vec![0; width as usize * height as usize * 4];
    t.download(&mut pixels, width as usize * 4);
    let (l, top, r, b) = (
        g.x.max(0.),
        g.y.max(0.),
        (g.x + g.width).min(width as f64),
        (g.y + g.height).min(height as f64),
    );
    for fx in [0.2, 0.5, 0.8] {
        for fy in [0.2, 0.5, 0.8] {
            let x = (l + (r - l) * fx).floor() as i32;
            let y = (top + (b - top) * fy).floor() as i32;
            let i = ((y * width + x) * 4) as usize;
            let red = ((x as f64 + 0.5 - g.x) / g.width * 255.).clamp(0., 255.);
            let green = ((y as f64 + 0.5 - g.y) / g.height * 255.).clamp(0., 255.);
            assert!(
                pixels[i + 3] > 127
                    && (pixels[i + 2] as f64 - red).abs() <= 6.
                    && (pixels[i + 1] as f64 - green).abs() <= 6.,
                "zoom/pan snapshot pixel mismatch {g:?} at {x},{y}"
            );
        }
    }
}
pub(super) fn synthetic(app: &adw::Application) {
    for size in [(1600, 1200), (1200, 1600), (32, 64)] {
        let w = adw::ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .build();
        let p = gtk::Picture::builder()
            .can_shrink(true)
            .hexpand(true)
            .vexpand(true)
            .build();
        w.set_content(Some(&p));
        w.present();
        wait(|| w.is_mapped());
        settle();
        let mut bytes = Vec::new();
        for y in 0..size.1 {
            for x in 0..size.0 {
                bytes.extend_from_slice(&[
                    ((x as f64 + 0.5) / size.0 as f64 * 255.) as u8,
                    ((y as f64 + 0.5) / size.1 as f64 * 255.) as u8,
                    83,
                    255,
                ]);
            }
        }
        let tex: gdk::Texture = gdk::MemoryTexture::new(
            size.0,
            size.1,
            gdk::MemoryFormat::R8g8b8a8,
            &glib::Bytes::from_owned(bytes),
            size.0 as usize * 4,
        )
        .upcast();
        let source = Rc::new(RefCell::new(Some(tex.clone())));
        let state = Rc::new(RefCell::new(Transform::default()));
        let action = gio::SimpleAction::new_stateful(
            "image-mode",
            Some(glib::VariantTy::STRING),
            &"actual".to_variant(),
        );
        image_view::install_interaction(
            &p,
            source.clone(),
            state.clone(),
            &action,
            Rc::new(std::cell::Cell::new(pivra::viewing::Resampling::Smooth)),
            Rc::new(std::cell::Cell::new(true)),
        );
        let survivor = adw::ApplicationWindow::builder()
            .application(app)
            .default_width(480)
            .default_height(320)
            .build();
        let other = gtk::Picture::builder()
            .can_shrink(true)
            .hexpand(true)
            .vexpand(true)
            .build();
        survivor.set_content(Some(&other));
        survivor.present();
        wait(|| survivor.is_mapped());
        settle();
        let other_state = Rc::new(RefCell::new(Transform::default()));
        let other_action = gio::SimpleAction::new_stateful(
            "image-mode",
            Some(glib::VariantTy::STRING),
            &"actual".to_variant(),
        );
        image_view::install_interaction(
            &other,
            Rc::new(RefCell::new(Some(tex.clone()))),
            other_state.clone(),
            &other_action,
            Rc::new(std::cell::Cell::new(pivra::viewing::Resampling::Smooth)),
            Rc::new(std::cell::Cell::new(true)),
        );
        image_view::present(&other, Some(&tex), ViewMode::Actual);
        let other_paintable = other.paintable();
        for mode in ViewMode::ALL {
            state.borrow_mut().reset();
            action.set_state(&mode.id().to_variant());
            image_view::present(&p, Some(&tex), mode);
            let vp = (p.width() as f64, p.height() as f64);
            let image = (size.0 as f64, size.1 as f64);
            let point = (vp.0 * 0.4, vp.1 * 0.4);
            let before = state.borrow_mut().geometry(image, vp, mode).unwrap();
            wheel(&p, point.0, point.1, -2.);
            let after = state.borrow_mut().geometry(image, vp, mode).unwrap();
            if before.width >= vp.0 && before.height >= vp.1 {
                assert!(
                    ((point.0 - before.x) / before.width - (point.0 - after.x) / after.width).abs()
                        < 1e-9
                );
                assert!(
                    ((point.1 - before.y) / before.height - (point.1 - after.y) / after.height)
                        .abs()
                        < 1e-9
                );
            }
            let paintable = p.paintable();
            let g = state.borrow_mut().geometry(image, vp, mode).unwrap();
            snapshot(&w, &p, g);
            wheel(&p, point.0, point.1, 1.);
            assert_eq!(p.paintable(), paintable, "input replaced cached paintable");
            let burst = controller::<gtk::GestureDrag>(&p);
            burst.emit_by_name::<()>("drag-begin", &[&0_f64, &0_f64]);
            for i in 0..1000 {
                let x = if i % 2 == 0 { -500_f64 } else { 500_f64 };
                let y = (i % 17) as f64 * 30.;
                burst.emit_by_name::<()>("drag-update", &[&x, &y]);
            }
            burst.emit_by_name::<()>("drag-end", &[&500_f64, &0_f64]);
            for (x, y) in [(1e5, 1e5), (-1e5, 1e5), (1e5, -1e5), (-1e5, -1e5)] {
                drag(&p, x, y);
                let g = state.borrow_mut().geometry(image, vp, mode).unwrap();
                snapshot(&w, &p, g);
            }
            for _ in 0..100 {
                wheel(&p, point.0, point.1, -10.);
            }
            let g = state.borrow_mut().geometry(image, vp, mode).unwrap();
            assert!((g.width / image.0 - 32.).abs() < 1e-9);
            snapshot(&w, &p, g);
            for _ in 0..100 {
                wheel(&p, point.0, point.1, 10.);
            }
            let g = state.borrow_mut().geometry(image, vp, mode).unwrap();
            assert!((g.width / image.0 - 0.1).abs() < 1e-9);
            snapshot(&w, &p, g);
            // An old gesture may not continue after a mode/load reset.
            let d = controller::<gtk::GestureDrag>(&p);
            d.emit_by_name::<()>("drag-begin", &[&0_f64, &0_f64]);
            state.borrow_mut().reset();
            image_view::present(&p, Some(&tex), mode);
            d.emit_by_name::<()>("drag-update", &[&500_f64, &500_f64]);
            assert_eq!(
                state.borrow_mut().geometry(image, vp, mode),
                mode.geometry(image, vp)
            );
            d.emit_by_name::<()>("drag-end", &[&500_f64, &500_f64]);
            assert_eq!(source.borrow().as_ref(), Some(&tex));
            assert_eq!(other.paintable(), other_paintable);
            assert_eq!(
                other_state.borrow_mut().geometry(
                    image,
                    (other.width() as f64, other.height() as f64),
                    ViewMode::Actual
                ),
                ViewMode::Actual.geometry(image, (other.width() as f64, other.height() as f64))
            );
            // Resize manual %100 geometry, then reset and verify fitting axes center.
            state.borrow_mut().reset();
            action.set_state(&"actual".to_variant());
            image_view::present(&p, Some(&tex), ViewMode::Actual);
            drag(&p, -100., -80.);
            w.set_default_size(480, 320);
            settle();
            let resized = (p.width() as f64, p.height() as f64);
            let g = state
                .borrow_mut()
                .geometry(image, resized, ViewMode::Actual)
                .unwrap();
            snapshot(&w, &p, g);
            w.set_default_size(800, 600);
            settle();
        }
        // Explicit release/cancellation ignores late updates; close during a drag.
        let d = controller::<gtk::GestureDrag>(&p);
        d.emit_by_name::<()>("drag-begin", &[&0_f64, &0_f64]);
        d.emit_by_name::<()>("cancel", &[&None::<gdk::EventSequence>]);
        let before = state.borrow_mut().geometry(
            (size.0 as f64, size.1 as f64),
            (p.width() as f64, p.height() as f64),
            ViewMode::Actual,
        );
        d.emit_by_name::<()>("drag-update", &[&300_f64, &300_f64]);
        assert_eq!(
            before,
            state.borrow_mut().geometry(
                (size.0 as f64, size.1 as f64),
                (p.width() as f64, p.height() as f64),
                ViewMode::Actual
            )
        );
        d.emit_by_name::<()>("drag-begin", &[&0_f64, &0_f64]);
        w.close();
        wheel(&other, 100., 100., -1.);
        assert_eq!(other_action.state().unwrap().str(), Some("manual"));
        survivor.close();
    }
    println!(
        "PASS synthetic production scroll/drag controllers, cached paintable, gradient snapshots, limits, four edges/corners, reset and stale gesture rejection; NOT TESTED secure decode or physical mouse"
    );
}
pub(super) fn native(w: &adw::ApplicationWindow) {
    let p = picture(w);
    let original = p.paintable().unwrap();
    select(w, ViewMode::Actual);
    drag(&p, -200., -100.);
    let vp = (p.width() as f64, p.height() as f64);
    wheel(&p, vp.0 * 0.4, vp.1 * 0.4, -2.);
    assert_eq!(state(w), "manual");
    drag(&p, -200., -100.);
    settle();
    assert!(p.paintable().is_some());
    wheel(&p, vp.0 * 0.4, vp.1 * 0.4, 1.);
    select(w, ViewMode::Fit);
    assert_eq!(p.paintable(), Some(original));
    assert_eq!(state(w), "fit");
}
