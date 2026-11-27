//! Photo clicks reuse manual slideshow navigation; no additional timer.
use adw::prelude::*;
use gtk::{gio, glib};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub(super) fn install(window: &adw::ApplicationWindow, picture: &gtk::Picture) {
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
    let moved = Rc::new(Cell::new(false));
    let pressed = Rc::new(RefCell::new(None::<gtk::gdk::Paintable>));
    if let Some(drag) = drag {
        let moved = moved.clone();
        let weak = picture.downgrade();
        drag.connect_drag_update(move |_, x, y| {
            if let Some(picture) = weak.upgrade() {
                if picture.drag_check_threshold(0, 0, x as i32, y as i32) {
                    moved.set(true);
                }
            }
        });
    }
    let begin = pressed.clone();
    let motion = moved.clone();
    let weak = picture.downgrade();
    click.connect_pressed(move |_, _, _, _| {
        motion.set(false);
        *begin.borrow_mut() = weak.upgrade().and_then(|p| p.paintable());
    });
    let source = pressed.clone();
    let weak = picture.downgrade();
    let owner = window.downgrade();
    click.connect_released(move |_, _, x, y| {
        let source = source.borrow_mut().take();
        let (Some(window), Some(picture)) = (owner.upgrade(), weak.upgrade()) else {
            return;
        };
        if !moved.get()
            && super::slideshow::active(&window)
            && picture.is_visible()
            && source.is_some()
            && source == picture.paintable()
            && x >= 0.
            && y >= 0.
            && x < f64::from(picture.width())
            && y < f64::from(picture.height())
        {
            gio::prelude::ActionGroupExt::activate_action(&window, "slideshow-next", None);
        }
    });
    let cancelled = pressed.clone();
    click.connect_cancel(move |_, _| {
        cancelled.borrow_mut().take();
    });
    let closing = pressed.clone();
    window.connect_close_request(move |_| {
        closing.borrow_mut().take();
        glib::Propagation::Proceed
    });
    window.connect_destroy(move |_| {
        pressed.borrow_mut().take();
    });
}
