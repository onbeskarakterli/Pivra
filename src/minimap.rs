//! Reuses the current texture and Transform. Frame observation never drives a clock.
use gtk::{gdk, gio, glib, prelude::*};
use pivra::{
    image_pipeline::Phase,
    viewing::{Geometry, Resampling, Transform, ViewMode, visible_region},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
pub struct Map {
    root: gtk::Overlay,
    mini: gtk::Picture,
    canvas: gtk::DrawingArea,
    picture: glib::WeakRef<gtk::Picture>,
    texture: Rc<RefCell<Option<gdk::Texture>>>,
    transform: Rc<RefCell<Transform>>,
    mode: gio::SimpleAction,
    filter: Rc<Cell<Resampling>>,
    phase: Rc<Cell<Phase>>,
    region: Cell<Option<Geometry>>,
    size: Cell<(i32, i32)>,
    slideshow: Cell<bool>,
    closed: Cell<bool>,
    clock: RefCell<Option<(glib::WeakRef<gdk::FrameClock>, glib::SignalHandlerId)>>,
}
impl Map {
    pub fn new(
        picture: &gtk::Picture,
        texture: Rc<RefCell<Option<gdk::Texture>>>,
        transform: Rc<RefCell<Transform>>,
        mode: &gio::SimpleAction,
        filter: Rc<Cell<Resampling>>,
        phase: Rc<Cell<Phase>>,
    ) -> Rc<Self> {
        let root = gtk::Overlay::new();
        root.set_widget_name("image-minimap");
        root.add_css_class("osd");
        root.set_halign(gtk::Align::End);
        root.set_valign(gtk::Align::Start);
        root.set_margin_top(12);
        root.set_margin_end(12);
        root.set_size_request(160, 120);
        root.set_tooltip_text(Some("Click or drag to pan"));
        root.set_visible(false);
        let mini = gtk::Picture::new();
        mini.set_widget_name("minimap-image");
        mini.set_can_shrink(true);
        mini.set_content_fit(gtk::ContentFit::Contain);
        mini.set_hexpand(true);
        mini.set_vexpand(true);
        mini.set_can_target(false);
        root.set_child(Some(&mini));
        let canvas = gtk::DrawingArea::new();
        canvas.set_hexpand(true);
        canvas.set_vexpand(true);
        root.add_overlay(&canvas);
        root.set_measure_overlay(&canvas, false);
        let this = Rc::new(Self {
            root,
            mini,
            canvas,
            picture: picture.downgrade(),
            texture,
            transform,
            mode: mode.clone(),
            filter,
            phase,
            region: Cell::new(None),
            size: Cell::new((0, 0)),
            slideshow: Cell::new(false),
            closed: Cell::new(false),
            clock: RefCell::new(None),
        });
        let weak = Rc::downgrade(&this);
        this.canvas.set_draw_func(move |_, cr, width, height| {
            let Some(this) = weak.upgrade() else {
                return;
            };
            let Some(region) = this.region.get() else {
                return;
            };
            let (iw, ih) = this.size.get();
            let Some(image) =
                ViewMode::Fit.geometry((iw as f64, ih as f64), (width as f64, height as f64))
            else {
                return;
            };
            let x = image.x + image.width * region.x;
            let y = image.y + image.height * region.y;
            let w = image.width * region.width;
            let h = image.height * region.height;
            cr.rectangle(x, y, w, h);
            cr.set_source_rgba(0., 0., 0., 0.8);
            cr.set_line_width(3.);
            let _ = cr.stroke_preserve();
            cr.set_source_rgba(1., 1., 1., 0.95);
            cr.set_line_width(1.5);
            let _ = cr.stroke();
        });
        let drag = gtk::GestureDrag::new();
        drag.set_button(1);
        let origin = Rc::new(Cell::new((0., 0.)));
        let start = origin.clone();
        let weak = Rc::downgrade(&this);
        drag.connect_drag_begin(move |gesture, x, y| {
            start.set((x, y));
            gesture.set_state(gtk::EventSequenceState::Claimed);
            if let Some(this) = weak.upgrade() {
                this.pan(x, y);
            }
        });
        let weak = Rc::downgrade(&this);
        drag.connect_drag_update(move |_, dx, dy| {
            if let Some(this) = weak.upgrade() {
                let (x, y) = origin.get();
                this.pan(x + dx, y + dy);
            }
        });
        this.root.add_controller(drag);
        let weak = Rc::downgrade(&this);
        picture.connect_map(move |_| {
            if let Some(this) = weak.upgrade() {
                this.bind_clock();
            }
        });
        let weak = Rc::downgrade(&this);
        picture.connect_unmap(move |_| {
            if let Some(this) = weak.upgrade() {
                this.disconnect();
                this.hide();
            }
        });
        if picture.is_mapped() {
            this.bind_clock();
        }
        this
    }
    pub fn widget(&self) -> &gtk::Overlay {
        &self.root
    }
    fn disconnect(&self) {
        if let Some((weak, id)) = self.clock.borrow_mut().take() {
            if let Some(clock) = weak.upgrade() {
                clock.disconnect(id);
            }
        }
    }
    fn bind_clock(self: &Rc<Self>) {
        self.disconnect();
        let Some(clock) = self.picture.upgrade().and_then(|p| p.frame_clock()) else {
            return;
        };
        let weak = Rc::downgrade(self);
        let id = clock.connect_after_paint(move |_| {
            if let Some(this) = weak.upgrade() {
                this.refresh();
            }
        });
        self.clock.replace(Some((clock.downgrade(), id)));
        self.refresh();
    }
    pub fn hide(&self) {
        self.root.set_visible(false);
        self.mini.set_paintable(None::<&gdk::Paintable>);
        self.region.set(None);
    }
    pub fn set_slideshow(&self, active: bool) {
        self.slideshow.set(active);
        if active {
            self.hide();
        } else {
            self.refresh();
        }
    }
    pub fn close(&self) {
        self.closed.set(true);
        self.disconnect();
        self.hide();
    }
    fn refresh(&self) {
        if self.closed.get() || self.slideshow.get() || self.phase.get() != Phase::FullReady {
            self.hide();
            return;
        }
        let Some(picture) = self.picture.upgrade().filter(|p| p.is_mapped()) else {
            self.hide();
            return;
        };
        let Some(texture) = self.texture.borrow().clone() else {
            self.hide();
            return;
        };
        let mode = self
            .mode
            .state()
            .and_then(|v| v.str().and_then(ViewMode::from_id))
            .unwrap_or(ViewMode::Actual);
        let viewport = (picture.width() as f64, picture.height() as f64);
        let image = (texture.width() as f64, texture.height() as f64);
        let region = self
            .transform
            .borrow_mut()
            .geometry(image, viewport, mode)
            .and_then(|g| visible_region(g, viewport));
        let Some(region) = region else {
            self.hide();
            return;
        };
        let changed = self.region.replace(Some(region)) != Some(region);
        self.size.set((texture.width(), texture.height()));
        let paintable = texture.upcast_ref::<gdk::Paintable>();
        if self.mini.paintable().as_ref() != Some(paintable) {
            self.mini.set_paintable(Some(paintable));
        }
        if !self.root.is_visible() {
            self.root.set_visible(true);
            self.canvas.queue_draw();
        } else if changed {
            self.canvas.queue_draw();
        }
    }
    fn pan(&self, x: f64, y: f64) {
        if !x.is_finite() || !y.is_finite() || !self.root.is_visible() {
            return;
        }
        let Some(picture) = self.picture.upgrade() else {
            return;
        };
        let Some(texture) = self.texture.borrow().clone() else {
            return;
        };
        let image = (texture.width() as f64, texture.height() as f64);
        let viewport = (picture.width() as f64, picture.height() as f64);
        let Some(map) = ViewMode::Fit.geometry(
            image,
            (self.canvas.width() as f64, self.canvas.height() as f64),
        ) else {
            return;
        };
        let point = (
            ((x - map.x) / map.width).clamp(0., 1.),
            ((y - map.y) / map.height).clamp(0., 1.),
        );
        let mode = self
            .mode
            .state()
            .and_then(|v| v.str().and_then(ViewMode::from_id))
            .unwrap_or(ViewMode::Actual);
        let Some(g) = self.transform.borrow_mut().geometry(image, viewport, mode) else {
            return;
        };
        if self.transform.borrow_mut().pan(
            image,
            viewport,
            mode,
            (
                viewport.0 / 2. - point.0 * g.width - g.x,
                viewport.1 / 2. - point.1 * g.height - g.y,
            ),
        ) {
            super::manual(
                &picture,
                &texture,
                self.transform.clone(),
                mode,
                &self.mode,
                self.filter.clone(),
            );
            self.refresh();
        }
    }
}
