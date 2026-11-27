//! Native application selection backed by GIO; no synchronous GTK portal probe.
use super::{FileContext, show_latest_toast};
use adw::prelude::*;
use gtk::{gio, glib};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone)]
struct Choice {
    id: String,
    name: String,
}
fn applications(cancel: &gio::Cancellable) -> Vec<Choice> {
    let _timing = pivra::performance::Span::new("open_with_apps_worker");
    let mut result: Vec<_> = gio::AppInfo::all()
        .into_iter()
        .take_while(|_| !cancel.is_cancelled())
        .filter(|app| app.should_show() && (app.supports_files() || app.supports_uris()))
        .filter_map(|app| {
            Some(Choice {
                id: app.id()?.to_string(),
                name: app.display_name().chars().take(200).collect(),
            })
        })
        .filter(|app| app.id != "io.github.onbeskarakterli.Pivra.desktop")
        .collect();
    result.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    result.dedup_by(|a, b| a.id == b.id);
    result.truncate(512);
    result
}
fn launch(id: String, uri: String, cancel: gio::Cancellable) -> Result<(), String> {
    let _timing = pivra::performance::Span::new("open_with_launch_worker");
    if cancel.is_cancelled() {
        return Ok(());
    }
    let context = glib::MainContext::new();
    context
        .with_thread_default(|| {
            let app = gio::DesktopAppInfo::new(&id)
                .ok_or("Selected application is no longer installed")?;
            let future = gio::GioFuture::new(&app, move |app, _, result| {
                app.launch_uris_async(
                    &[uri.as_str()],
                    None::<&gio::AppLaunchContext>,
                    Some(&cancel),
                    move |r| result.resolve(r),
                );
            });
            context.block_on(future).map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?
}
pub(super) fn install(
    window: &adw::ApplicationWindow,
    context: Rc<RefCell<FileContext>>,
    toasts: &adw::ToastOverlay,
    current_toast: Rc<RefCell<Option<adw::Toast>>>,
) {
    let pending = Rc::new(RefCell::new(None::<gio::Cancellable>));
    let dialog_slot = Rc::new(RefCell::new(None::<glib::WeakRef<adw::MessageDialog>>));
    let cache = Rc::new(RefCell::new(None::<Vec<Choice>>));
    let monitor = gio::AppInfoMonitor::get();
    let cached = cache.clone();
    let handler = RefCell::new(Some(monitor.connect_changed(move |_| {
        cached.borrow_mut().take();
    })));
    window.connect_destroy(move |_| {
        if let Some(id) = handler.borrow_mut().take() {
            monitor.disconnect(id);
        }
    });
    let action = gio::SimpleAction::new("open-with", None);
    let weak = window.downgrade();
    let tokens = pending.clone();
    let dialogs = dialog_slot.clone();
    let toasts = toasts.downgrade();
    action.connect_activate(move |_, _| {
        let _timing = pivra::performance::Span::new("open_with_dialog_ui");
        let Some(window) = weak.upgrade().filter(|w| w.is_visible()) else {
            return;
        };
        if tokens.borrow().is_some() {
            return;
        }
        let Some(path) = context
            .borrow()
            .collection
            .current()
            .map(|i| i.path().to_owned())
        else {
            return;
        };
        super::context_menu::dismiss(&window);
        super::file_chooser::set_dialogs_enabled(&window, false);
        let cancel = gio::Cancellable::new();
        tokens.replace(Some(cancel.clone()));
        let dialog = adw::MessageDialog::new(
            Some(&window),
            Some("Open With"),
            Some(&path.file_name().unwrap_or_default().to_string_lossy()),
        );
        dialog.set_widget_name("open-with-dialog");
        dialog.set_destroy_with_parent(true);
        dialog.add_responses(&[("cancel", "Cancel"), ("open", "Open")]);
        dialog.set_close_response("cancel");
        dialog.set_response_appearance("open", adw::ResponseAppearance::Suggested);
        dialog.set_response_enabled("open", false);
        let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
        let status = gtk::Label::new(Some("Loading installed applications…"));
        let selector = gtk::DropDown::from_strings(&[]);
        selector.set_widget_name("open-with-applications");
        selector.set_enable_search(true);
        content.append(&status);
        content.append(&selector);
        dialog.set_extra_child(Some(&content));
        dialogs.replace(Some(dialog.downgrade()));
        let choices = Rc::new(RefCell::new(Vec::<Choice>::new()));
        let weak_window = window.downgrade();
        let active = tokens.clone();
        let active_dialogs = dialogs.clone();
        let token = cancel.clone();
        let selected = choices.clone();
        let toasts = toasts.clone();
        let current = current_toast.clone();
        let select = selector.downgrade();
        let uri = gio::File::for_path(path).uri().to_string();
        dialog.connect_response(None, move |_, response| {
            if active.borrow().as_ref() != Some(&token) {
                return;
            }
            active_dialogs.borrow_mut().take();
            let choice = select
                .upgrade()
                .and_then(|s| selected.borrow().get(s.selected() as usize).cloned());
            if response != "open" || choice.is_none() {
                token.cancel();
                active.borrow_mut().take();
                if let Some(w) = weak_window.upgrade() {
                    super::file_chooser::set_dialogs_enabled(&w, true);
                }
                return;
            }
            let choice = choice.unwrap();
            let uri = uri.clone();
            let cancel = token.clone();
            let token = token.clone();
            let active = active.clone();
            let weak = weak_window.clone();
            let toasts = toasts.clone();
            let current = current.clone();
            glib::MainContext::default().spawn_local(async move {
                let result = gio::spawn_blocking(move || launch(choice.id, uri, cancel)).await;
                if active.borrow().as_ref() != Some(&token) {
                    return;
                }
                active.borrow_mut().take();
                if let Some(w) = weak.upgrade().filter(|w| w.is_visible()) {
                    super::file_chooser::set_dialogs_enabled(&w, true);
                    let error = match result {
                        Ok(Ok(())) => None,
                        Ok(Err(e)) => Some(e),
                        Err(_) => Some("Application launch worker failed".into()),
                    };
                    if let (Some(error), Some(toasts)) = (error, toasts.upgrade()) {
                        show_latest_toast(
                            &toasts,
                            &current,
                            adw::Toast::new(&format!("Could not open with application: {error}")),
                        );
                    }
                }
            });
        });
        dialog.present();
        let cached = cache.borrow().clone();
        let cache = cache.clone();
        let token = cancel.clone();
        let tokens = tokens.clone();
        let weak_dialog = dialog.downgrade();
        let selector = selector.downgrade();
        let status = status.downgrade();
        glib::MainContext::default().spawn_local(async move {
            let result = if let Some(apps) = cached {
                Ok(apps)
            } else {
                gio::spawn_blocking(move || applications(&cancel)).await
            };
            if token.is_cancelled() || tokens.borrow().as_ref() != Some(&token) {
                return;
            }
            if let (Some(dialog), Some(selector), Some(status)) =
                (weak_dialog.upgrade(), selector.upgrade(), status.upgrade())
            {
                let Ok(apps) = result else {
                    status.set_text("Could not list installed applications. Close and try again.");
                    return;
                };
                let names: Vec<_> = apps.iter().map(|a| a.name.as_str()).collect();
                selector.set_model(Some(&gtk::StringList::new(&names)));
                if !apps.is_empty() {
                    selector.set_selected(0);
                }
                status.set_text(if apps.is_empty() {
                    "No installed application can open files or URIs."
                } else {
                    "Choose an installed application"
                });
                dialog.set_response_enabled("open", !apps.is_empty());
                *cache.borrow_mut() = Some(apps.clone());
                *choices.borrow_mut() = apps;
            }
        });
    });
    window.add_action(&action);
    let tokens = pending.clone();
    let dialogs = dialog_slot.clone();
    window.connect_close_request(move |_| {
        if let Some(token) = tokens.borrow_mut().take() {
            token.cancel();
        }
        if let Some(dialog) = dialogs.borrow_mut().take().and_then(|w| w.upgrade()) {
            dialog.close();
        }
        glib::Propagation::Proceed
    });
    window.connect_destroy(move |_| {
        if let Some(token) = pending.borrow_mut().take() {
            token.cancel();
        }
        if let Some(dialog) = dialog_slot.borrow_mut().take().and_then(|w| w.upgrade()) {
            dialog.close();
        }
    });
}
