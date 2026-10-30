//! Per-window fullscreen intent; compositor state owns presentation and chrome.
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use std::{cell::{Cell, RefCell}, rc::Rc, time::Duration};

fn request(window: &adw::ApplicationWindow, pending: &Cell<Option<bool>>, target: bool) {
    pending.set(Some(target));
    window.set_fullscreened(target);
}

pub(crate) fn install(
    window: &adw::ApplicationWindow,
    picture: &gtk::Picture,
    video: &gtk::Picture,
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
                gesture.set_state(gtk::EventSequenceState::Claimed);
                if let Some(drag) = &photo_drag {
                    drag.reset();
                }
                gio::prelude::ActionGroupExt::activate_action(&window, "fullscreen", None);
            }
        }
    });

    // Video single-click toggles playback only after the multi-click interval.
    // A second click cancels it, so fullscreen never modifies playback state.
    let video_click = gtk::GestureClick::new();
    video_click.set_button(1);
    video_click.set_propagation_phase(gtk::PropagationPhase::Capture);
    video.add_controller(video_click.clone());
    let delayed = Rc::new(RefCell::new(None::<glib::SourceId>));
    let weak = window.downgrade();
    let video_ref = video.downgrade();
    let pending_click = delayed.clone();
    video_click.connect_pressed(move |gesture, count, _, _| {
        if let Some(id) = pending_click.borrow_mut().take() {
            id.remove();
        }
        if count == 2 {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            if let Some(window) = weak.upgrade() {
                gio::prelude::ActionGroupExt::activate_action(&window, "fullscreen", None);
            }
        } else if count == 1 {
            let video_ref = video_ref.clone();
            let pending = pending_click.clone();
            // Keep single-click response quick. Double-clicks slower than 300 ms
            // may still trigger playback before fullscreen.
            let id = glib::timeout_add_local_once(Duration::from_millis(300), move || {
                pending.borrow_mut().take();
                if let Some(video) = video_ref.upgrade() {
                    if !video.is_visible() {
                        return;
                    }
                    if let Some(stream) = video
                        .paintable()
                        .and_then(|paintable| paintable.downcast::<gtk::MediaStream>().ok())
                    {
                        if stream.is_playing() {
                            stream.pause();
                        } else {
                            stream.play();
                        }
                    }
                }
            });
            pending_click.replace(Some(id));
        }
    });
    let cancel = delayed.clone();
    window.connect_destroy(move |_| {
        if let Some(id) = cancel.borrow_mut().take() {
            id.remove();
        }
    });
}
