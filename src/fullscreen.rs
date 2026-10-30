//! Per-window fullscreen intent; compositor state owns presentation and chrome.
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use std::{cell::Cell, rc::Rc};

fn request(window: &adw::ApplicationWindow, pending: &Cell<Option<bool>>, target: bool) {
    pending.set(Some(target));
    window.set_fullscreened(target);
}

pub(crate) fn install(
    window: &adw::ApplicationWindow,
    picture: &gtk::Picture,
    chrome: &[gtk::Widget],
) {
    let pending = Rc::new(Cell::new(None));
    let action =
        gio::SimpleAction::new_stateful("fullscreen", None, &window.is_fullscreen().to_variant());
    let weak = window.downgrade();
    let intent = pending.clone();
    action.connect_activate(move |_, _| {
        if let Some(window) = weak.upgrade() {
            let target = !intent.get().unwrap_or_else(|| window.is_fullscreen());
            request(&window, &intent, target);
        }
    });
    window.add_action(&action);
    let controls: Vec<_> = chrome.iter().map(|w| w.downgrade()).collect();
    let intent = pending.clone();
    window.connect_fullscreened_notify(move |window| {
        let actual = window.is_fullscreen();
        action.set_state(&actual.to_variant());
        for control in &controls {
            if let Some(control) = control.upgrade() {
                control.set_visible(!actual);
            }
        }
        if intent.get() == Some(actual) {
            intent.set(None);
        } else if let Some(target) = intent.get() {
            // An older asynchronous configure must not override the last F/Esc.
            window.set_fullscreened(target);
        }
    });
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak = window.downgrade();
    keys.connect_key_pressed(move |_, key, _, modifiers| {
        if modifiers.intersects(
            gdk::ModifierType::CONTROL_MASK
                | gdk::ModifierType::ALT_MASK
                | gdk::ModifierType::SUPER_MASK
                | gdk::ModifierType::META_MASK,
        ) {
            return glib::Propagation::Proceed;
        }
        let Some(window) = weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        match key {
            gdk::Key::Escape => {
                // A second Escape closes even if the compositor has not yet
                // acknowledged the first request to leave fullscreen.
                if pending.get() == Some(false)
                    || (!window.is_fullscreen() && pending.get() != Some(true))
                {
                    window.close();
                } else {
                    request(&window, &pending, false);
                }
            }
            gdk::Key::f | gdk::Key::F => {
                let mut focus = gtk::prelude::GtkWindowExt::focus(&window);
                while let Some(widget) = focus {
                    if widget.is::<gtk::Editable>() || widget.is::<gtk::TextView>() {
                        return glib::Propagation::Proceed;
                    }
                    focus = widget.parent();
                }
                gio::prelude::ActionGroupExt::activate_action(&window, "fullscreen", None);
            }
            _ => return glib::Propagation::Proceed,
        }
        glib::Propagation::Stop
    });
    window.add_controller(keys);

    // Only Picture receives this gesture: overlay siblings and chrome are not
    // descendants, so their button/menu/thumbnail events cannot bubble here.
    let click = gtk::GestureClick::new();
    click.set_button(1);
    click.set_propagation_phase(gtk::PropagationPhase::Capture);
    let drag = picture
        .observe_controllers()
        .iter::<glib::Object>()
        .filter_map(Result::ok)
        .find_map(|c| c.downcast::<gtk::GestureDrag>().ok());
    picture.add_controller(click.clone());
    if let Some(drag) = &drag {
        click.group_with(drag);
    }
    let weak = window.downgrade();
    click.connect_pressed(move |gesture, count, _, _| {
        if count != 2 {
            return;
        }
        if let Some(window) = weak.upgrade() {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            if let Some(drag) = &drag {
                drag.reset();
            }
            gio::prelude::ActionGroupExt::activate_action(&window, "fullscreen", None);
        }
    });
}
