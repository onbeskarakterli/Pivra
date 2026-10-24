//! Each window owns its ordered catalog, keyboard actions and cancellable view.

use crate::{image_view::ImageView, input::OpenedFile};
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use pivra::{APP_NAME, catalog::MediaCollection, discovery, media::MediaKind, settings::Settings};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

struct FileContext {
    collection: MediaCollection,
    suffix: Option<String>,
    warning: bool,
    revision: u64,
}

fn update_title(window: &adw::ApplicationWindow, title: &adw::WindowTitle, context: &FileContext) {
    let Some(selected) = context.collection.current() else {
        window.set_title(Some("No media selected — Pivra"));
        title.set_title("No media selected");
        title.set_subtitle("Use arrow keys to select remaining media.");
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
            revision: 0,
        }));
        let banner = gtk::Revealer::new();
        let notice = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        notice.set_halign(gtk::Align::Center);
        notice.append(&gtk::Label::new(Some("Dosya değiştirildi — Yenile")));
        let refresh_button = gtk::Button::with_label("Yenile");
        notice.append(&refresh_button);
        banner.set_child(Some(&notice));
        viewport.prepend(&banner);
        let view = ImageView::new(&window);
        let menu = gio::Menu::new();
        for mode in pivra::viewing::ViewMode::ALL {
            let item = gio::MenuItem::new(Some(mode.label()), None);
            item.set_action_and_target_value(Some("win.image-mode"), Some(&mode.id().to_variant()));
            menu.append_item(&item);
        }
        let quality = gio::Menu::new();
        for filter in pivra::viewing::Resampling::ALL {
            let item = gio::MenuItem::new(Some(filter.label()), None);
            item.set_action_and_target_value(
                Some("win.image-resampling"),
                Some(&filter.id().to_variant()),
            );
            quality.append_item(&item);
        }
        menu.append_section(Some(pivra::viewing::Resampling::MENU), &quality);
        let mode_button = gtk::MenuButton::builder()
            .label("Fit")
            .tooltip_text(pivra::viewing::ViewMode::HELP)
            .menu_model(&menu)
            .build();
        header.pack_end(&mode_button);
        let weak_button = mode_button.downgrade();
        let action = window
            .lookup_action("image-mode")
            .expect("view mode action");
        action.connect_notify_local(Some("state"), move |action, _| {
            if let Some(button) = weak_button.upgrade() {
                let state = action.state();
                let id = state.as_ref().and_then(|v| v.str()).unwrap_or("fit");
                button.set_label(if id == "manual" {
                    pivra::viewing::ViewMode::MANUAL_LABEL
                } else {
                    pivra::viewing::ViewMode::from_id(id)
                        .unwrap_or_default()
                        .label()
                });
            }
        });
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
            let banner = banner.clone();
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
                    context.revision = context.revision.wrapping_add(1);
                    banner.set_reveal_child(false);
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
            let mode = match key {
                gdk::Key::F6 => Some("fit"),
                gdk::Key::F7 => Some("fill"),
                gdk::Key::F8 => Some("actual"),
                _ => None,
            };
            if let Some(mode) = mode {
                if let Some(window) = weak_window.upgrade() {
                    gio::prelude::ActionGroupExt::activate_action(
                        &window,
                        "image-mode",
                        Some(&mode.to_variant()),
                    );
                }
                return glib::Propagation::Stop;
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
        let reload = gio::SimpleAction::new("reload-media", None);
        let refresh_context = context.clone();
        let refresh_view = view.clone();
        let refresh_banner = banner.clone();
        reload.connect_activate(move |_, _| {
            let context = refresh_context.borrow();
            let Some(item) = context.collection.current() else {
                return;
            };
            if item.kind() == MediaKind::Video
                || !item
                    .path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|ext| {
                        ["jpg", "jpeg", "png"]
                            .iter()
                            .any(|s| ext.eq_ignore_ascii_case(s))
                    })
            {
                refresh_banner.set_reveal_child(false);
                display_selected(&refresh_view, &context);
                return;
            }
            let path = item.path().to_owned();
            let revision = context.revision;
            drop(context);
            let context = refresh_context.clone();
            let banner = refresh_banner.downgrade();
            refresh_view.load_observed(
                path,
                || {},
                move |success| {
                    if success && context.borrow().revision == revision {
                        if let Some(banner) = banner.upgrade() {
                            banner.set_reveal_child(false);
                        }
                    }
                },
            );
        });
        window.add_action(&reload);
        let weak_window = window.downgrade();
        refresh_button.connect_clicked(move |_| {
            if let Some(window) = weak_window.upgrade() {
                gio::prelude::ActionGroupExt::activate_action(&window, "reload-media", None);
            }
        });
        window.present();
        watch(
            &window,
            &title,
            &placeholder,
            &banner,
            view,
            context,
            opened,
        );
    } else {
        window.present();
    }
}

// Only main-context callbacks own this state. Blocking scans receive paths and
// an atomic cancellation flag, never GTK objects. One scan and one timer maximum.
struct FolderWatch {
    window: glib::WeakRef<adw::ApplicationWindow>,
    title: glib::WeakRef<adw::WindowTitle>,
    status: glib::WeakRef<gtk::Label>,
    banner: glib::WeakRef<gtk::Revealer>,
    context: Rc<RefCell<FileContext>>,
    view: ImageView,
    directory: PathBuf,
    cancelled: Arc<AtomicBool>,
    monitor: RefCell<Option<gio::FileMonitor>>,
    timer: RefCell<Option<glib::SourceId>>,
    running: Cell<bool>,
    dirty: Cell<bool>,
    initialized: Cell<bool>,
    link_monitors: RefCell<Vec<gio::FileMonitor>>,
    link_targets: RefCell<Vec<(PathBuf, PathBuf)>>,
}

impl FolderWatch {
    fn stop(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
        if let Some(timer) = self.timer.borrow_mut().take() {
            timer.remove();
        }
        for monitor in self.link_monitors.borrow_mut().drain(..) {
            monitor.cancel();
        }
        if let Some(monitor) = self.monitor.borrow_mut().take() {
            monitor.cancel();
        }
    }

    fn watch_links(self: &Rc<Self>, targets: Vec<(PathBuf, PathBuf)>) {
        if *self.link_targets.borrow() == targets {
            return;
        }
        for monitor in self.link_monitors.borrow_mut().drain(..) {
            monitor.cancel();
        }
        self.link_targets.replace(targets.clone());
        // Directory events refer to target names, not aliases. Follow only file
        // candidate symlinks in the same lexical folder, including dangling aliases.
        for (alias, target) in targets {
            match gio::File::for_path(target).monitor_file(
                gio::FileMonitorFlags::WATCH_MOVES,
                None::<&gio::Cancellable>,
            ) {
                Ok(monitor) => {
                    let weak = Rc::downgrade(self);
                    monitor.connect_changed(move |_, _, _, event| {
                        let Some(state) = weak.upgrade() else {
                            return;
                        };
                        if state.cancelled.load(Ordering::Relaxed) {
                            return;
                        }
                        let mut context = state.context.borrow_mut();
                        if context
                            .collection
                            .current()
                            .is_some_and(|item| item.path() == alias)
                            && !matches!(
                                event,
                                gio::FileMonitorEvent::Deleted
                                    | gio::FileMonitorEvent::MovedOut
                                    | gio::FileMonitorEvent::Unmounted
                                    | gio::FileMonitorEvent::PreUnmount
                            )
                        {
                            context.revision = context.revision.wrapping_add(1);
                            if let Some(banner) = state.banner.upgrade() {
                                banner.set_reveal_child(true);
                            }
                        }
                        drop(context);
                        state.schedule();
                    });
                    self.link_monitors.borrow_mut().push(monitor);
                }
                Err(error) => eprintln!("{APP_NAME}: symlink target monitor unavailable: {error}"),
            }
        }
    }

    fn schedule(self: &Rc<Self>) {
        if self.cancelled.load(Ordering::Relaxed) {
            return;
        }
        self.dirty.set(true);
        if self.running.get() || self.timer.borrow().is_some() {
            return;
        }
        let state = self.clone();
        // Fixed trailing window: sustained events cannot postpone scans forever.
        let timer = glib::timeout_add_local_once(Duration::from_millis(120), move || {
            state.timer.borrow_mut().take();
            state.scan();
        });
        self.timer.replace(Some(timer));
    }

    fn scan(self: &Rc<Self>) {
        if self.cancelled.load(Ordering::Relaxed) {
            return;
        }
        self.running.set(true);
        self.dirty.set(false);
        let directory = self.directory.clone();
        let cancelled = self.cancelled.clone();
        let job = gio::spawn_blocking(move || {
            discovery::scan_directory(&directory, &cancelled).map(|report| {
                let mut links: Vec<_> = std::fs::read_dir(&directory)
                    .into_iter()
                    .flatten()
                    .take_while(|_| !cancelled.load(Ordering::Relaxed))
                    .filter_map(|entry| {
                        let path = entry.ok()?.path();
                        discovery::candidate_kind(&path)?;
                        if !std::fs::symlink_metadata(&path).ok()?.is_symlink() {
                            return None;
                        }
                        let target = std::fs::read_link(&path).ok()?;
                        let target = if target.is_absolute() {
                            target
                        } else {
                            directory.join(target)
                        };
                        let target = std::fs::canonicalize(&target).unwrap_or(target);
                        Some((path, target))
                    })
                    .collect();
                links.sort();
                (report, links)
            })
        });
        let state = self.clone();
        glib::MainContext::default().spawn_local(async move {
            let result = job.await.unwrap_or_else(|_| Err(std::io::Error::other("folder worker failed")));
            state.running.set(false);
            if state.cancelled.load(Ordering::Relaxed) { return; }
            if state.dirty.get() { state.schedule(); return; }
            let (Some(window), Some(title), Some(status)) =
                (state.window.upgrade(), state.title.upgrade(), state.status.upgrade()) else { return; };
            let mut context = state.context.borrow_mut();
            match result {
                Ok((report, links)) => {
                    state.watch_links(links);
                    let warnings = report.issues.len() + usize::from(state.monitor.borrow().is_none());
                    for issue in &report.issues {
                        eprintln!("{APP_NAME}: folder scan incomplete: {:?}: {}", issue.path, issue.error);
                    }
                    let missing = context.collection.replace_preserving(report.collection.items().to_vec());
                    if missing {
                        context.revision = context.revision.wrapping_add(1);
                        state.view.show_message("Selected media was removed or is inaccessible. Use arrow keys to continue.");
                        if let Some(banner) = state.banner.upgrade() { banner.set_reveal_child(false); }
                    }
                    let count = context.collection.items().len();
                    let suffix = if warnings == 0 {
                        format!("{count} {}", if count == 1 { "candidate" } else { "candidates" })
                    } else { format!("{count} {}, {warnings} {}", if count == 1 { "candidate" } else { "candidates" }, if warnings == 1 { "warning" } else { "warnings" }) };
                    status.set_text(&if state.monitor.borrow().is_none() {
                        format!("{suffix}. Folder monitoring unavailable; reopen to update.")
                    } else { format!("{suffix}.") });
                    context.suffix = Some(suffix);
                    context.warning = warnings != 0;
                }
                Err(error) => {
                    eprintln!("{APP_NAME}: folder scan failed: {error}");
                    if state.initialized.get() {
                        context.collection.replace_preserving(vec![]);
                    }
                    context.suffix = Some("folder scan failed".into());
                    context.warning = true;
                    if state.initialized.get() {
                        state.view.show_message("Folder unavailable. Media cannot be accessed.");
                    }
                    if let Some(banner) = state.banner.upgrade() { banner.set_reveal_child(false); }
                    status.set_text("Folder scan failed. Folder unavailable.");
                }
            }
            state.initialized.set(true);
            update_title(&window, &title, &context);
        });
    }
}

fn watch(
    window: &adw::ApplicationWindow,
    title: &adw::WindowTitle,
    status: &gtk::Label,
    banner: &gtk::Revealer,
    view: ImageView,
    context: Rc<RefCell<FileContext>>,
    opened: OpenedFile,
) {
    let directory = opened
        .item
        .path()
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_owned();
    let state = Rc::new(FolderWatch {
        window: window.downgrade(),
        title: title.downgrade(),
        status: status.downgrade(),
        banner: banner.downgrade(),
        context,
        view,
        directory: directory.clone(),
        cancelled: Arc::new(AtomicBool::new(false)),
        monitor: RefCell::new(None),
        timer: RefCell::new(None),
        running: Cell::new(false),
        dirty: Cell::new(false),
        initialized: Cell::new(false),
        link_monitors: RefCell::new(vec![]),
        link_targets: RefCell::new(vec![]),
    });
    match gio::File::for_path(&directory).monitor_directory(
        gio::FileMonitorFlags::WATCH_MOVES,
        None::<&gio::Cancellable>,
    ) {
        Ok(monitor) => {
            let weak = Rc::downgrade(&state);
            monitor.connect_changed(move |_, file, other, event| {
                let Some(state) = weak.upgrade() else {
                    return;
                };
                if state.cancelled.load(Ordering::Relaxed) {
                    return;
                }
                let path = file.path();
                let mut context = state.context.borrow_mut();
                let selected = context
                    .collection
                    .current()
                    .map(|item| item.path().to_owned());
                if matches!(
                    event,
                    gio::FileMonitorEvent::Renamed | gio::FileMonitorEvent::Moved
                ) {
                    if let (Some(old), Some(new)) =
                        (path.as_ref(), other.and_then(|file| file.path()))
                    {
                        context.collection.rename_selected(old, &new);
                    }
                }
                let changed = matches!(
                    event,
                    gio::FileMonitorEvent::Changed
                        | gio::FileMonitorEvent::ChangesDoneHint
                        | gio::FileMonitorEvent::AttributeChanged
                        | gio::FileMonitorEvent::Created
                        | gio::FileMonitorEvent::MovedIn
                        | gio::FileMonitorEvent::Renamed
                        | gio::FileMonitorEvent::Moved
                );
                if changed
                    && selected.as_ref().is_some_and(|selected| {
                        path.as_ref() == Some(selected)
                            || other.and_then(|file| file.path()).as_ref() == Some(selected)
                    })
                {
                    if let Some(banner) = state.banner.upgrade() {
                        context.revision = context.revision.wrapping_add(1);
                        banner.set_reveal_child(true);
                    }
                }
                drop(context);
                state.schedule();
            });
            state.monitor.replace(Some(monitor));
        }
        Err(error) => {
            eprintln!("{APP_NAME}: folder monitor unavailable: {error}");
            status
                .set_text("Folder monitoring unavailable; reopen the file to update its catalog.");
        }
    }
    let closing = state.clone();
    window.connect_close_request(move |_| {
        closing.stop();
        glib::Propagation::Proceed
    });
    let destroying = state.clone();
    window.connect_destroy(move |_| destroying.stop());
    state.scan();
}
