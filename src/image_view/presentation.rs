//! Stateless picture presentation; callers retain their existing transforms.
use super::{actual_paintable, present};
use gtk::{gio, prelude::*};
use pivra::viewing::{Resampling, Transform, ViewMode};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub(super) fn manual(
    picture: &gtk::Picture,
    texture: &gtk::gdk::Texture,
    transform: Rc<RefCell<Transform>>,
    mode: ViewMode,
    action: &gio::SimpleAction,
    filter: Rc<Cell<Resampling>>,
) {
    picture.set_overflow(gtk::Overflow::Hidden);
    picture.set_content_fit(gtk::ContentFit::Fill);
    if let Some(current) = picture
        .paintable()
        .and_then(|p| p.downcast::<actual_paintable::ActualPaintable>().ok())
        .filter(|p| p.has_transform(&transform) && p.has_texture(texture))
    {
        current.invalidate_contents();
    } else {
        picture.set_paintable(Some(&actual_paintable::ActualPaintable::transformed(
            texture, transform, mode, filter,
        )));
    }
    action.set_state(&"manual".to_variant());
}

/// Keep a single transformed paintable for all frames of one animation.
pub(in super::super) fn present_animation_frame(
    picture: &gtk::Picture,
    texture: &gtk::gdk::Texture,
    id: &str,
    transform: Rc<RefCell<Transform>>,
    filter: Rc<Cell<Resampling>>,
) {
    let mode = ViewMode::from_id(id).unwrap_or(ViewMode::Actual);
    if let Some(current) = picture
        .paintable()
        .and_then(|p| p.downcast::<actual_paintable::ActualPaintable>().ok())
        .filter(|p| p.has_transform(&transform))
    {
        // Manual zoom lives in Transform, not in the incoming frame texture.
        current.set_mode(mode);
        current.update_texture(texture);
        return;
    }
    picture.set_overflow(gtk::Overflow::Hidden);
    picture.set_content_fit(gtk::ContentFit::Fill);
    picture.set_paintable(Some(&actual_paintable::ActualPaintable::transformed(
        texture, transform, mode, filter,
    )));
}

pub(in super::super) fn present_filtered(
    picture: &gtk::Picture,
    texture: Option<&gtk::gdk::Texture>,
    id: &str,
    transform: Rc<RefCell<Transform>>,
    filter: Rc<Cell<Resampling>>,
) {
    let mode = ViewMode::from_id(id).unwrap_or(ViewMode::Actual);
    if id != "manual" && filter.get() == Resampling::Smooth && mode != ViewMode::Actual {
        present(picture, texture, mode);
        return;
    }
    let Some(texture) = texture else {
        return;
    };
    if let Some(current) = picture
        .paintable()
        .and_then(|p| p.downcast::<actual_paintable::ActualPaintable>().ok())
        .filter(|p| p.has_transform(&transform) && p.has_texture(texture))
    {
        current.set_mode(mode);
        current.invalidate_contents();
        return;
    }
    picture.set_overflow(gtk::Overflow::Hidden);
    picture.set_content_fit(gtk::ContentFit::Fill);
    picture.set_paintable(Some(&actual_paintable::ActualPaintable::transformed(
        texture, transform, mode, filter,
    )));
}
