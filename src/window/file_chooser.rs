//! One reusable native chooser; preference I/O never precedes showing it.
use adw::prelude::*;
use gtk::{gio, glib};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub(super) fn set_dialogs_enabled(window: &adw::ApplicationWindow, enabled: bool) {
    for name in ["choose-media", "open-with"] {
        if let Some(action) = window
            .lookup_action(name)
            .and_downcast::<gio::SimpleAction>()
        {
            action.set_enabled(enabled);
        }
    }
}
pub(super) fn install(window: &adw::ApplicationWindow, app: &adw::Application) {
    let cached = Rc::new(RefCell::new(None::<gio::File>));
    let revision = Rc::new(Cell::new(0u64));
    let warm = gio::Cancellable::new();
    let work_cancel = warm.clone();
    let cache = cached.clone();
    let version = revision.clone();
    glib::MainContext::default().spawn_local(async move {
        let path = gio::spawn_blocking(|| {
            let _timing = pivra::performance::Span::new("open_file_folder_worker");
            super::open_folder::read_last_open_folder()
        })
        .await
        .ok()
        .flatten();
        if !work_cancel.is_cancelled() && version.get() == 0 {
            *cache.borrow_mut() = path.map(gio::File::for_path);
        }
    });
    if let Some(action) = window
        .lookup_action("open-file")
        .and_downcast::<gio::SimpleAction>()
    {
        let cache = cached.clone();
        action.connect_activate(move |_, value| {
            if let Some(uri) = value.and_then(|v| v.str()) {
                revision.set(revision.get().wrapping_add(1));
                *cache.borrow_mut() = gio::File::for_uri(uri).parent();
            }
        });
    }
    let dialog = gtk::FileDialog::new();
    dialog.set_title("Open media");
    let pending = Rc::new(RefCell::new(None::<gio::Cancellable>));
    let choose = gio::SimpleAction::new("choose-media", None);
    let weak = window.downgrade();
    let tokens = pending.clone();
    choose.connect_activate(move |_, _| {
        let Some(window) = weak.upgrade().filter(|w| w.is_visible()) else {
            return;
        };
        if tokens.borrow().is_some() {
            return;
        }
        super::context_menu::dismiss(&window);
        set_dialogs_enabled(&window, false);
        dialog.set_initial_folder(cached.borrow().as_ref());
        let cancel = gio::Cancellable::new();
        tokens.replace(Some(cancel.clone()));
        let tokens = tokens.clone();
        let weak = window.downgrade();
        let token = cancel.clone();
        let _timing = pivra::performance::Span::new("open_file_gtk_call_ui");
        dialog.open(Some(&window), Some(&cancel), move |result| {
            if tokens.borrow().as_ref() != Some(&token) {
                return;
            }
            tokens.borrow_mut().take();
            if let Some(window) = weak.upgrade().filter(|w| w.is_visible()) {
                set_dialogs_enabled(&window, true);
                match result {
                    Ok(file) => gio::prelude::ActionGroupExt::activate_action(
                        &window,
                        "open-file",
                        Some(&file.uri().to_variant()),
                    ),
                    Err(error)
                        if error.matches(gtk::DialogError::Dismissed)
                            || error.matches(gtk::DialogError::Cancelled) => {}
                    Err(error) => eprintln!("Pivra: could not open file chooser: {error}"),
                }
            }
        });
    });
    window.add_action(&choose);
    app.set_accels_for_action("win.choose-media", &["<Control>o"]);
    let closing = pending.clone();
    let closing_warm = warm.clone();
    window.connect_close_request(move |_| {
        closing_warm.cancel();
        if let Some(cancel) = closing.borrow_mut().take() {
            cancel.cancel();
        }
        glib::Propagation::Proceed
    });
    window.connect_destroy(move |_| {
        warm.cancel();
        if let Some(cancel) = pending.borrow_mut().take() {
            cancel.cancel();
        }
    });
}
