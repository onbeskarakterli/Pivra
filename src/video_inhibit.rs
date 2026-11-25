//! Video-only IDLE inhibition. Slideshow retains its separate paused-session lease.
use gtk::{glib, prelude::*};
use pivra::inhibit::{Change, Lease};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
pub struct VideoInhibit {
    window: glib::WeakRef<adw::ApplicationWindow>,
    app: glib::WeakRef<gtk::Application>,
    stream: RefCell<Option<(glib::WeakRef<gtk::MediaFile>, Vec<glib::SignalHandlerId>)>>,
    lease: RefCell<Lease>,
    closed: Cell<bool>,
}
impl VideoInhibit {
    pub fn new(window: &adw::ApplicationWindow) -> Rc<Self> {
        Rc::new(Self {
            window: window.downgrade(),
            app: window
                .application()
                .map(|a| a.downgrade())
                .unwrap_or_default(),
            stream: RefCell::new(None),
            lease: RefCell::default(),
            closed: Cell::new(false),
        })
    }
    fn apply(&self, active: bool) {
        let change = self.lease.borrow_mut().update(active);
        let Some(app) = self.app.upgrade() else {
            return;
        };
        match change {
            Change::Acquire(epoch) => {
                let window = self.window.upgrade();
                let cookie = app.inhibit(
                    window.as_ref(),
                    gtk::ApplicationInhibitFlags::IDLE,
                    Some("Video playback"),
                );
                // Do not hold a RefCell borrow across session IPC/reentrant GTK.
                let stale = self.lease.borrow_mut().complete(epoch, cookie);
                if let Some(cookie) = stale {
                    app.uninhibit(cookie);
                }
                if cookie == 0 {
                    eprintln!(
                        "Pivra: video display-sleep inhibition unavailable for this playback session"
                    );
                }
            }
            Change::Release(cookie) => app.uninhibit(cookie),
            Change::None => {}
        }
    }
    fn refresh(&self) {
        let active = !self.closed.get()
            && self.window.upgrade().is_some_and(|w| w.is_visible())
            && self
                .stream
                .borrow()
                .as_ref()
                .and_then(|(m, _)| m.upgrade())
                .is_some_and(|m| {
                    m.is_prepared()
                        && m.has_video()
                        && m.is_playing()
                        && !m.is_ended()
                        && m.error().is_none()
                });
        self.apply(active);
    }
    pub fn bind(self: &Rc<Self>, media: &gtk::MediaFile) {
        self.unbind();
        let mut handlers = Vec::new();
        for property in ["prepared", "has-video", "playing", "ended", "error"] {
            let weak = Rc::downgrade(self);
            handlers.push(media.connect_notify_local(Some(property), move |_, _| {
                if let Some(this) = weak.upgrade() {
                    this.refresh();
                }
            }));
        }
        self.stream.replace(Some((media.downgrade(), handlers)));
        self.refresh();
    }
    pub fn unbind(&self) {
        if let Some((weak, handlers)) = self.stream.borrow_mut().take() {
            if let Some(media) = weak.upgrade() {
                for id in handlers {
                    media.disconnect(id);
                }
            }
        }
        self.apply(false);
    }
    pub fn close(&self) {
        self.closed.set(true);
        self.unbind();
    }
}
impl Drop for VideoInhibit {
    fn drop(&mut self) {
        self.unbind();
    }
}
