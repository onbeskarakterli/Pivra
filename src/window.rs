//! Native windows own their catalog; filesystem scanning runs on a worker.

use crate::input::OpenedFile;
use adw::prelude::*;
use gtk::{gio, glib};
use pivra::{APP_NAME, catalog::MediaCollection, discovery, settings::Settings};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

struct FileContext {
    opened: OpenedFile,
    collection: MediaCollection,
    discovery_finished: bool,
}

pub(super) fn build(app: &adw::Application, settings: &Settings, file: Option<OpenedFile>) {
    let header = adw::HeaderBar::new();
    let title = adw::WindowTitle::new(APP_NAME, "");
    header.set_title_widget(Some(&title));
    let viewport = gtk::Box::new(gtk::Orientation::Vertical, 0);
    viewport.set_hexpand(true);
    viewport.set_vexpand(true);
    let placeholder = gtk::Label::new(None);
    placeholder.set_wrap(true);
    placeholder.set_valign(gtk::Align::Center);
    placeholder.set_vexpand(true);
    if file.is_some() {
        placeholder.set_text("Discovering neighboring media…");
        viewport.append(&placeholder);
    }
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&header);
    content.append(&viewport);
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_width(settings.window_width)
        .default_height(settings.window_height)
        .content(&content)
        .build();
    if let Some(opened) = file {
        let collection = MediaCollection::new(vec![opened.item.clone()]);
        let context = Rc::new(RefCell::new(FileContext {
            opened,
            collection,
            discovery_finished: false,
        }));
        // The map callback retains the per-window model without globals.
        let mapped = context.clone();
        let mapped_title = title.clone();
        window.connect_map(move |window| {
            let context = mapped.borrow();
            if context.discovery_finished {
                return;
            }
            let name = context
                .opened
                .item
                .path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            window.set_title(Some(&format!("{name} — {APP_NAME}")));
            mapped_title.set_title(&name);
            mapped_title.set_subtitle("Local photo");
        });
        let image =
            crate::image_view::build(&window, context.borrow().opened.item.path().to_owned());
        viewport.prepend(&image);
        placeholder.set_vexpand(false);
        window.present();
        discover(&window, &title, &placeholder, context);
    } else {
        window.present();
    }
}

fn discover(
    window: &adw::ApplicationWindow,
    title: &adw::WindowTitle,
    placeholder: &gtk::Label,
    context: Rc<RefCell<FileContext>>,
) {
    let path = context.borrow().opened.item.path().to_owned();
    let cancelled = Arc::new(AtomicBool::new(false));
    let closing = cancelled.clone();
    window.connect_close_request(move |_| {
        closing.store(true, Ordering::Relaxed);
        glib::Propagation::Proceed
    });
    let destroying = cancelled.clone();
    window.connect_destroy(move |_| destroying.store(true, Ordering::Relaxed));
    let window = window.downgrade();
    let title = title.downgrade();
    let placeholder = placeholder.downgrade();
    // Only std paths, atomics and model data cross the worker boundary.
    let job = gio::spawn_blocking(move || discovery::scan_neighbors(&path, &cancelled));
    glib::MainContext::default().spawn_local(async move {
        let result = job
            .await
            .unwrap_or_else(|_| Err(std::io::Error::other("folder worker failed")));
        let (Some(window), Some(title), Some(placeholder)) =
            (window.upgrade(), title.upgrade(), placeholder.upgrade())
        else {
            return;
        };
        if !window.is_visible() {
            return;
        }
        let mut context = context.borrow_mut();
        context.discovery_finished = true;
        match result {
            Ok(report) => {
                let warnings = report.issues.len();
                for issue in report.issues {
                    eprintln!(
                        "{APP_NAME}: folder scan incomplete: {:?}: {}",
                        issue.path, issue.error
                    );
                }
                context.collection = report.collection;
                // Selection comes from the catalog, not the directory's first entry.
                let selected = context
                    .collection
                    .current()
                    .expect("scanner selected opened file");
                let name = selected
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                let count = context.collection.items().len();
                let candidates = format!(
                    "{count} {}",
                    if count == 1 {
                        "candidate"
                    } else {
                        "candidates"
                    }
                );
                let suffix = if warnings == 0 {
                    candidates
                } else {
                    format!(
                        "{candidates}, {warnings} {}",
                        if warnings == 1 { "warning" } else { "warnings" }
                    )
                };
                window.set_title(Some(&format!("{name} — {APP_NAME} ({suffix})")));
                title.set_title(&name);
                title.set_subtitle(if warnings == 0 {
                    "Local photo"
                } else {
                    "Folder scan incomplete."
                });
                placeholder.set_text(&format!("{suffix}."));
            }
            Err(error) => {
                eprintln!("{APP_NAME}: folder scan failed: {error}");
                let name = context
                    .opened
                    .item
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                window.set_title(Some(&format!("{name} — {APP_NAME} (folder scan failed)")));
                title.set_subtitle("Folder scan failed.");
                placeholder.set_text("Folder scan failed. Keeping only the opened file.");
            }
        }
    });
}
