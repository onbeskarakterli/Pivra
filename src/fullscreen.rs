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
    let ordinary = Rc::new(Cell::new(window.is_fullscreen()));
    let slideshow = Rc::new(Cell::new(false));
    let action =
        gio::SimpleAction::new_stateful("fullscreen", None, &window.is_fullscreen().to_variant());
    let weak = window.downgrade();
    let intent = pending.clone();
    let normal = ordinary.clone();
    let slide = slideshow.clone();
    action.connect_activate(move |action, _| {
        if let Some(window) = weak.upgrade() {
            let target = !normal.get();
            normal.set(target);
            action.set_state(&target.to_variant());
            request(&window, &intent, target || slide.get());
        }
    });
    window.add_action(&action);
    let forced = gio::SimpleAction::new_stateful("slideshow-fullscreen", None, &false.to_variant());
    let weak = window.downgrade();
    let normal = ordinary.clone();
    let slide = slideshow.clone();
    let intent = pending.clone();
    forced.connect_change_state(move |action, value| {
        let Some(target) = value.and_then(|v| v.get::<bool>()) else {
            return;
        };
        slide.set(target);
        action.set_state(&target.to_variant());
        if let Some(window) = weak.upgrade() {
            request(&window, &intent, target || normal.get());
        }
    });
    window.add_action(&forced);
    let controls: Vec<_> = chrome.iter().map(|w| w.downgrade()).collect();
    let intent = pending.clone();
    let normal = ordinary.clone();
    let slide = slideshow.clone();
    window.connect_fullscreened_notify(move |window| {
        let actual = window.is_fullscreen();
        if !slide.get() && intent.get().is_none() {
            normal.set(actual);
        }
        action.set_state(&normal.get().to_variant());
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
                if window
                    .lookup_action("slideshow")
                    .and_then(|a| a.state())
                    .and_then(|v| v.get::<bool>())
                    == Some(true)
                {
                    gio::prelude::ActionGroupExt::activate_action(&window, "slideshow-stop", None);
                    return glib::Propagation::Stop;
                }
                ordinary.set(false);
                if let Some(action) = window
                    .lookup_action("fullscreen")
                    .and_downcast::<gio::SimpleAction>()
                {
                    action.set_state(&false.to_variant());
                }
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
            gdk::Key::f | gdk::Key::F | gdk::Key::F11 => {
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

    // Photo double-click: fullscreen only. Retain the pan gesture grouping.
    let photo_click = gtk::GestureClick::new();
    photo_click.set_button(1);
    photo_click.set_propagation_phase(gtk::PropagationPhase::Capture);
    let photo_drag = picture
        .observe_controllers()
        .iter::<glib::Object>()
        .filter_map(Result::ok)
        .find_map(|c| c.downcast::<gtk::GestureDrag>().ok());
    picture.add_controller(photo_click.clone());
    if let Some(drag) = &photo_drag {
        photo_click.group_with(drag);
    }
    let weak = window.downgrade();
    photo_click.connect_pressed(move |gesture, count, _, _| {
        if count == 2 {
            if let Some(window) = weak.upgrade() {
                if window
                    .lookup_action("slideshow")
                    .and_then(|a| a.state())
                    .and_then(|v| v.get::<bool>())
                    == Some(true)
                {
                    return;
                }
                gesture.set_state(gtk::EventSequenceState::Claimed);
                if let Some(drag) = &photo_drag {
                    drag.reset();
                }
                gio::prelude::ActionGroupExt::activate_action(&window, "fullscreen", None);
            }
        }
    });
}
