//! One owner for transient popup lifetime and the cancellable hover generation.
use adw::prelude::*;
use gtk::{gdk, glib};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

pub(super) struct Menus {
    root: glib::WeakRef<gtk::Popover>,
    child: RefCell<Option<glib::WeakRef<gtk::Popover>>>,
    pending: RefCell<Option<glib::SourceId>>,
    generation: Cell<u64>,
}
impl Menus {
    pub(super) fn new(root: &gtk::Popover, window: &adw::ApplicationWindow) -> Rc<Self> {
        let state = Rc::new(Self {
            root: root.downgrade(),
            child: RefCell::new(None),
            pending: RefCell::new(None),
            generation: Cell::new(0),
        });
        let closing = state.clone();
        root.connect_unmap(move |_| closing.close_child());
        let closing = state.clone();
        window.connect_is_active_notify(move |w| {
            if !w.is_active() && !Self::owner_active(w) {
                closing.close();
            }
        });
        let weak = Rc::downgrade(&state);
        window.connect_realize(move |window| {
            if let Some(state) = weak.upgrade() {
                state.watch_surface(window);
            }
        });
        if window.is_realized() {
            state.watch_surface(window);
        }
        let closing = state.clone();
        window.connect_suspended_notify(move |w| {
            if w.is_suspended() {
                closing.close();
            }
        });
        let closing = state.clone();
        window.connect_unmap(move |_| closing.close());
        let closing = state.clone();
        window.connect_close_request(move |_| {
            closing.close();
            glib::Propagation::Proceed
        });
        let closing = state.clone();
        window.connect_destroy(move |_| {
            closing.close();
            if let Some(root) = closing.root.upgrade() {
                root.unparent();
            }
        });
        state
    }
    pub(super) fn owner_active(window: &adw::ApplicationWindow) -> bool {
        // A native popup grab can send a synthetic focus-out to GtkWindow on
        // X11. The compositor's owning toplevel stays FOCUSED in that case.
        window
            .surface()
            .and_downcast::<gdk::Toplevel>()
            .map_or_else(
                || window.is_active(),
                |s| s.state().contains(gdk::ToplevelState::FOCUSED),
            )
    }
    fn watch_surface(self: &Rc<Self>, window: &adw::ApplicationWindow) {
        if let Some(surface) = window.surface().and_downcast::<gdk::Toplevel>() {
            let weak = window.downgrade();
            let state = Rc::downgrade(self);
            surface.connect_notify_local(Some("state"), move |surface, _| {
                if let (Some(window), Some(state)) = (weak.upgrade(), state.upgrade()) {
                    if !Self::owner_active(&window)
                        || surface.state().contains(gdk::ToplevelState::MINIMIZED)
                    {
                        state.close();
                    }
                }
            });
        }
    }
    fn cancel_hover(&self) {
        self.generation.set(self.generation.get().wrapping_add(1));
        if let Some(id) = self.pending.borrow_mut().take() {
            id.remove();
        }
    }
    pub(super) fn close_child(&self) {
        self.cancel_hover();
        let child = self.child.borrow_mut().take().and_then(|c| c.upgrade());
        if let Some(child) = child {
            child.popdown();
        }
    }
    pub(super) fn close(&self) {
        self.close_child();
        if let Some(root) = self.root.upgrade() {
            root.popdown();
        }
    }
    fn open_child(&self, child: &gtk::Popover) {
        self.cancel_hover();
        if child.is_visible() {
            return;
        }
        self.close_child();
        if !self.root.upgrade().is_some_and(|p| p.is_mapped()) {
            return;
        }
        self.child.replace(Some(child.downgrade()));
        child.popup();
        if let Some(first) = child.child().and_then(|content| content.first_child()) {
            first.grab_focus();
        }
    }
    pub(super) fn watch_other(self: &Rc<Self>, button: &gtk::Button) {
        let motion = gtk::EventControllerMotion::new();
        let state = Rc::downgrade(self);
        let weak = button.downgrade();
        motion.connect_enter(move |_, _, _| {
            if let Some(state) = state.upgrade() {
                state.close_child();
            }
            if let Some(button) = weak.upgrade() {
                button.grab_focus();
            }
        });
        button.add_controller(motion);
        let focus = gtk::EventControllerFocus::new();
        let state = Rc::downgrade(self);
        focus.connect_enter(move |_| {
            if let Some(state) = state.upgrade() {
                state.close_child();
            }
        });
        button.add_controller(focus);
    }
    pub(super) fn watch_parent(self: &Rc<Self>, button: &gtk::Button, popup: &gtk::Popover) {
        popup.set_parent(button);
        popup.set_position(gtk::PositionType::Right);
        popup.set_valign(gtk::Align::Start);
        button.update_property(&[gtk::accessible::Property::HasPopup(true)]);
        button.update_state(&[gtk::accessible::State::Expanded(Some(false))]);
        let weak = button.downgrade();
        popup.connect_visible_notify(move |popup| {
            if let Some(button) = weak.upgrade() {
                button.update_state(&[gtk::accessible::State::Expanded(Some(popup.is_visible()))]);
            }
        });
        let motion = gtk::EventControllerMotion::new();
        let state = Rc::downgrade(self);
        let weak = button.downgrade();
        let child = popup.downgrade();
        motion.connect_enter(move |_, _, _| {
            let (Some(state), Some(button), Some(child)) =
                (state.upgrade(), weak.upgrade(), child.upgrade())
            else {
                return;
            };
            state.cancel_hover();
            button.grab_focus();
            if child.is_visible() {
                return;
            }
            let generation = state.generation.get();
            let state_ref = Rc::downgrade(&state);
            let weak = button.downgrade();
            let child = child.downgrade();
            let id = glib::timeout_add_local_once(Duration::from_millis(200), move || {
                let Some(state) = state_ref.upgrade() else {
                    return;
                };
                state.pending.borrow_mut().take();
                if state.generation.get() != generation {
                    return;
                }
                if let (Some(button), Some(child)) = (weak.upgrade(), child.upgrade()) {
                    if button.is_mapped() {
                        state.open_child(&child);
                    }
                }
            });
            *state.pending.borrow_mut() = Some(id);
        });
        let state = Rc::downgrade(self);
        motion.connect_leave(move |_| {
            // Preserve an open child while crossing the gap. Only another root
            // row (or an explicit dismissal) ends that child session.
            if let Some(state) = state.upgrade() {
                state.cancel_hover();
            }
        });
        button.add_controller(motion);
        let state = Rc::downgrade(self);
        let child = popup.downgrade();
        button.connect_clicked(move |_| {
            if let (Some(state), Some(child)) = (state.upgrade(), child.upgrade()) {
                state.open_child(&child);
            }
        });
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let state = Rc::downgrade(self);
        let child = popup.downgrade();
        keys.connect_key_pressed(move |_, key, _, modifiers| {
            if key == gdk::Key::Right && modifiers.is_empty() {
                if let (Some(state), Some(child)) = (state.upgrade(), child.upgrade()) {
                    state.open_child(&child);
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        button.add_controller(keys);
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let state = Rc::downgrade(self);
        let parent = button.downgrade();
        keys.connect_key_pressed(move |_, key, _, modifiers| {
            if matches!(key, gdk::Key::Escape | gdk::Key::Left) && modifiers.is_empty() {
                if let Some(state) = state.upgrade() {
                    state.close_child();
                }
                if let Some(parent) = parent.upgrade() {
                    parent.grab_focus();
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        popup.add_controller(keys);
        let state = Rc::downgrade(self);
        button.connect_unmap(move |_| {
            if let Some(state) = state.upgrade() {
                state.close_child();
            }
        });
        let child = popup.clone();
        button.connect_destroy(move |_| {
            child.unparent();
        });
    }
}
