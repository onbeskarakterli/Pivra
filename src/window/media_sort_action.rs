//! The viewer action for changing the catalog order without reloading media.
use super::{FileContext, update_thumbnail_catalog, update_title};
use crate::image_view::ImageView;
use adw::prelude::*;
use gtk::{gio, glib};
use pivra::{media_sort::Order, settings};
use std::{cell::RefCell, rc::Rc};

pub(super) const ACTION: &str = "media-sort";

pub(super) fn install(
    window: &adw::ApplicationWindow,
    title: &adw::WindowTitle,
    context: Rc<RefCell<FileContext>>,
    view: &ImageView,
) {
    let start = context.borrow().order;
    let action = gio::SimpleAction::new_stateful(
        ACTION,
        Some(glib::VariantTy::STRING),
        &start.id().to_variant(),
    );
    let weak = window.downgrade();
    let title = title.downgrade();
    let view = view.clone();
    action.connect_activate(move |action, value| {
        let Some(order) = value.and_then(|value| value.str()).and_then(Order::from_id) else {
            return;
        };
        let (Some(window), Some(title)) = (weak.upgrade(), title.upgrade()) else {
            return;
        };
        let mut context = context.borrow_mut();
        if context.order == order {
            return;
        }
        context.order = order;
        context.collection.sort_preserving(order);
        settings::save_media_sort(order);
        action.set_state(&order.id().to_variant());
        update_title(&window, &title, &context);
        update_thumbnail_catalog(&view, &context);
        view.set_neighbors(context.collection.photo_neighbors());
    });
    window.add_action(&action);
}
