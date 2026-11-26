//! Reveal the selected file with the supported, fully asynchronous desktop API.
use super::{FileContext, show_latest_toast};
use adw::prelude::*;
use gtk::{gio, glib};
use std::{cell::RefCell, rc::Rc};

pub(super) fn install(
    window: &adw::ApplicationWindow,
    context: Rc<RefCell<FileContext>>,
    toasts: &adw::ToastOverlay,
    current_toast: Rc<RefCell<Option<adw::Toast>>>,
) {
    let pending = Rc::new(RefCell::new(None::<gio::Cancellable>));
    let action = gio::SimpleAction::new("show-in-files", None);
    let weak = window.downgrade();
    let tokens = pending.clone();
    let toasts = toasts.downgrade();
    action.connect_activate(move |action, _| {
        let _dispatch = pivra::performance::Span::new("show_files_dispatch_ui");
        let Some(window) = weak.upgrade().filter(|w| w.is_visible()) else {
            return;
        };
        if tokens.borrow().is_some() {
            return;
        }
        let Some(uri) = context.borrow().collection.current()
            .map(|item| gio::File::for_path(item.path()).uri().to_string()) else {
            return;
        };
        super::context_menu::dismiss(&window);
        let cancel = gio::Cancellable::new();
        tokens.replace(Some(cancel.clone()));
        action.set_enabled(false);
        let token = cancel.clone();
        let tokens = tokens.clone();
        let action = action.downgrade();
        let weak = window.downgrade();
        let toasts = toasts.clone();
        let current = current_toast.clone();
        let waiting = pivra::performance::Span::new("show_files_service_wait");
        let done = move |result: Result<(), glib::Error>| {
            let _waiting = waiting;
            if tokens.borrow().as_ref() != Some(&token) {
                return;
            }
            tokens.borrow_mut().take();
            let Some(_window) = weak.upgrade().filter(|w| w.is_visible()) else {
                return;
            };
            if let Some(action) = action.upgrade() {
                action.set_enabled(true);
            }
            if let Err(error) = result {
                if error.matches(gio::IOErrorEnum::Cancelled) {
                    return;
                }
                eprintln!("Pivra: could not show selected file in Files: {error}");
                if let Some(toasts) = toasts.upgrade() {
                    show_latest_toast(
                        &toasts, &current,
                        adw::Toast::new("Could not show the selected file in Files. The desktop file manager may be unavailable."),
                    );
                }
            }
        };
        let work_cancel = cancel.clone();
        gio::bus_get(gio::BusType::Session, Some(&cancel), move |result| {
            match result {
                Err(error) => done(Err(error)),
                Ok(bus) => bus.call(
                    Some("org.freedesktop.FileManager1"),
                    "/org/freedesktop/FileManager1",
                    "org.freedesktop.FileManager1", "ShowItems",
                    Some(&(vec![uri], "").to_variant()),
                    Some(glib::VariantTy::UNIT), gio::DBusCallFlags::NONE,
                    25_000, Some(&work_cancel), move |r| done(r.map(|_| ())),
                ),
            }
        });
    });
    window.add_action(&action);
    let closing = pending.clone();
    window.connect_close_request(move |_| {
        if let Some(cancel) = closing.borrow_mut().take() {
            cancel.cancel();
        }
        glib::Propagation::Proceed
    });
    window.connect_destroy(move |_| {
        if let Some(cancel) = pending.borrow_mut().take() {
            cancel.cancel();
        }
    });
}
