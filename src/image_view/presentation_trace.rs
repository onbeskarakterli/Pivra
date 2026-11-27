//! One-shot frame evidence, enabled only with PIVRA_PERF. No pixel readback.
use adw::prelude::*;
use gtk::{gdk, glib};
use std::{cell::RefCell, path::Path, rc::Rc, time::Instant};

pub(super) struct Observer {
    picture: glib::WeakRef<gtk::Picture>,
    pending: RefCell<Option<(u64, Instant, glib::WeakRef<gdk::Paintable>)>>,
    frame: RefCell<Option<(gdk::FrameClock, glib::SignalHandlerId)>>,
    signals: RefCell<Vec<glib::SignalHandlerId>>,
}
impl Observer {
    pub(super) fn new(picture: &gtk::Picture, window: &adw::ApplicationWindow) -> Option<Rc<Self>> {
        if !pivra::performance::enabled() {
            return None;
        }
        let this = Rc::new(Self {
            picture: picture.downgrade(),
            pending: RefCell::new(None),
            frame: RefCell::new(None),
            signals: RefCell::new(vec![]),
        });
        let weak = Rc::downgrade(&this);
        this.signals
            .borrow_mut()
            .push(picture.connect_map(move |_| {
                if let Some(this) = weak.upgrade() {
                    this.arm();
                }
            }));
        let weak = Rc::downgrade(&this);
        this.signals
            .borrow_mut()
            .push(picture.connect_unmap(move |_| {
                if let Some(this) = weak.upgrade() {
                    this.cancel();
                }
            }));
        let weak = Rc::downgrade(&this);
        window.connect_close_request(move |_| {
            if let Some(this) = weak.upgrade() {
                this.cancel();
            }
            glib::Propagation::Proceed
        });
        Some(this)
    }
    pub(super) fn ready(self: &Rc<Self>, path: &Path) {
        let Some(paintable) = self.picture.upgrade().and_then(|p| p.paintable()) else {
            return;
        };
        self.pending.replace(Some((
            pivra::performance::media_id(path),
            Instant::now(),
            paintable.downgrade(),
        )));
        self.arm();
    }
    fn arm(self: &Rc<Self>) {
        if self.pending.borrow().is_none() || self.frame.borrow().is_some() {
            return;
        }
        let Some(clock) = self
            .picture
            .upgrade()
            .filter(|p| p.is_mapped())
            .and_then(|p| p.frame_clock())
        else {
            return;
        };
        let weak = Rc::downgrade(self);
        let id = clock.connect_after_paint(move |_| {
            if let Some(this) = weak.upgrade() {
                if let Some((id, start, expected)) = this.pending.borrow_mut().take() {
                    let matching = this
                        .picture
                        .upgrade()
                        .filter(|p| p.is_mapped())
                        .and_then(|p| p.paintable())
                        .is_some_and(|p| expected.upgrade().as_ref() == Some(&p));
                    pivra::performance::emit(
                        if matching {
                            "image_frame_after_paint"
                        } else {
                            "image_frame_unobserved"
                        },
                        if matching {
                            "gtk_frame_not_scanout"
                        } else {
                            "superseded"
                        },
                        start.elapsed().as_micros(),
                        id,
                    );
                }
                this.cancel();
            }
        });
        self.frame.replace(Some((clock, id)));
    }
    fn cancel(&self) {
        self.pending.borrow_mut().take();
        if let Some((clock, id)) = self.frame.borrow_mut().take() {
            clock.disconnect(id);
        }
    }
}
impl Drop for Observer {
    fn drop(&mut self) {
        self.cancel();
        if let Some(picture) = self.picture.upgrade() {
            for id in self.signals.get_mut().drain(..) {
                picture.disconnect(id);
            }
        }
    }
}
