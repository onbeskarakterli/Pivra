//! Per-window orchestration. A single bounded main-context observer owns no decoder.
use super::{FileContext, display_selected, update_navigation_actions, update_title};
use crate::image_view::ImageView;
use adw::prelude::*;
use gtk::{gio, glib};
use pivra::{
    image_pipeline::Phase,
    media::MediaKind,
    slideshow::{Clock, target},
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
    stream: RefCell<Option<glib::WeakRef<gtk::MediaFile>>>,
    timer: RefCell<Option<glib::SourceId>>,
    inhibitor: Cell<u32>,
    inhibitor_app: RefCell<Option<glib::WeakRef<gtk::Application>>>,
    ended_seen: Cell<bool>,
}
impl Session {
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
            if media.is_ended() && media.is_seekable() {
                media.seek(0);
            }
            media.play();
        }
        self.arm();
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
        if let Some(id) = self.timer.borrow_mut().take() {
            id.remove();
        }
        self.stream.borrow_mut().take();
        self.failed.borrow_mut().clear();
        self.last_path.borrow_mut().take();
        self.ended_seen.set(false);
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
    }
    pub(super) fn pause(&self) {
        if !self.running.get() {
            return;
        }
        self.clock.borrow_mut().toggle_pause(self.origin.elapsed());
        let paused = self.clock.borrow().paused;
        self.view.set_slideshow_paused(paused);
        self.state("slideshow-paused", paused);
        // Inhibitor intentionally survives pause.
    }
    pub(super) fn navigate(&self, forward: bool) {
        if !self.running.get() {
            return;
        }
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
        if context.collection.selected_index() == Some(index) {
            if let Some(media) = self.view.current_video() {
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
        self.stream.borrow_mut().take();
        context.collection.select(index);
        context.revision = context.revision.wrapping_add(1);
        update_title(&window, &self.title, &context);
        display_selected(&self.view, &context);
    }
    fn poll(&self) {
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
            self.stream.borrow_mut().take();
            // A same-path revision comes from the existing file monitor. A
            // cancelled/changed current request must not leave slideshow waiting
            // forever for readiness. Navigation already starts its own request.
            if old_revision.is_some() && same_path && item.is_some() {
                if let Some(window) = self.window.upgrade() {
                    gio::prelude::ActionGroupExt::activate_action(&window, "reload-media", None);
                }
            }
        }
        let Some(item) = item else {
            self.navigate(true);
            return;
        };
        let paused = self.clock.borrow().paused;
        if item.kind() == MediaKind::Video {
            if let Some(media) = self.view.current_video() {
                let same = self
                    .stream
                    .borrow()
                    .as_ref()
                    .and_then(|s| s.upgrade())
                    .is_some_and(|s| s == media);
                if !same {
                    self.ended_seen.set(false);
                    self.stream.replace(Some(media.downgrade()));
                    if paused {
                        media.pause();
                    } else {
                        media.play();
                    }
                }
                if !media.is_ended() {
                    self.ended_seen.set(false);
                }
                if media.error().is_none()
                    && self.clock.borrow().video_completed(media.is_ended())
                    && !self.ended_seen.replace(true)
                {
                    if std::env::var_os("PIVRA_TRACE_SLIDESHOW").is_some_and(|v| v == "1") {
                        eprintln!("PIVRA_SLIDESHOW event=video-ended path={:?}", item.path());
                    }
                    drop(media);
                    self.navigate(true);
                    return;
                }
            }
            if !paused && self.view.slideshow_video_failed() {
                self.fail(item.path().to_owned());
            }
        } else {
            match self.view.phase() {
                Phase::FullReady => {
                    self.clock.borrow_mut().photo_ready(self.origin.elapsed());
                    let due = self.clock.borrow_mut().due(self.origin.elapsed());
                    if due {
                        self.navigate(true);
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
    fn fail(&self, path: PathBuf) {
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
        self.navigate(true);
    }
}

pub(super) fn install(
    window: &adw::ApplicationWindow,
    title: &adw::WindowTitle,
    context: Rc<RefCell<FileContext>>,
    view: ImageView,
    menu: &gio::Menu,
) -> Rc<Session> {
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
        ended_seen: Cell::new(false),
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
            .filter(|s| pivra::slideshow::INTERVALS.contains(s))
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
    let section = gio::Menu::new();
    section.append(Some("Start slideshow (F5)"), Some("win.slideshow"));
    for seconds in pivra::slideshow::INTERVALS {
        let item = gio::MenuItem::new(Some(&format!("Photo interval: {seconds} s")), None);
        item.set_action_and_target_value(
            Some("win.slideshow-interval"),
            Some(&seconds.to_variant()),
        );
        section.append_item(&item);
    }
    menu.append_section(Some("Slideshow"), &section);
    let closing = this.clone();
    window.connect_close_request(move |_| {
        closing.stop();
        glib::Propagation::Proceed
    });
    let destroyed = this.clone();
    window.connect_destroy(move |_| destroyed.stop());
    this
}
