//! One transient render-node snapshot; no CPU readback or second image decoder.
use gtk::{glib, prelude::*};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
pub struct Transition {
    enabled: Cell<bool>,
    old: gtk::Picture,
    tick: RefCell<Option<gtk::TickCallbackId>>,
}
impl Transition {
    pub fn new(overlay: &gtk::Overlay) -> Rc<Self> {
        let old = gtk::Picture::new();
        old.set_widget_name("slideshow-photo-transition");
        old.set_content_fit(gtk::ContentFit::Fill);
        old.set_can_shrink(true);
        old.set_hexpand(true);
        old.set_vexpand(true);
        old.set_can_target(false);
        old.set_visible(false);
        overlay.add_overlay(&old);
        overlay.set_measure_overlay(&old, false);
        Rc::new(Self {
            enabled: Cell::new(false),
            old,
            tick: RefCell::new(None),
        })
    }
    pub fn enable(&self, enabled: bool) {
        self.enabled.set(enabled);
        if !enabled {
            self.clear();
        }
    }
    pub fn clear(&self) {
        if let Some(id) = self.tick.borrow_mut().take() {
            id.remove();
        }
        self.old.set_paintable(None::<&gtk::gdk::Paintable>);
        self.old.set_visible(false);
    }
    pub fn capture(&self, picture: &gtk::Picture, ready: bool) {
        self.clear();
        if !self.enabled.get() || !ready || picture.width() <= 0 || picture.height() <= 0 {
            return;
        }
        if let Some(image) = picture.paintable() {
            let snapshot = gtk::Snapshot::new();
            image.snapshot(&snapshot, picture.width() as f64, picture.height() as f64);
            let paintable = snapshot.to_paintable(Some(&gtk::graphene::Size::new(
                picture.width() as f32,
                picture.height() as f32,
            )));
            self.old.set_paintable(paintable.as_ref());
        }
    }
    pub fn present(self: &Rc<Self>, success: bool) {
        if !success || !self.enabled.get() {
            self.clear();
            return;
        }
        if self.old.paintable().is_none() {
            return;
        }
        self.old.set_opacity(1.0);
        self.old.set_visible(true);
        let start = Cell::new(None);
        let weak = Rc::downgrade(self);
        let id = self.old.add_tick_callback(move |_, clock| {
            let Some(this) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let begin = start.get().unwrap_or_else(|| {
                start.set(Some(clock.frame_time()));
                clock.frame_time()
            });
            let progress = ((clock.frame_time() - begin) as f64 / 300_000.).clamp(0., 1.);
            this.old.set_opacity(1. - progress);
            if progress >= 1. {
                this.tick.borrow_mut().take();
                this.old.set_visible(false);
                this.old.set_paintable(None::<&gtk::gdk::Paintable>);
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
        self.tick.replace(Some(id));
    }
}
