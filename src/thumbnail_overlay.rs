//! Floating thumbnail controls; shared Enabled preference, per-window visibility.
use gtk::prelude::*;
use gtk::{gdk, glib};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};
thread_local! { static CSS_INSTALLED: Cell<bool> = const { Cell::new(false) }; }
pub(crate) const CELL_WIDTH: i32 = 96;
pub(crate) const CELL_HEIGHT: i32 = 80;
fn install_css(display: &gtk::gdk::Display) {
    CSS_INSTALLED.with(|installed| {
        if installed.replace(true) { return; }
        let css=gtk::CssProvider::new();
        css.load_from_string("\
            .pivra-thumbnail-panel { background-color: alpha(@window_bg_color, 0.50); border-radius: 0; border-top: 1px solid alpha(white, 0.22); padding: 8px; }\n\
            .pivra-thumbnail-toggle { background-image: none; background-color: alpha(@window_bg_color, 0.50); border-radius: 10px 10px 0 0; border: 1px solid alpha(white, 0.22); border-bottom: 0; box-shadow: none; padding: 3px 16px; min-width: 112px; min-height: 24px; }\n\
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
        .height_request(112)
        .child(&items)
        .build();
    strip.set_widget_name("thumbnail-strip");
    strip.add_css_class("pivra-thumbnail-panel");
    strip.set_margin_start(0);
    strip.set_margin_end(0);
    strip.set_visible(false);
    strip.set_min_content_width(0);
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
    strip.set_valign(gtk::Align::End);
    // The panel touches the image viewport edge; neither sibling is measured.
    strip.set_margin_bottom(0);
    for widget in [strip.upcast_ref::<gtk::Widget>(), button.upcast_ref()] {
        overlay.add_overlay(widget);
        overlay.set_measure_overlay(widget, false);
        overlay.set_clip_overlay(widget, true);
    }
    let preference = pivra::settings::thumbnail_preference();
    let state = Rc::new(Visibility {
        available,
        preference: preference.clone(),
        strip: strip.downgrade(),
        button: button.downgrade(),
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
            state.render(state.near.get());
            // Enabled also changes the height of the interaction region.
            if let Some(y) = state.pointer_y.get() {
                state.motion(y);
            }
        }
    });
    preference.subscribe(&listener);
    state.listener.replace(Some(listener));
    state.render(false);
    let click = state.clone();
    button.connect_clicked(move |_| {
        if click.available {
            click.preference.set_enabled(!click.preference.enabled());
        }
    });
    let motion = gtk::EventControllerMotion::new();
    motion.set_propagation_phase(gtk::PropagationPhase::Capture);
    let enter = state.clone();
    motion.connect_enter(move |_, _, y| enter.motion(y));
    let moving = state.clone();
    motion.connect_motion(move |_, _, y| moving.motion(y));
    let leave = state.clone();
    motion.connect_leave(move |_| {
        leave.pointer_y.set(None);
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
                key_state.schedule(1000);
            }
        }
        glib::Propagation::Stop
    });
    window.as_ref().add_controller(keys);
    let mapped = state.clone();
    overlay.connect_map(move |overlay| {
        if let Some(clock) = overlay.frame_clock() {
            let weak = Rc::downgrade(&mapped);
            let handler = clock.connect_after_paint(move |_| {
                if let Some(state) = weak.upgrade() {
                    state.place_toggle();
                    if let Some(overlay) = state.overlay.upgrade() {
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
    ThumbnailStrip::new(items, strip)
}
type PreferenceListener = Rc<dyn Fn(bool)>;
struct Visibility {
    available: bool,
    preference: Rc<pivra::settings::ThumbnailPreference>,
    strip: glib::WeakRef<gtk::ScrolledWindow>,
    button: glib::WeakRef<gtk::ToggleButton>,
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
        let show = show && self.available && !self.closed.get();
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
        self.place_toggle();
    }
    fn place_toggle(&self) {
        let (Some(panel), Some(button), Some(overlay)) = (
            self.strip.upgrade(),
            self.button.upgrade(),
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
    }
    fn motion(self: &Rc<Self>, y: f64) {
        self.pointer_y.set(Some(y));
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
        self.near.set(false);
        if self.shown.get() && self.timeout.borrow().is_none() {
            self.schedule(500);
        }
    }
    fn schedule(self: &Rc<Self>, milliseconds: u64) {
        self.cancel();
        let weak = Rc::downgrade(self);
        let id = glib::timeout_add_local_once(Duration::from_millis(milliseconds), move || {
            if let Some(state) = weak.upgrade() {
                state.timeout.borrow_mut().take();
                if !state.near.get() {
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
