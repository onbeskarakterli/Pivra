//! Batch file mutations, using the same identity and no-clobber rules as single actions.
use super::{
    TrashReceipt, TrashSource, context_menu, file_chooser,
    undo::{self, Operation, Undo},
};
use adw::prelude::*;
use gtk::{gio, glib};
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Mutex};
#[path = "batch_operations/transfer.rs"]
mod transfer;

pub(super) const MAX_BATCH_FILES: usize = 128;

pub(super) fn within_batch_limit(history: &Undo, count: usize) -> bool {
    if count <= MAX_BATCH_FILES {
        return true;
    }
    history.notify(&format!(
        "Select at most {MAX_BATCH_FILES} files for one operation"
    ));
    false
}

pub(super) struct TrashGroup(Mutex<Vec<TrashReceipt>>);

pub(super) struct GroupRestore {
    pub removed: Vec<PathBuf>,
    pub paths: Vec<PathBuf>,
    pub remaining: usize,
    pub message: String,
}

impl TrashGroup {
    pub(super) fn file_count(&self) -> usize {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .len()
    }

    pub(super) fn restore(&self) -> Result<GroupRestore, String> {
        let mut pending = self.0.lock().map_err(|_| "Trash history unavailable")?;
        let total = pending.len();
        let mut paths = Vec::new();
        let mut error = None;
        pending.retain(|item| match item.restore() {
            Ok(()) => {
                paths.push(item.path().to_owned());
                false
            }
            Err(message) => {
                if error.is_none() {
                    error = Some(message);
                }
                true
            }
        });
        let message = if let Some(error) = error {
            format!("{} of {total} files restored · {error}", paths.len())
        } else if paths.len() == 1 {
            "File restored".into()
        } else {
            format!("{} files restored", paths.len())
        };
        Ok(GroupRestore {
            removed: Vec::new(),
            paths,
            remaining: pending.len(),
            message,
        })
    }
}

pub(super) fn trash(window: &adw::ApplicationWindow, paths: Vec<PathBuf>, history: &Rc<Undo>) {
    if !within_batch_limit(history, paths.len()) {
        return;
    }
    let Some(permit) = history.begin() else {
        return;
    };
    let count = paths.len();
    context_menu::dismiss(window);
    let capture = gio::spawn_blocking(move || {
        paths
            .into_iter()
            .map(TrashSource::capture)
            .collect::<Vec<_>>()
    });
    let weak = window.downgrade();
    let history = history.clone();
    glib::MainContext::default().spawn_local(async move {
        let Some(window) = weak.upgrade().filter(|window| window.is_visible()) else {
            return;
        };
        let Ok(sources) = capture.await else {
            history.notify("Could not prepare the selected files");
            return;
        };
        if !window.is_visible() {
            return;
        }
        let confirm = gtk::AlertDialog::builder()
            .message(format!("Move {count} files to Trash?"))
            .buttons(["Cancel", "Move to Trash"])
            .cancel_button(0)
            .default_button(0)
            .modal(true)
            .build();
        let hold = window.application().map(|app| app.downgrade());
        confirm.choose(Some(&window), gio::Cancellable::NONE, move |answer| {
            if answer != Ok(1) {
                return;
            }
            let hold = hold.and_then(|app| app.upgrade()).map(|app| app.hold());
            let apply = gio::spawn_blocking(move || {
                undo::run_file_operation(|| {
                    let mut moved = Vec::new();
                    let mut failure = None;
                    for source in sources {
                        match source.and_then(TrashSource::trash) {
                            Ok(receipt) => moved.push(receipt),
                            Err(error) => {
                                if failure.is_none() {
                                    failure = Some(error);
                                }
                            }
                        }
                    }
                    (moved, failure)
                })
            });
            glib::MainContext::default().spawn_local(async move {
                let _permit = permit;
                let _hold = hold;
                let Ok((moved, failure)) = apply.await else {
                    history.notify("Could not move files to Trash");
                    return;
                };
                let done = moved.len();
                let title = if let Some(error) = failure {
                    format!("{done} of {count} files moved to Trash · {error}")
                } else {
                    format!("{done} files moved to Trash")
                };
                if done > 0 {
                    let removed: Vec<_> = moved.iter().map(|item| item.path().to_owned()).collect();
                    history.removed(&removed);
                    history.record(Operation::TrashGroup(TrashGroup(Mutex::new(moved))), &title);
                } else {
                    history.notify(&title);
                }
            });
        });
    });
}

pub(super) struct MoveGroup(Mutex<Vec<transfer::Moved>>);

impl MoveGroup {
    pub(super) fn file_count(&self) -> usize {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .len()
    }

    pub(super) fn restore(&self) -> Result<GroupRestore, String> {
        let mut pending = self.0.lock().map_err(|_| "Move history unavailable")?;
        let total = pending.len();
        let (mut removed, mut restored) = (Vec::new(), Vec::new());
        let mut failure = None;
        pending.retain(|item| match item.undo() {
            Ok((from, to)) => {
                removed.push(from);
                restored.push(to);
                false
            }
            Err(error) => {
                failure.get_or_insert(error);
                true
            }
        });
        let message = if let Some(error) = failure {
            format!("{} of {total} moves undone · {error}", restored.len())
        } else {
            "Move undone".into()
        };
        Ok(GroupRestore {
            removed,
            paths: restored,
            remaining: pending.len(),
            message,
        })
    }
}

pub(super) fn install_transfers(
    window: &adw::ApplicationWindow,
    context: Rc<RefCell<super::FileContext>>,
    view: &crate::image_view::ImageView,
    history: &Rc<Undo>,
) {
    for (name, moving) in [("copy-to", false), ("move-to", true)] {
        let action = gio::SimpleAction::new(name, None);
        let weak = window.downgrade();
        let context = context.clone();
        let view = view.clone();
        let history = history.clone();
        action.connect_activate(move |_, _| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut paths = view.marked_thumbnails();
            if paths.is_empty() {
                let context = context.borrow();
                let Some(item) = context.collection.current() else {
                    return;
                };
                paths.push(item.path().to_owned());
            }
            if !within_batch_limit(&history, paths.len()) {
                return;
            }
            let Some(permit) = history.begin() else {
                return;
            };
            context_menu::dismiss(&window);
            let verb = if moving { "Move" } else { "Copy" };
            let chooser = file_chooser::media_dialog(&format!("{verb} Media To"));
            chooser.set_initial_folder(paths[0].parent().map(gio::File::for_path).as_ref());
            let count = paths.len();
            let capture = gio::spawn_blocking(move || {
                paths
                    .into_iter()
                    .map(transfer::Source::capture)
                    .collect::<Vec<_>>()
            });
            let weak = window.downgrade();
            let history = history.clone();
            chooser.select_folder(Some(&window), gio::Cancellable::NONE, move |result| {
                let path = match result {
                    Ok(file) => match file.path() {
                        Some(path) => path,
                        None => {
                            history.notify("Choose a local destination folder");
                            return;
                        }
                    },
                    Err(error)
                        if error.matches(gtk::DialogError::Dismissed)
                            || error.matches(gtk::DialogError::Cancelled) =>
                    {
                        return;
                    }
                    Err(error) => {
                        history.notify(&format!("Could not open the destination: {error}"));
                        return;
                    }
                };
                glib::MainContext::default().spawn_local(async move {
                    let Ok(sources) = capture.await else {
                        history.notify("Could not prepare the selected files");
                        return;
                    };
                    let location = path.to_string_lossy().into_owned();
                    let job = gio::spawn_blocking(move || transfer::Directory::open(path));
                    let to = match job.await {
                        Ok(Ok(to)) => to,
                        _ => {
                            history.notify("Could not access the destination folder");
                            return;
                        }
                    };
                    let Some(window) = weak.upgrade().filter(|window| window.is_visible()) else {
                        return;
                    };
                    let confirm = gtk::AlertDialog::builder()
                        .message(format!("{verb} {count} files?"))
                        .detail(format!(
                            "{location}\nChoose what to do when a filename already exists."
                        ))
                        .buttons(["Cancel", "Skip Existing", "Keep Both"])
                        .cancel_button(0)
                        .default_button(0)
                        .modal(true)
                        .build();
                    let weak = window.downgrade();
                    confirm.choose(Some(&window), gio::Cancellable::NONE, move |answer| {
                        let collision = match answer {
                            Ok(1) => transfer::Collision::Skip,
                            Ok(2) => transfer::Collision::KeepBoth,
                            _ => return,
                        };
                        let Some(window) = weak.upgrade().filter(|window| window.is_visible())
                        else {
                            return;
                        };
                        let dialog = adw::MessageDialog::new(
                            Some(&window),
                            Some("File transfer"),
                            Some("Transferring media…"),
                        );
                        dialog.add_response("cancel", "Cancel");
                        dialog.set_close_response("cancel");
                        let cancel = gio::Cancellable::new();
                        let token = cancel.clone();
                        dialog.connect_response(None, move |_, _| token.cancel());
                        let closing = cancel.clone();
                        let on_close = window.connect_close_request(move |_| {
                            closing.cancel();
                            glib::Propagation::Proceed
                        });
                        let owner = window.downgrade();
                        dialog.present();
                        let hold = window.application().map(|app| app.hold());
                        let job = gio::spawn_blocking(move || {
                            undo::run_file_operation(|| {
                                let (mut created, mut removed, mut receipts) =
                                    (Vec::new(), Vec::new(), Vec::new());
                                let mut failure = None;
                                let mut skipped = 0;
                                for source in sources {
                                    if cancel.is_cancelled() {
                                        failure = Some("Cancelled".into());
                                        break;
                                    }
                                    let source = match source {
                                        Ok(source) => source,
                                        Err(error) => {
                                            failure.get_or_insert_with(|| error.to_string());
                                            continue;
                                        }
                                    };
                                    match source.transfer(&to, moving, collision, &cancel) {
                                        Ok(result) => {
                                            created.push(result.name.clone());
                                            if moving {
                                                let receipt = transfer::Moved::new(
                                                    source,
                                                    to.clone(),
                                                    result,
                                                );
                                                removed.push(receipt.original_path().to_owned());
                                                receipts.push(receipt);
                                            }
                                        }
                                        Err(error)
                                            if error.kind()
                                                == std::io::ErrorKind::AlreadyExists =>
                                        {
                                            skipped += 1
                                        }
                                        Err(error) => {
                                            failure.get_or_insert_with(|| error.to_string());
                                        }
                                    }
                                }
                                (created, removed, receipts, failure, skipped)
                            })
                        });
                        glib::MainContext::default().spawn_local(async move {
                            let _permit = permit;
                            let _hold = hold;
                            let result = job.await;
                            if let Some(window) = owner.upgrade() {
                                window.disconnect(on_close);
                            }
                            dialog.close();
                            let Ok((created, removed, receipts, failure, skipped)) = result else {
                                history.notify("File transfer worker failed");
                                return;
                            };
                            let done = created.len();
                            history.removed(&removed);
                            history.restored_many(created);
                            let verb = if moving { "moved" } else { "copied" };
                            let suffix = if let Some(failure) = failure {
                                format!(" · {failure}")
                            } else if skipped > 0 {
                                format!(" · {skipped} skipped")
                            } else {
                                String::new()
                            };
                            let title = if done == count {
                                format!("{done} files {verb}")
                            } else {
                                format!("{done} of {count} files {verb}{suffix}")
                            };
                            if receipts.is_empty() {
                                history.notify(&title);
                            } else {
                                history.record(
                                    Operation::MoveGroup(MoveGroup(Mutex::new(receipts))),
                                    &title,
                                );
                            }
                        });
                    });
                });
            });
        });
        window.add_action(&action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_group_only_retries_failed_items() {
        let from = tempfile::tempdir().unwrap();
        let to = tempfile::tempdir().unwrap();
        let target = transfer::Directory::open(to.path().to_owned()).unwrap();
        let cancel = gio::Cancellable::new();
        let receipts = ["one.png", "two.mp4"]
            .into_iter()
            .map(|name| {
                let path = from.path().join(name);
                std::fs::write(&path, name).unwrap();
                let source = transfer::Source::capture(path).unwrap();
                let result = source
                    .transfer(&target, true, transfer::Collision::Skip, &cancel)
                    .unwrap();
                transfer::Moved::new(source, target.clone(), result)
            })
            .collect();
        let group = MoveGroup(Mutex::new(receipts));
        let blocked = from.path().join("two.mp4");
        std::fs::write(&blocked, b"unrelated").unwrap();
        let first = group.restore().unwrap();
        assert_eq!(
            (first.paths.len(), first.removed.len(), first.remaining),
            (1, 1, 1)
        );
        assert_eq!(std::fs::read(&blocked).unwrap(), b"unrelated");
        std::fs::remove_file(&blocked).unwrap();
        let last = group.restore().unwrap();
        assert_eq!(
            (last.paths.len(), last.removed.len(), last.remaining),
            (1, 1, 0)
        );
        assert_eq!(std::fs::read(blocked).unwrap(), b"two.mp4");
        assert_eq!(
            std::fs::read(from.path().join("one.png")).unwrap(),
            b"one.png"
        );
    }
}
