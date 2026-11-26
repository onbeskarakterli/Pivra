//! Existing strip unit tests; module path and assertions are unchanged.
use super::*;
use pivra::thumbnail_manager::Permit;
#[test]
fn admission_cancellation_and_permanent_failures_are_not_retryable() {
    assert!(matches!(provider_failure(Error::Busy), Work::Busy));
    assert!(matches!(
        provider_failure(Error::Cancelled),
        Work::Cancelled
    ));
    for error in [
        Error::Ineligible,
        Error::InvalidSource("bad data".into()),
        Error::SourceUnavailable("permission denied".into()),
        Error::GenerationFailed("unknown decoder error".into()),
        Error::ProviderUnavailable,
    ] {
        assert!(matches!(provider_failure(error), Work::Failed(_)));
    }
    assert!(matches!(
        provider_failure(Error::TimedOut),
        Work::TransientFailure(_)
    ));
    assert!(matches!(
        provider_failure(Error::ProviderTemporarilyUnavailable),
        Work::TransientFailure(_)
    ));
}
#[test]
fn automatic_failure_retry_is_bounded_to_one_without_reset() {
    let mut retries = 0;
    assert!(retry_failure_once(&mut retries));
    for _ in 0..1000 {
        assert!(!retry_failure_once(&mut retries));
    }
    assert_eq!(retries, 1);
}
#[test]
fn over_budget_visible_entries_do_not_spin_until_viewport_changes() {
    let mut entries: Vec<_> = (0..65)
        .map(|index| Entry {
            index: Rc::new(Cell::new(index)),
            path: PathBuf::from(index.to_string()),
            widgets: None,
            phase: Phase::Evicted,
            texture: None,
            keys: None,
            cost: 0,
            used: 0,
            failure_retries: 0,
        })
        .collect();
    let last = Cell::new(None);
    reconsider_evicted(&mut entries, &(0..64), &last, None);
    assert!(entries[..64].iter().all(|e| e.phase == Phase::Pending));
    for e in &mut entries[..64] {
        e.phase = Phase::Evicted;
    } // RAM eviction after admission
    reconsider_evicted(&mut entries, &(0..64), &last, None);
    assert!(entries[..64].iter().all(|e| e.phase == Phase::Evicted));
    reconsider_evicted(&mut entries, &(1..65), &last, None);
    assert!(entries[1..].iter().all(|e| e.phase == Phase::Pending));
    assert!(entries[0].phase == Phase::Evicted);
}
#[test]
fn deferred_click_rejects_catalog_replacement_and_index_reuse() {
    let entries = vec![Entry {
        index: Rc::new(Cell::new(2)),
        path: PathBuf::from("/pictures/new.jpg"),
        widgets: None,
        phase: Phase::Pending,
        texture: None,
        keys: None,
        cost: 0,
        used: 0,
        failure_retries: 0,
    }];
    assert!(!fallback_target_valid(
        10,
        11,
        2,
        std::path::Path::new("/pictures/old.jpg"),
        &entries
    ));
    assert!(!fallback_target_valid(
        10,
        10,
        2,
        std::path::Path::new("/pictures/old.jpg"),
        &entries
    ));
    assert!(fallback_target_valid(
        10,
        10,
        2,
        std::path::Path::new("/pictures/new.jpg"),
        &entries
    ));
    assert!(!fallback_target_valid(
        10,
        10,
        3,
        std::path::Path::new("/pictures/new.jpg"),
        &entries
    ));
}
#[test]
fn rgba8_density_and_conservative_upload_charge_fit_both_caps() {
    for scale in 1..=4 {
        for (w, h) in [(512, 512), (4096, 3072), (64, 32), (1, 32768)] {
            let (width, height) = display_size(w, h, scale);
            assert!(width > 0 && height > 0 && width <= w && height <= h);
            assert!(width <= CELL_WIDTH * 2 && height <= CELL_HEIGHT * 2);
            // One texture must fit. The cache is bounded by BOTH count
            // and bytes; large thumbnails require eviction below LIMIT.
            let cost = display_cost(width, height).unwrap();
            assert!(cost <= BUDGET);
            assert!((BUDGET / cost).min(LIMIT) >= 1);
            assert!((BUDGET / cost).min(LIMIT) * cost <= BUDGET);
        }
    }
    assert_eq!(texture_cost(512, 512), Some(8 * 1024 * 1024));
    assert_eq!(display_cost(160, 160), Some(204800));
}
#[test]
fn evicted_offscreen_selection_is_reconsidered_without_paint_retry_loop() {
    let mut entries: Vec<_> = (0..5)
        .map(|index| Entry {
            index: Rc::new(Cell::new(index)),
            path: PathBuf::new(),
            widgets: None,
            phase: Phase::Evicted,
            texture: None,
            keys: None,
            cost: 0,
            used: 0,
            failure_retries: 0,
        })
        .collect();
    let last = Cell::new(None);
    reconsider_evicted(&mut entries, &(0..1), &last, Some(4));
    assert!(entries[0].phase == Phase::Pending && entries[4].phase == Phase::Pending);
    assert!(entries[1..4].iter().all(|e| e.phase == Phase::Evicted));
    entries[4].phase = Phase::Evicted;
    reconsider_evicted(&mut entries, &(0..1), &last, Some(4));
    assert!(entries[4].phase == Phase::Evicted);
}
#[test]
fn generated_memory_texture_owns_pixels_and_never_invents_cache_identity() {
    use pivra::freedesktop_generator::{GeneratedThumbnail, Metrics};
    use pivra::freedesktop_thumbnail::SizeClass;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("source.png");
    std::fs::write(&path, b"controlled source identity only").unwrap();
    let key = FileKey::read(&path).unwrap();
    let t = GeneratedThumbnail {
        source: key.clone(),
        class: SizeClass::Normal,
        width: 4,
        height: 2,
        stride: 16,
        pixels: glib::Bytes::from_owned([20u8, 40, 60, 128].repeat(8)),
        metrics: Metrics {
            decoded_width: 4,
            decoded_height: 2,
            frame_bytes: 32,
            download_bytes: 32,
            output_bytes: 32,
            decode_time: std::time::Duration::ZERO,
            resize_time: std::time::Duration::ZERO,
        },
    };
    let cancel = gio::Cancellable::new();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let Work::Hit(texture, source, cache, _) = memory_work(&t, &cancel, deadline, 1) else {
        panic!("memory pixels");
    };
    assert_eq!(source, key);
    assert!(cache.is_none());
    assert_eq!((texture.width(), texture.height()), (4, 2));
    assert!(!matches!(
        memory_work(&t, &cancel, std::time::Instant::now(), 1),
        Work::Hit(..)
    ));
    std::fs::write(&path, b"changed").unwrap();
    assert!(!matches!(
        memory_work(&t, &cancel, deadline, 1),
        Work::Hit(..)
    ));
    cancel.cancel();
    assert!(!matches!(
        memory_work(&t, &cancel, deadline, 1),
        Work::Hit(..)
    ));
    drop(t);
    let mut downloader = gdk::TextureDownloader::new(&texture);
    downloader.set_format(gdk::MemoryFormat::R8g8b8a8);
    let (bytes, _) = downloader.download_bytes();
    for pixel in bytes.chunks_exact(4) {
        for (actual, expected) in pixel.iter().zip([20u8, 40, 60, 128]) {
            assert!(actual.abs_diff(expected) <= 1);
        }
    }
}
#[test]
fn global_worker_bound_deduplicates_and_releases_paths() {
    let root = tempfile::tempdir().unwrap();
    let a = root.path().join("a");
    let b = root.path().join("b");
    let c = root.path().join("c");
    let first = Permit::take(&a).unwrap();
    assert!(Permit::take(&a).is_none());
    let second = Permit::take(&b).unwrap();
    assert!(Permit::take(&c).is_none());
    drop(first);
    assert!(Permit::take(&c).is_some());
    drop(second);
    assert_eq!(Permit::active_count(), 0);
}
