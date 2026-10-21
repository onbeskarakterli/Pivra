//! Each window owns its ordered catalog, keyboard actions and cancellable view.

use crate::{image_view::ImageView, input::OpenedFile};
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use pivra::{APP_NAME, catalog::MediaCollection, discovery, media::MediaKind, settings::Settings};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

struct FileContext {
    collection: MediaCollection,
    suffix: Option<String>,
    warning: bool,
}

fn update_title(window: &adw::ApplicationWindow, title: &adw::WindowTitle, context: &FileContext) {
    let Some(selected) = context.collection.current() else {
        return;
    };
    let name = selected
        .path()
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let suffix = context
        .suffix
        .as_ref()
        .map(|s| format!(" ({s})"))
        .unwrap_or_default();
    window.set_title(Some(&format!("{name} — {APP_NAME}{suffix}")));
    title.set_title(&name);
    title.set_subtitle(if context.warning {
        "Folder scan incomplete."
    } else if selected.kind() == MediaKind::Video {
        "Video playback is not available yet."
    } else {
        "Local photo"
    });
}

fn display_selected(view: &ImageView, context: &FileContext) {
    let Some(item) = context.collection.current() else {
        view.show_message("No media selected.");
        return;
    };
    if item.kind() == MediaKind::Video {
        view.show_message("Video playback is not available yet.");
    } else if item
        .path()
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| {
            ["jpg", "jpeg", "png"]
                .iter()
                .any(|s| ext.eq_ignore_ascii_case(s))
        })
    {
        view.load(item.path().to_owned());
    } else {
        view.show_message("This image format is not supported yet. JPEG and PNG only.");
    }
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
        let context = Rc::new(RefCell::new(FileContext {
            collection: MediaCollection::new(vec![opened.item.clone()]),
            suffix: None,
            warning: false,
        }));
        let view = ImageView::new(&window);
        viewport.prepend(view.widget());
        placeholder.set_vexpand(false);
        update_title(&window, &title, &context.borrow());
        display_selected(&view, &context.borrow());
        for (name, forward) in [("previous-media", false), ("next-media", true)] {
            let action = gio::SimpleAction::new(name, None);
            let weak_window = window.downgrade();
            let title = title.clone();
            let context = context.clone();
            let view = view.clone();
            action.connect_activate(move |_, _| {
                let Some(window) = weak_window.upgrade() else {
                    return;
                };
                let mut context = context.borrow_mut();
                let changed = if forward {
                    context.collection.select_next()
                } else {
                    context.collection.select_previous()
                };
                if changed {
                    update_title(&window, &title, &context);
                    display_selected(&view, &context);
                }
            });
            window.add_action(&action);
        }
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak_window = window.downgrade();
        keys.connect_key_pressed(move |_, key, _, modifiers| {
            if modifiers.intersects(
                gdk::ModifierType::CONTROL_MASK
                    | gdk::ModifierType::ALT_MASK
                    | gdk::ModifierType::SUPER_MASK,
            ) {
                return glib::Propagation::Proceed;
            }
            let name = match key {
                gdk::Key::Left => "previous-media",
                gdk::Key::Right => "next-media",
                _ => return glib::Propagation::Proceed,
            };
            if let Some(window) = weak_window.upgrade() {
                gio::prelude::ActionGroupExt::activate_action(&window, name, None);
            }
            glib::Propagation::Stop
        });
        window.add_controller(keys);
        window.present();
        discover(&window, &title, &placeholder, context, opened);
    } else {
        window.present();
    }
}

fn discover(
    window: &adw::ApplicationWindow,
    title: &adw::WindowTitle,
    placeholder: &gtk::Label,
    context: Rc<RefCell<FileContext>>,
    opened: OpenedFile,
) {
    let path = opened.item.path().to_owned();
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
                placeholder.set_text(&format!("{suffix}."));
                context.suffix = Some(suffix);
                context.warning = warnings != 0;
            }
            Err(error) => {
                eprintln!("{APP_NAME}: folder scan failed: {error}");
                context.suffix = Some("folder scan failed".into());
                context.warning = true;
                placeholder.set_text("Folder scan failed. Keeping only the opened file.");
            }
        }
        update_title(&window, &title, &context);
    });
}
