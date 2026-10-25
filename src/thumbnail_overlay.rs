//! Floating thumbnail controls; shared Enabled preference, per-window visibility.
use gtk::prelude::*;
use gtk::{gdk, glib};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};
thread_local! { static CSS_INSTALLED: Cell<bool> = const { Cell::new(false) }; }
pub(crate) const CELL_WIDTH: i32 = 112;
pub(crate) const CELL_HEIGHT: i32 = 94;
fn install_css(display: &gtk::gdk::Display) {
    CSS_INSTALLED.with(|installed| {
        if installed.replace(true) { return; }
        let css=gtk::CssProvider::new();
        css.load_from_string("\
            .pivra-thumbnail-panel { background-color: alpha(@window_bg_color, 0.50); border-radius: 0; border-top: 1px solid alpha(white, 0.22); padding: 8px; }\n\
            .pivra-thumbnail-toggle { background-image: none; background-color: alpha(@window_bg_color, 0.50); border-radius: 10px 10px 0 0; border: 1px solid alpha(white, 0.22); border-bottom: 0; box-shadow: none; padding: 3px 16px; min-width: 112px; min-height: 24px; }\n\
            .pivra-thumbnail-pin { background-image: none; background-color: alpha(@window_bg_color, 0.50); border-radius: 10px 10px 0 0; border: 1px solid alpha(white, 0.22); border-bottom: 0; box-shadow: none; padding: 3px 8px; min-height: 24px; }\n\
            .pivra-thumbnail-pin.pivra-pinned { background-color: alpha(@accent_bg_color, 0.85); color: @accent_fg_color; border-color: alpha(@accent_color, 0.95); }\n\
            .pivra-thumbnail-panel scrollbar.horizontal { opacity: 0; }\n\\
            .pivra-thumbnail-cell { background-image: none; background-color: transparent; padding: 0; min-width: 0; min-height: 0; border: none; border-radius: 0; box-shadow: none; }\n\
            .pivra-thumbnail-cell.selected { box-shadow: none; }\n\
            .pivra-thumbnail-selection { background: transparent; border: 0; padding: 0; box-shadow: none; }\n\
            .pivra-thumbnail-selection.selected { box-shadow: inset 0 0 0 2px @accent_color; }\n");
        gtk::style_context_add_provider_for_display(display,&css,gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    });
}
/// Paint-only selection outline never changes selected/unselected allocation.
#[allow(dead_code)] // Cell content is supplied only by the explicit TH-02 probe for now.
pub(crate) fn cell(child: &impl IsA<gtk::Widget>, selected: bool) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.add_css_class("pivra-thumbnail-cell");
    if selected {
        frame.add_css_class("selected");
    }
    frame.set_size_request(CELL_WIDTH, CELL_HEIGHT);
    frame.set_valign(gtk::Align::Center);
    // Overlay content cannot enlarge the fixed cell's natural request.
    let content = gtk::Overlay::new();
    content.add_overlay(child);
    content.set_measure_overlay(child, false);
    content.set_clip_overlay(child, true);
    frame.set_child(Some(&content));
    frame
}
#[path = "thumbnail_strip.rs"]
#[allow(dead_code)] // Standalone ImageView probes exercise only subsets of window integration.
mod strip;
pub(crate) use strip::{ThumbnailStats, ThumbnailStrip};
pub(crate) fn install(
    window: &impl IsA<gtk::Window>,
    overlay: &gtk::Overlay,
    available: bool,
) -> ThumbnailStrip {
    install_css(&overlay.display());
    let items = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    items.set_widget_name("thumbnail-items");
    items.append(&gtk::Label::new(Some("No thumbnails available")));
    let strip = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .overlay_scrolling(true)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .propagate_natural_width(false)
        .height_request(128)
        .child(&items)
        .build();
    strip.set_widget_name("thumbnail-strip");
    strip.add_css_class("pivra-thumbnail-panel");
    strip.set_margin_start(0);
    strip.set_margin_end(0);
    strip.set_visible(false);
    strip.set_min_content_width(0);
    // An invisible, full-width resize hit area covers the strip's top edge.
    // It is a separate overlay so horizontal thumbnail gestures stay intact.
    let resize_edge = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    resize_edge.set_hexpand(true);
    resize_edge.set_halign(gtk::Align::Fill);
    resize_edge.set_valign(gtk::Align::End);
    resize_edge.set_height_request(10);
    resize_edge.set_can_target(false);
    resize_edge.set_visible(false);
    resize_edge.set_cursor_from_name(Some("ns-resize"));
    let button = gtk::ToggleButton::builder()
        .icon_name("pan-up-symbolic")
        .tooltip_text("Show thumbnail strip")
        .halign(gtk::Align::Center)
        .build();
    button.set_widget_name("thumbnail-toggle");
    button.add_css_class("pivra-thumbnail-toggle");
    // Independent overlay siblings: no full-width box beneath the strip that
    // can paint a dock background or intercept input around the small button.
    button.set_valign(gtk::Align::End);
    button.set_margin_bottom(0);
    button.set_visible(available);
    let pin = gtk::ToggleButton::builder()
        .icon_name("bookmark-new-symbolic")
        .tooltip_text("Keep thumbnail strip visible")
        .build();
    pin.add_css_class("pivra-thumbnail-pin");
    pin.set_active(false);
    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    controls.set_halign(gtk::Align::Center);
    controls.set_valign(gtk::Align::End);
    controls.append(&button);
    controls.append(&pin);
    controls.set_visible(available);
    strip.set_valign(gtk::Align::End);
    // The panel touches the image viewport edge; neither sibling is measured.
    strip.set_margin_bottom(0);
    for widget in [strip.upcast_ref::<gtk::Widget>(), controls.upcast_ref()] {
        overlay.add_overlay(widget);
        overlay.set_measure_overlay(widget, false);
        overlay.set_clip_overlay(widget, true);
    }
    // Added last so the edge receives pointer events above the scroller.
    overlay.add_overlay(&resize_edge);
    overlay.set_measure_overlay(&resize_edge, false);
    overlay.set_clip_overlay(&resize_edge, true);
    let thumbnails = ThumbnailStrip::new(items, strip.clone());
    let saved_height = pivra::settings::load_thumbnail_height();
    strip.set_height_request(saved_height);
    thumbnails.resize_cells(saved_height);
    // When the window is maximized/fullscreen, prewarm newly exposed cells
    // even if the thumbnail strip has never been revealed.
    let resize_preload = thumbnails.clone();
    window.connect_notify_local(Some("width"), move |_, _| {
        resize_preload.refresh_hidden();
    });
    let preference = pivra::settings::thumbnail_preference();
    let saved_pinned = pivra::settings::load_thumbnail_pinned();
    if saved_pinned && !preference.enabled() {
        preference.set_enabled(true);
    }
    pin.set_active(saved_pinned);
    if saved_pinned {
        pin.add_css_class("pivra-pinned");
        pin.set_tooltip_text(Some("Unpin thumbnail strip"));
    }
    let state = Rc::new(Visibility {
        available,
        preference: preference.clone(),
        strip: strip.downgrade(),
        resize_edge: resize_edge.downgrade(),
        resizing: Cell::new(false),
        button: button.downgrade(),
        pin: pin.downgrade(),
        controls: controls.downgrade(),
        pinned: Cell::new(saved_pinned),
        overlay: overlay.downgrade(),
        near: Cell::new(false),
        pointer_y: Cell::new(None),
        allocated_height: Cell::new(-1),
        closed: Cell::new(false),
        layout: RefCell::new(None),
        shown: Cell::new(false),
        timeout: RefCell::new(None),
        listener: RefCell::new(None),
    });
    let weak = Rc::downgrade(&state);
    let listener: Rc<dyn Fn(bool)> = Rc::new(move |_| {
        if let Some(state) = weak.upgrade() {
            state.cancel();
            state.render(state.pinned.get() || state.near.get());
            // Enabled also changes the height of the interaction region.
            if let Some(y) = state.pointer_y.get() {
                state.motion(y);
            }
        }
    });
    preference.subscribe(&listener);
    state.listener.replace(Some(listener));
    state.render(saved_pinned);
    let click = state.clone();
    button.connect_clicked(move |_| {
        if click.available {
            click.preference.set_enabled(!click.preference.enabled());
        }
    });
    let pin_click = state.clone();
    pin.connect_clicked(move |_| {
        let pinned = !pin_click.pinned.get();
        pin_click.pinned.set(pinned);
        pivra::settings::save_thumbnail_pinned(pinned);
        pin_click.cancel();
        if pinned && !pin_click.preference.enabled() {
            pin_click.preference.set_enabled(true);
        }
        if let Some(pin) = pin_click.pin.upgrade() {
            pin.set_active(pinned);
            pin.set_icon_name(if pinned { "bookmark-new-symbolic" } else { "bookmark-new-symbolic" });
            if pinned { pin.add_css_class("pivra-pinned"); } else { pin.remove_css_class("pivra-pinned"); }
            pin.set_tooltip_text(Some(if pinned {
                "Unpin thumbnail strip"
            } else {
                "Keep thumbnail strip visible"
            }));
        }
        pin_click.render(pinned || pin_click.near.get() || pin_click.shown.get());
        if !pinned && !pin_click.near.get() {
            pin_click.schedule(3000);
        }
    });
    let resize = gtk::GestureDrag::new();
    resize.set_button(1);
    let start_height = Rc::new(Cell::new(saved_height));
    let begin_state = state.clone();
    let begin_height = start_height.clone();
    resize.connect_drag_begin(move |gesture, _, y| {
        let Some(strip) = begin_state.strip.upgrade() else { return; };
        let Some(overlay) = begin_state.overlay.upgrade() else { return; };
        let top = f64::from(overlay.height() - strip.height_request());
        if !begin_state.shown.get() || !begin_state.preference.enabled()
            || y < top - 10.0 || y > top + 10.0
        {
            gesture.set_state(gtk::EventSequenceState::Denied);
            return;
        }
        begin_height.set(strip.height_request().max(128));
        begin_state.resizing.set(true);
        begin_state.cancel();
        gesture.set_state(gtk::EventSequenceState::Claimed);
    });
    let drag_state = state.clone();
    let resize_thumbnails = thumbnails.clone();
    resize.connect_drag_update(move |gesture, _, delta_y| {
        if !drag_state.resizing.get() || !delta_y.is_finite() { return; }
        if let (Some(strip), Some(overlay)) = (drag_state.strip.upgrade(), drag_state.overlay.upgrade()) {
            let ceiling = (overlay.height() - 48).clamp(128, 560);
            let height = (f64::from(start_height.get()) - delta_y).round()
                .clamp(128.0, f64::from(ceiling)) as i32;
            if strip.height_request() != height {
                strip.set_height_request(height);
                resize_thumbnails.resize_cells(height);
                drag_state.place_toggle();
            }
            gesture.set_state(gtk::EventSequenceState::Claimed);
        }
    });
    let end_state = state.clone();
    resize.connect_drag_end(move |_, _, _| {
        if end_state.resizing.get() {
            if let Some(strip) = end_state.strip.upgrade() {
                pivra::settings::save_thumbnail_height(strip.height_request());
            }
        }
        end_state.resizing.set(false);
        end_state.place_toggle();
        if !end_state.pinned.get() && !end_state.near.get() { end_state.schedule(3000); }
    });
    let cancel_state = state.clone();
    resize.connect_cancel(move |_, _| {
        cancel_state.resizing.set(false);
        if !cancel_state.pinned.get() && !cancel_state.near.get() { cancel_state.schedule(3000); }
    });
    resize.set_propagation_phase(gtk::PropagationPhase::Capture);
    overlay.add_controller(resize);
    let motion = gtk::EventControllerMotion::new();
    motion.set_propagation_phase(gtk::PropagationPhase::Capture);
    let enter = state.clone();
    motion.connect_enter(move |_, _, y| enter.motion(y));
    let moving = state.clone();
    motion.connect_motion(move |_, _, y| moving.motion(y));
    let leave = state.clone();
    motion.connect_leave(move |_| {
        leave.pointer_y.set(None);
        leave.update_resize_cursor(None);
        leave.away();
    });
    overlay.add_controller(motion);
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let key_state = state.clone();
    let weak_window = window.downgrade();
    keys.connect_key_pressed(move |_, key, _, modifiers| {
        if !key_state.available
            || !matches!(key, gdk::Key::t | gdk::Key::T)
            || modifiers.intersects(
                gdk::ModifierType::CONTROL_MASK
                    | gdk::ModifierType::ALT_MASK
                    | gdk::ModifierType::SUPER_MASK
                    | gdk::ModifierType::META_MASK,
            )
        {
            return glib::Propagation::Proceed;
        }
        if let Some(window) = weak_window.upgrade() {
            let mut focus = gtk::prelude::GtkWindowExt::focus(&window);
            while let Some(widget) = focus {
                if widget.is::<gtk::Editable>() || widget.is::<gtk::TextView>() {
                    return glib::Propagation::Proceed;
                }
                focus = widget.parent();
            }
        }
        key_state
            .preference
            .set_enabled(!key_state.preference.enabled());
        if key_state.preference.enabled() {
            key_state.render(true);
            if !key_state.near.get() {
                key_state.schedule(3000);
            }
        }
        glib::Propagation::Stop
    });
    window.as_ref().add_controller(keys);
    let mapped = state.clone();
    let resize_preload = thumbnails.clone();
    overlay.connect_map(move |overlay| {
        if let Some(clock) = overlay.frame_clock() {
            let weak = Rc::downgrade(&mapped);
            let preload = resize_preload.clone();
            let previous_width = Rc::new(Cell::new(-1i32));
            let width_state = previous_width.clone();
            let handler = clock.connect_after_paint(move |_| {
                if let Some(state) = weak.upgrade() {
                    state.place_toggle();
                    if let Some(overlay) = state.overlay.upgrade() {
                        let width = overlay.width();
                        if width > 0 && width_state.replace(width) != width {
                            preload.refresh_hidden();
                        }
                        let height = overlay.height();
                        if state.allocated_height.replace(height) != height {
                            if let Some(y) = state.pointer_y.get() {
                                state.motion(y);
                            }
                        }
                    }
                }
            });
            mapped.layout.replace(Some((clock, handler)));
        }
    });
    let closing = state.clone();
    window.connect_close_request(move |_| {
        closing.closed.set(true);
        closing.disconnect_layout();
        closing.cancel();
        closing.render(false);
        glib::Propagation::Proceed
    });
    let unmap = state;
    overlay.connect_unmap(move |_| {
        unmap.pointer_y.set(None);
        unmap.disconnect_layout();
        unmap.near.set(false);
        unmap.cancel();
        unmap.render(false);
    });
    thumbnails
}
type PreferenceListener = Rc<dyn Fn(bool)>;
struct Visibility {
    available: bool,
    preference: Rc<pivra::settings::ThumbnailPreference>,
    strip: glib::WeakRef<gtk::ScrolledWindow>,
    resize_edge: glib::WeakRef<gtk::Box>,
    resizing: Cell<bool>,
    button: glib::WeakRef<gtk::ToggleButton>,
    pin: glib::WeakRef<gtk::ToggleButton>,
    controls: glib::WeakRef<gtk::Box>,
    pinned: Cell<bool>,
    overlay: glib::WeakRef<gtk::Overlay>,
    near: Cell<bool>,
    pointer_y: Cell<Option<f64>>,
    allocated_height: Cell<i32>,
    closed: Cell<bool>,
    layout: RefCell<Option<(gdk::FrameClock, glib::SignalHandlerId)>>,
    shown: Cell<bool>,
    timeout: RefCell<Option<glib::SourceId>>,
    listener: RefCell<Option<PreferenceListener>>,
}
impl Visibility {
    fn disconnect_layout(&self) {
        if let Some((clock, handler)) = self.layout.borrow_mut().take() {
            clock.disconnect(handler);
        }
    }
    fn cancel(&self) {
        if let Some(id) = self.timeout.borrow_mut().take() {
            id.remove();
        }
    }
    fn render(&self, show: bool) {
        let show = (show || self.pinned.get() || self.resizing.get()) && self.available && !self.closed.get();
        self.shown.set(show);
        if let Some(strip) = self.strip.upgrade() {
            strip.set_visible(show && self.preference.enabled());
        }
        if let Some(button) = self.button.upgrade() {
            let enabled = self.preference.enabled();
            button.set_active(enabled);
            let icon = if enabled {
                "pan-down-symbolic"
            } else {
                "pan-up-symbolic"
            };
            if button.icon_name().as_deref() != Some(icon) {
                button.set_icon_name(icon);
            }
            button.set_tooltip_text(Some(if enabled {
                "Disable thumbnail strip"
            } else {
                "Enable thumbnail strip"
            }));
            // Above the visible panel; otherwise only the small bottom trigger.
            button.set_visible(show);
        }
        if let Some(controls) = self.controls.upgrade() {
            controls.set_visible(show);
        }
        if let Some(edge) = self.resize_edge.upgrade() {
            edge.set_visible(show && self.preference.enabled());
        }
        if !show { self.update_resize_cursor(None); }
        self.place_toggle();
    }
    fn place_toggle(&self) {
        let (Some(panel), Some(button), Some(overlay)) = (
            self.strip.upgrade(),
            self.controls.upgrade(),
            self.overlay.upgrade(),
        ) else {
            return;
        };
        let margin = if panel.is_visible() {
            let height = if panel.is_mapped() {
                panel
                    .compute_bounds(&overlay)
                    .filter(|b| {
                        b.height() > 0.
                            && b.y() >= 0.
                            && (b.y() + b.height() - overlay.height() as f32).abs() <= 1.
                    })
                    .map(|b| (overlay.height() as f32 - b.y()).ceil() as i32)
            } else {
                None
            };
            height.unwrap_or_else(|| panel.measure(gtk::Orientation::Vertical, -1).1)
        } else {
            0
        };
        if button.margin_bottom() != margin {
            button.set_margin_bottom(margin);
        }
        if let Some(edge) = self.resize_edge.upgrade() {
            let edge_margin = (margin - 5).max(0);
            if edge.margin_bottom() != edge_margin {
                edge.set_margin_bottom(edge_margin);
            }
        }
    }
    fn update_resize_cursor(&self, y: Option<f64>) {
        let (Some(overlay), Some(strip)) = (self.overlay.upgrade(), self.strip.upgrade()) else { return; };
        let top = f64::from(overlay.height() - strip.height_request());
        let on_border = self.shown.get() && self.preference.enabled()
            && y.is_some_and(|position| (position - top).abs() <= 10.0);
        let cursor = if on_border { Some("ns-resize") } else { None };
        overlay.set_cursor_from_name(cursor);
        strip.set_cursor_from_name(cursor);
    }
    fn motion(self: &Rc<Self>, y: f64) {
        self.pointer_y.set(Some(y));
        self.update_resize_cursor(Some(y));
        let Some(overlay) = self.overlay.upgrade() else {
            return;
        };
        let button_height = self.button.upgrade().map_or(32, |b| b.height().max(32));
        let panel_height = self
            .strip
            .upgrade()
            .map_or(128, |s| s.height().max(s.height_request() + 16));
        let region = if self.preference.enabled() {
            panel_height + button_height + 16
        } else {
            button_height + 16
        };
        if y >= f64::from((overlay.height() - region).max(0)) && y <= f64::from(overlay.height()) {
            self.near.set(true);
            self.cancel();
            self.render(true);
        } else {
            self.away();
        }
    }
    fn away(self: &Rc<Self>) {
        if self.resizing.get() { return; }
        self.near.set(false);
        if self.shown.get() && !self.pinned.get() && self.timeout.borrow().is_none() {
            self.schedule(3000);
        }
    }
    fn schedule(self: &Rc<Self>, milliseconds: u64) {
        self.cancel();
        let weak = Rc::downgrade(self);
        let id = glib::timeout_add_local_once(Duration::from_millis(milliseconds), move || {
            if let Some(state) = weak.upgrade() {
                state.timeout.borrow_mut().take();
                if !state.near.get() && !state.pinned.get() {
                    state.render(false);
                }
            }
        });
        self.timeout.replace(Some(id));
    }
}
impl Drop for Visibility {
    fn drop(&mut self) {
        self.disconnect_layout();
        self.cancel();
    }
}
