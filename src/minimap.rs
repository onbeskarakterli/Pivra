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
        root.set_hexpand(false);
        root.set_vexpand(false);
        root.set_halign(gtk::Align::End);
        root.set_valign(gtk::Align::End);
        root.set_margin_bottom(16);
        root.set_margin_end(16);
        root.set_opacity(0.88);
        root.set_tooltip_text(Some("Click or drag to pan"));
        root.set_visible(false);
        let mini = gtk::Picture::new();
        mini.set_widget_name("minimap-image");
        mini.set_can_shrink(true);
        mini.set_content_fit(gtk::ContentFit::Contain);
        mini.set_can_target(false);
        // Unmeasured overlay children cannot propagate the full texture's
        // natural dimensions. The root request below controls the floating box.
        root.add_overlay(&mini);
        root.set_measure_overlay(&mini, false);
        root.set_clip_overlay(&mini, true);
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
        for mapped in [true, false] {
            let weak = Rc::downgrade(&this);
            let sync = move |_: &gtk::Overlay| {
                if let Some(this) = weak.upgrade() {
                    this.sync_navigation();
                }
            };
            if mapped {
                this.root.connect_map(sync);
            } else {
                this.root.connect_unmap(sync);
            }
        }
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
            cr.set_line_width(2.);
            let _ = cr.stroke_preserve();
            cr.set_source_rgba(1., 1., 1., 0.95);
            cr.set_line_width(1.);
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
    fn is_navigation(widget: &gtk::Widget) -> bool {
        matches!(
            widget.widget_name().as_str(),
            "thumbnail-strip" | "thumbnail-controls" | "thumbnail-resize-edge"
        ) || widget
            .downcast_ref::<gtk::Button>()
            .and_then(|b| b.action_name())
            .is_some_and(|a| matches!(a.as_str(), "win.previous-media" | "win.next-media"))
    }
    fn sync_navigation(&self) {
        let Some(parent) = self.root.parent() else {
            return;
        };
        // Container-level mask: each control still owns its normal `visible`
        // flag (hover, pin, enabled, etc.). No preference or timer is changed.
        // Map/unmap makes the mask effective before either UI can be painted.
        let show = !self.root.is_mapped();
        let mut child = parent.first_child();
        while let Some(widget) = child {
            child = widget.next_sibling();
            if Self::is_navigation(&widget) {
                widget.set_child_visible(show);
            }
        }
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
    /// Placement uses allocated UI bounds only; image transforms still use the
    /// full Picture viewport for both the indicator and interactive panning.
    fn place(&self, picture: &gtk::Picture, image: (f64, f64)) -> bool {
        let Some(parent) = picture.parent() else {
            return false;
        };
        let Some(photo) = picture.compute_bounds(&parent) else {
            return false;
        };
        let (width, height) = (picture.width() as f64, picture.height() as f64);
        let (mut left, mut top, mut right, mut bottom) =
            (16_f64, 16_f64, width - 16., height - 16.);
        let mut reserve = |widget: &gtk::Widget, vertical: bool| {
            if !widget.is_visible() || !widget.is_mapped() {
                return;
            }
            let Some(b) = widget.compute_bounds(picture) else {
                return;
            };
            let (x, y, w, h) = (
                b.x() as f64,
                b.y() as f64,
                b.width() as f64,
                b.height() as f64,
            );
            if w <= 0. || h <= 0. || x >= width || y >= height || x + w <= 0. || y + h <= 0. {
                return;
            }
            if vertical {
                if x + w / 2. < width / 2. {
                    left = left.max(x + w + 16.);
                } else {
                    right = right.min(x - 16.);
                }
            } else if y + h / 2. < height / 2. {
                top = top.max(y + h + 16.);
            } else {
                bottom = bottom.min(y - 16.);
            }
        };
        // Inspect siblings/scroller bars, not every thumbnail cell on each frame.
        // Navigation is mutually exclusive with the map, so it does not reduce
        // the map's usable area even on the first frame before masking it.
        let mut child = parent.first_child();
        while let Some(widget) = child {
            child = widget.next_sibling();
            if Self::is_navigation(&widget) {
                continue;
            }
            if let Some(bar) = widget.downcast_ref::<gtk::Scrollbar>() {
                reserve(&widget, bar.orientation() == gtk::Orientation::Vertical);
            }
            if let Some(scroll) = widget.downcast_ref::<gtk::ScrolledWindow>() {
                reserve(scroll.hscrollbar().upcast_ref(), false);
                reserve(scroll.vscrollbar().upcast_ref(), true);
            }
        }
        let available = ((right - left).min(560.), (bottom - top).min(440.));
        if available.0 < 1. || available.1 < 1. {
            return false;
        }
        let scale = (available.0 / image.0).min(available.1 / image.1);
        self.root.set_size_request(
            (image.0 * scale).floor().max(1.) as i32,
            (image.1 * scale).floor().max(1.) as i32,
        );
        self.root
            .set_margin_start((photo.x() as f64 + left).ceil() as i32);
        self.root
            .set_margin_top((photo.y() as f64 + top).ceil() as i32);
        self.root
            .set_margin_end((parent.width() as f64 - photo.x() as f64 - right).ceil() as i32);
        self.root
            .set_margin_bottom((parent.height() as f64 - photo.y() as f64 - bottom).ceil() as i32);
        true
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
        if !self.place(&picture, image) {
            self.hide();
            return;
        }
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
