//! One current-file rename dialog, reusing the live viewer and folder monitor.
use super::*;
use std::{io, path::Path};

pub(super) fn with_metadata<T>(
    from: &Path,
    to: &Path,
    rename: impl FnOnce() -> io::Result<T>,
) -> io::Result<T> {
    let fields = pivra::media_information::lookup(from);
    let result = rename()?;
    if let Ok(key) = pivra::image_cache::FileKey::read(to) {
        pivra::media_information::remember(key, fields);
    }
    Ok(result)
}

pub(super) fn install(
    window: &adw::ApplicationWindow,
    context: Rc<RefCell<FileContext>>,
    undo: Rc<undo::Undo>,
) {
    let action = gio::SimpleAction::new("rename-media", None);
    let weak_window = window.downgrade();
    action.connect_activate(move |action, _| {
        let Some(path) = context
            .borrow()
            .collection
            .current()
            .map(|i| i.path().to_owned())
        else {
            return;
        };
        let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_owned) else {
            return;
        };
        action.set_enabled(false);
        let action = action.downgrade();
        let weak_window = weak_window.clone();
        let context = context.clone();
        let undo = undo.clone();
        glib::MainContext::default().spawn_local(async move {
            let source = path.clone();
            let ticket = gio::spawn_blocking(move || pivra::rename::Ticket::capture(&source)).await;
            let Some(window) = weak_window.upgrade().filter(|w| w.is_visible()) else {
                return;
            };
            let ticket = match ticket {
                Ok(Ok(t)) => Arc::new(t),
                other => {
                    if let Some(a) = action.upgrade() {
                        a.set_enabled(true);
                    }
                    let error = match other {
                        Ok(Err(error)) => error.to_string(),
                        _ => "worker failed".into(),
                    };
                    undo.notify(&format!("Could not prepare rename: {error}"));
                    return;
                }
            };
            if context
                .borrow()
                .collection
                .current()
                .is_none_or(|i| i.path() != path)
            {
                if let Some(a) = action.upgrade() {
                    a.set_enabled(true);
                }
                return;
            }
            let dialog = adw::MessageDialog::new(Some(&window), Some("Rename media"), None);
            let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
            let entry = gtk::Entry::new();
            entry.set_text(&name);
            entry.set_activates_default(true);
            entry.set_widget_name("rename-filename");
            let error = gtk::Label::new(None);
            error.set_wrap(true);
            error.add_css_class("error");
            error.set_visible(false);
            body.append(&entry);
            body.append(&error);
            dialog.set_extra_child(Some(&body));
            dialog.add_responses(&[("cancel", "Cancel"), ("rename", "Rename")]);
            dialog.set_response_appearance("rename", adw::ResponseAppearance::Suggested);
            dialog.set_default_response(Some("rename"));
            dialog.set_close_response("cancel");
            let weak = dialog.downgrade();
            entry.connect_changed(move |entry| {
                let invalid = pivra::rename::validate_name(entry.text().as_str()).err();
                error.set_text(invalid.unwrap_or(""));
                error.set_visible(invalid.is_some());
                if let Some(d) = weak.upgrade() {
                    d.set_response_enabled("rename", invalid.is_none());
                }
            });
            let stem_end =
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map_or(name.chars().count(), |s| s.chars().count()) as i32;
            let response_entry = entry.clone();
            dialog.connect_response(None, move |_, response| {
                if response != "rename" {
                    if let Some(a) = action.upgrade() {
                        a.set_enabled(true);
                    }
                    return;
                }
                let name = response_entry.text().to_string();
                if pivra::rename::validate_name(&name).is_err() {
                    if let Some(a) = action.upgrade() {
                        a.set_enabled(true);
                    }
                    return;
                }
                let Some(permit) = undo.begin() else {
                    if let Some(action) = action.upgrade() {
                        action.set_enabled(true);
                    }
                    return;
                };
                let target = path.with_file_name(&name);
                let marker = undo.mark_rename(&path, &target);
                let ticket = ticket.clone();
                let path = path.clone();
                let action = action.clone();
                let undo = undo.clone();
                let job = gio::spawn_blocking(move || {
                    undo::run_file_operation(|| {
                        with_metadata(&path, &target, || ticket.rename_with_undo(&name))
                    })
                });
                glib::MainContext::default().spawn_local(async move {
                    let _permit = permit;
                    let result = job.await;
                    if let Some(action) = action.upgrade() {
                        action.set_enabled(true);
                    }
                    match result {
                        Ok(Ok(Some(rename))) => {
                            let (original, renamed) = rename.paths();
                            undo.renamed(original, renamed);
                            undo.record(undo::Operation::Rename(rename), "File renamed");
                        }
                        Ok(Ok(None)) => (),
                        other => {
                            undo.clear_marker(marker);
                            let error = match other {
                                Ok(Err(error)) => error.to_string(),
                                _ => "worker failed".into(),
                            };
                            undo.notify(&format!("Could not rename file: {error}"));
                        }
                    }
                });
            });
            dialog.present();
            entry.grab_focus();
            entry.select_region(0, stem_end);
        });
    });
    window.add_action(&action);
}
