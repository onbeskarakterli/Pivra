//! Slideshow timing and bounded catalog traversal. No GTK clocks or decoder.
use crate::{catalog::MediaCollection, media::MediaKind};
use std::{collections::HashSet, path::PathBuf, time::Duration};
pub const DEFAULT_SECONDS: u32 = 5;
pub const INTERVALS: [u32; 5] = [3, 5, 10, 15, 30];
pub fn valid_interval(seconds: u32) -> bool {
    (1..=3600).contains(&seconds)
}
pub fn parse_interval(text: &str) -> Option<u32> {
    let text = text.trim();
    if text.is_empty() || text.len() > 4 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok().filter(|s| valid_interval(*s))
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Effect {
    #[default]
    Fade,
    None,
}
impl Effect {
    pub fn id(self) -> &'static str {
        match self {
            Self::Fade => "fade",
            Self::None => "none",
        }
    }
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "fade" => Some(Self::Fade),
            "none" => Some(Self::None),
            _ => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Fade => "Fade",
            Self::None => "No transition",
        }
    }
}
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
/// GTK's ended property is a level, not proof of a new successful completion.
/// Observe stream notifications before play (including very short clips), then
/// consume on the main-context observer, never dismantling a signal's emitter.
#[derive(Clone, Copy, Default, Debug)]
pub struct VideoState {
    pub prepared: bool,
    pub playing: bool,
    pub seeking: bool,
    pub ended: bool,
    pub failed: bool,
}
#[derive(Default, Debug)]
pub struct VideoCompletion {
    live: bool,
    ended: bool,
    pending: bool,
}
impl VideoCompletion {
    /// Returns whether a validated completion is pending (not yet consumed).
    pub fn observe(&mut self, state: VideoState) -> bool {
        let rising = state.ended && !self.ended;
        self.ended = state.ended;
        if !state.prepared || state.failed || state.seeking {
            self.live = false;
            self.pending = false;
            return false;
        }
        if !state.ended {
            self.pending = false;
            if state.playing {
                self.live = true;
            }
        } else if rising && self.live && !state.playing {
            self.pending = true;
            self.live = false;
        }
        self.pending
    }
    pub fn take(&mut self, paused: bool, state: VideoState) -> bool {
        if state.failed || !state.prepared || state.seeking || !state.ended || state.playing {
            self.pending = false;
        }
        !paused && std::mem::take(&mut self.pending)
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
    #[test]
    fn custom_interval_validation() {
        for (text, expected) in [("1", 1), ("7", 7), ("3600", 3600), (" 15 ", 15)] {
            assert_eq!(parse_interval(text), Some(expected));
        }
        for text in [
            "", "0", "3601", "-1", "+5", "1.5", "5\n10", "hello", "999999", "５",
        ] {
            assert_eq!(parse_interval(text), None, "{text:?}");
        }
        for seconds in INTERVALS {
            assert!(valid_interval(seconds));
        }
        assert!(!valid_interval(u32::MAX));
    }
    #[test]
    fn only_supported_effects_round_trip() {
        for effect in [Effect::Fade, Effect::None] {
            assert_eq!(Effect::from_id(effect.id()), Some(effect));
        }
        assert_eq!(Effect::from_id("spin"), None);
    }
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
        let mut end = VideoCompletion::default();
        end.observe(playing());
        assert!(!end.take(false, playing()));
        assert!(!c.due(at(999)));
        end.observe(ended());
        assert!(!end.take(true, ended()));
        assert!(end.take(false, ended()));
        assert!(!end.take(false, ended()));
    }
    fn playing() -> VideoState {
        VideoState {
            prepared: true,
            playing: true,
            ..VideoState::default()
        }
    }
    fn ended() -> VideoState {
        VideoState {
            prepared: true,
            ended: true,
            ..VideoState::default()
        }
    }
    #[test]
    fn initial_or_unprepared_ended_is_not_a_new_eos() {
        let mut end = VideoCompletion::default();
        end.observe(ended());
        assert!(!end.take(false, ended()));
        end.observe(VideoState {
            prepared: false,
            ..ended()
        });
        assert!(!end.take(false, ended()));
    }
    #[test]
    fn errors_and_seeks_cannot_complete_a_video() {
        for state in [
            VideoState {
                failed: true,
                ..ended()
            },
            VideoState {
                seeking: true,
                ..ended()
            },
        ] {
            let mut end = VideoCompletion::default();
            end.observe(playing());
            end.observe(state);
            assert!(!end.take(false, state));
            end.observe(ended());
            assert!(!end.take(false, ended()));
            end.observe(playing());
            end.observe(ended());
            assert!(end.take(false, ended()));
        }
    }
    #[test]
    fn pause_is_not_eos_and_short_video_notifications_are_not_lost() {
        let mut end = VideoCompletion::default();
        end.observe(playing());
        end.observe(VideoState {
            playing: false,
            ..playing()
        });
        assert!(!end.take(
            false,
            VideoState {
                playing: false,
                ..playing()
            }
        ));
        end.observe(playing());
        end.observe(ended()); // Both notifications can occur before the next poll.
        assert!(end.take(false, ended()));
    }
    #[test]
    fn replacement_and_seek_after_eos_discard_pending_completion() {
        let mut old = VideoCompletion::default();
        old.observe(playing());
        old.observe(ended());
        let mut replacement = VideoCompletion::default();
        assert!(!replacement.take(false, ended()));
        old.observe(VideoState {
            seeking: true,
            ..ended()
        });
        assert!(!old.take(false, ended()));
        // A queued EOS cannot outlive an error/restart before the next poll,
        // even if no further notification was delivered to this observer.
        for invalid in [
            VideoState {
                failed: true,
                ..ended()
            },
            VideoState {
                prepared: false,
                ..ended()
            },
            playing(),
        ] {
            let mut queued = VideoCompletion::default();
            queued.observe(playing());
            queued.observe(ended());
            assert!(!queued.take(false, invalid));
            assert!(!queued.take(false, ended()));
        }
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
