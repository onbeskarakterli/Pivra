//! Production paintable/GSK output; explicit synthetic fixture, never decode PASS.
use super::*;
use pivra::viewing::{Resampling, Transform};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn filtered(w: &adw::ApplicationWindow, filter: Resampling) {
    gio::prelude::ActionGroupExt::activate_action(
        w,
        "image-resampling",
        Some(&filter.id().to_variant()),
    );
}
fn pixels(w: &adw::ApplicationWindow, p: &gtk::Picture) -> Vec<u8> {
    settle();
    let (width, height) = (p.width(), p.height());
    let s = gtk::Snapshot::new();
    gtk::WidgetPaintable::new(Some(p)).snapshot(&s, width as f64, height as f64);
    let n = s.to_node().expect("resampling render node");
    let bounds = n.bounds();
    assert!(
        bounds.x() >= -1.
            && bounds.y() >= -1.
            && bounds.x() + bounds.width() <= width as f32 + 1.
            && bounds.y() + bounds.height() <= height as f32 + 1.,
        "filter changed clip bounds"
    );
    let t = w.renderer().unwrap().render_texture(
        n,
        Some(&gtk::graphene::Rect::new(
            0.,
            0.,
            width as f32,
            height as f32,
        )),
    );
    let mut bytes = vec![0; width as usize * height as usize * 4];
    t.download(&mut bytes, width as usize * 4);
    bytes
}
pub(super) fn synthetic(app: &adw::Application) {
    let w = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(400)
        .default_height(300)
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
    // Exact endpoint colors; Cairo reference below checks encoded-channel bilinear interpolation.
    let bytes = glib::Bytes::from_owned(vec![
        0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
    ]);
    let source: gdk::Texture =
        gdk::MemoryTexture::new(2, 2, gdk::MemoryFormat::R8g8b8a8, &bytes, 8).upcast();
    let state = Rc::new(RefCell::new(Transform::default()));
    let choice = Rc::new(Cell::new(Resampling::Smooth));
    let mode = gio::SimpleAction::new_stateful(
        "image-mode",
        Some(glib::VariantTy::STRING),
        &"actual".to_variant(),
    );
    image_view::install_interaction(
        &p,
        Rc::new(RefCell::new(Some(source.clone()))),
        state.clone(),
        &mode,
        choice.clone(),
    );
    let cases = [0.1_f64, 0.5, 1., 2., 4., 8., 2.5, 32.]
        .into_iter()
        .map(|scale| (scale, None))
        .chain(ViewMode::ALL.into_iter().map(|mode| (1., Some(mode))));
    for (requested, preset) in cases {
        state.borrow_mut().reset();
        let vp = (p.width() as f64, p.height() as f64);
        if preset.is_none() {
            state.borrow_mut().zoom(
                (2., 2.),
                vp,
                ViewMode::Actual,
                (vp.0 / 2., vp.1 / 2.),
                -requested.ln() / 1.2_f64.ln(),
            );
        }
        let selected = preset.unwrap_or(ViewMode::Actual);
        let g = state.borrow_mut().geometry((2., 2.), vp, selected).unwrap();
        let scale = g.width / 2.;
        if preset.is_none() {
            assert!((scale - requested).abs() < 1e-9);
        }
        let mut results = Vec::new();
        for f in Resampling::ALL {
            choice.set(f);
            image_view::present_filtered(
                &p,
                Some(&source),
                preset.map_or("manual", ViewMode::id),
                state.clone(),
                choice.clone(),
            );
            let before = state.borrow_mut().geometry((2., 2.), vp, selected);
            let output = pixels(&w, &p);
            assert_eq!(before, state.borrow_mut().geometry((2., 2.), vp, selected));
            let mut intermediate = 0;
            let mut inside = 0;
            for y in 0..p.height() {
                for x in 0..p.width() {
                    // Exclude anti-aliased *outer clip* pixels; inspect the image interior.
                    let sx = (x as f64 + 0.5 - g.x) / g.width * 2.;
                    let sy = (y as f64 + 0.5 - g.y) / g.height * 2.;
                    if !(0.01..1.99).contains(&sx) || !(0.01..1.99).contains(&sy) {
                        continue;
                    }
                    let i = ((y * p.width() + x) * 4) as usize;
                    if output[i + 3] != 255 {
                        continue;
                    }
                    inside += 1;
                    let value = output[i];
                    assert_eq!(output[i + 1], value);
                    assert_eq!(output[i + 2], value);
                    if f == Resampling::Nearest {
                        assert!(
                            value == 0 || value == 255,
                            "nearest introduced color {value} at {scale}x"
                        );
                        // Skip exact source-pixel boundaries where either tie choice is legal.
                        if (sx - 1.).abs() > 0.02 && (sy - 1.).abs() > 0.02 {
                            let expected = if (sx.floor() as i32 + sy.floor() as i32) % 2 == 0 {
                                0
                            } else {
                                255
                            };
                            assert_eq!(value, expected, "nearest spatial sampling");
                        }
                    } else {
                        if value > 0 && value < 255 {
                            intermediate += 1;
                        }
                        // Explicit bilinear interpolation between source texel centers.
                        let fx = (sx - 0.5).clamp(0., 1.);
                        let fy = (sy - 0.5).clamp(0., 1.);
                        let expected = 255. * (fx * (1. - fy) + (1. - fx) * fy);
                        assert!(
                            (value as f64 - expected).abs() <= 5.,
                            "smooth is not bilinear: {value} expected {expected}, scale {scale}"
                        );
                    }
                }
            }
            if scale >= 2. {
                assert!(inside > 0);
                if f == Resampling::Smooth {
                    assert!(intermediate > 0, "smooth did not interpolate");
                }
            }
            results.push(output);
        }
        if scale >= 2. {
            assert_ne!(
                results[0], results[1],
                "quality selection has no render effect"
            );
        }
        assert!((g.width - 2. * scale).abs() < 1e-9 && (g.height - 2. * scale).abs() < 1e-9);
    }
    // Deterministic downsampling with enough opaque output pixels to inspect.
    let mut checker = Vec::new();
    for y in 0..16 {
        for x in 0..16 {
            let v = if (x + y) % 2 == 0 { 0 } else { 255 };
            checker.extend_from_slice(&[v, v, v, 255]);
        }
    }
    let checker: gdk::Texture = gdk::MemoryTexture::new(
        16,
        16,
        gdk::MemoryFormat::R8g8b8a8,
        &glib::Bytes::from_owned(checker),
        64,
    )
    .upcast();
    state.borrow_mut().reset();
    let vp = (p.width() as f64, p.height() as f64);
    state.borrow_mut().zoom(
        (16., 16.),
        vp,
        ViewMode::Actual,
        (vp.0 / 2., vp.1 / 2.),
        -(0.5_f64).ln() / 1.2_f64.ln(),
    );
    choice.set(Resampling::Nearest);
    image_view::present_filtered(&p, Some(&checker), "manual", state.clone(), choice.clone());
    let nearest = pixels(&w, &p);
    let again = pixels(&w, &p);
    assert_eq!(nearest, again, "downscale is nondeterministic");
    let mut count = 0;
    for px in nearest.chunks_exact(4).filter(|p| p[3] == 255) {
        assert!(px[0] == 0 || px[0] == 255);
        count += 1;
    }
    assert!(count >= 16);
    choice.set(Resampling::Smooth);
    image_view::present_filtered(&p, Some(&checker), "manual", state.clone(), choice.clone());
    let smooth = pixels(&w, &p);
    assert_ne!(nearest, smooth);
    assert!(
        smooth
            .chunks_exact(4)
            .any(|p| p[3] == 255 && p[0] > 0 && p[0] < 255)
    );
    let held = p.paintable().unwrap();
    let weak = held.downgrade();
    for i in 0..1000 {
        choice.set(if i % 2 == 0 {
            Resampling::Nearest
        } else {
            Resampling::Smooth
        });
        image_view::present_filtered(&p, Some(&checker), "manual", state.clone(), choice.clone());
        assert_eq!(p.paintable().as_ref(), Some(&held));
    }
    drop(held);
    state.borrow_mut().reset();
    choice.set(Resampling::Smooth);
    image_view::present_filtered(&p, Some(&checker), "fit", state.clone(), choice.clone());
    assert!(weak.upgrade().is_none(), "filter paintable leaked");
    // Cover a real-sized gradient's Fit/Fill/Actual and pan/resize without resetting geometry.
    let mut gradient = Vec::with_capacity(1600 * 1200 * 4);
    for y in 0..1200 {
        for x in 0..1600 {
            gradient.extend_from_slice(&[
                ((x as f64 + 0.5) / 1600. * 255.) as u8,
                ((y as f64 + 0.5) / 1200. * 255.) as u8,
                83,
                255,
            ]);
        }
    }
    let source: gdk::Texture = gdk::MemoryTexture::new(
        1600,
        1200,
        gdk::MemoryFormat::R8g8b8a8,
        &glib::Bytes::from_owned(gradient),
        1600 * 4,
    )
    .upcast();
    for preset in ViewMode::ALL {
        state.borrow_mut().reset();
        let vp = (p.width() as f64, p.height() as f64);
        let manual = state
            .borrow_mut()
            .pan((1600., 1200.), vp, preset, (-90., -40.));
        let before = state.borrow_mut().geometry((1600., 1200.), vp, preset);
        let mut renders = Vec::new();
        for f in [Resampling::Nearest, Resampling::Smooth, Resampling::Nearest] {
            choice.set(f);
            image_view::present_filtered(
                &p,
                Some(&source),
                if manual { "manual" } else { preset.id() },
                state.clone(),
                choice.clone(),
            );
            let output = pixels(&w, &p);
            let g = before.unwrap();
            for fx in [0.2, 0.5, 0.8] {
                for fy in [0.2, 0.5, 0.8] {
                    let left = g.x.max(0.);
                    let top = g.y.max(0.);
                    let right = (g.x + g.width).min(vp.0);
                    let bottom = (g.y + g.height).min(vp.1);
                    let x = (left + (right - left) * fx).floor() as i32;
                    let y = (top + (bottom - top) * fy).floor() as i32;
                    let i = ((y * p.width() + x) * 4) as usize;
                    let red = (x as f64 + 0.5 - g.x) / g.width * 255.;
                    let green = (y as f64 + 0.5 - g.y) / g.height * 255.;
                    assert!(
                        output[i + 3] == 255
                            && (output[i + 2] as f64 - red).abs() <= 3.
                            && (output[i + 1] as f64 - green).abs() <= 3.,
                        "filter moved rendered zoom/pan geometry"
                    );
                }
            }
            renders.push(output);
            assert_eq!(
                before,
                state.borrow_mut().geometry((1600., 1200.), vp, preset)
            );
        }
        assert_eq!(renders[0], renders[2], "nearest roundtrip moved the image");
    }
    w.close();
    println!(
        "PASS synthetic real-GSK resampling: 2/4/8x, fractional, min/max/downscale, nearest palette/spatial samples, smooth bilinear pixels, distinct outputs, unchanged geometry; NOT TESTED secure decode"
    );
}
pub(super) fn native(w: &adw::ApplicationWindow) {
    let p = picture(w);
    let original = p.paintable().unwrap();
    assert_eq!(
        w.lookup_action("image-resampling")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("smooth")
    );
    zoom_pan_cases::native(w);
    select(w, ViewMode::Fill);
    filtered(w, Resampling::Nearest);
    let a = pixels(w, &p);
    filtered(w, Resampling::Smooth);
    let b = pixels(w, &p);
    if original.intrinsic_width() < p.width() && original.intrinsic_height() < p.height() {
        assert_ne!(a, b, "native upscaled image showed no filtering difference");
    }
    // Preserve a real manual zoom/pan while toggling during an active drag.
    select(w, ViewMode::Actual);
    let list = p.observe_controllers();
    let motion = (0..list.n_items())
        .find_map(|i| {
            list.item(i)
                .unwrap()
                .downcast::<gtk::EventControllerMotion>()
                .ok()
        })
        .unwrap();
    let scroll = (0..list.n_items())
        .find_map(|i| {
            list.item(i)
                .unwrap()
                .downcast::<gtk::EventControllerScroll>()
                .ok()
        })
        .unwrap();
    let drag = (0..list.n_items())
        .find_map(|i| list.item(i).unwrap().downcast::<gtk::GestureDrag>().ok())
        .unwrap();
    motion.emit_by_name::<()>("enter", &[&200_f64, &200_f64]);
    scroll.emit_by_name::<bool>("scroll", &[&0_f64, &(-2_f64.ln() / 1.2_f64.ln())]);
    drag.emit_by_name::<()>("drag-begin", &[&200_f64, &200_f64]);
    drag.emit_by_name::<()>("drag-update", &[&-80_f64, &-60_f64]);
    let before = pixels(w, &p);
    let paintable = p.paintable();
    let blank = Rc::new(Cell::new(false));
    let observed = blank.clone();
    let handler = p.connect_paintable_notify(move |picture| {
        if picture.paintable().is_none() {
            observed.set(true);
        }
    });
    filtered(w, Resampling::Nearest);
    pixels(w, &p);
    filtered(w, Resampling::Smooth);
    let after = pixels(w, &p);
    assert_eq!(before, after, "filter round trip moved/reloaded the image");
    assert_eq!(paintable, p.paintable());
    assert!(!blank.get());
    p.disconnect(handler);
    drag.emit_by_name::<()>("drag-update", &[&-100_f64, &-80_f64]);
    drag.emit_by_name::<()>("drag-end", &[&-100_f64, &-80_f64]);
    select(w, ViewMode::Fit);
    assert_eq!(p.paintable(), Some(original));
}

/// Exact-color PNG is decoded through the production secure loader, not injected.
pub(super) fn native_checker(app: &adw::Application) {
    let dir = tempfile::tempdir().unwrap();
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/resampling-checker.png");
    let original = std::fs::read(fixture).unwrap();
    std::fs::write(dir.path().join("1.png"), &original).unwrap();
    let other = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images/tall.png");
    std::fs::copy(&other, dir.path().join("2.png")).unwrap();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images/broken.png"),
        dir.path().join("3.png"),
    )
    .unwrap();
    std::fs::copy(&other, dir.path().join("4.png")).unwrap();
    let w = open_window(app, &dir.path().join("1.png"));
    filtered(&w, Resampling::Nearest);
    let p = loaded(&w);
    wait(|| w.title().is_some_and(|s| s.ends_with("(4 candidates)")));
    let old = p.paintable().unwrap();
    assert_eq!((old.intrinsic_width(), old.intrinsic_height()), (16, 16));
    let vp = (p.width() as f64, p.height() as f64);
    let list = p.observe_controllers();
    let motion = (0..list.n_items())
        .find_map(|i| {
            list.item(i)
                .unwrap()
                .downcast::<gtk::EventControllerMotion>()
                .ok()
        })
        .unwrap();
    let scroll = (0..list.n_items())
        .find_map(|i| {
            list.item(i)
                .unwrap()
                .downcast::<gtk::EventControllerScroll>()
                .ok()
        })
        .unwrap();
    motion.emit_by_name::<()>("enter", &[&(vp.0 / 2.), &(vp.1 / 2.)]);
    scroll.emit_by_name::<bool>("scroll", &[&0_f64, &(-8_f64.ln() / 1.2_f64.ln())]);
    assert_eq!(state(&w), "manual");
    let paintable = p.paintable();
    filtered(&w, Resampling::Nearest);
    let nearest = pixels(&w, &p);
    let mut opaque = 0;
    for px in nearest.chunks_exact(4).filter(|p| p[3] == 255) {
        assert!(
            px[0] == 0 || px[0] == 255,
            "secure nearest introduced intermediate color"
        );
        assert_eq!(px[0], px[1]);
        assert_eq!(px[1], px[2]);
        opaque += 1;
    }
    assert!(opaque > 100);
    filtered(&w, Resampling::Smooth);
    let smooth = pixels(&w, &p);
    assert_ne!(nearest, smooth);
    assert!(
        smooth
            .chunks_exact(4)
            .any(|p| p[3] == 255 && p[0] > 0 && p[0] < 255),
        "secure smooth did not interpolate"
    );
    assert_eq!(
        p.paintable(),
        paintable,
        "manual filter toggle replaced cached paintable"
    );
    assert_eq!(state(&w), "manual");
    // Preference survives a new image and remains local to this window.
    filtered(&w, Resampling::Nearest);
    gio::prelude::ActionGroupExt::activate_action(&w, "next-media", None);
    loaded(&w);
    assert_eq!(
        w.lookup_action("image-resampling")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("nearest")
    );
    assert_eq!(state(&w), "fit");
    assert_eq!(p.paintable().unwrap().intrinsic_width(), 32);
    gio::prelude::ActionGroupExt::activate_action(&w, "next-media", None);
    wait(|| {
        p.next_sibling()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap()
            .text()
            .starts_with("Unable")
    });
    assert!(p.paintable().is_none());
    assert!(
        !p.next_sibling()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap()
            .text()
            .contains("Loader process exited early")
    );
    gio::prelude::ActionGroupExt::activate_action(&w, "next-media", None);
    loaded(&w);
    assert_eq!(
        w.lookup_action("image-resampling")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("nearest")
    );
    filtered(&w, Resampling::Smooth);
    let decoded = p.paintable().unwrap().downcast::<gdk::Texture>().unwrap();
    filtered(&w, Resampling::Nearest);
    filtered(&w, Resampling::Smooth);
    assert_eq!(p.paintable(), Some(decoded.upcast()));
    let second_dir = tempfile::tempdir().unwrap();
    std::fs::write(second_dir.path().join("only.png"), &original).unwrap();
    let second = open_window(app, &second_dir.path().join("only.png"));
    loaded(&second);
    filtered(&w, Resampling::Nearest);
    assert_eq!(
        second
            .lookup_action("image-resampling")
            .unwrap()
            .state()
            .unwrap()
            .str(),
        Some("smooth")
    );
    filtered(&w, Resampling::Smooth);
    w.close();
    assert!(second.is_visible());
    second.close();
    assert_eq!(std::fs::read(dir.path().join("1.png")).unwrap(), original);
    assert_eq!(
        std::fs::read(dir.path().join("2.png")).unwrap(),
        std::fs::read(other).unwrap()
    );
    println!(
        "PASS secure PNG checker nearest palette, smooth interpolation, cached paintable/texture, new-image preference, two-window isolation and source integrity"
    );
}
