//! One current-file rename dialog, reusing the live viewer and folder monitor.
use super::*;
pub(super) fn install(
    window: &adw::ApplicationWindow,
    title: &adw::WindowTitle,
    context: Rc<RefCell<FileContext>>,
    view: ImageView,
    banner: &gtk::Revealer,
    watches: Rc<RefCell<Option<Rc<FolderWatch>>>>,
    toasts: &adw::ToastOverlay,
    current_toast: Rc<RefCell<Option<adw::Toast>>>,
) {
    let action = gio::SimpleAction::new("rename-media", None);
    let weak_window = window.downgrade();
    let title = title.downgrade();
    let banner = banner.downgrade();
    let toasts = toasts.downgrade();
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
        let title = title.clone();
        let banner = banner.clone();
        let toasts = toasts.clone();
        let current_toast = current_toast.clone();
        let context = context.clone();
        let view = view.clone();
        let watches = watches.clone();
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
                    if let Some(t) = toasts.upgrade() {
                        show_latest_toast(
                            &t,
                            &current_toast,
                            adw::Toast::new(&format!(
                                "Could not prepare rename: {}",
                                match other {
                                    Ok(Err(e)) => e.to_string(),
                                    _ => "worker failed".into(),
                                }
                            )),
                        );
                    }
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
                let target = path.parent().unwrap().join(&name);
                context.borrow_mut().own_rename = Some((path.clone(), target));
                let ticket = ticket.clone();
                let path = path.clone();
                let context = context.clone();
                let view = view.clone();
                let watches = watches.clone();
                let action = action.clone();
                let weak_window = weak_window.clone();
                let title = title.clone();
                let banner = banner.clone();
                let toasts = toasts.clone();
                let current_toast = current_toast.clone();
                glib::MainContext::default().spawn_local(async move {
                    let old = path.clone();
                    let result = gio::spawn_blocking(move || {
                        let fields = pivra::media_information::lookup(&old);
                        let new = ticket.rename(&name)?;
                        if let Ok(key) = pivra::image_cache::FileKey::read(&new) {
                            pivra::media_information::remember(key, fields);
                        }
                        Ok::<_, std::io::Error>(new)
                    })
                    .await;
                    if let Some(a) = action.upgrade() {
                        a.set_enabled(true);
                    }
                    let Some(window) = weak_window.upgrade().filter(|w| w.is_visible()) else {
                        return;
                    };
                    match result {
                        Ok(Ok(new)) => {
                            let mut context = context.borrow_mut();
                            context.collection.renamed(&path, &new);
                            context.revision = context.revision.wrapping_add(1);
                            if let Some(title) = title.upgrade() {
                                update_title(&window, &title, &context);
                            }
                            if let Some(banner) = banner.upgrade() {
                                banner.set_reveal_child(false);
                            }
                            view.media_renamed(&path, &new);
                            update_thumbnail_catalog(&view, &context);
                            view.set_neighbors(context.collection.photo_neighbors());
                            drop(context);
                            if let Some(watch) = watches.borrow().as_ref() {
                                watch.schedule();
                            }
                        }
                        other => {
                            context.borrow_mut().own_rename = None;
                            if let Some(t) = toasts.upgrade() {
                                show_latest_toast(
                                    &t,
                                    &current_toast,
                                    adw::Toast::new(&format!(
                                        "Could not rename file: {}",
                                        match other {
                                            Ok(Err(e)) => e.to_string(),
                                            _ => "worker failed".into(),
                                        }
                                    )),
                                );
                            }
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
