//! Batch file mutations, using the same identity and no-clobber rules as single actions.
use super::{
    TrashReceipt, TrashSource, context_menu,
    undo::{self, Operation, Undo},
};
use adw::prelude::*;
use gtk::{gio, glib};
use std::{path::PathBuf, rc::Rc, sync::Mutex};

pub(super) struct TrashGroup(Mutex<Vec<TrashReceipt>>);

pub(super) struct TrashRestore {
    pub paths: Vec<PathBuf>,
    pub remaining: usize,
    pub message: String,
}

impl TrashGroup {
    pub(super) fn restore(&self) -> Result<TrashRestore, String> {
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
        Ok(TrashRestore {
            paths,
            remaining: pending.len(),
            message,
        })
    }
}

pub(super) fn trash(window: &adw::ApplicationWindow, paths: Vec<PathBuf>, history: &Rc<Undo>) {
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
