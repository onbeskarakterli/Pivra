//! Main-context folder monitoring and bounded background reconciliation.
use super::{FileContext, update_thumbnail_catalog, update_title};
use crate::{image_view::ImageView, input::OpenedFile};
use adw::prelude::*;
use gtk::{gio, glib};
use pivra::{APP_NAME, discovery};
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

// Only main-context callbacks own this state. Blocking scans receive paths and
// an atomic cancellation flag, never GTK objects. One scan and one timer maximum.
pub(super) struct FolderWatch {
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
    fallback_poll: RefCell<Option<glib::SourceId>>,
    running: Cell<bool>,
    dirty: Cell<bool>,
    initialized: Cell<bool>,
    link_monitors: RefCell<Vec<gio::FileMonitor>>,
    link_targets: RefCell<Vec<(PathBuf, PathBuf)>>,
}

impl FolderWatch {
    pub(super) fn directory(&self) -> &std::path::Path {
        &self.directory
    }

    pub(super) fn stop(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
        if let Some(timer) = self.timer.borrow_mut().take() {
            timer.remove();
        }
        if let Some(poll) = self.fallback_poll.borrow_mut().take() {
            poll.remove();
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
                        pivra::performance::emit("folder_monitor_event", "symlink", 0, 0);
                        let Some(state) = weak.upgrade() else {
                            return;
                        };
                        if state.cancelled.load(Ordering::Relaxed) {
                            return;
                        }
                        state
                            .view
                            .invalidate_cache_paths(std::slice::from_ref(&alias));
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

    pub(super) fn schedule(self: &Rc<Self>) {
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
        if self.context.borrow().own_batch_rename {
            self.dirty.set(true);
            return;
        }
        self.running.set(true);
        self.dirty.set(false);
        let directory = self.directory.clone();
        let cancelled = self.cancelled.clone();
        pivra::performance::emit("folder_scan_submitted", "queued", 0, 0);
        let waiting = pivra::performance::Span::new("folder_scan_queue_wait");
        let job = gio::spawn_blocking(move || {
            drop(waiting);
            let _timing = pivra::performance::Span::new("folder_scan_worker");
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
            if state.context.borrow().own_batch_rename {
                state.dirty.set(true);
                return;
            }
            if state.dirty.get() {
                pivra::performance::emit("folder_scan_superseded", "dirty", 0, 0);
                state.schedule(); return;
            }
            let (Some(window), Some(title), Some(status)) =
                (state.window.upgrade(), state.title.upgrade(), state.status.upgrade()) else { return; };
            let _timing = pivra::performance::Span::new("folder_catalog_apply_ui");
            let mut context = state.context.borrow_mut();
            match result {
                Ok((report, links)) => {
                    context.own_rename = None;
                    state.watch_links(links);
                    let warnings = report.issues.len() + usize::from(state.monitor.borrow().is_none());
                    for issue in &report.issues {
                        eprintln!("{APP_NAME}: folder scan incomplete: {:?}: {}", issue.path, issue.error);
                    }
                    // A periodic fallback scan must not cancel thumbnail work
                    // when the directory's media catalog is unchanged.
                    let mut items = report.collection.items().to_vec();
                    if context.order != pivra::media_sort::Order::default() {
                        context.order.sort(&mut items);
                    }
                    let catalog_changed = context.collection.items().iter().map(|item| item.path())
                        .ne(items.iter().map(|item| item.path()));
                    // A metadata-only refresh must still update the in-memory keys.
                    let missing = context.collection.replace_preserving(items);
                    if missing {
                        context.revision = context.revision.wrapping_add(1);
                        state.view.show_message("Selected media was removed or is inaccessible. Use arrow keys to continue.");
                        if let Some(banner) = state.banner.upgrade() { banner.set_reveal_child(false); }
                    }
                    pivra::performance::emit("folder_catalog_populated", "ready", 0, context.collection.items().len() as u64);
                    state.view.set_neighbors(context.collection.photo_neighbors());
                    if catalog_changed {
                        update_thumbnail_catalog(&state.view, &context);
                        state.view.invalidate_thumbnails();
                    }
                    state.view.finish_initial_thumbnail_catalog(context.collection.selected_index());
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
                    update_thumbnail_catalog(&state.view, &context);
                    // Revalidate the reconciled catalog: rebuilding the model may
                    // cancel an older metadata epoch (e.g. replace + delete).
                    state.view.invalidate_thumbnails();
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

pub(super) fn watch(
    window: &adw::ApplicationWindow,
    title: &adw::WindowTitle,
    status: &gtk::Label,
    banner: &gtk::Revealer,
    view: ImageView,
    context: Rc<RefCell<FileContext>>,
    opened: OpenedFile,
) -> Rc<FolderWatch> {
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
        fallback_poll: RefCell::new(None),
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
                pivra::performance::emit("folder_monitor_event", "directory", 0, 0);
                let Some(state) = weak.upgrade() else {
                    return;
                };
                if state.cancelled.load(Ordering::Relaxed) {
                    return;
                }
                let path = file.path();
                let affected: Vec<_> = path
                    .iter()
                    .cloned()
                    .chain(other.and_then(|f| f.path()))
                    .collect();
                state.view.invalidate_cache_paths(&affected);
                if affected
                    .iter()
                    .any(|path| pivra::rename::is_batch_temporary(path))
                {
                    state.schedule();
                    return;
                }
                let mut retarget = None;
                let mut context = state.context.borrow_mut();
                if context.own_batch_rename {
                    state.dirty.set(true);
                    return;
                }
                let own_rename = context.own_rename.as_ref().is_some_and(|(old, new)| {
                    path.as_ref() == Some(old) && other.and_then(|f| f.path()).as_ref() == Some(new)
                });
                if own_rename {
                    context.own_rename = None;
                }
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
                        if selected.as_ref() == Some(old) {
                            retarget = Some((old.clone(), new.clone()));
                        }
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
                    && !own_rename
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
                if let Some((old, new)) = retarget {
                    state.view.retarget_pending(&old, &new);
                }
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
    // Some backends do not deliver directory notifications reliably.
    // Periodically reconcile the catalog as a fallback while this window lives.
    let weak = Rc::downgrade(&state);
    let poll = glib::timeout_add_local(Duration::from_secs(5), move || {
        let Some(state) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        if state.cancelled.load(Ordering::Relaxed) {
            return glib::ControlFlow::Break;
        }
        if !state.running.get() {
            pivra::performance::emit("folder_periodic_reconcile", "existing_poll", 0, 0);
            state.schedule();
        }
        glib::ControlFlow::Continue
    });
    state.fallback_poll.replace(Some(poll));
    state.scan();
    state
}
