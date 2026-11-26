//! Per-window orchestration. A single bounded main-context observer owns no decoder.
use super::{FileContext, display_selected, update_navigation_actions, update_title};
use crate::image_view::ImageView;
use adw::prelude::*;
use gtk::{gio, glib};
use pivra::{
    image_pipeline::Phase,
    media::MediaKind,
    slideshow::{Clock, VideoCompletion, VideoState, target},
};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

pub(super) fn active(window: &adw::ApplicationWindow) -> bool {
    window
        .lookup_action("slideshow")
        .and_then(|a| a.state())
        .and_then(|v| v.get::<bool>())
        .unwrap_or(false)
}
struct VideoWatch {
    media: glib::WeakRef<gtk::MediaFile>,
    handlers: Vec<glib::SignalHandlerId>,
}
fn video_state(media: &gtk::MediaFile) -> VideoState {
    VideoState {
        prepared: media.is_prepared(),
        playing: media.is_playing(),
        seeking: media.is_seeking(),
        ended: media.is_ended(),
        failed: media.error().is_some(),
    }
}
pub(super) struct Session {
    window: glib::WeakRef<adw::ApplicationWindow>,
    title: adw::WindowTitle,
    context: Rc<RefCell<FileContext>>,
    view: ImageView,
    running: Cell<bool>,
    clock: RefCell<Clock>,
    origin: Instant,
    revision: Cell<Option<u64>>,
    last_path: RefCell<Option<PathBuf>>,
    failed: RefCell<HashSet<PathBuf>>,
    stream: RefCell<Option<VideoWatch>>,
    timer: RefCell<Option<glib::SourceId>>,
    inhibitor: Cell<u32>,
    inhibitor_app: RefCell<Option<glib::WeakRef<gtk::Application>>>,
    video_end: RefCell<VideoCompletion>,
    toasts: glib::WeakRef<adw::ToastOverlay>,
    current_toast: Rc<RefCell<Option<adw::Toast>>>,
    notice: RefCell<Option<glib::WeakRef<adw::Toast>>>,
}
impl Session {
    fn dismiss_notice(&self) {
        if let Some(toast) = self.notice.borrow_mut().take().and_then(|w| w.upgrade()) {
            toast.dismiss();
        }
    }
    fn notify(&self, message: &str) {
        if let Some(overlay) = self.toasts.upgrade() {
            let toast = adw::Toast::new(message);
            toast.set_timeout(2);
            self.notice.replace(Some(toast.downgrade()));
            super::show_latest_toast(&overlay, &self.current_toast, toast.clone());
            // Dismiss after two seconds even when GTK pauses toast timeouts
            // (for example when the pointer rests over the notification).
            let weak_toast = toast.downgrade();
            glib::timeout_add_local_once(Duration::from_secs(2), move || {
                if let Some(toast) = weak_toast.upgrade() {
                    toast.dismiss();
                }
            });
        }
    }
    fn unbind_video(&self) {
        if let Some(watch) = self.stream.borrow_mut().take() {
            if let Some(media) = watch.media.upgrade() {
                for handler in watch.handlers {
                    media.disconnect(handler);
                }
            }
        }
        *self.video_end.borrow_mut() = VideoCompletion::default();
    }
    fn bind_video(self: &Rc<Self>, media: &gtk::MediaFile) {
        if !self.running.get() {
            return;
        }
        self.unbind_video();
        let mut handlers = Vec::new();
        for property in ["prepared", "playing", "seeking", "ended", "error"] {
            let weak = Rc::downgrade(self);
            handlers.push(media.connect_notify_local(Some(property), move |media, _| {
                let Some(this) = weak.upgrade() else {
                    return;
                };
                if this.running.get()
                    && this.view.current_video().as_ref() == Some(media)
                    && this
                        .stream
                        .borrow()
                        .as_ref()
                        .and_then(|w| w.media.upgrade())
                        .as_ref()
                        == Some(media)
                {
                    // Record only: poll() consumes completion outside this signal.
                    let state = video_state(media);
                    let accepted = this.video_end.borrow_mut().observe(state);
                    if property == "ended" && state.ended
                        && std::env::var_os("PIVRA_TRACE_SLIDESHOW").is_some_and(|v| v == "1")
                    {
                        eprintln!("PIVRA_SLIDESHOW event=video-end-notify time_us={} media={:p} prepared={} playing={} seeking={} failed={} accepted={accepted}",
                            glib::monotonic_time(), media.as_ptr(), state.prepared, state.playing, state.seeking, state.failed);
                    }
                }
            }));
        }
        self.stream.replace(Some(VideoWatch {
            media: media.downgrade(),
            handlers,
        }));
        self.video_end.borrow_mut().observe(video_state(media));
    }
    // Opt-in transition boundaries only, never per-frame logging or GL calls.
    fn trace(&self, event: &str, path: Option<&std::path::Path>) {
        if std::env::var_os("PIVRA_TRACE_SLIDESHOW").is_some_and(|v| v == "1") {
            eprintln!(
                "PIVRA_SLIDESHOW event={event} time_us={} path={path:?} phase={:?}",
                glib::monotonic_time(),
                self.view.phase()
            );
        }
    }
    fn state(&self, name: &str, value: bool) {
        if let Some(window) = self.window.upgrade() {
            if let Some(action) = window
                .lookup_action(name)
                .and_downcast::<gio::SimpleAction>()
            {
                action.set_state(&value.to_variant());
            }
        }
    }
    pub(super) fn start(self: &Rc<Self>) {
        if self.running.get() || self.context.borrow().collection.current().is_none() {
            return;
        }
        let Some(window) = self.window.upgrade().filter(|w| w.is_visible()) else {
            return;
        };
        self.running.set(true);
        self.failed.borrow_mut().clear();
        self.clock.borrow_mut().paused = false;
        self.clock.borrow_mut().reset();
        self.revision.set(None);
        self.last_path.borrow_mut().take();
        self.view.set_slideshow(true);
        {
            let context = self.context.borrow();
            if context
                .collection
                .current()
                .is_some_and(|i| i.kind() == MediaKind::Image)
                && self.view.phase() != Phase::FullReady
            {
                // load() deduplicates a live foreground request and restarts
                // a previously cancelled request through normal admission.
                display_selected(&self.view, &context);
            }
        }
        self.state("slideshow", true);
        self.state("slideshow-paused", false);
        if let Some(action) = window.lookup_action("slideshow-fullscreen") {
            action.change_state(&true.to_variant());
        }
        update_navigation_actions(&window, &self.context.borrow());
        if !self.running.get() || !window.is_visible() {
            self.stop();
            return;
        }
        if let Some(app) = window.application() {
            let cookie = app.inhibit(
                Some(&window),
                gtk::ApplicationInhibitFlags::IDLE,
                Some("Slideshow active (including pause)"),
            );
            if !self.running.get() || !window.is_visible() {
                if cookie != 0 {
                    app.uninhibit(cookie);
                }
                self.stop();
                return;
            }
            self.inhibitor.set(cookie);
            self.inhibitor_app.replace(Some(app.downgrade()));
            if cookie == 0 {
                eprintln!("Pivra: display-sleep inhibition unavailable for this slideshow session");
            }
        }
        if let Some(media) = self.view.current_video() {
            self.bind_video(&media);
            if media.is_ended() && media.is_seekable() {
                media.seek(0);
            }
            media.play();
        }
        self.arm();
        self.notify("Slideshow started");
    }
    fn arm(self: &Rc<Self>) {
        if self.timer.borrow().is_some() || !self.running.get() {
            return;
        }
        let weak = Rc::downgrade(self);
        let id = glib::timeout_add_local_once(Duration::from_millis(100), move || {
            let Some(this) = weak.upgrade() else {
                return;
            };
            this.timer.borrow_mut().take();
            this.poll();
            this.arm();
        });
        self.timer.replace(Some(id));
    }
    pub(super) fn stop(&self) {
        if !self.running.replace(false) {
            return;
        }
        self.trace("stop-begin", None);
        if let Some(id) = self.timer.borrow_mut().take() {
            id.remove();
        }
        self.unbind_video();
        self.failed.borrow_mut().clear();
        self.last_path.borrow_mut().take();
        self.clock.borrow_mut().reset();
        self.view.set_slideshow(false);
        self.state("slideshow", false);
        self.state("slideshow-paused", false);
        let cookie = self.inhibitor.replace(0);
        if let Some(app) = self
            .inhibitor_app
            .borrow_mut()
            .take()
            .and_then(|app| app.upgrade())
        {
            if cookie != 0 {
                app.uninhibit(cookie);
            }
        }
        if let Some(window) = self.window.upgrade() {
            if let Some(action) = window.lookup_action("slideshow-fullscreen") {
                action.change_state(&false.to_variant());
            }
            update_navigation_actions(&window, &self.context.borrow());
        }
        self.trace("stop-complete", None);
        if self.window.upgrade().is_some_and(|w| w.is_visible()) {
            self.notify("Slideshow finished");
        } else {
            self.dismiss_notice();
        }
    }
    pub(super) fn pause(&self) {
        if !self.running.get() {
            return;
        }
        self.clock.borrow_mut().toggle_pause(self.origin.elapsed());
        let paused = self.clock.borrow().paused;
        self.view.set_slideshow_paused(paused);
        self.state("slideshow-paused", paused);
        if paused {
            self.notify("Slideshow paused");
        } else {
            self.dismiss_notice();
        }
        // Inhibitor intentionally survives pause.
    }
    pub(super) fn navigate(self: &Rc<Self>, forward: bool) {
        self.advance(forward, "manual");
    }
    fn advance(self: &Rc<Self>, forward: bool, reason: &str) {
        if !self.running.get() {
            return;
        }
        self.trace(reason, None);
        let Some(window) = self.window.upgrade() else {
            self.stop();
            return;
        };
        let mut context = self.context.borrow_mut();
        let next = {
            let failed = self.failed.borrow();
            target(&context.collection, forward, &failed)
        };
        let Some(index) = next else {
            drop(context);
            self.stop();
            self.view
                .show_message("Slideshow stopped: no readable supported media remains.");
            return;
        };
        self.clock.borrow_mut().reset();
        self.unbind_video();
        if context.collection.selected_index() == Some(index) {
            if let Some(media) = self.view.current_video() {
                self.bind_video(&media);
                if media.is_seekable() {
                    media.seek(0);
                    if !self.clock.borrow().paused {
                        media.play();
                    }
                } else {
                    display_selected(&self.view, &context);
                }
            }
            return;
        }
        context.collection.select(index);
        context.revision = context.revision.wrapping_add(1);
        update_title(&window, &self.title, &context);
        let path = context.collection.current().map(|i| i.path());
        self.trace("display-selected-begin", path);
        display_selected(&self.view, &context);
        self.trace("display-selected-return", path);
    }
    fn poll(self: &Rc<Self>) {
        if !self.running.get() {
            return;
        }
        let (revision, item) = {
            let c = self.context.borrow();
            (c.revision, c.collection.current().cloned())
        };
        let old_revision = self.revision.replace(Some(revision));
        let path = item.as_ref().map(|i| i.path().to_owned());
        let same_path = *self.last_path.borrow() == path;
        self.last_path.replace(path);
        if old_revision != Some(revision) {
            self.clock.borrow_mut().reset();
            // A same-path revision comes from the existing file monitor. A
            // cancelled/changed current request must not leave slideshow waiting
            // forever for readiness. Navigation already starts its own request.
            if old_revision.is_some() && same_path && item.is_some() {
                self.trace("catalog-revision-reload", item.as_ref().map(|i| i.path()));
                if let Some(window) = self.window.upgrade() {
                    gio::prelude::ActionGroupExt::activate_action(&window, "reload-media", None);
                }
            }
        }
        let Some(item) = item else {
            self.advance(true, "catalog-empty-or-removed");
            return;
        };
        let paused = self.clock.borrow().paused;
        if item.kind() == MediaKind::Video {
            // The photo deadline is never armed or consumed for a video, even
            // across delayed initialization, interval edits or catalog updates.
            self.clock.borrow_mut().reset();
            if let Some(media) = self.view.current_video() {
                let same = self
                    .stream
                    .borrow()
                    .as_ref()
                    .and_then(|s| s.media.upgrade())
                    .is_some_and(|s| s == media);
                if !same {
                    self.bind_video(&media);
                    if paused {
                        media.pause();
                    } else {
                        media.play();
                    }
                }
                let complete = self
                    .video_end
                    .borrow_mut()
                    .take(paused, video_state(&media));
                if complete {
                    if std::env::var_os("PIVRA_TRACE_SLIDESHOW").is_some_and(|v| v == "1") {
                        eprintln!(
                            "PIVRA_SLIDESHOW event=video-ended path={:?} time_us={} media={:p} type={} prepared={} seekable={} duration={} timestamp={}",
                            item.path(),
                            glib::monotonic_time(),
                            media.as_ptr(),
                            media.type_(),
                            media.is_prepared(),
                            media.is_seekable(),
                            media.duration(),
                            media.timestamp()
                        );
                    }
                    // This is the main-context observer, not a GStreamer signal
                    // handler. Do not retain its stream across stop_video().
                    drop(media);
                    self.trace("eos-observer-reference-released", Some(item.path()));
                    self.advance(true, "video-eos");
                    return;
                }
            }
            if !paused && self.view.slideshow_video_failed() {
                // A decoder failure is not successful completion. Keep its
                // visible error and selection; manual arrows remain available.
                self.trace("video-error-paused", Some(item.path()));
                self.pause();
            }
        } else {
            match self.view.phase() {
                Phase::FullReady => {
                    self.clock.borrow_mut().photo_ready(self.origin.elapsed());
                    let due = self.clock.borrow_mut().due(self.origin.elapsed());
                    if due {
                        self.advance(true, "photo-deadline");
                    }
                }
                Phase::Failed if !paused => self.fail(item.path().to_owned()),
                _ if !pivra::discovery::supported_photo(item.path()) && !paused => {
                    self.fail(item.path().to_owned())
                }
                _ => self.clock.borrow_mut().reset(),
            }
        }
    }
    fn fail(self: &Rc<Self>, path: PathBuf) {
        {
            let context = self.context.borrow();
            let mut failed = self.failed.borrow_mut();
            failed.insert(path);
            if failed.len() > context.collection.items().len() {
                let current: HashSet<_> = context
                    .collection
                    .items()
                    .iter()
                    .map(|i| i.path())
                    .collect();
                failed.retain(|path| current.contains(path.as_path()));
            }
        }
        self.advance(true, "photo-error");
    }
}

pub(super) fn install(
    window: &adw::ApplicationWindow,
    title: &adw::WindowTitle,
    context: Rc<RefCell<FileContext>>,
    view: ImageView,
    toasts: &adw::ToastOverlay,
    current_toast: Rc<RefCell<Option<adw::Toast>>>,
) -> Rc<Session> {
    if let Some(picture) = view.widget().child().and_downcast::<gtk::Picture>() {
        super::slideshow_interaction::install(window, &picture);
    }
    let seconds = pivra::settings::load_slideshow_interval();
    let this = Rc::new(Session {
        window: window.downgrade(),
        title: title.clone(),
        context,
        view,
        running: Cell::new(false),
        clock: RefCell::new(Clock::new(seconds)),
        origin: Instant::now(),
        revision: Cell::new(None),
        last_path: RefCell::new(None),
        failed: RefCell::new(HashSet::new()),
        stream: RefCell::new(None),
        timer: RefCell::new(None),
        inhibitor: Cell::new(0),
        inhibitor_app: RefCell::new(None),
        video_end: RefCell::new(VideoCompletion::default()),
        toasts: toasts.downgrade(),
        current_toast,
        notice: RefCell::new(None),
    });
    let weak = Rc::downgrade(&this);
    this.view.on_video_stream(move |media| {
        if let Some(this) = weak.upgrade() {
            if let Some(media) = media {
                this.bind_video(media);
            } else {
                this.unbind_video();
            }
        }
    });
    for (name, kind) in [
        ("slideshow", 0),
        ("slideshow-stop", 1),
        ("slideshow-pause", 2),
        ("slideshow-next", 3),
        ("slideshow-previous", 4),
    ] {
        let action = if kind == 0 {
            gio::SimpleAction::new_stateful(name, None, &false.to_variant())
        } else {
            gio::SimpleAction::new(name, None)
        };
        let state = this.clone();
        action.connect_activate(move |_, _| match kind {
            0 => state.start(),
            1 => state.stop(),
            2 => state.pause(),
            3 => state.navigate(true),
            _ => state.navigate(false),
        });
        window.add_action(&action);
    }
    window.add_action(&gio::SimpleAction::new_stateful(
        "slideshow-paused",
        None,
        &false.to_variant(),
    ));
    let interval = gio::SimpleAction::new_stateful(
        "slideshow-interval",
        Some(glib::VariantTy::UINT32),
        &seconds.to_variant(),
    );
    let state = this.clone();
    interval.connect_activate(move |action, value| {
        let Some(seconds) = value
            .and_then(|v| v.get::<u32>())
            .filter(|s| pivra::slideshow::valid_interval(*s))
        else {
            return;
        };
        state
            .clock
            .borrow_mut()
            .set_interval(seconds, state.origin.elapsed());
        action.set_state(&seconds.to_variant());
        pivra::settings::save_slideshow_interval(seconds);
    });
    window.add_action(&interval);
    let effect = pivra::settings::load_slideshow_effect();
    this.view.set_slideshow_effect(effect);
    let transition = gio::SimpleAction::new_stateful(
        "slideshow-transition",
        Some(glib::VariantTy::STRING),
        &effect.id().to_variant(),
    );
    let state = this.clone();
    transition.connect_activate(move |action, value| {
        let Some(effect) = value
            .and_then(|v| v.str())
            .and_then(pivra::slideshow::Effect::from_id)
        else {
            return;
        };
        state.view.set_slideshow_effect(effect);
        action.set_state(&effect.id().to_variant());
        pivra::settings::save_slideshow_effect(effect);
    });
    window.add_action(&transition);
    let closing = this.clone();
    window.connect_close_request(move |_| {
        closing.stop();
        closing.dismiss_notice();
        glib::Propagation::Proceed
    });
    let destroyed = this.clone();
    window.connect_destroy(move |_| {
        destroyed.stop();
        destroyed.dismiss_notice();
    });
    this
}
