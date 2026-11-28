//! Window-local history for successful filesystem operations.

use super::{
    FileContext, FolderWatch, TrashReceipt, batch_operations, display_selected, show_latest_toast,
    update_thumbnail_catalog, update_title,
};
use crate::image_view::ImageView;
use adw::prelude::*;
use gtk::{gio, glib};
use pivra::{
    media::MediaItem,
    rename::{BatchReversal, RenameResult, RenameStep, Reversal},
};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
};

const HISTORY_LIMIT: usize = 32;
const HISTORY_FILE_LIMIT: usize = 192;
static FILE_OPERATIONS: Mutex<()> = Mutex::new(());

pub(super) fn run_file_operation<T>(operation: impl FnOnce() -> T) -> T {
    // Different windows may operate on the same file. Keep the lock off GTK's thread.
    let _guard = FILE_OPERATIONS
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    operation()
}

pub(super) enum Operation {
    Rename(Reversal),
    Trash(TrashReceipt),
    TrashGroup(batch_operations::TrashGroup),
    MoveGroup(batch_operations::MoveGroup),
    RenameGroup(BatchReversal),
}

enum Outcome {
    Single,
    Group(batch_operations::GroupRestore),
    Renamed(RenameResult),
}

impl Operation {
    fn file_count(&self) -> usize {
        match self {
            Self::Rename(_) | Self::Trash(_) => 1,
            Self::TrashGroup(group) => group.file_count(),
            Self::MoveGroup(group) => group.file_count(),
            Self::RenameGroup(group) => group.file_count(),
        }
    }

    fn undo(&self) -> Result<Outcome, String> {
        match self {
            Self::Rename(rename) => {
                let (original, renamed) = rename.paths();
                super::rename_dialog::with_metadata(renamed, original, || rename.undo())
                    .map(|()| Outcome::Single)
                    .map_err(|error| error.to_string())
            }
            Self::Trash(trash) => trash.restore().map(|()| Outcome::Single),
            Self::TrashGroup(group) => group.restore().map(Outcome::Group),
            Self::MoveGroup(group) => group.restore().map(Outcome::Group),
            Self::RenameGroup(group) => group.undo().map(Outcome::Renamed),
        }
    }
}

#[derive(Default)]
struct History {
    next: u64,
    entries: VecDeque<(u64, Arc<Operation>, usize)>,
}

impl History {
    fn record(&mut self, operation: Operation) -> u64 {
        self.next += 1;
        let files = operation.file_count();
        let mut reserved: usize = self.entries.iter().map(|(_, _, count)| count).sum();
        // Evict complete chronological operations. Never truncate a group.
        while self.entries.len() >= HISTORY_LIMIT || reserved + files > HISTORY_FILE_LIMIT {
            let Some((_, _, count)) = self.entries.pop_front() else {
                break;
            };
            reserved -= count;
        }
        self.entries
            .push_back((self.next, Arc::new(operation), files));
        self.next
    }

    fn latest(&self, requested: Option<u64>) -> Option<(u64, Arc<Operation>)> {
        self.entries
            .back()
            .filter(|(id, _, _)| requested.is_none_or(|requested| requested == *id))
            .map(|(id, operation, _)| (*id, operation.clone()))
    }
}

pub(super) struct Undo {
    history: RefCell<History>,
    busy: Cell<bool>,
    action: gio::SimpleAction,
    window: glib::WeakRef<adw::ApplicationWindow>,
    overlay: glib::WeakRef<adw::ToastOverlay>,
    current_toast: Rc<RefCell<Option<adw::Toast>>>,
    title: glib::WeakRef<adw::WindowTitle>,
    banner: glib::WeakRef<gtk::Revealer>,
    context: Rc<RefCell<FileContext>>,
    view: ImageView,
    watches: Rc<RefCell<Option<Rc<FolderWatch>>>>,
}

impl Undo {
    pub(super) fn install(
        window: &adw::ApplicationWindow,
        overlay: &adw::ToastOverlay,
        current_toast: Rc<RefCell<Option<adw::Toast>>>,
        title: &adw::WindowTitle,
        banner: &gtk::Revealer,
        context: Rc<RefCell<FileContext>>,
        view: ImageView,
        watches: Rc<RefCell<Option<Rc<FolderWatch>>>>,
    ) -> Rc<Self> {
        let action = gio::SimpleAction::new("undo", None);
        action.set_enabled(false);
        let manager = Rc::new(Self {
            history: RefCell::default(),
            busy: Cell::new(false),
            action: action.clone(),
            window: window.downgrade(),
            overlay: overlay.downgrade(),
            current_toast,
            title: title.downgrade(),
            banner: banner.downgrade(),
            context,
            view,
            watches,
        });
        let weak = Rc::downgrade(&manager);
        action.connect_activate(move |_, _| {
            if let Some(manager) = weak.upgrade() {
                manager.request(None);
            }
        });
        window.add_action(&action);
        manager
    }

    fn update_action(&self) {
        self.action
            .set_enabled(!self.busy.get() && self.history.borrow().latest(None).is_some());
    }

    pub(super) fn begin(self: &Rc<Self>) -> Option<Permit> {
        if self.busy.replace(true) {
            self.notify("Another file operation is in progress");
            return None;
        }
        self.update_action();
        Some(Permit(self.clone()))
    }

    fn show(&self, toast: adw::Toast) {
        if let Some(overlay) = self.overlay.upgrade() {
            show_latest_toast(&overlay, &self.current_toast, toast);
        }
    }

    pub(super) fn notify(&self, title: &str) {
        self.show(adw::Toast::new(&glib::markup_escape_text(title)));
    }

    pub(super) fn record(self: &Rc<Self>, operation: Operation, title: &str) {
        let id = self.history.borrow_mut().record(operation);
        self.update_action();
        let toast = adw::Toast::new(&glib::markup_escape_text(title));
        toast.set_button_label(Some("Undo"));
        let weak = Rc::downgrade(self);
        toast.connect_button_clicked(move |toast| {
            if let Some(manager) = weak.upgrade() {
                let is_current = manager.current_toast.borrow().as_ref() == Some(toast);
                if is_current {
                    manager.request(Some(id));
                }
            }
        });
        self.show(toast);
    }

    fn request(self: &Rc<Self>, requested: Option<u64>) {
        let latest = self.history.borrow().latest(requested);
        let Some((_, operation)) = latest else {
            return;
        };
        let Some(permit) = self.begin() else {
            return;
        };
        let marker = match operation.as_ref() {
            Operation::Rename(rename) => {
                let (original, renamed) = rename.paths();
                self.mark_rename(renamed, original)
            }
            Operation::RenameGroup(_) => {
                self.begin_rename_group();
                None
            }
            Operation::Trash(_) | Operation::TrashGroup(_) | Operation::MoveGroup(_) => None,
        };
        let worker_operation = operation.clone();
        let job = gio::spawn_blocking(move || run_file_operation(|| worker_operation.undo()));
        let hold = self
            .window
            .upgrade()
            .and_then(|window| window.application())
            .map(|app| app.hold());
        let manager = self.clone();
        glib::MainContext::default().spawn_local(async move {
            let _permit = permit;
            let _hold = hold;
            match job.await {
                Ok(Ok(Outcome::Renamed(result))) => {
                    let done = result.completed.len();
                    manager.finish_rename_group(&result.completed);
                    if result.failed == 0 {
                        manager.history.borrow_mut().entries.pop_back();
                        manager.notify("Rename undone");
                    } else {
                        let total = done + result.failed;
                        let error = result.error.as_deref().unwrap_or("Could not restore name");
                        manager.notify(&format!("{done} of {total} renames undone · {error}"));
                    }
                }
                Ok(Ok(Outcome::Group(restore))) => {
                    if restore.remaining == 0 {
                        manager.history.borrow_mut().entries.pop_back();
                    }
                    manager.removed(&restore.removed);
                    manager.restored_many(restore.paths);
                    manager.notify(&restore.message);
                }
                Ok(Ok(Outcome::Single)) => {
                    manager.history.borrow_mut().entries.pop_back();
                    match operation.as_ref() {
                        Operation::Rename(rename) => {
                            let (original, renamed) = rename.paths();
                            manager.renamed(renamed, original);
                            manager.notify("Rename undone");
                        }
                        Operation::Trash(trash) => {
                            manager.restored_many(vec![trash.path().to_owned()]);
                            manager.notify("File restored");
                        }
                        Operation::TrashGroup(_)
                        | Operation::MoveGroup(_)
                        | Operation::RenameGroup(_) => {
                            unreachable!("group result handled separately")
                        }
                    }
                }
                result => {
                    if matches!(operation.as_ref(), Operation::RenameGroup(_)) {
                        manager.finish_rename_group(&[]);
                    }
                    manager.clear_marker(marker);
                    let message = result
                        .ok()
                        .and_then(Result::err)
                        .unwrap_or_else(|| "file worker failed".into());
                    manager.notify(&format!("Undo failed: {message}"));
                }
            }
        });
    }

    fn watch_for(&self, path: &Path) -> Option<Rc<FolderWatch>> {
        self.watches
            .borrow()
            .as_ref()
            .filter(|watch| path.parent() == Some(watch.directory()))
            .cloned()
    }

    pub(super) fn mark_rename(
        &self,
        from: &Path,
        to: &Path,
    ) -> Option<(std::path::PathBuf, std::path::PathBuf)> {
        if from == to || self.watch_for(from).is_none() {
            return None;
        }
        let pair = (from.to_owned(), to.to_owned());
        self.context.borrow_mut().own_rename = Some(pair.clone());
        Some(pair)
    }

    pub(super) fn clear_marker(&self, marker: Option<(std::path::PathBuf, std::path::PathBuf)>) {
        let mut context = self.context.borrow_mut();
        if context.own_rename == marker {
            context.own_rename = None;
        }
    }

    pub(super) fn renamed(&self, from: &Path, to: &Path) {
        let Some(watch) = self.watch_for(to) else {
            return;
        };
        let mut context = self.context.borrow_mut();
        context.collection.renamed(from, to);
        let order = context.order;
        if order != pivra::media_sort::Order::default() {
            context.collection.sort_preserving(order);
        }
        context.revision = context.revision.wrapping_add(1);
        self.view.media_renamed(from, to);
        self.reconcile(&context, false);
        watch.schedule();
    }

    pub(super) fn begin_rename_group(&self) {
        self.context.borrow_mut().own_batch_rename = true;
    }

    pub(super) fn finish_rename_group(&self, changed: &[RenameStep]) {
        let mut context = self.context.borrow_mut();
        context.own_batch_rename = false;
        let Some(watch) = self.watches.borrow().clone() else {
            return;
        };
        watch.schedule();
        let changes: std::collections::HashMap<_, _> = changed
            .iter()
            .map(RenameStep::paths)
            .filter(|(old, _)| old.parent() == Some(watch.directory()))
            .collect();
        if changes.is_empty() {
            return;
        }
        let selection = context
            .collection
            .current()
            .map(|item| item.path().to_owned());
        let mut items: Vec<_> = context
            .collection
            .items()
            .iter()
            .map(|item| {
                changes
                    .get(item.path())
                    .map_or_else(|| item.clone(), |new| item.at_path(*new))
            })
            .collect();
        context.order.sort(&mut items);
        context.collection.replace_preserving(items);
        if let Some(old) = selection
            && let Some(new) = changes.get(old.as_path())
        {
            if let Some(index) = context
                .collection
                .items()
                .iter()
                .position(|item| item.path() == *new)
            {
                context.collection.select(index);
                context.revision = context.revision.wrapping_add(1);
                self.view.media_renamed(&old, new);
            }
        }
        self.reconcile(&context, false);
    }

    pub(super) fn removed(&self, paths: &[PathBuf]) {
        let Some(watch) = self.watches.borrow().clone() else {
            return;
        };
        let paths: std::collections::HashSet<_> = paths
            .iter()
            .filter(|path| path.parent() == Some(watch.directory()))
            .map(PathBuf::as_path)
            .collect();
        if paths.is_empty() {
            return;
        }
        let mut context = self.context.borrow_mut();
        let old_index = context.collection.selected_index().unwrap_or(0);
        let items = context
            .collection
            .items()
            .iter()
            .filter(|item| !paths.contains(item.path()))
            .cloned()
            .collect();
        let removed_active = context.collection.replace_preserving(items);
        if removed_active {
            let next = old_index.min(context.collection.items().len().saturating_sub(1));
            context.collection.select(next);
            context.revision = context.revision.wrapping_add(1);
        }
        self.reconcile(&context, removed_active);
        watch.schedule();
    }

    pub(super) fn restored_many(&self, paths: Vec<PathBuf>) {
        let Some(watch) = self.watches.borrow().clone() else {
            return;
        };
        let additions: Vec<_> = paths
            .into_iter()
            .filter(|path| path.parent() == Some(watch.directory()))
            .filter_map(|path| {
                pivra::discovery::candidate_kind(&path).map(|kind| MediaItem::new(path, kind))
            })
            .collect();
        if additions.is_empty() {
            return;
        }
        let mut context = self.context.borrow_mut();
        let new_selection = additions[0].path().to_owned();
        let mut items = context.collection.items().to_vec();
        let known: std::collections::HashSet<_> =
            items.iter().map(|item| item.path().to_owned()).collect();
        items.extend(
            additions
                .into_iter()
                .filter(|item| !known.contains(item.path())),
        );
        context.order.sort(&mut items);
        context.collection.replace_preserving(items);
        let select_restored = context.collection.current().is_none();
        if select_restored {
            if let Some(index) = context
                .collection
                .items()
                .iter()
                .position(|item| item.path() == new_selection)
            {
                context.collection.select(index);
                context.revision = context.revision.wrapping_add(1);
            }
        }
        self.reconcile(&context, select_restored);
        watch.schedule();
    }

    fn reconcile(&self, context: &FileContext, load: bool) {
        if let (Some(window), Some(title)) = (self.window.upgrade(), self.title.upgrade()) {
            update_title(&window, &title, context);
        }
        if load {
            display_selected(&self.view, context);
        } else {
            update_thumbnail_catalog(&self.view, context);
            self.view
                .set_neighbors(context.collection.photo_neighbors());
        }
        if let Some(banner) = self.banner.upgrade() {
            banner.set_reveal_child(false);
        }
    }
}

pub(super) struct Permit(Rc<Undo>);

impl Drop for Permit {
    fn drop(&mut self) {
        self.0.busy.set(false);
        self.0.update_action();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pivra::rename::Ticket;

    #[test]
    fn grouped_history_evicts_whole_older_operations_and_preserves_newest_id() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("original.png");
        std::fs::write(&source, b"media").unwrap();
        let ticket = Arc::new(Ticket::capture(&source).unwrap());
        let item = RenameStep::new(ticket, "renamed.png").unwrap();
        let mut history = History::default();
        let record = |history: &mut History, count| {
            history.record(Operation::RenameGroup(BatchReversal::new(vec![
                item.clone();
                count
            ])))
        };
        let oldest = record(&mut history, 96);
        let middle = record(&mut history, 96);
        let newest = record(&mut history, 96);
        assert_eq!(history.entries.len(), 2);
        assert_eq!(history.entries.front().unwrap().0, middle);
        assert!(history.latest(Some(oldest)).is_none() && history.latest(Some(middle)).is_none());
        assert_eq!(history.latest(None).unwrap().0, newest);
        assert_eq!(history.latest(Some(newest)).unwrap().1.file_count(), 96);
        history.entries.pop_back();
        assert_eq!(history.latest(None).unwrap().0, middle);
    }
}
