//! Slideshow timing and bounded catalog traversal. No GTK clocks or decoder.
use crate::{catalog::MediaCollection, media::MediaKind};
use std::{collections::HashSet, path::PathBuf, time::Duration};
pub const DEFAULT_SECONDS: u32 = 5;
pub const INTERVALS: [u32; 5] = [3, 5, 10, 15, 30];
#[derive(Debug)]
pub struct Clock {
    pub paused: bool,
    interval: Duration,
    remaining: Duration,
    deadline: Option<Duration>,
    ready: bool,
}
impl Clock {
    pub fn new(seconds: u32) -> Self {
        let interval = Duration::from_secs(u64::from(seconds.clamp(1, 3600)));
        Self {
            paused: false,
            interval,
            remaining: interval,
            deadline: None,
            ready: false,
        }
    }
    pub fn reset(&mut self) {
        self.deadline = None;
        self.remaining = self.interval;
        self.ready = false;
    }
    pub fn photo_ready(&mut self, now: Duration) {
        if !self.ready {
            self.ready = true;
            if !self.paused {
                self.deadline = Some(now + self.remaining);
            }
        }
    }
    pub fn toggle_pause(&mut self, now: Duration) {
        self.paused = !self.paused;
        if self.paused {
            if let Some(deadline) = self.deadline.take() {
                self.remaining = deadline.saturating_sub(now);
            }
        } else if self.ready {
            self.deadline = Some(now + self.remaining);
        }
    }
    pub fn due(&mut self, now: Duration) -> bool {
        if !self.paused && self.deadline.is_some_and(|d| now >= d) {
            self.deadline = None;
            true
        } else {
            false
        }
    }
    pub fn video_completed(&self, ended: bool) -> bool {
        !self.paused && ended
    }
    pub fn set_interval(&mut self, seconds: u32, now: Duration) {
        let paused = self.paused;
        let ready = self.ready;
        *self = Self::new(seconds);
        self.paused = paused;
        if ready {
            self.photo_ready(now);
        }
    }
}
/// Exactly one bounded pass, including wrap and single-item replay. Failed paths
/// remain in the catalog; only this slideshow session skips them.
pub fn target(
    collection: &MediaCollection,
    forward: bool,
    failed: &HashSet<PathBuf>,
) -> Option<usize> {
    let n = collection.items().len();
    if n == 0 {
        return None;
    }
    let start = collection
        .selected_index()
        .unwrap_or(if forward { n - 1 } else { 0 });
    (1..=n)
        .map(|step| {
            if forward {
                (start + step) % n
            } else {
                (start + n - step) % n
            }
        })
        .find(|&i| {
            let item = &collection.items()[i];
            !failed.contains(item.path())
                && (item.kind() == MediaKind::Video
                    || crate::discovery::supported_photo(item.path()))
        })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::MediaItem;
    fn at(s: u64) -> Duration {
        Duration::from_secs(s)
    }
    #[test]
    fn loading_has_no_timer_and_ready_starts_full_interval() {
        let mut c = Clock::new(5);
        assert!(!c.due(at(99)));
        c.photo_ready(at(100));
        c.photo_ready(at(103));
        assert!(!c.due(at(104)));
        assert!(c.due(at(105)));
        assert!(!c.due(at(106)));
    }
    #[test]
    fn pause_freezes_remaining_and_resume_does_not_restart_interval() {
        let mut c = Clock::new(5);
        c.photo_ready(at(0));
        c.toggle_pause(at(2));
        assert!(!c.due(at(100)));
        c.toggle_pause(at(100));
        assert!(!c.due(at(102)));
        assert!(c.due(at(103)));
    }
    #[test]
    fn manual_navigation_resets_clock_and_keeps_pause() {
        let mut c = Clock::new(5);
        c.photo_ready(at(0));
        c.toggle_pause(at(1));
        c.reset();
        c.photo_ready(at(9));
        assert!(!c.due(at(100)));
        c.toggle_pause(at(100));
        assert!(!c.due(at(104)));
        assert!(c.due(at(105)));
    }
    #[test]
    fn video_switch_cancels_photo_deadline() {
        let mut c = Clock::new(5);
        c.photo_ready(at(0));
        c.reset();
        assert!(!c.due(at(1000)));
        c.photo_ready(at(1000));
        assert!(c.due(at(1005)));
    }
    #[test]
    fn interval_change_and_zero_remaining_are_deterministic() {
        let mut c = Clock::new(5);
        c.photo_ready(at(0));
        c.set_interval(10, at(3));
        assert!(!c.due(at(12)));
        c.toggle_pause(at(13));
        c.toggle_pause(at(99));
        assert!(c.due(at(99)));
    }
    #[test]
    fn video_waits_for_eos_and_paused_eos_is_deferred() {
        let mut c = Clock::new(3);
        assert!(!c.video_completed(false));
        assert!(!c.due(at(999)));
        c.toggle_pause(at(0));
        assert!(!c.video_completed(true));
        c.toggle_pause(at(1));
        assert!(c.video_completed(true));
    }
    #[test]
    fn loop_includes_video_and_skips_failed_unsupported_without_mutating_catalog() {
        let mut cat = MediaCollection::new(vec![
            MediaItem::new("1.png", MediaKind::Image),
            MediaItem::new("2.mp4", MediaKind::Video),
            MediaItem::new("3.unknown", MediaKind::Image),
        ]);
        assert_eq!(target(&cat, true, &HashSet::new()), Some(1));
        cat.select(1);
        assert_eq!(target(&cat, true, &HashSet::new()), Some(0));
        assert_eq!(cat.selected_index(), Some(1));
        let failed = HashSet::from([PathBuf::from("1.png"), PathBuf::from("2.mp4")]);
        assert_eq!(target(&cat, false, &failed), None);
        cat.replace_preserving(vec![]);
        assert_eq!(target(&cat, true, &failed), None);
    }
    #[test]
    fn singleton_and_removed_selection_wrap_safely() {
        let mut cat = MediaCollection::new(vec![MediaItem::new("1.png", MediaKind::Image)]);
        assert_eq!(target(&cat, false, &HashSet::new()), Some(0));
        cat.replace_preserving(vec![
            MediaItem::new("2.png", MediaKind::Image),
            MediaItem::new("3.png", MediaKind::Image),
        ]);
        assert_eq!(target(&cat, true, &HashSet::new()), Some(0));
        assert_eq!(target(&cat, false, &HashSet::new()), Some(1));
    }
}
