//! One frame-clock transition layer, retaining rendered nodes, never pixel copies.
use gtk::{glib, prelude::*};
use pivra::slideshow::Effect;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub struct Transition {
    enabled: Cell<bool>,
    effect: Cell<Effect>,
    old: gtk::Picture,
    picture: glib::WeakRef<gtk::Picture>,
    source: gtk::WidgetPaintable,
    captured: RefCell<Option<gtk::gsk::RenderNode>>,
    size: Cell<(i32, i32)>,
    tick: RefCell<Option<gtk::TickCallbackId>>,
    waiting: RefCell<Option<(gtk::gdk::FrameClock, glib::SignalHandlerId)>>,
    idle: RefCell<Option<glib::SourceId>>,
}
impl Transition {
    pub fn new(overlay: &gtk::Overlay) -> Rc<Self> {
        let picture = overlay.child().unwrap().downcast::<gtk::Picture>().unwrap();
        let old = gtk::Picture::new();
        old.set_widget_name("slideshow-photo-transition");
        old.set_content_fit(gtk::ContentFit::Contain);
        old.set_can_shrink(true);
        old.set_hexpand(true);
        old.set_vexpand(true);
        old.set_can_target(false);
        old.set_visible(false);
        overlay.add_overlay(&old);
        overlay.set_measure_overlay(&old, false);
        Rc::new(Self {
            enabled: Cell::new(false),
            effect: Cell::new(Effect::Fade),
            picture: picture.downgrade(),
            source: gtk::WidgetPaintable::new(None::<&gtk::Widget>),
            old,
            captured: RefCell::new(None),
            size: Cell::new((0, 0)),
            tick: RefCell::new(None),
            waiting: RefCell::new(None),
            idle: RefCell::new(None),
        })
    }
    pub fn enable(&self, enabled: bool) {
        self.enabled.set(enabled);
        if !enabled {
            self.clear();
        }
        self.update_source();
    }
    fn update_source(&self) {
        let picture = self
            .picture
            .upgrade()
            .filter(|_| self.enabled.get() && self.effect.get() != Effect::None);
        self.source.set_widget(picture.as_ref());
    }
    pub fn shutdown(&self) {
        self.enable(false);
    }
    pub fn set_effect(&self, effect: Effect) {
        self.effect.set(effect);
        self.clear();
        self.update_source();
    }
    pub fn clear(&self) {
        if let Some(id) = self.tick.borrow_mut().take() {
            id.remove();
        }
        if let Some((clock, id)) = self.waiting.borrow_mut().take() {
            clock.disconnect(id);
        }
        if let Some(id) = self.idle.borrow_mut().take() {
            id.remove();
        }
        self.captured.borrow_mut().take();
        self.old.set_paintable(None::<&gtk::gdk::Paintable>);
        self.old.set_visible(false);
    }
    #[expect(
        deprecated,
        reason = "GTK 4.16 compatibility: Widget::lookup_color requires GTK 4.20; keep the public named-theme lookup instead of hardcoded colors"
    )]
    fn background(&self) -> gtk::gdk::RGBA {
        self.old
            .style_context()
            .lookup_color("window_bg_color")
            .unwrap_or(gtk::gdk::RGBA::BLACK)
    }
    fn snapshot(&self) -> Option<gtk::gsk::RenderNode> {
        let (w, h) = self.size.get();
        let image = self.source.current_image();
        if image.intrinsic_width() != w || image.intrinsic_height() != h {
            return None;
        }
        let snapshot = gtk::Snapshot::new();
        image.snapshot(&snapshot, f64::from(w), f64::from(h));
        let node = snapshot.to_node()?;
        let snapshot = gtk::Snapshot::new();
        snapshot.append_color(
            &self.background(),
            &gtk::graphene::Rect::new(0., 0., w as f32, h as f32),
        );
        snapshot.append_node(&node);
        snapshot.to_node()
    }
    fn display(&self, node: &gtk::gsk::RenderNode) {
        let (w, h) = self.size.get();
        let snapshot = gtk::Snapshot::new();
        snapshot.append_node(node);
        self.old.set_paintable(
            snapshot
                .to_paintable(Some(&gtk::graphene::Size::new(w as f32, h as f32)))
                .as_ref(),
        );
    }
    pub fn capture(&self, picture: &gtk::Picture, ready: bool) {
        self.clear();
        if !self.enabled.get()
            || self.effect.get() == Effect::None
            || !ready
            || picture.width() <= 0
            || picture.height() <= 0
        {
            return;
        }
        self.size.set((picture.width(), picture.height()));
        if let Some(node) = self.snapshot() {
            self.display(&node);
            self.captured.replace(Some(node));
        }
    }
    pub fn present(self: &Rc<Self>, success: bool) {
        if !success || !self.enabled.get() {
            self.clear();
            return;
        }
        if self.captured.borrow().is_none() {
            return;
        }
        self.old.set_opacity(1.);
        self.old.set_visible(true);
        if self.effect.get() == Effect::Fade {
            self.animate(None);
            return;
        }
        let Some(picture) = self.picture.upgrade() else {
            self.clear();
            return;
        };
        let Some(clock) = picture.frame_clock() else {
            self.clear();
            return;
        };
        let weak = Rc::downgrade(self);
        let id = clock.connect_after_paint(move |_| {
            let Some(this) = weak.upgrade() else {
                return;
            };
            if let Some((clock, id)) = this.waiting.borrow_mut().take() {
                clock.disconnect(id);
            }
            // WidgetPaintable publishes the freshly rendered image in GTK's
            // HIGH-priority idle. Run after that publication, without a sleep.
            let weak = Rc::downgrade(&this);
            let id = glib::idle_add_local_once(move || {
                let Some(this) = weak.upgrade() else {
                    return;
                };
                this.idle.borrow_mut().take();
                if this
                    .picture
                    .upgrade()
                    .is_none_or(|p| (p.width(), p.height()) != this.size.get())
                {
                    this.clear();
                    return;
                }
                if let Some(node) = this.snapshot() {
                    this.animate(Some(node));
                } else {
                    this.clear();
                }
            });
            this.idle.replace(Some(id));
        });
        self.waiting.replace(Some((clock, id)));
        picture.queue_draw();
    }
    fn animate(self: &Rc<Self>, new: Option<gtk::gsk::RenderNode>) {
        let Some(old) = self.captured.borrow().clone() else {
            return;
        };
        let start = Cell::new(None);
        let weak = Rc::downgrade(self);
        let effect = self.effect.get();
        let id = self.old.add_tick_callback(move |_, clock| {
            let Some(this) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let begin = start.get().unwrap_or_else(|| {
                start.set(Some(clock.frame_time()));
                clock.frame_time()
            });
            let p = ((clock.frame_time() - begin) as f64 / 300_000.).clamp(0., 1.);
            if p >= 1.
                || this
                    .picture
                    .upgrade()
                    .is_none_or(|pic| (pic.width(), pic.height()) != this.size.get())
            {
                this.tick.borrow_mut().take();
                this.clear();
                return glib::ControlFlow::Break;
            }
            if effect == Effect::Fade {
                this.old.set_opacity(1. - p);
            } else if let Some(new) = &new {
                this.display(&this.compose(&old, new, effect, p));
            }
            glib::ControlFlow::Continue
        });
        self.tick.replace(Some(id));
    }
    fn compose(
        &self,
        old: &gtk::gsk::RenderNode,
        new: &gtk::gsk::RenderNode,
        effect: Effect,
        p: f64,
    ) -> gtk::gsk::RenderNode {
        let (w, h) = self.size.get();
        let (w, h) = (w as f32, h as f32);
        let snapshot = gtk::Snapshot::new();
        snapshot.push_clip(&gtk::graphene::Rect::new(0., 0., w, h));
        snapshot.append_color(&self.background(), &gtk::graphene::Rect::new(0., 0., w, h));
        let draw = |node: &gtk::gsk::RenderNode, x: f32, y: f32, scale: f32, alpha: f64| {
            snapshot.save();
            snapshot.translate(&gtk::graphene::Point::new(x + w / 2., y + h / 2.));
            snapshot.scale(scale, scale);
            snapshot.translate(&gtk::graphene::Point::new(-w / 2., -h / 2.));
            snapshot.push_opacity(alpha);
            snapshot.append_node(node);
            snapshot.pop();
            snapshot.restore();
        };
        let p = p as f32;
        match effect {
            Effect::SlideLeft | Effect::SlideRight | Effect::SlideUp | Effect::SlideDown => {
                let (x, y) = match effect {
                    Effect::SlideLeft => (-w, 0.),
                    Effect::SlideRight => (w, 0.),
                    Effect::SlideUp => (0., -h),
                    _ => (0., h),
                };
                draw(old, x * p, y * p, 1., 1.);
                draw(new, -x * (1. - p), -y * (1. - p), 1., 1.);
            }
            Effect::ZoomIn | Effect::ZoomOut => {
                draw(old, 0., 0., 1., 1.);
                let scale = if effect == Effect::ZoomIn {
                    0.85 + 0.15 * p
                } else {
                    1.15 - 0.15 * p
                };
                draw(new, 0., 0., scale, f64::from(p));
            }
            _ => {
                draw(new, 0., 0., 1., 1.);
                draw(old, 0., 0., 1., f64::from(1. - p));
            }
        }
        snapshot.pop();
        snapshot.to_node().expect("bounded transition layers")
    }
}
