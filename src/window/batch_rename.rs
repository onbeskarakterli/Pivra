//! Native multi-file name preview and serialized application.
use super::{
    context_menu,
    undo::{self, Operation, Undo},
};
use crate::image_view::ImageView;
use adw::prelude::*;
use gtk::{gio, glib};
use pivra::rename::{BatchReversal, RenameStep, Ticket, validate_name};
use std::{cell::RefCell, collections::HashSet, ffi::OsString, path::PathBuf, rc::Rc, sync::Arc};

enum Mode<'a> {
    Number {
        base: &'a str,
        start: u64,
        padding: usize,
    },
    Prefix(&'a str),
    Suffix(&'a str),
    Replace {
        find: &'a str,
        replacement: &'a str,
    },
}

fn propose(paths: &[PathBuf], mode: Mode<'_>) -> Result<Vec<String>, String> {
    paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("Filename is not valid UTF-8")?;
            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .ok_or("Missing media extension")?;
            let base = match mode {
                Mode::Number {
                    base,
                    start,
                    padding,
                } => {
                    let number = start
                        .checked_add(index as u64)
                        .ok_or("Number is too large")?;
                    format!("{base}{number:0padding$}")
                }
                Mode::Prefix(prefix) => format!("{prefix}{stem}"),
                Mode::Suffix(suffix) => format!("{stem}{suffix}"),
                Mode::Replace { find: "", .. } => return Err("Enter text to find".into()),
                Mode::Replace { find, replacement } => stem.replace(find, replacement),
            };
            if base.trim().is_empty() {
                return Err("The filename stem cannot be empty".into());
            }
            let name = format!("{base}.{extension}");
            validate_name(&name)?;
            Ok(name)
        })
        .collect()
}

fn validate_plan(
    paths: &[PathBuf],
    names: &[String],
    occupied: &HashSet<OsString>,
) -> Result<(), String> {
    let original: HashSet<_> = paths.iter().filter_map(|path| path.file_name()).collect();
    let mut unique = HashSet::new();
    for name in names {
        let name = std::ffi::OsStr::new(name);
        if !unique.insert(name) {
            return Err(format!(
                "More than one file would become {}",
                name.to_string_lossy()
            ));
        }
        if occupied.contains(name) && !original.contains(name) {
            return Err(format!("{} already exists", name.to_string_lossy()));
        }
    }
    if paths
        .iter()
        .zip(names)
        .all(|(path, name)| path.file_name().is_some_and(|old| old == name.as_str()))
    {
        return Err("No names would change".into());
    }
    Ok(())
}

struct Preview {
    dialog: glib::WeakRef<adw::MessageDialog>,
    mode: gtk::DropDown,
    primary: gtk::Entry,
    replacement: gtk::Entry,
    number_row: gtk::Box,
    start: gtk::SpinButton,
    padding: gtk::SpinButton,
    names: gtk::TextBuffer,
    warning: gtk::Label,
    paths: Vec<PathBuf>,
    occupied: HashSet<OsString>,
}

impl Preview {
    fn current(&self) -> Result<Vec<String>, String> {
        let primary = self.primary.text();
        let replacement = self.replacement.text();
        let mode = match self.mode.selected() {
            0 => Mode::Number {
                base: &primary,
                start: self.start.value_as_int() as u64,
                padding: self.padding.value_as_int() as usize,
            },
            1 => Mode::Prefix(&primary),
            2 => Mode::Suffix(&primary),
            _ => Mode::Replace {
                find: &primary,
                replacement: &replacement,
            },
        };
        propose(&self.paths, mode)
    }

    fn refresh(&self) {
        let mode = self.mode.selected();
        self.number_row.set_visible(mode == 0);
        self.replacement.set_visible(mode == 3);
        self.primary.set_placeholder_text(Some(
            ["Base name", "Prefix", "Suffix", "Find"][mode.min(3) as usize],
        ));
        let result = self.current();
        let mut text = String::new();
        for (index, path) in self.paths.iter().enumerate() {
            use std::fmt::Write;
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let proposed = result
                .as_ref()
                .ok()
                .and_then(|names| names.get(index))
                .map(String::as_str)
                .unwrap_or("—");
            let _ = writeln!(text, "{name}  →  {proposed}");
        }
        self.names.set_text(&text);
        let validation =
            result.and_then(|names| validate_plan(&self.paths, &names, &self.occupied));
        self.warning
            .set_label(validation.as_ref().err().map_or("", String::as_str));
        self.warning.set_visible(validation.is_err());
        if let Some(dialog) = self.dialog.upgrade() {
            dialog.set_response_enabled("apply", validation.is_ok());
        }
    }
}

pub(super) fn install(window: &adw::ApplicationWindow, view: &ImageView, undo: &Rc<Undo>) {
    let action = gio::SimpleAction::new("batch-rename", None);
    let window_weak = window.downgrade();
    let view = view.clone();
    let undo = undo.clone();
    action.connect_activate(move |_, _| {
        let Some(window) = window_weak.upgrade() else {
            return;
        };
        let paths = view.marked_thumbnails();
        if paths.len() < 2 {
            return;
        }
        if !super::batch_operations::within_batch_limit(&undo, paths.len()) {
            return;
        }
        let Some(permit) = undo.begin() else {
            return;
        };
        context_menu::dismiss(&window);
        let weak = window.downgrade();
        let worker_paths = paths.clone();
        let capture = gio::spawn_blocking(move || prepare(&worker_paths));
        let undo = undo.clone();
        glib::MainContext::default().spawn_local(async move {
            let result = capture.await;
            let Some(window) = weak.upgrade().filter(|window| window.is_visible()) else {
                return;
            };
            match result {
                Ok(Ok((tickets, occupied))) => {
                    show(&window, paths, tickets, occupied, undo, permit)
                }
                Ok(Err(error)) => undo.notify(&format!("Could not prepare batch rename: {error}")),
                Err(_) => undo.notify("Could not prepare batch rename"),
            }
        });
    });
    window.add_action(&action);
}

fn prepare(paths: &[PathBuf]) -> std::io::Result<(Vec<Arc<Ticket>>, HashSet<OsString>)> {
    let parent = paths
        .first()
        .and_then(|path| path.parent())
        .ok_or(std::io::Error::other("Missing folder"))?;
    if paths.iter().any(|path| path.parent() != Some(parent)) {
        return Err(std::io::Error::other("Batch rename requires one folder"));
    }
    let occupied = std::fs::read_dir(parent)?
        .map(|entry| entry.map(|item| item.file_name()))
        .collect::<Result<_, _>>()?;
    let tickets = paths
        .iter()
        .map(|path| Ticket::capture(path).map(Arc::new))
        .collect::<Result<_, _>>()?;
    Ok((tickets, occupied))
}

fn show(
    window: &adw::ApplicationWindow,
    paths: Vec<PathBuf>,
    tickets: Vec<Arc<Ticket>>,
    occupied: HashSet<OsString>,
    undo: Rc<Undo>,
    permit: undo::Permit,
) {
    let dialog = adw::MessageDialog::new(Some(window), Some("Rename Selected Files"), None);
    dialog.add_responses(&[("cancel", "Cancel"), ("apply", "Rename")]);
    dialog.set_close_response("cancel");
    dialog.set_response_appearance("apply", adw::ResponseAppearance::Suggested);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let mode = gtk::DropDown::from_strings(&[
        "Sequential numbering",
        "Add prefix",
        "Add suffix",
        "Find and replace",
    ]);
    let primary = gtk::Entry::new();
    primary.set_text("IMG_");
    let replacement = gtk::Entry::builder()
        .placeholder_text("Replace with")
        .build();
    let number_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let start = gtk::SpinButton::with_range(0., 999_999., 1.);
    start.set_value(1.);
    let padding = gtk::SpinButton::with_range(1., 12., 1.);
    padding.set_value(3.);
    for widget in [
        gtk::Label::new(Some("Start")).upcast::<gtk::Widget>(),
        start.clone().upcast(),
        gtk::Label::new(Some("Digits")).upcast(),
        padding.clone().upcast(),
    ] {
        number_row.append(&widget);
    }
    let names = gtk::TextView::builder()
        .editable(false)
        .cursor_visible(false)
        .monospace(true)
        .build();
    let scroll = gtk::ScrolledWindow::builder()
        .child(&names)
        .min_content_width(420)
        .min_content_height(160)
        .build();
    let warning = gtk::Label::builder().wrap(true).build();
    warning.add_css_class("error");
    for widget in [
        mode.clone().upcast::<gtk::Widget>(),
        primary.clone().upcast(),
        replacement.clone().upcast(),
        number_row.clone().upcast(),
        scroll.upcast(),
        warning.clone().upcast(),
    ] {
        body.append(&widget);
    }
    dialog.set_extra_child(Some(&body));
    let preview = Rc::new(Preview {
        dialog: dialog.downgrade(),
        mode,
        primary,
        replacement,
        number_row,
        start,
        padding,
        names: names.buffer(),
        warning,
        paths,
        occupied,
    });
    let weak = Rc::downgrade(&preview);
    preview.mode.connect_selected_notify(move |_| {
        if let Some(state) = weak.upgrade() {
            state.refresh();
        }
    });
    for entry in [&preview.primary, &preview.replacement] {
        let weak = Rc::downgrade(&preview);
        entry.connect_changed(move |_| {
            if let Some(state) = weak.upgrade() {
                state.refresh();
            }
        });
    }
    for spin in [&preview.start, &preview.padding] {
        let weak = Rc::downgrade(&preview);
        spin.connect_value_changed(move |_| {
            if let Some(state) = weak.upgrade() {
                state.refresh();
            }
        });
    }
    preview.refresh();
    let hold = window.application().map(|app| app.downgrade());
    let pending = RefCell::new(Some((tickets, permit)));
    dialog.connect_response(None, move |_, response| {
        let Some((tickets, permit)) = pending.borrow_mut().take() else {
            return;
        };
        if response != "apply" {
            return;
        }
        let steps = (|| {
            let names = preview.current()?;
            validate_plan(&preview.paths, &names, &preview.occupied)?;
            tickets
                .into_iter()
                .zip(names)
                .map(|(ticket, name)| {
                    RenameStep::new(ticket, &name).map_err(|error| error.to_string())
                })
                .collect::<Result<Vec<_>, _>>()
        })();
        let steps = match steps {
            Ok(steps) => steps,
            Err(error) => {
                undo.notify(&error);
                return;
            }
        };
        undo.begin_rename_group();
        let job = gio::spawn_blocking(move || {
            undo::run_file_operation(|| pivra::rename::apply_batch(steps))
        });
        let undo = undo.clone();
        let hold = hold
            .as_ref()
            .and_then(|app| app.upgrade())
            .map(|app| app.hold());
        glib::MainContext::default().spawn_local(async move {
            let _permit = permit;
            let _hold = hold;
            let Ok(result) = job.await else {
                undo.finish_rename_group(&[]);
                undo.notify("Batch rename worker failed");
                return;
            };
            undo.finish_rename_group(&result.completed);
            let done = result.completed.len();
            let total = done + result.failed;
            let title = if let Some(error) = result.error {
                format!("{done} of {total} files renamed · {error}")
            } else {
                format!("{done} files renamed")
            };
            if done > 0 {
                undo.record(
                    Operation::RenameGroup(BatchReversal::new(result.completed)),
                    &title,
                );
            } else {
                undo.notify(&title);
            }
        });
    });
    dialog.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_name_modes_preserve_extensions_and_reject_conflicts() {
        let paths: Vec<_> = ["a.jpg", "a.png", "other.JPEG"]
            .into_iter()
            .map(PathBuf::from)
            .collect();
        assert_eq!(
            propose(
                &paths,
                Mode::Number {
                    base: "IMG_",
                    start: 12,
                    padding: 3
                }
            )
            .unwrap(),
            ["IMG_012.jpg", "IMG_013.png", "IMG_014.JPEG"]
        );
        assert_eq!(
            propose(&paths, Mode::Prefix("trip_")).unwrap(),
            ["trip_a.jpg", "trip_a.png", "trip_other.JPEG"]
        );
        assert_eq!(
            propose(&paths, Mode::Suffix("_edit")).unwrap(),
            ["a_edit.jpg", "a_edit.png", "other_edit.JPEG"]
        );
        assert_eq!(
            propose(
                &paths,
                Mode::Replace {
                    find: "other",
                    replacement: "new"
                }
            )
            .unwrap()[2],
            "new.JPEG"
        );
        assert!(propose(&paths, Mode::Prefix("bad/")).is_err());
        assert!(
            propose(
                &paths,
                Mode::Replace {
                    find: "",
                    replacement: "x"
                }
            )
            .is_err()
        );
        let occupied = HashSet::from([OsString::from("trip_a.jpg")]);
        assert!(
            validate_plan(
                &paths,
                &propose(&paths, Mode::Prefix("trip_")).unwrap(),
                &occupied
            )
            .is_err()
        );
        assert!(
            validate_plan(
                &paths,
                &["same.png".into(), "same.png".into(), "next.jpg".into()],
                &occupied
            )
            .is_err()
        );
    }
}
