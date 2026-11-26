//! Existing photo and video input controllers.
use super::manual;
use adw::prelude::*;
use gtk::{gio, glib};
use pivra::viewing::{Resampling, Transform, ViewMode};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

/// Attached only to the photo widget: header/menu/banner events never enter here.
/// Video uses GTK's current MediaFile paintable, with only a lightweight
/// snapshot transform. No extra decoder or video frame buffering is involved.
pub(super) fn install_video_interaction(
    video: &gtk::Picture,
    stream: Rc<RefCell<Option<gtk::MediaFile>>>,
    transform: Rc<RefCell<Transform>>,
    mode_action: &gio::SimpleAction,
    zoom_wheel: Rc<Cell<bool>>,
) {
    let pointer = Rc::new(Cell::new((0.0, 0.0)));
    let motion = gtk::EventControllerMotion::new();
    let entered = pointer.clone();
    motion.connect_enter(move |_, x, y| entered.set((x, y)));
    let moved = pointer.clone();
    motion.connect_motion(move |_, x, y| moved.set((x, y)));
    video.add_controller(motion);

    let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
    scroll.set_propagation_phase(gtk::PropagationPhase::Capture);
    let source = stream.clone();
    let state = transform.clone();
    let action = mode_action.clone();
    let weak = video.downgrade();
    scroll.connect_scroll(move |controller, _, dy| {
        if dy == 0.0 {
            return glib::Propagation::Stop;
        }
        let zoom = zoom_wheel.get()
            || controller.current_event_state().intersects(
                gtk::gdk::ModifierType::BUTTON1_MASK | gtk::gdk::ModifierType::CONTROL_MASK,
            );
        let Some(video) = weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        if !zoom {
            if let Some(window) = video.root().and_downcast::<adw::ApplicationWindow>() {
                let direction = if dy > 0.0 {
                    "next-media"
                } else {
                    "previous-media"
                };
                gio::prelude::ActionGroupExt::activate_action(&window, direction, None);
            }
            return glib::Propagation::Stop;
        }
        let dimensions = source.borrow().as_ref().map(|media| {
            (
                media.intrinsic_width() as f64,
                media.intrinsic_height() as f64,
            )
        });
        let Some(dimensions) = dimensions else {
            return glib::Propagation::Proceed;
        };
        let mode = action
            .state()
            .and_then(|value| value.str().and_then(ViewMode::from_id))
            .unwrap_or(ViewMode::Fit);
        if state.borrow_mut().zoom(
            dimensions,
            (video.width() as f64, video.height() as f64),
            mode,
            pointer.get(),
            dy,
        ) {
            if let Some(paintable) = video.paintable() {
                paintable.invalidate_contents();
            }
            action.set_state(&"manual".to_variant());
        }
        glib::Propagation::Stop
    });
    video.add_controller(scroll);

    let drag = gtk::GestureDrag::new();
    drag.set_button(1);
    drag.set_propagation_phase(gtk::PropagationPhase::Capture);
    let last = Rc::new(Cell::new(None::<(f64, f64)>));
    let begin = last.clone();
    let begin_stream = stream.clone();
    drag.connect_drag_begin(move |gesture, _, _| {
        if begin_stream.borrow().is_some() {
            begin.set(Some((0.0, 0.0)));
            gesture.set_state(gtk::EventSequenceState::Claimed);
        }
    });
    let offset = last.clone();
    let weak = video.downgrade();
    let action = mode_action.clone();
    drag.connect_drag_update(move |_, x, y| {
        let Some(previous) = offset.get() else {
            return;
        };
        let Some(video) = weak.upgrade() else {
            return;
        };
        let dimensions = stream.borrow().as_ref().map(|media| {
            (
                media.intrinsic_width() as f64,
                media.intrinsic_height() as f64,
            )
        });
        let Some(dimensions) = dimensions else {
            return;
        };
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        offset.set(Some((x, y)));
        let mode = action
            .state()
            .and_then(|value| value.str().and_then(ViewMode::from_id))
            .unwrap_or(ViewMode::Fit);
        if transform.borrow_mut().pan(
            dimensions,
            (video.width() as f64, video.height() as f64),
            mode,
            (x - previous.0, y - previous.1),
        ) {
            if let Some(paintable) = video.paintable() {
                paintable.invalidate_contents();
            }
            action.set_state(&"manual".to_variant());
        }
    });
    let end = last.clone();
    drag.connect_drag_end(move |_, _, _| end.set(None));
    drag.connect_cancel(move |_, _| last.set(None));
    video.add_controller(drag);
}

pub(in super::super) fn install_interaction(
    picture: &gtk::Picture,
    texture: Rc<RefCell<Option<gtk::gdk::Texture>>>,
    transform: Rc<RefCell<Transform>>,
    action: &gio::SimpleAction,
    filter: Rc<Cell<Resampling>>,
    zoom_wheel: Rc<Cell<bool>>,
) {
    let pointer = Rc::new(Cell::new((0., 0.)));
    let motion = gtk::EventControllerMotion::new();
    let entered = pointer.clone();
    motion.connect_enter(move |_, x, y| entered.set((x, y)));
    let position = pointer.clone();
    motion.connect_motion(move |_, x, y| position.set((x, y)));
    picture.add_controller(motion);
    let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
    scroll.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak = picture.downgrade();
    let source = texture.clone();
    let state = transform.clone();
    let mode_action = action.clone();
    let scroll_filter = filter.clone();
    scroll.connect_scroll(move |controller, _, dy| {
        let zoom = zoom_wheel.get()
            || controller.current_event_state().intersects(
                gtk::gdk::ModifierType::BUTTON1_MASK | gtk::gdk::ModifierType::CONTROL_MASK,
            );
        if !zoom {
            if let Some(picture) = weak.upgrade() {
                if let Some(window) = picture.root().and_downcast::<adw::ApplicationWindow>() {
                    let action = if dy > 0.0 {
                        "next-media"
                    } else {
                        "previous-media"
                    };
                    if dy != 0.0 {
                        gio::prelude::ActionGroupExt::activate_action(&window, action, None);
                    }
                }
            }
            return glib::Propagation::Stop;
        }
        let Some(picture) = weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        if !picture.is_mapped() {
            return glib::Propagation::Proceed;
        }
        let source = source.borrow();
        let Some(texture) = source.as_ref() else {
            return glib::Propagation::Proceed;
        };
        let mode = mode_action
            .state()
            .and_then(|v| v.str().and_then(ViewMode::from_id))
            .unwrap_or(ViewMode::Actual);
        let changed = state.borrow_mut().zoom(
            (texture.width() as f64, texture.height() as f64),
            (picture.width() as f64, picture.height() as f64),
            mode,
            pointer.get(),
            dy,
        );
        if changed {
            manual(
                &picture,
                texture,
                state.clone(),
                mode,
                &mode_action,
                scroll_filter.clone(),
            );
        }
        glib::Propagation::Stop
    });
    picture.add_controller(scroll);
    let drag = gtk::GestureDrag::new();
    drag.set_button(1);
    drag.set_propagation_phase(gtk::PropagationPhase::Capture);
    let last = Rc::new(Cell::new(None::<(f64, f64)>));
    let start = last.clone();
    let source = texture.clone();
    let drag_generation = Rc::new(Cell::new(0));
    let began_generation = drag_generation.clone();
    let began_transform = transform.clone();
    drag.connect_drag_begin(move |gesture, _, _| {
        if source.borrow().is_some() && gesture.widget().is_some_and(|w| w.is_mapped()) {
            began_generation.set(began_transform.borrow().generation());
            start.set(Some((0., 0.)));
            gesture.set_state(gtk::EventSequenceState::Claimed);
        }
    });
    let weak = picture.downgrade();
    let offsets = last.clone();
    let state = transform;
    let mode_action = action.clone();
    drag.connect_drag_update(move |_, x, y| {
        if drag_generation.get() != state.borrow().generation() {
            // Zoom while holding the left button changes the transform
            // generation; rebase the drag instead of disabling it permanently.
            drag_generation.set(state.borrow().generation());
            offsets.set(Some((x, y)));
            return;
        }
        let Some(previous) = offsets.get() else {
            return;
        };
        let Some(picture) = weak.upgrade() else {
            return;
        };
        if !picture.is_mapped() {
            offsets.set(None);
            return;
        }
        let source = texture.borrow();
        let Some(texture) = source.as_ref() else {
            offsets.set(None);
            return;
        };
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        offsets.set(Some((x, y)));
        let mode = mode_action
            .state()
            .and_then(|v| v.str().and_then(ViewMode::from_id))
            .unwrap_or(ViewMode::Actual);
        let changed = state.borrow_mut().pan(
            (texture.width() as f64, texture.height() as f64),
            (picture.width() as f64, picture.height() as f64),
            mode,
            (x - previous.0, y - previous.1),
        );
        if changed {
            manual(
                &picture,
                texture,
                state.clone(),
                mode,
                &mode_action,
                filter.clone(),
            );
        }
    });
    let end = last.clone();
    drag.connect_drag_end(move |_, _, _| end.set(None));
    drag.connect_cancel(move |_, _| last.set(None));
    picture.add_controller(drag);
}
