//! Controlled GTK/GIO cache test by default; --native-thumbnails requires real
//! secure Glycin cache pixels and GNOME generation. Synthetic MAIN image always.
use super::*;
use std::{
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};
fn cache_path(path: &Path) -> PathBuf {
    let mut hash = glib::Checksum::new(glib::ChecksumType::Md5).unwrap();
    hash.update(gio::File::for_path(path).uri().as_bytes());
    glib::user_cache_dir()
        .join("thumbnails/normal")
        .join(format!("{}.png", hash.string().unwrap()))
}
fn text_chunk(kind: &str, value: &str, out: &mut Vec<u8>) {
    let data = [kind.as_bytes(), &[0], value.as_bytes()].concat();
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let chunk = [b"tEXt".as_slice(), &data].concat();
    out.extend_from_slice(&chunk);
    let mut crc = !0u32;
    for b in chunk {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320u32.wrapping_mul(crc & 1));
        }
    }
    out.extend_from_slice(&(!crc).to_be_bytes());
}
pub(super) fn cache(path: &Path) {
    let png = include_bytes!("../../tests/fixtures/images/wide.png");
    let mut bytes = png[..33].to_vec();
    text_chunk(
        "Thumb::URI",
        gio::File::for_path(path).uri().as_str(),
        &mut bytes,
    );
    text_chunk(
        "Thumb::MTime",
        &std::fs::metadata(path).unwrap().mtime().to_string(),
        &mut bytes,
    );
    text_chunk(
        "Thumb::Size",
        &std::fs::metadata(path).unwrap().len().to_string(),
        &mut bytes,
    );
    bytes.extend_from_slice(&png[33..]);
    let target = cache_path(path);
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(target, bytes).unwrap();
}
#[track_caller]
fn until(mut predicate: impl FnMut() -> bool) {
    let limit = if std::env::args().any(|s| s == "--native-thumbnails") {
        90
    } else {
        25
    };
    let end = Instant::now() + Duration::from_secs(limit);
    while !predicate() {
        assert!(Instant::now() < end, "TH-05 catalog/decode deadline");
        pump();
    }
}
fn count(row: &gtk::Box) -> usize {
    let mut n = 0;
    let mut c = row.first_child();
    while let Some(w) = c {
        n += 1;
        c = w.next_sibling();
    }
    n
}
fn button(row: &gtk::Box, index: usize) -> gtk::Button {
    let mut c = row.first_child();
    while let Some(w) = c {
        if w.widget_name() == format!("thumbnail-cell-{index}") {
            return w.downcast().unwrap();
        }
        c = w.next_sibling();
    }
    panic!("missing thumbnail {index}");
}

fn real_pixels(row: &gtk::Box, index: usize, dimensions: Option<(i32, i32)>, check_red: bool) {
    let content = button(row, index).child().unwrap();
    let mut child = content.first_child();
    let picture = loop {
        let w = child.expect("thumbnail Picture");
        if let Ok(p) = w.clone().downcast::<gtk::Picture>() {
            break p;
        }
        child = w.next_sibling();
    };
    let texture = picture
        .paintable()
        .unwrap()
        .downcast::<gdk::Texture>()
        .unwrap();
    if let Some(size) = dimensions {
        assert_eq!((texture.width(), texture.height()), size);
    } else {
        // The system thumbnailer may upscale/downscale. Verify the new portrait
        // content, not an assumed backend output size (old cached image was wide).
        assert!(
            texture.width() > 0
                && texture.height() == texture.width() * 2
                && texture.height() <= 1024
        );
    }
    println!(
        "PASS secure cache pixels index={index}: {}x{}",
        texture.width(),
        texture.height()
    );
    if check_red {
        let stride = texture.width() as usize * 4;
        let mut bytes = vec![0; stride * texture.height() as usize];
        texture.download(&mut bytes, stride);
        let at = texture.height() as usize / 4 * stride + texture.width() as usize / 4 * 4;
        assert_eq!(
            &bytes[at..at + 4],
            &[0, 0, 255, 255],
            "real cached PNG red quadrant"
        );
    }
}
// Exercises production ImageView/overlay key admission without any decoder.
// TEST cells and main pixels are synthetic; this is not generation evidence.
pub(super) fn overlay_shortcut(app: &adw::Application) {
    pivra::settings::thumbnail_preference().set_enabled(false);
    let (first, view, picture) = open(app);
    let (other, other_view, _) = open(app);
    let strip: gtk::ScrolledWindow = find(view.widget().upcast_ref(), "thumbnail-strip");
    let other_strip: gtk::ScrolledWindow =
        find(other_view.widget().upcast_ref(), "thumbnail-strip");
    assert!(!strip.is_visible() && !other_strip.is_visible());
    let before = image_bytes(&first, &picture);
    let size = (picture.width(), picture.height());
    assert!(
        key(&first, gdk::ModifierType::empty()),
        "Freedesktop T key was rejected"
    );
    until(|| strip.is_mapped());
    assert!(pivra::settings::thumbnail_preference().enabled());
    assert!(!other_strip.is_visible(), "reveal must remain window-local");
    assert_eq!((picture.width(), picture.height()), size);
    assert_eq!(
        image_bytes(&first, &picture),
        before,
        "T changed main pixels"
    );
    assert!(key(&first, gdk::ModifierType::empty()));
    assert!(!strip.is_visible() && !other_strip.is_visible());
    assert!(!pivra::settings::thumbnail_preference().enabled());
    assert!(key(&other, gdk::ModifierType::empty()));
    until(|| other_strip.is_mapped());
    // Thumbnail preference is shared between windows. Re-enabling it from the
    // second window may reveal the first strip if its pointer is still near
    // the bottom; do not mistake that valid behavior for a routing failure.
    assert!(first.is_mapped() && other.is_mapped());
    assert!(other_strip.is_visible());
    assert!(key(&other, gdk::ModifierType::empty()));
    first.close();
    other.close();
    println!(
        "PASS Freedesktop T opens mapped strip, toggles off, independent window reveal; TEST pixels only, no generation claim"
    );
}

pub(super) fn run(app: &adw::Application) {
    let native = std::env::args().any(|s| s == "--native-thumbnails");
    println!("STAGE catalog: automated checks; windows are test-driven until MANUAL instructions");
    if native {
        println!("STAGE secure decoder prerequisite (one attempt)");
        // One secure prerequisite; failure is NOT RUN, not 120 failed retries.
        let gate = gio::spawn_blocking(|| {
            glib::MainContext::new().block_on(native_image_loader::load(
                Path::new("tests/fixtures/images/wide.png"),
                &gio::Cancellable::new(),
            ))
        });
        match glib::MainContext::default().block_on(gate) {
            Ok(Ok(_)) => {}
            error => {
                eprintln!(
                    "NOT RUN secure thumbnail prerequisite: {}",
                    match error {
                        Ok(Err(e)) => e,
                        _ => "worker failure".into(),
                    }
                );
                std::process::exit(77);
            }
        }
    }
    let root = tempfile::tempdir().unwrap();
    let photos: Vec<_> = (0..120)
        .map(|index| {
            let path = root.path().join(format!(
                "photo{index}.{}",
                if index % 2 == 0 { "png" } else { "jpg" }
            ));
            std::fs::write(
                &path,
                if index % 2 == 0 {
                    include_bytes!("../../tests/fixtures/images/wide.png").as_slice()
                } else {
                    include_bytes!("../../tests/fixtures/images/no-exif.jpg").as_slice()
                },
            )
            .unwrap();
            (index, path)
        })
        .collect();
    for (_, path) in &photos {
        cache(path);
    }
    // Initially hidden, then first measured reveal: first/middle/last selections.
    for selected in [0, 79, 119] {
        // Enabled is a shared persisted preference. A preceding case enables it;
        // reset explicitly before opening the next window so hidden-start
        // assertions do not depend on iteration order or prior user settings.
        pivra::settings::thumbnail_preference().set_enabled(false);
        let calls_before = THUMB_LOADS.load(Ordering::SeqCst);
        let attempts_before = THUMB_ATTEMPTS.lock().unwrap().len();
        let mut order_before = THUMB_ORDER.lock().unwrap().len();
        let (window, view, picture) = open(app);
        if selected == 79 {
            resize_until(&window, &picture, 320, 300, || picture.width() < 400);
            // Deliberately present the initial singleton before the completed
            // scan, so an early index0 alignment cannot consume index79's anchor.
            view.set_thumbnail_catalog(&[(0, photos[79].1.clone())], Some(0));
            let hidden: gtk::Box = find(view.widget().upcast_ref(), "thumbnail-items");
            pause(25);
            assert_eq!(count(&hidden), 0);
            assert_eq!(THUMB_LOADS.load(Ordering::SeqCst), calls_before);
            set_enabled(&view, true);
            until(|| view.thumbnail_stats().completed == 1 && !view.thumbnail_stats().running);
            image_bytes(&window, &picture);
            order_before = THUMB_ORDER.lock().unwrap().len();
        }
        view.set_thumbnail_catalog(&photos, Some(selected));
        view.finish_initial_thumbnail_catalog(Some(selected));
        if selected != 79 {
            let hidden_row: gtk::Box = find(view.widget().upcast_ref(), "thumbnail-items");
            pause(25);
            assert_eq!(count(&hidden_row), 0, "hidden strip constructed cells");
            // Hidden prewarm is intentional: allow only the initial viewport
            // batch, while keeping the strip unconstructed until reveal.
            // The full catalog below still asserts exactly one decode per
            // source (apart from explicitly accounted cancellations).
            assert!(
                THUMB_LOADS.load(Ordering::SeqCst) - calls_before <= 24,
                "hidden prewarm exceeded its bounded viewport batch"
            );
        }
        set_enabled(&view, true);
        let row: gtk::Box = find(view.widget().upcast_ref(), "thumbnail-items");
        let strip: gtk::ScrolledWindow = find(view.widget().upcast_ref(), "thumbnail-strip");
        until(|| count(&row) == 120);
        image_bytes(&window, &picture);
        until(|| {
            let b = button(&row, selected).compute_bounds(&row).unwrap();
            let adj = strip.hadjustment();
            f64::from(b.x()) + 1. >= adj.value()
                && f64::from(b.x() + b.width()) <= adj.value() + adj.page_size() + 1.
        });
        assert!(strip.hadjustment().page_size() <= f64::from(picture.width()));
        println!(
            "PASS first measured reveal: selected={selected}/119 page={} scroll={}",
            strip.hadjustment().page_size(),
            strip.hadjustment().value()
        );
        if selected != 79 {
            view.clear_thumbnail_select();
            until(|| !view.thumbnail_stats().running);
            window.close();
            continue;
        }
        if std::env::args().any(|s| s == "--catalog-cancel-race") {
            assert!(
                !native,
                "controlled timing must not impersonate native decoder evidence"
            );
            // Interrupt an admitted background decode twice, then restore the
            // original viewport. Workers deliberately ignore cancellation until return.
            until(|| THUMB_LOADS.load(Ordering::SeqCst) >= calls_before + 10);
            let original = strip.hadjustment().value();
            let before = THUMB_LOADS.load(Ordering::SeqCst);
            strip.hadjustment().set_value(strip.hadjustment().upper());
            until(|| THUMB_LOADS.load(Ordering::SeqCst) > before);
            let before_second = THUMB_LOADS.load(Ordering::SeqCst);
            let middle = button(&row, 40).compute_bounds(&row).unwrap();
            strip.hadjustment().set_value(f64::from(middle.x()));
            until(|| THUMB_LOADS.load(Ordering::SeqCst) > before_second);
            strip.hadjustment().set_value(original);
        }
        until(|| view.thumbnail_stats().completed == 120 && !view.thumbnail_stats().running);
        let cancelled = audit_attempts(&photos, attempts_before);
        if std::env::args().any(|s| s == "--catalog-cancel-race") {
            assert_eq!(
                cancelled, 2,
                "controlled case must exercise both priority cancellations"
            );
        }
        let stats = view.thumbnail_stats();
        assert_eq!(
            THUMB_LOADS.load(Ordering::SeqCst) - calls_before,
            120 + cancelled,
            "each extra decode must be accounted for by a cancelled attempt"
        );
        // Use actual GTK allocations: thumbnail width varies with strip height.
        let viewport_start = strip.hadjustment().value();
        let first_visible = (0..120)
            .find(|&index| {
                let bounds = button(&row, index).compute_bounds(&row).unwrap();
                f64::from(bounds.x() + bounds.width()) > viewport_start
            })
            .expect("at least one thumbnail intersects the viewport");
        assert_eq!(
            THUMB_ORDER.lock().unwrap()[order_before],
            cache_path(&photos[first_visible].1),
            "visible entries must precede background"
        );
        assert_eq!(stats.failed, 0, "cache thumbnails failed: {stats:?}");
        assert!(stats.textures <= 64 && stats.bytes <= 16 * 1024 * 1024);
        assert_eq!(stats.total, 120);
        if native {
            assert!(strip.is_mapped(), "native pixels require a mapped strip");
            real_pixels(&row, 79, Some((64, 32)), true);
        }
        let initial_offset = strip.hadjustment().value();
        resize_until(&window, &picture, 600, 500, || picture.width() > 500);
        point(&view, true);
        assert!(
            (strip.hadjustment().value() - initial_offset).abs() < 1.,
            "resize reran initial alignment"
        );
        let first = row.first_child().unwrap();
        let selected_view = view.clone();
        let selection_photos = photos.clone();
        view.on_thumbnail_select(move |index| {
            selected_view.set_thumbnail_catalog(&selection_photos, Some(index))
        });
        let target_bounds = button(&row, 103).compute_bounds(&row).unwrap();
        strip.hadjustment().set_value(f64::from(target_bounds.x()));
        image_bytes(&window, &picture);
        let offset = strip.hadjustment().value();
        button(&row, 103).emit_clicked();
        image_bytes(&window, &picture);
        assert_eq!(row.first_child().unwrap(), first, "selection rebuilt cells");
        assert!(
            (strip.hadjustment().value() - offset).abs() < 1.,
            "thumbnail click reset scroll"
        );
        let cell_overlay: gtk::Overlay = button(&row, 103).child().unwrap().downcast().unwrap();
        let mut child = cell_overlay.first_child();
        let mut selected_frame = None;
        while let Some(widget) = child {
            if widget.has_css_class("pivra-thumbnail-selection") {
                selected_frame = Some(widget);
                break;
            }
            child = widget.next_sibling();
        }
        assert!(selected_frame.expect("thumbnail selection frame").has_css_class("selected"));
        assert!(!button(&row, 103).gets_focus_on_click());
        for n in [104, 105, 106, 105, 104, 103] {
            view.set_thumbnail_catalog(&photos, Some(n));
        }
        image_bytes(&window, &picture);
        assert!((strip.hadjustment().value() - offset).abs() < 1.);
        until(|| !view.thumbnail_stats().running);
        let loads = THUMB_LOADS.load(Ordering::SeqCst);
        for _ in 0..8 {
            view.set_thumbnail_catalog(&photos, Some(103));
        }
        image_bytes(&window, &picture);
        pause(100);
        assert_eq!(
            THUMB_LOADS.load(Ordering::SeqCst),
            loads,
            "stable selection redecoded thumbnails"
        );
        for n in [100, 103, 119] {
            let bounds = button(&row, n).compute_bounds(&row).unwrap();
            assert_eq!(
                (bounds.width() as i32, bounds.height() as i32),
                (image_view::thumbnail_overlay::CELL_WIDTH, image_view::thumbnail_overlay::CELL_HEIGHT),
                "thumbnail cell geometry must match the configured dimensions"
            );
        }
        println!(
            "PASS all120 cache entries processed, stable click/rapid-selection scroll and cells, bounded paintables, no repeated stable decode: {stats:?}; native-cache-decode={native}"
        );
        // Delete one cached image: GNOME generation must repair it in native mode;
        // cache-only environments expose a controlled placeholder, never fake success.
        let generation_path = &photos[103].1;
        std::fs::remove_file(cache_path(generation_path)).unwrap();
        view.invalidate_thumbnails();
        until(|| view.thumbnail_stats().completed == 120 && !view.thumbnail_stats().running);
        if native {
            assert_eq!(
                view.thumbnail_stats().failed,
                0,
                "native GNOME generation failed"
            );
        } else {
            assert_eq!(
                view.thumbnail_stats().failed,
                1,
                "missing cache must be explicit"
            );
        }
        std::fs::write(cache_path(&photos[104].1), b"invalid PNG thumbnail cache").unwrap();
        view.invalidate_thumbnails();
        until(|| view.thumbnail_stats().completed == 120 && !view.thumbnail_stats().running);
        if native {
            assert_eq!(
                view.thumbnail_stats().failed,
                0,
                "corrupt cache regeneration"
            );
        } else {
            assert_eq!(
                view.thumbnail_stats().failed,
                2,
                "corrupt and missing must remain explicit failures"
            );
        }
        // Source identity change must invalidate only affected entries, not all120.
        let before = THUMB_LOADS.load(Ordering::SeqCst);
        let replacement = root.path().join("replacement");
        std::fs::write(
            &replacement,
            include_bytes!("../../tests/fixtures/images/tall.png"),
        )
        .unwrap();
        std::fs::rename(replacement, &photos[102].1).unwrap();
        view.invalidate_thumbnails();
        until(|| view.thumbnail_stats().completed == 120 && !view.thumbnail_stats().running);
        assert!(
            THUMB_LOADS.load(Ordering::SeqCst) - before < 4,
            "source change redecoded entire strip"
        );
        if native {
            assert_eq!(view.thumbnail_stats().failed, 0);
            real_pixels(&row, 102, None, true);
        }
        if !native {
            cache(&photos[106].1);
            CORRUPT_DURING_LOAD.store(true, Ordering::SeqCst);
            view.invalidate_thumbnails();
            until(|| view.thumbnail_stats().completed == 120 && !view.thumbnail_stats().running);
            assert!(
                !CORRUPT_DURING_LOAD.load(Ordering::SeqCst),
                "race path did not decode"
            );
            let count = THUMB_LOADS.load(Ordering::SeqCst);
            pause(180);
            assert_eq!(
                THUMB_LOADS.load(Ordering::SeqCst),
                count,
                "identity race retried without invalidation"
            );
        }
        // A removed source becomes an explicit failure; navigation/cells survive.
        std::fs::remove_file(&photos[101].1).unwrap();
        view.invalidate_thumbnails();
        until(|| view.thumbnail_stats().completed == 120 && !view.thumbnail_stats().running);
        assert!(view.thumbnail_stats().failed >= 1);
        assert_eq!(row.first_child().unwrap(), first);
        // Combined membership + content change must revalidate AFTER reconcile.
        let replacement = root.path().join("combined-save");
        std::fs::write(
            &replacement,
            include_bytes!("../../tests/fixtures/images/tall.png"),
        )
        .unwrap();
        std::fs::rename(replacement, &photos[108].1).unwrap();
        let next: Vec<_> = photos
            .iter()
            .filter(|(n, _)| *n != 101)
            .enumerate()
            .map(|(n, (_, p))| (n, p.clone()))
            .collect();
        view.set_thumbnail_catalog(&next, Some(102));
        view.invalidate_thumbnails();
        until(|| view.thumbnail_stats().completed == 119 && !view.thumbnail_stats().running);
        assert_eq!(row.first_child().unwrap(), first);
        if native {
            real_pixels(&row, 107, None, true);
        } else {
            let changed = button(&row, 107)
                .child()
                .unwrap()
                .first_child()
                .unwrap()
                .downcast::<gtk::Picture>()
                .unwrap();
            assert!(
                changed.paintable().is_none(),
                "reconciled source kept a stale thumbnail"
            );
            assert!(view.thumbnail_stats().failed >= 1);
        }
        // Repeated scroll priority changes and close with admitted work.
        for n in [0., 11000., 2000., 9000., 4000.] {
            strip.hadjustment().set_value(n);
            pump();
        }
        view.clear_thumbnail_select();
        window.close();
        pause(100);
        println!(
            "PASS missing cache/source replace/delete, repeated priority cancellation and close; no native generation claim unless --native-thumbnails"
        );
    }
    // A stable failure must not be admitted again simply because hover reveals
    // the strip. Only a source/catalog invalidation may explicitly retry it.
    let (failed_window, failed_view, _) = open(app);
    failed_view.set_thumbnail_catalog(&[(0, root.path().join("missing-source.png"))], Some(0));
    set_enabled(&failed_view, true);
    until(|| {
        failed_view.thumbnail_stats().completed == 1 && !failed_view.thumbnail_stats().running
    });
    assert_eq!(failed_view.thumbnail_stats().failed, 1);
    let admissions = failed_view.thumbnail_stats().started;
    point(&failed_view, false);
    let failed_strip: gtk::ScrolledWindow =
        find(failed_view.widget().upcast_ref(), "thumbnail-strip");
    until(|| !failed_strip.is_visible());
    point(&failed_view, true);
    until(|| !failed_view.thumbnail_stats().running);
    pause(80);
    assert_eq!(
        failed_view.thumbnail_stats().started,
        admissions,
        "hover retried an unchanged failed source"
    );
    failed_view.clear_thumbnail_select();
    failed_window.close();
    production_windows(app, &photos);
}

fn production_windows(app: &adw::Application, photos: &[(usize, PathBuf)]) {
    let mut windows = Vec::new();
    for selected in [79, 10] {
        let before = app.windows();
        window::build(
            app,
            &pivra::settings::Settings::default(),
            Some(input::OpenedFile::from_argument(photos[selected].1.as_os_str()).unwrap()),
        );
        let window = app
            .windows()
            .into_iter()
            .find(|w| !before.contains(w))
            .unwrap()
            .downcast::<adw::ApplicationWindow>()
            .unwrap();
        let row: gtk::Box = find(window.upcast_ref(), "thumbnail-items");
        let strip: gtk::ScrolledWindow = find(window.upcast_ref(), "thumbnail-strip");
        let overlay = strip.parent().unwrap().downcast::<gtk::Overlay>().unwrap();
        let motion = controller::<gtk::EventControllerMotion>(&overlay);
        until(|| window.is_mapped() && overlay.height() > 0);
        motion.emit_by_name::<()>("motion", &[&20f64, &(overlay.height() as f64 - 4.)]);
        // Cell construction is incremental and pauses while the strip is hidden.
        // Keep the strip exposed until the final catalog cell is built.
        let deadline = Instant::now() + Duration::from_secs(20);
        while !contains(window.upcast_ref(), "thumbnail-cell-118") {
            if !strip.is_mapped() {
                motion.emit_by_name::<()>(
                    "motion",
                    &[&20f64, &(overlay.height() as f64 - 4.)],
                );
            }
            assert!(
                Instant::now() < deadline,
                "production catalog cell deadline: title={:?} row_children={} strip_visible={} strip_mapped={} overlay_height={} row_mapped={} cell_79={} cell_118={}",
                window.title(),
                count(&row),
                strip.is_visible(),
                strip.is_mapped(),
                overlay.height(),
                row.is_mapped(),
                contains(window.upcast_ref(), "thumbnail-cell-79"),
                contains(window.upcast_ref(), "thumbnail-cell-118"),
            );
            pump();
        } //119 entries after deletion
        // Keep the motion controller connected until the strip is mapped and
        // its incremental cells are created; disconnect only for stable probing.
        overlay.remove_controller(&motion);
        let end = Instant::now() + Duration::from_secs(20);
        loop {
            let selected_button = button(&row, selected);
            if let Some(bounds) = selected_button.compute_bounds(&row) {
                let a = strip.hadjustment();
                if strip.is_mapped()
                    && bounds.width() > 0.
                    && f64::from(bounds.x()) + 1. >= a.value()
                    && f64::from(bounds.x() + bounds.width()) <= a.value() + a.page_size() + 1.
                {
                    break;
                }
            }
            assert!(
                Instant::now() < end,
                "production singleton-to-folder initial alignment"
            );
            pump();
        }
        windows.push((window, row, strip, motion, overlay));
    }
    let (first, row, strip, motion, overlay) = &windows[0];
    let other_title = windows[1].0.title();
    let original = row.first_child().unwrap();
    strip.hadjustment().set_value(100. * 104.);
    pause(60);
    let offset = strip.hadjustment().value();
    // Global catalog index104 is photo105 after photo101 was deleted.
    button(row, 104).emit_clicked();
    until(|| {
        first
            .title()
            .is_some_and(|s| s.contains(photos[105].1.file_name().unwrap().to_str().unwrap()))
    });
    assert_eq!(row.first_child().unwrap(), original);
    assert!((strip.hadjustment().value() - offset).abs() < 1.);
    assert_eq!(windows[1].0.title(), other_title);
    motion.emit_by_name::<()>("motion", &[&20f64, &20f64]);
    until(|| !strip.is_visible());
    gio::prelude::ActionGroupExt::activate_action(first, "next-media", None);
    until(|| {
        first
            .title()
            .is_some_and(|s| s.contains(photos[106].1.file_name().unwrap().to_str().unwrap()))
    });
    assert!(
        !strip.is_visible(),
        "Right unexpectedly revealed hidden strip"
    );
    gio::prelude::ActionGroupExt::activate_action(first, "previous-media", None);
    until(|| {
        first
            .title()
            .is_some_and(|s| s.contains(photos[105].1.file_name().unwrap().to_str().unwrap()))
    });
    assert!(!strip.is_visible());
    assert_eq!(windows[1].0.title(), other_title);
    motion.emit_by_name::<()>("motion", &[&20f64, &(overlay.height() as f64 - 4.)]);
    first.close();
    pause(80);
    assert!(windows[1].0.is_visible());
    assert_eq!(windows[1].0.title(), other_title);
    windows[1].0.close();
    println!(
        "PASS production window.rs singleton/async folder scan alignment, real click-to-file title, stable scroll/cells, Left/Right actions without reveal, two-window independence/close (foreground pixels synthetic unless --native-thumbnails)"
    );
    if std::env::args().any(|s| s == "--manual") {
        let old = app.windows();
        for n in [79, 10] {
            window::build(
                app,
                &pivra::settings::Settings::default(),
                Some(input::OpenedFile::from_argument(photos[n].1.as_os_str()).unwrap()),
            );
        }
        let manual: Vec<_> = app
            .windows()
            .into_iter()
            .filter(|w| !old.contains(w))
            .collect();
        println!(
            "MANUAL TH-05: two production windows, PRIVATE TEST JPEG/PNG fixtures. native secure pixels={}. Move to bottom, scroll/click, Left/Right, T, resize. Close both to finish.",
            std::env::args().any(|s| s == "--native-thumbnails")
        );
        let loop_ = glib::MainLoop::new(None, false);
        let remaining = std::rc::Rc::new(std::cell::Cell::new(manual.len()));
        for w in manual {
            let quit = loop_.clone();
            let count = remaining.clone();
            w.connect_close_request(move |_| {
                count.set(count.get() - 1);
                if count.get() == 0 {
                    quit.quit();
                }
                glib::Propagation::Proceed
            });
        }
        loop_.run();
    }
}

// Large cache frames expose the 16 MiB limit long before the 64-entry cap.
// Synthetic decoded pixels only; this does not claim secure decoder evidence.
pub(super) fn retention(app: &adw::Application) {
    let root = tempfile::tempdir().unwrap();
    let mut survivors = Vec::new();
    for total in [81, 120] {
        let photos: Vec<_> = (0..total)
            .map(|index| {
                let path = root.path().join(format!("{total}-{index}.png"));
                std::fs::write(
                    &path,
                    include_bytes!("../../tests/fixtures/images/wide.png"),
                )
                .unwrap();
                cache(&path);
                (index, path)
            })
            .collect();
        let (window, view, picture) = open(app);
        view.set_thumbnail_catalog(&photos, Some(0));
        view.finish_initial_thumbnail_catalog(Some(0));
        image_bytes(&window, &picture);
        set_enabled(&view, true);
        let row: gtk::Box = find(view.widget().upcast_ref(), "thumbnail-items");
        let strip: gtk::ScrolledWindow = find(view.widget().upcast_ref(), "thumbnail-strip");
        let cell_deadline = Instant::now() + Duration::from_secs(5);
        while count(&row) != total {
            assert!(
                Instant::now() < cell_deadline,
                "cells={} total={total} mapped={} visible={} {:?}",
                count(&row),
                strip.is_mapped(),
                strip.is_visible(),
                view.thumbnail_stats()
            );
            pump();
        }
        image_bytes(&window, &picture);
        point(&view, true);
        let deadline = Instant::now() + Duration::from_secs(25);
        while view.thumbnail_stats().completed != total || view.thumbnail_stats().running {
            assert!(
                Instant::now() < deadline,
                "retention deadline: {:?}, mapped={} page={} scroll={}",
                view.thumbnail_stats(),
                strip.is_mapped(),
                strip.hadjustment().page_size(),
                strip.hadjustment().value()
            );
            pump();
        }
        let visible_ready = || {
            let adj = strip.hadjustment();
            (0..total).all(|n| {
                let b = button(&row, n);
                let bounds = b.compute_bounds(&row).unwrap();
                f64::from(bounds.x()) >= adj.value() + adj.page_size()
                    || f64::from(bounds.x() + bounds.width()) <= adj.value()
                    || thumb_picture(&b).paintable().is_some()
            })
        };
        let verify = || {
            let adj = strip.hadjustment();
            let mut visible = 0;
            for n in 0..total {
                let b = button(&row, n);
                let bounds = b.compute_bounds(&row).unwrap();
                if f64::from(bounds.x()) < adj.value() + adj.page_size()
                    && f64::from(bounds.x() + bounds.width()) > adj.value()
                {
                    visible += 1;
                    assert!(
                        thumb_picture(&b).paintable().is_some(),
                        "visible thumbnail {n}/{total} evicted after background load: {:?}",
                        view.thumbnail_stats()
                    );
                }
            }
            assert!(visible > 0);
            assert!(
                thumb_picture(&button(&row, 0)).paintable().is_some(),
                "selected thumbnail evicted"
            );
            let texture = thumb_picture(&button(&row, 0))
                .paintable()
                .unwrap()
                .downcast::<gdk::Texture>()
                .unwrap();
            let mut download = gdk::TextureDownloader::new(&texture);
            download.set_format(gdk::MemoryFormat::R8g8b8a8);
            let (bytes, _) = download.download_bytes();
            assert_eq!(
                &bytes[..4],
                &[180, 80, 40, 255],
                "normalized thumbnail pixels changed"
            );
            let stats = view.thumbnail_stats();
            assert_eq!(stats.failed, 0);
            assert!(stats.textures <= 64 && stats.bytes <= 16 * 1024 * 1024);
            println!("MEASURE total={total} visible={visible} {stats:?}");
        };
        verify();
        let first = thumb_picture(&button(&row, 0)).paintable();
        let started = view.thumbnail_stats().started;
        pause(180);
        assert_eq!(
            view.thumbnail_stats().started,
            started,
            "stable viewport decode loop"
        );
        assert_eq!(thumb_picture(&button(&row, 0)).paintable(), first);
        let evicted = (0..total)
            .find(|&n| thumb_picture(&button(&row, n)).paintable().is_none())
            .expect("capacity bound must evict offscreen cells");
        for target in [total - 1, evicted, 0] {
            let bounds = button(&row, target).compute_bounds(&row).unwrap();
            strip.hadjustment().set_value(f64::from(bounds.x()));
            until(|| visible_ready() && !view.thumbnail_stats().running);
            verify();
            let admissions = view.thumbnail_stats().started;
            pause(100);
            assert_eq!(
                view.thumbnail_stats().started,
                admissions,
                "scroll settled into decode loop"
            );
        }
        assert!(
            view.thumbnail_stats().started > total as u64,
            "scroll did not rehydrate evicted cells"
        );
        resize_until(&window, &picture, 320, 320, || picture.width() < 400);
        point(&view, true);
        pause(100);
        verify();
        set_enabled(&view, false);
        set_enabled(&view, true);
        until(|| !view.thumbnail_stats().running);
        verify();
        // Keep the first independent texture set alive while testing the second.
        survivors.push((window, view));
    }
    for (w, v) in survivors {
        v.clear_thumbnail_select();
        w.close();
    }
    pause(100);
    println!(
        "PASS 81/120 large-cache retention, selected and visible pixels, bounded RAM/count, scroll-return, stable decode, resize/hide and independent windows; synthetic pixels only"
    );
}
fn thumb_picture(button: &gtk::Button) -> gtk::Picture {
    let content = button.child().unwrap();
    let mut child = content.first_child();
    while let Some(w) = child {
        if let Ok(p) = w.clone().downcast::<gtk::Picture>() {
            return p;
        }
        child = w.next_sibling();
    }
    panic!("missing thumbnail picture");
}

fn audit_attempts(photos: &[(usize, PathBuf)], start: usize) -> usize {
    let attempts = THUMB_ATTEMPTS.lock().unwrap();
    let slice = &attempts[start..];
    for (n, path) in photos {
        let cached = cache_path(path);
        let matching: Vec<_> = slice
            .iter()
            .enumerate()
            .filter(|(_, a)| a.path == cached)
            .collect();
        if matching.is_empty() {
            println!(
                "DECODE missing source={} index={n} cache={}",
                path.display(),
                cached.display()
            );
        }
        if matching.len() != 1 {
            for (order, a) in &matching {
                println!(
                    "DECODE source={} index={n} cache={} attempt={} completed={:?} cancelled={}",
                    path.display(),
                    cached.display(),
                    start + order,
                    a.completed,
                    a.token.is_cancelled()
                );
            }
        }
    }
    let expected: Vec<_> = photos.iter().map(|(_, path)| cache_path(path)).collect();
    let cancelled = validate_attempts(
        &expected,
        slice
            .iter()
            .map(|a| (a.path.as_path(), a.token.is_cancelled(), a.completed)),
    )
    .unwrap_or_else(|error| panic!("per-file decode audit failed: {error}"));
    println!(
        "DECODE audit total={} cancelled={} unique-successes={}",
        slice.len(),
        cancelled,
        expected.len()
    );
    cancelled
}

// An aborted worker may have decoded pixels, but its result is not published.
// Accept exactly one uncancelled successful decode per expected cache file;
// never excuse an extra successful decode merely because totals are close.
fn validate_attempts<'a>(
    expected: &[PathBuf],
    attempts: impl Iterator<Item = (&'a Path, bool, Option<bool>)>,
) -> Result<usize, String> {
    let mut successes: std::collections::HashMap<_, usize> =
        expected.iter().map(|p| (p.as_path(), 0)).collect();
    let mut cancelled_count = 0;
    for (path, cancelled, completed) in attempts {
        let count = successes
            .get_mut(path)
            .ok_or_else(|| format!("unknown cache input {}", path.display()))?;
        let completed =
            completed.ok_or_else(|| format!("unfinished attempt {}", path.display()))?;
        if cancelled {
            cancelled_count += 1;
        } else if completed {
            *count += 1;
        } else {
            return Err(format!("uncancelled decoder failure {}", path.display()));
        }
    }
    for (path, count) in successes {
        if count != 1 {
            return Err(format!(
                "{} has {count} uncancelled successful decodes, expected 1",
                path.display()
            ));
        }
    }
    Ok(cancelled_count)
}

#[cfg(test)]
mod attempt_tests {
    use super::*;
    #[test]
    fn only_cancelled_extras_are_allowed() {
        let paths = [PathBuf::from("a"), PathBuf::from("b")];
        assert_eq!(
            validate_attempts(
                &paths,
                [
                    (paths[0].as_path(), true, Some(false)),
                    (paths[1].as_path(), true, Some(true)),
                    (paths[0].as_path(), false, Some(true)),
                    (paths[1].as_path(), false, Some(true)),
                ]
                .into_iter()
            ),
            Ok(2)
        );
    }
    #[test]
    fn duplicate_success_missing_failure_unknown_and_unfinished_are_rejected() {
        let paths = [PathBuf::from("a")];
        let a = paths[0].as_path();
        assert!(validate_attempts(&paths, [(a, false, Some(true)); 2].into_iter()).is_err());
        assert!(validate_attempts(&paths, [(a, true, Some(true))].into_iter()).is_err());
        assert!(validate_attempts(&paths, [(a, false, Some(false))].into_iter()).is_err());
        assert!(validate_attempts(&paths, [(a, true, None)].into_iter()).is_err());
        assert!(
            validate_attempts(
                &paths,
                [(Path::new("other"), false, Some(true))].into_iter()
            )
            .is_err()
        );
    }
}
