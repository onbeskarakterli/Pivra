//! Per-view admission, cancellation and identity. No paths, GTK objects or decoder.
use gtk::{gio, glib, prelude::*};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};
const MAX_RUNNING: usize = 3;
const MAX_QUEUE_WAIT: Duration = Duration::from_secs(5);
#[derive(Clone, Copy, Debug, Default)]
#[allow(dead_code)] // Probe diagnostics; never request identity or product UI.
pub(crate) struct LoadStats {
    pub running: usize,
    pub pending: usize,
    pub started: u64,
    pub completed: u64,
    pub max_running: usize,
    pub dropped_pending: u64,
    pub suppressed: u64,
    pub timed_out: u64,
}
struct Pending {
    token: gio::Cancellable,
    start: Box<dyn FnOnce()>,
    timer: glib::SourceId,
}
#[derive(Default)]
pub(super) struct LoadSlot {
    current: RefCell<Option<gio::Cancellable>>,
    active: RefCell<Vec<gio::Cancellable>>,
    pending: RefCell<Option<Pending>>,
    stats: Cell<LoadStats>,
    promoted: RefCell<Option<(gio::Cancellable, gio::Cancellable)>>,
}
impl LoadSlot {
    pub fn begin(&self) -> gio::Cancellable {
        self.cancel();
        let token = gio::Cancellable::new();
        self.current.replace(Some(token.clone()));
        token
    }
    pub fn cancel(&self) {
        self.promoted.borrow_mut().take();
        let current = self.current.borrow_mut().take();
        let pending = self.pending.borrow_mut().take();
        if let Some(pending) = pending {
            pending.timer.remove();
            let mut s = self.stats.get();
            s.dropped_pending = s.dropped_pending.saturating_add(1);
            self.stats.set(s);
        }
        // Release RefCell borrows before cancellation can invoke observers.
        let active = self.active.borrow().clone();
        if let Some(token) = current {
            token.cancel();
        }
        for token in active {
            token.cancel();
        }
    }
    pub fn begin_promoting(&self, background: &gio::Cancellable) -> gio::Cancellable {
        let current = self.current.borrow_mut().take();
        let pending = self.pending.borrow_mut().take();
        if let Some(pending) = pending {
            pending.timer.remove();
            let mut stats = self.stats.get();
            stats.dropped_pending = stats.dropped_pending.saturating_add(1);
            self.stats.set(stats);
        }
        if let Some(current) = current {
            current.cancel();
        }
        for token in self.active.borrow().clone() {
            if &token != background {
                token.cancel();
            }
        }
        let token = gio::Cancellable::new();
        self.current.replace(Some(token.clone()));
        self.promoted
            .replace(Some((token.clone(), background.clone())));
        token
    }
    pub fn try_background(&self, token: &gio::Cancellable, start: impl FnOnce()) -> bool {
        if !self.active.borrow().is_empty() || self.pending.borrow().is_some() {
            return false;
        }
        self.active.borrow_mut().push(token.clone());
        let mut s = self.stats.get();
        s.started = s.started.saturating_add(1);
        s.max_running = s.max_running.max(1);
        self.stats.set(s);
        start();
        true
    }
    pub fn accepts(&self, token: &gio::Cancellable) -> bool {
        !token.is_cancelled() && self.current.borrow().as_ref() == Some(token)
    }
    pub fn dispatch(
        self: &Rc<Self>,
        token: &gio::Cancellable,
        start: impl FnOnce() + 'static,
        on_timeout: impl FnOnce() + 'static,
    ) {
        self.dispatch_for(token, start, on_timeout, MAX_QUEUE_WAIT);
    }
    fn dispatch_for(
        self: &Rc<Self>,
        token: &gio::Cancellable,
        start: impl FnOnce() + 'static,
        on_timeout: impl FnOnce() + 'static,
        wait: Duration,
    ) {
        if !self.accepts(token) {
            return;
        }
        if self.active.borrow().len() < MAX_RUNNING {
            self.start(token.clone(), Box::new(start));
            return;
        }
        let weak = Rc::downgrade(self);
        let waiting = token.clone();
        let timer = glib::timeout_add_local_once(wait, move || {
            let Some(slot) = weak.upgrade() else {
                return;
            };
            if !slot.accepts(&waiting) {
                return;
            }
            let pending = slot.pending.borrow_mut().take();
            if pending.is_none() {
                return;
            }
            let mut s = slot.stats.get();
            s.timed_out = s.timed_out.saturating_add(1);
            slot.stats.set(s);
            // This is admission timeout, not a decoder completion or kill.
            on_timeout();
        });
        self.pending.replace(Some(Pending {
            token: token.clone(),
            start: Box::new(start),
            timer,
        }));
    }
    fn start(&self, token: gio::Cancellable, start: Box<dyn FnOnce()>) {
        self.active.borrow_mut().push(token);
        let mut s = self.stats.get();
        s.started = s.started.saturating_add(1);
        s.max_running = s.max_running.max(self.active.borrow().len());
        self.stats.set(s);
        start();
    }
    pub fn finish(&self, token: &gio::Cancellable) {
        {
            let mut active = self.active.borrow_mut();
            let Some(index) = active.iter().position(|active| active == token) else {
                return;
            };
            active.remove(index);
        }
        if self
            .promoted
            .borrow()
            .as_ref()
            .is_some_and(|(_, background)| background == token)
        {
            self.promoted.borrow_mut().take();
        }
        let mut s = self.stats.get();
        s.completed = s.completed.saturating_add(1);
        self.stats.set(s);
        let pending = self.pending.borrow_mut().take();
        if let Some(pending) = pending {
            pending.timer.remove();
            if self.accepts(&pending.token) {
                self.start(pending.token, pending.start);
            }
        }
    }
    pub fn suppress(&self) {
        let mut s = self.stats.get();
        s.suppressed = s.suppressed.saturating_add(1);
        self.stats.set(s);
    }
    pub fn current_in_flight(&self) -> bool {
        let current = self.current.borrow();
        let Some(token) = current.as_ref().filter(|t| !t.is_cancelled()) else {
            return false;
        };
        self.promoted
            .borrow()
            .as_ref()
            .is_some_and(|(owner, _)| owner == token)
            || self.active.borrow().contains(token)
            || self
                .pending
                .borrow()
                .as_ref()
                .is_some_and(|p| &p.token == token)
    }
    pub fn stats(&self) -> LoadStats {
        let mut s = self.stats.get();
        s.running = self.active.borrow().len();
        s.pending = usize::from(self.pending.borrow().is_some());
        s
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    static CONTEXT: std::sync::Mutex<()> = std::sync::Mutex::new(());
    #[test]
    fn same_path_identity_never_reuses_cancelled_token_and_windows_are_independent() {
        let a = LoadSlot::default();
        let b = LoadSlot::default();
        let other = b.begin();
        let first_a = a.begin();
        let middle = a.begin();
        let last_a = a.begin();
        assert_ne!(first_a, last_a);
        assert!(first_a.is_cancelled() && middle.is_cancelled());
        assert!(!a.accepts(&first_a));
        assert!(a.accepts(&last_a) && b.accepts(&other));
        for _ in 0..10000 {
            a.begin();
        }
        a.cancel();
        assert!(b.accepts(&other));
    }
    #[test]
    fn admission_is_bounded_and_only_latest_pending_starts() {
        let _serial = CONTEXT.lock().unwrap();
        let context = glib::MainContext::default();
        let _guard = context.acquire().unwrap();
        // No timers needed until saturation; local timers use default context.
        let slot = Rc::new(LoadSlot::default());
        let starts = Rc::new(Cell::new(0));
        let mut active = Vec::new();
        for _ in 0..3 {
            let token = slot.begin();
            let count = starts.clone();
            slot.dispatch(
                &token,
                move || count.set(count.get() + 1),
                || panic!("unexpected timeout"),
            );
            active.push(token);
        }
        for _ in 0..100 {
            let token = slot.begin();
            let count = starts.clone();
            slot.dispatch(
                &token,
                move || count.set(count.get() + 1),
                || panic!("unexpected timeout"),
            );
        }
        assert_eq!(starts.get(), 3);
        assert_eq!(slot.stats().running, 3);
        assert_eq!(slot.stats().pending, 1);
        slot.finish(&active[0]);
        assert_eq!(starts.get(), 4);
        assert_eq!(slot.stats().running, 3);
        assert_eq!(slot.stats().pending, 0);
        assert_eq!(slot.stats().max_running, 3);
        assert_eq!(slot.stats().dropped_pending, 99);
        let next = slot.begin();
        slot.dispatch(
            &next,
            || panic!("duplicate finish admitted new work"),
            || panic!("unexpected timeout"),
        );
        slot.finish(&active[0]);
        assert_eq!(
            (
                slot.stats().running,
                slot.stats().pending,
                slot.stats().started
            ),
            (3, 1, 4)
        );
        slot.cancel();
    }
    #[test]
    fn timeout_is_not_worker_completion_and_close_drops_pending_without_start() {
        let _serial = CONTEXT.lock().unwrap();
        let context = glib::MainContext::default();
        let _guard = context.acquire().unwrap();
        let slot = Rc::new(LoadSlot::default());
        let mut active = Vec::new();
        for _ in 0..3 {
            let token = slot.begin();
            slot.dispatch(&token, || {}, || panic!("unexpected"));
            active.push(token);
        }
        let token = slot.begin();
        let expired = Rc::new(Cell::new(false));
        let observed = expired.clone();
        slot.dispatch_for(
            &token,
            || panic!("queued request started"),
            move || observed.set(true),
            Duration::ZERO,
        );
        while !expired.get() {
            context.iteration(false);
        }
        assert_eq!(slot.stats().timed_out, 1);
        assert_eq!(slot.stats().completed, 0);
        assert_eq!(slot.stats().running, 3);
        assert_eq!(slot.stats().pending, 0);
        let token = slot.begin();
        slot.dispatch(
            &token,
            || panic!("closed pending started"),
            || panic!("closed timer fired"),
        );
        slot.cancel();
        for token in active {
            assert!(token.is_cancelled());
            slot.finish(&token);
        }
        assert_eq!(slot.stats().running, 0);
        assert_eq!(slot.stats().pending, 0);
    }
    #[test]
    fn background_leaves_foreground_capacity_and_promotes_without_duplicate_job() {
        let _serial = CONTEXT.lock().unwrap();
        let slot = Rc::new(LoadSlot::default());
        let background = gio::Cancellable::new();
        assert!(slot.try_background(&background, || {}));
        assert!(!slot.try_background(&gio::Cancellable::new(), || panic!("second background")));
        let selected = slot.begin_promoting(&background);
        assert!(slot.accepts(&selected));
        assert!(!background.is_cancelled());
        assert!(slot.current_in_flight());
        assert_eq!(slot.stats().started, 1);
        let next = slot.begin();
        assert!(background.is_cancelled());
        slot.dispatch(&next, || {}, || panic!("foreground delayed"));
        assert_eq!(slot.stats().running, 2);
        slot.finish(&background);
        assert!(slot.accepts(&next));
        slot.finish(&next);
        assert_eq!(slot.stats().running, 0);
    }
}
