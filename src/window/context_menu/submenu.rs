//! Public GTK custom-menu slot with a coordinated, right-facing submenu.
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use pivra::viewing::ViewMode;
use std::{cell::RefCell, rc::Rc, time::Duration};

pub(super) fn install(root: &gtk::PopoverMenu, window: &adw::ApplicationWindow) {
    let modes = gio::Menu::new();
    for mode in ViewMode::ALL {
        let item = gio::MenuItem::new(Some(mode.label()), None);
        item.set_action_and_target_value(Some("win.image-mode"), Some(&mode.id().to_variant()));
        modes.append_item(&item);
    }
    let popup = gtk::PopoverMenu::from_model_full(&modes, gtk::PopoverMenuFlags::NESTED);
    popup.set_widget_name("display-mode-submenu");
    popup.set_has_arrow(false);
    popup.set_valign(gtk::Align::Start);
    let button = gtk::MenuButton::builder()
        .label("Display Mode")
        .direction(gtk::ArrowType::Right)
        .has_frame(false)
        .hexpand(true)
        .popover(&popup)
        .build();
    button.set_widget_name("display-mode-submenu-button");
    // GtkMenuButton supplies a real child GdkPopup, including screen-edge flip,
    // click activation and accessibility. No private GtkModelButton hooks.
    root.add_child(&button, "display-mode");
    let pending = Rc::new(RefCell::new(None::<glib::SourceId>));
    let motion = gtk::EventControllerMotion::new();
    let timer = pending.clone();
    let weak = button.downgrade();
    let parent = root.downgrade();
    motion.connect_enter(move |_, _, _| {
        if timer.borrow().is_some() {
            return;
        }
        let weak = weak.clone();
        let parent = parent.clone();
        let fired = timer.clone();
        let id = glib::timeout_add_local_once(Duration::from_millis(200), move || {
            fired.borrow_mut().take();
            if let (Some(button), Some(parent)) = (weak.upgrade(), parent.upgrade()) {
                if button.is_mapped() && parent.is_mapped() && !button.is_active() {
                    button.popup();
                }
            }
        });
        *timer.borrow_mut() = Some(id);
    });
    let timer = pending.clone();
    motion.connect_leave(move |_| {
        // Crossing into the child popup only cancels a pending open. It must
        // never pop down an already open submenu or its parent.
        if let Some(id) = timer.borrow_mut().take() {
            id.remove();
        }
    });
    button.add_controller(motion);
    let timer = pending;
    let child = popup.downgrade();
    button.connect_unmap(move |_| {
        if let Some(id) = timer.borrow_mut().take() {
            id.remove();
        }
        if let Some(child) = child.upgrade() {
            child.set_visible(false);
        }
    });
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak = button.downgrade();
    keys.connect_key_pressed(move |_, key, _, modifiers| {
        if key == gdk::Key::Right && modifiers.is_empty() {
            if let Some(button) = weak.upgrade() {
                button.popup();
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    button.add_controller(keys);
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak = button.downgrade();
    keys.connect_key_pressed(move |_, key, _, modifiers| {
        if matches!(key, gdk::Key::Left | gdk::Key::Escape) && modifiers.is_empty() {
            if let Some(button) = weak.upgrade() {
                button.popdown();
                button.grab_focus();
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    popup.add_controller(keys);
    // A mode action, including reselecting the same mode, is a normal menu
    // activation and closes the whole chain. Returning focus is not activation.
    if let Some(action) = window
        .lookup_action("image-mode")
        .and_downcast::<gio::SimpleAction>()
    {
        let parent = root.downgrade();
        let handler = action.connect_activate(move |_, _| {
            if let Some(parent) = parent.upgrade() {
                // GTK may already have hidden the child before dispatching the
                // action. Root visibility is the correct chain-lifetime guard.
                if parent.is_visible() {
                    parent.popdown();
                }
            }
        });
        let action = action.downgrade();
        let handler = RefCell::new(Some(handler));
        button.connect_destroy(move |_| {
            if let (Some(action), Some(handler)) = (action.upgrade(), handler.borrow_mut().take()) {
                action.disconnect(handler);
            }
        });
    }
}
