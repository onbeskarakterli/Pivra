//! TH-04/05 stable cells and bounded, viewport-prioritized thumbnail work.
use super::{CELL_HEIGHT, CELL_WIDTH};
use gtk::{gdk, gio, glib, prelude::*};
use pivra::{
    image_cache::{FileKey, texture_cost},
    thumbnail_manager::Manager,
    thumbnail_provider::Error,
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    path::PathBuf,
    rc::Rc,
};
const BUDGET: usize = 16 * 1024 * 1024;
const LIMIT: usize = 64;
#[cfg(test)]
use pivra::thumbnail_manager::Permit;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Pending,
    Loading,
    Ready,
    Evicted,
    Failed,
}
struct CellWidgets {
    button: gtk::Button,
    selection: gtk::Frame,
    picture: gtk::Picture,
    spinner: gtk::Spinner,
    icon: gtk::Image,
}
struct Entry {
    index: Rc<Cell<usize>>,
    path: PathBuf,
    widgets: Option<CellWidgets>,
    phase: Phase,
    texture: Option<gdk::Texture>,
    keys: Option<(FileKey, Option<FileKey>)>,
    cost: usize,
    used: u64,
}
type Selection = Box<dyn Fn(usize)>;
struct Active {
    path: PathBuf,
    token: gio::Cancellable,
    epoch: u64,
}
struct Inner {
    items: gtk::Box,
    scroller: gtk::ScrolledWindow,
    entries: RefCell<Vec<Entry>>,
    select: RefCell<Option<Selection>>,
    selected: Cell<Option<usize>>,
    initial: Cell<Option<usize>>,
    initialized: Cell<bool>,
    restore: Cell<Option<f64>>,
    epoch: Cell<u64>,
    clock: Cell<u64>,
    started: Cell<u64>,
    decoded_max: Cell<usize>,
    last_visible: Cell<Option<(usize, usize)>>,
    active: RefCell<Option<Active>>,
    building: RefCell<Option<glib::SourceId>>,
    retry: RefCell<Option<glib::SourceId>>,
    closed: Cell<bool>,
    frames: RefCell<Option<(gdk::FrameClock, glib::SignalHandlerId)>>,
    validation: RefCell<Option<gio::Cancellable>>,
    warming: RefCell<Option<gio::Cancellable>>,
}
#[derive(Debug, Default)]
pub(crate) struct ThumbnailStats {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub textures: usize,
    pub bytes: usize,
    pub running: bool,
    pub started: u64,
    pub decoded_max: usize,
    pub largest_texture: usize,
}
#[derive(Clone)]
pub(crate) struct ThumbnailStrip(Rc<Inner>);
impl ThumbnailStrip {
    pub(super) fn new(items: gtk::Box, scroller: gtk::ScrolledWindow) -> Self {
        let inner = Rc::new(Inner {
            items,
            scroller,
            entries: RefCell::new(Vec::new()),
            select: RefCell::new(None),
            selected: Cell::new(None),
            initial: Cell::new(None),
            initialized: Cell::new(false),
            restore: Cell::new(None),
            epoch: Cell::new(0),
            clock: Cell::new(0),
            started: Cell::new(0),
            decoded_max: Cell::new(0),
            last_visible: Cell::new(None),
            active: RefCell::new(None),
            building: RefCell::new(None),
            retry: RefCell::new(None),
            closed: Cell::new(false),
            frames: RefCell::new(None),
            validation: RefCell::new(None),
            warming: RefCell::new(None),
        });
        let weak = Rc::downgrade(&inner);
        inner.scroller.connect_map(move |strip| {
            if let Some(inner) = weak.upgrade() {
                // Let an already-running hidden decode publish its results.
                // Cancelling it on reveal dropped every in-flight thumbnail.
                if let Some(clock) = strip.frame_clock() {
                    let weak = Rc::downgrade(&inner);
                    let handler = clock.connect_after_paint(move |_| {
                        if let Some(i) = weak.upgrade() {
                            i.align();
                            i.drive();
                        }
                    });
                    inner.frames.replace(Some((clock, handler)));
                }
                inner.build();
                inner.revalidate(false);
                inner.scroller.queue_draw();
            }
        });
        let weak = Rc::downgrade(&inner);
        inner.scroller.connect_unmap(move |_| {
            if let Some(i) = weak.upgrade() {
                i.pause();
                i.disconnect();
            }
        });
        let weak = Rc::downgrade(&inner);
        inner
            .scroller
            .hadjustment()
            .connect_value_changed(move |_| {
                if let Some(i) = weak.upgrade() {
                    i.drive();
                }
            });
        let wheel = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::BOTH_AXES);
        let weak = Rc::downgrade(&inner);
        wheel.connect_scroll(move |_, dx, dy| {
            if let Some(i) = weak.upgrade() {
                i.initial.set(None);
                let adj = i.scroller.hadjustment();
                let delta = if dx.abs() > f64::EPSILON { dx } else { dy };
                adj.set_value((adj.value() + delta * 48.).clamp(
                    adj.lower(),
                    (adj.upper() - adj.page_size()).max(adj.lower()),
                ));
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        inner.scroller.add_controller(wheel);
        // Left-button horizontal drag works over thumbnail buttons as well.
        // GTK's drag threshold preserves a short click for photo selection.
        let drag = gtk::GestureDrag::new();
        drag.set_button(1);
        drag.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = Rc::downgrade(&inner);
        let origin = Rc::new(Cell::new(0.0f64));
        let start = origin.clone();
        drag.connect_drag_begin(move |_, _, _| {
            if let Some(i) = weak.upgrade() {
                start.set(i.scroller.hadjustment().value());
            }
        });
        let weak = Rc::downgrade(&inner);
        drag.connect_drag_update(move |gesture, dx, _| {
            if let Some(i) = weak.upgrade() {
                if !dx.is_finite() { return; }
                i.initial.set(None);
                let adj = i.scroller.hadjustment();
                let limit = (adj.upper() - adj.page_size()).max(adj.lower());
                adj.set_value((origin.get() - dx).clamp(adj.lower(), limit));
                gesture.set_state(gtk::EventSequenceState::Claimed);
            }
        });
        inner.scroller.add_controller(drag);
        Self(inner)
    }
    pub(crate) fn stats(&self) -> ThumbnailStats {
        let e = self.0.entries.borrow();
        ThumbnailStats {
            total: e.len(),
            completed: e
                .iter()
                .filter(|e| !matches!(e.phase, Phase::Pending | Phase::Loading))
                .count(),
            failed: e.iter().filter(|e| e.phase == Phase::Failed).count(),
            textures: e.iter().filter(|e| e.texture.is_some()).count(),
            bytes: e.iter().map(|e| e.cost).sum(),
            // An idle gap before the retry/after-paint admission is not a drain.
            // Hidden/closed strips intentionally leave deferred entries pending.
            running: self.0.active.borrow().is_some()
                || self.0.validation.borrow().is_some()
                || (!self.0.closed.get()
                    && self.0.scroller.is_mapped()
                    && (self.0.retry.borrow().is_some()
                        || self.0.building.borrow().is_some()
                        || e.iter()
                            .any(|e| matches!(e.phase, Phase::Pending | Phase::Loading)))),
            started: self.0.started.get(),
            decoded_max: self.0.decoded_max.get(),
            largest_texture: e.iter().map(|e| e.cost).max().unwrap_or(0),
        }
    }
    pub(crate) fn finish_initial_catalog(&self, selected: Option<usize>) {
        // The production window first supplies a singleton, then its asynchronous
        // folder scan. Only the first completed scan gets this initial anchor.
        if !self.0.initialized.replace(true) {
            self.0.initial.set(selected);
            self.0.restore.set(None);
            self.0.scroller.queue_draw();
            if !self.0.scroller.is_mapped() { self.0.warm_hidden(); }
        }
    }
    pub(crate) fn on_select(&self, callback: impl Fn(usize) + 'static) {
        self.0.select.replace(Some(Box::new(callback)));
    }
    pub(crate) fn clear_on_select(&self) {
        self.0.select.borrow_mut().take();
        self.0.closed.set(true);
        self.0.pause();
        if let Some(token) = self.0.warming.borrow_mut().take() { token.cancel(); }
        self.0.disconnect();
        if let Some(id) = self.0.building.borrow_mut().take() {
            id.remove();
        }
    }
    /// Keep keyboard navigation's selection inside the strip viewport without
    /// revealing a hidden strip or disturbing already-visible thumbnails.
    pub(crate) fn set_catalog(&self, photos: &[(usize, PathBuf)], selected: Option<usize>) {
        let i = &self.0;
        let changed = i.selected.replace(selected) != selected;
        if changed {
            i.last_visible.set(None);
        }
        // The same measured, after-paint alignment handles both initial reveal
        // and subsequent Left/Right navigation. While unmapped, retain the
        // target until the strip is shown again; never force it open.
        if i.initial.get().is_some() || (changed && i.initialized.get()) {
            i.initial.set(selected);
        }
        let same = {
            let e = i.entries.borrow();
            e.len() == photos.len() && e.iter().zip(photos).all(|(e, (_, p))| &e.path == p)
        };
        if !same {
            i.last_visible.set(None);
            i.epoch.set(i.epoch.get().wrapping_add(1));
            i.pause();
            if let Some(token) = i.warming.borrow_mut().take() { token.cancel(); }
            let first = i.entries.borrow().is_empty();
            let offset = i.scroller.hadjustment().value();
            let mut old: HashMap<_, _> = i
                .entries
                .borrow_mut()
                .drain(..)
                .map(|e| (e.path.clone(), e))
                .collect();
            let mut next = Vec::with_capacity(photos.len());
            for (index, path) in photos {
                let mut entry = old.remove(path).unwrap_or_else(|| Entry {
                    index: Rc::new(Cell::new(*index)),
                    path: path.clone(),
                    widgets: None,
                    phase: Phase::Pending,
                    texture: None,
                    keys: None,
                    cost: 0,
                    used: 0,
                });
                entry.index.set(*index);
                if entry.phase == Phase::Loading {
                    entry.phase = Phase::Pending;
                }
                next.push(entry);
            }
            while let Some(child) = i.items.first_child() {
                i.items.remove(&child);
            }
            for entry in &next {
                if let Some(w) = &entry.widgets {
                    i.items.append(&w.button);
                }
            }
            i.entries.replace(next);
            if first || i.initial.get().is_some() {
                i.initial.set(selected);
                i.restore.set(None);
            } else {
                i.restore.set(Some(offset));
            }
            if i.scroller.is_mapped() {
                i.build();
            }
        } else {
            for (entry, (index, _)) in i.entries.borrow().iter().zip(photos) {
                entry.index.set(*index);
            }
        }
        for entry in i.entries.borrow().iter() {
            if let Some(w) = &entry.widgets {
                w.button
                    .set_widget_name(&format!("thumbnail-cell-{}", entry.index.get()));
                if Some(entry.index.get()) == selected {
                    w.selection.add_css_class("selected");
                } else {
                    w.selection.remove_css_class("selected");
                }
            }
        }
        i.scroller.queue_draw();
        if i.initialized.get() && !i.scroller.is_mapped() { i.warm_hidden(); }
    }
    /// Called on a real source-folder rescan, not on photo selection.
    pub(crate) fn invalidate(&self) {
        self.0.revalidate(true);
    }
}
impl Inner {
    // Prime GNOME/Freedesktop disk caches after folder discovery even when
    // the overlay is hidden. Bound to a small initial batch and one worker;
    // visible-cell decoding remains owned by the normal viewport scheduler.
    fn warm_hidden(self: &Rc<Self>) {
        if self.closed.get() || self.scroller.is_mapped() || self.warming.borrow().is_some() { return; }
        let entries = self.entries.borrow();
        let selected = entries.iter().position(|e| Some(e.index.get()) == self.selected.get());
        // Match the initial strip alignment: selected item stays visible,
        // scrolling just enough to reveal it. Hidden widgets have no measured
        // viewport, so estimate the cell pitch from the fixed width + spacing.
        // Hidden scrollers and their intermediate parents report width 0.
        // Use the allocated window viewport rather than treating zero as one
        // cell; otherwise only the selected image gets warmed on startup.
        let viewport_width = self.scroller.root()
            .and_then(|root| root.downcast::<gtk::Window>().ok())
            .map(|window| window.width())
            .filter(|&width| width > 0)
            .unwrap_or(960);
        let slots = (((viewport_width - 16).max(CELL_WIDTH) / (CELL_WIDTH + 8)).max(1) as usize)
            .min(24)
            .min(entries.len());
        let start = selected
            .map(|n| n.saturating_add(1).saturating_sub(slots))
            .unwrap_or(0);
        let end = (start + slots).min(entries.len());
        let mut paths = Vec::with_capacity(slots);
        // Prioritize the selected image, then the neighbors actually expected
        // in the initial viewport, not unrelated images at the folder start.
        if let Some(n) = selected { paths.push(entries[n].path.clone()); }
        for e in &entries[start..end] {
            if !paths.contains(&e.path) { paths.push(e.path.clone()); }
        }
        drop(entries);
        if paths.is_empty() { return; }
        let token = gio::Cancellable::new();
        self.warming.replace(Some(token.clone()));
        let worker_token = token.clone();
        let scale = self.scroller.scale_factor().clamp(1, 2);
        let epoch = self.epoch.get();
        let job = gio::spawn_blocking(move || {
            let mut results = Vec::new();
            for path in paths {
                if worker_token.is_cancelled() { break; }
                // The hidden pass uses the SAME verified decode/resize path as
                // the visible scheduler. Merely resolving disk cache entries
                // leaves the GTK cells pending and forces a second decode.
                let work = match Manager::session().resolve(&path, &worker_token) {
                    Ok(response) => {
                        if let Some(candidate) = response.candidate() {
                            let cached = cache_work(candidate, &worker_token, response.deadline(), scale);
                            if matches!(cached, Work::Hit(..)) { cached }
                            else if let Some(generated) = response.memory() {
                                memory_work(generated, &worker_token, response.deadline(), scale)
                            } else { cached }
                        } else if let Some(generated) = response.memory() {
                            memory_work(generated, &worker_token, response.deadline(), scale)
                        } else { Work::Failed("Thumbnail unavailable".into()) }
                    }
                    Err(Error::Busy) => Work::Busy,
                    Err(Error::Cancelled) => Work::Cancelled,
                    Err(_) => Work::Failed("Thumbnail unavailable".into()),
                };
                results.push((path, work));
            }
            results
        });
        let weak = Rc::downgrade(self);
        glib::MainContext::default().spawn_local(async move {
            let results = job.await.unwrap_or_default();
            if let Some(i) = weak.upgrade() {
                let current = i.warming.borrow().as_ref().is_some_and(|t| t == &token);
                if !current || token.is_cancelled() || i.closed.get() || i.epoch.get() != epoch {
                    return;
                }
                i.warming.borrow_mut().take();
                let mut entries = i.entries.borrow_mut();
                for (path, work) in results {
                    if let Work::Hit(texture, source, cache, decoded_cost) = work {
                        let cost = display_cost(texture.width(), texture.height()).unwrap_or(usize::MAX);
                        if cost > BUDGET { continue; }
                        let Some(entry) = entries.iter_mut().find(|e| e.path == path) else { continue; };
                        if entry.phase != Phase::Pending { continue; }
                        i.decoded_max.set(i.decoded_max.get().max(decoded_cost));
                        entry.cost = cost;
                        entry.texture = Some(texture.clone());
                        entry.keys = Some((source, cache));
                        entry.phase = Phase::Ready;
                        if let Some(w) = &entry.widgets {
                            let ratio = (CELL_WIDTH as f64 / texture.width() as f64)
                                .min(CELL_HEIGHT as f64 / texture.height() as f64);
                            w.selection.set_size_request(
                                ((texture.width() as f64 * ratio).round() as i32).max(1),
                                ((texture.height() as f64 * ratio).round() as i32).max(1),
                            );
                            w.picture.set_paintable(Some(&texture));
                            w.icon.set_visible(false);
                        }
                    }
                }
                // Enforce the same display-cache limits after hidden warming.
                while entries.iter().map(|e| e.cost).sum::<usize>() > BUDGET
                    || entries.iter().filter(|e| e.texture.is_some()).count() > LIMIT
                {
                    let victim = entries.iter().enumerate()
                        .filter(|(_, e)| e.texture.is_some())
                        .filter(|(_, e)| Some(e.index.get()) != i.selected.get())
                        .min_by_key(|(_, e)| e.used)
                        .map(|(n, _)| n);
                    let Some(n) = victim else { break; };
                    let entry = &mut entries[n];
                    entry.texture = None;
                    entry.cost = 0;
                    entry.phase = Phase::Evicted;
                    if let Some(w) = &entry.widgets {
                        w.picture.set_paintable(None::<&gdk::Texture>);
                        w.icon.set_visible(true);
                    }
                }
                drop(entries);
                i.scroller.queue_draw();
            }
        });
    }
    fn revalidate(self: &Rc<Self>, retry_failures: bool) {
        self.epoch.set(self.epoch.get().wrapping_add(1));
        let epoch = self.epoch.get();
        self.pause();
        let samples: Vec<_> = self
            .entries
            .borrow()
            .iter()
            .filter(|e| e.keys.is_some() || (retry_failures && e.phase == Phase::Failed))
            .map(|e| (e.path.clone(), e.keys.clone()))
            .collect();
        if samples.is_empty() {
            self.scroller.queue_draw();
            return;
        }
        let token = gio::Cancellable::new();
        self.validation.replace(Some(token.clone()));
        let work = gio::spawn_blocking(move || {
            let mut dirty = Vec::new();
            for (path, keys) in samples {
                if token.is_cancelled() {
                    return None;
                }
                if keys.as_ref().is_none_or(|(source, cache)| {
                    FileKey::read(&path).ok().as_ref() != Some(source)
                        || cache.as_ref().is_some_and(|cache| {
                            FileKey::read(&cache.path).ok().as_ref() != Some(cache)
                        })
                }) {
                    dirty.push(path);
                }
            }
            Some(dirty)
        });
        let weak = Rc::downgrade(self);
        glib::MainContext::default().spawn_local(async move {
            let Ok(Some(dirty)) = work.await else {
                return;
            };
            let Some(i) = weak.upgrade() else {
                return;
            };
            if i.epoch.get() != epoch || i.closed.get() {
                return;
            }
            i.validation.borrow_mut().take();
            for e in i
                .entries
                .borrow_mut()
                .iter_mut()
                .filter(|e| dirty.contains(&e.path))
            {
                e.phase = Phase::Pending;
                e.texture = None;
                e.keys = None;
                e.cost = 0;
                if let Some(w) = &e.widgets {
                    w.picture.set_paintable(None::<&gdk::Texture>);
                    w.icon.set_visible(true);
                }
            }
            i.scroller.queue_draw();
            i.drive();
        });
    }
    fn disconnect(&self) {
        if let Some((clock, id)) = self.frames.borrow_mut().take() {
            clock.disconnect(id);
        }
    }
    fn pause(&self) {
        if !self.scroller.is_mapped() {
            if let Some(id) = self.building.borrow_mut().take() {
                id.remove();
            }
        }
        if let Some(token) = self.validation.borrow_mut().take() {
            token.cancel();
        }
        if let Some(active) = self.active.borrow().as_ref() {
            trace("cancel-pause-or-epoch", &active.path);
            active.token.cancel();
        }
        if let Some(id) = self.retry.borrow_mut().take() {
            id.remove();
        }
        for e in self.entries.borrow().iter() {
            if let Some(w) = &e.widgets {
                w.spinner.stop();
            }
        }
    }
    fn build(self: &Rc<Self>) {
        if self.building.borrow().is_some()
            || !self.entries.borrow().iter().any(|e| e.widgets.is_none())
        {
            return;
        }
        let weak = Rc::downgrade(self);
        let id = glib::idle_add_local(move || {
            let Some(i) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            if i.closed.get() || !i.scroller.is_mapped() {
                i.building.borrow_mut().take();
                return glib::ControlFlow::Break;
            }
            let mut entries = i.entries.borrow_mut();
            for entry in entries.iter_mut().filter(|e| e.widgets.is_none()).take(32) {
                let button = gtk::Button::new();
                button.add_css_class("pivra-thumbnail-cell");
                button.set_size_request(CELL_WIDTH, CELL_HEIGHT);
                button.set_valign(gtk::Align::Center);
                button.set_focus_on_click(false);
                button.set_widget_name(&format!("thumbnail-cell-{}", entry.index.get()));


                let content = gtk::Overlay::new();
                let selection = gtk::Frame::new(None);
                selection.add_css_class("pivra-thumbnail-selection");
                selection.set_halign(gtk::Align::Center);
                selection.set_valign(gtk::Align::Center);
                selection.set_can_target(false);
                selection.set_size_request(CELL_WIDTH, CELL_HEIGHT);
                if Some(entry.index.get()) == i.selected.get() {
                    selection.add_css_class("selected");
                }
                let picture = gtk::Picture::new();
                picture.set_content_fit(gtk::ContentFit::Contain);
                picture.set_can_shrink(true);
                // Hidden predecode can complete before a GTK cell exists.
                // Hydrate new cells from their already-ready textures.
                if let Some(texture) = entry.texture.as_ref() {
                    let ratio = (CELL_WIDTH as f64 / texture.width() as f64)
                        .min(CELL_HEIGHT as f64 / texture.height() as f64);
                    selection.set_size_request(
                        ((texture.width() as f64 * ratio).round() as i32).max(1),
                        ((texture.height() as f64 * ratio).round() as i32).max(1),
                    );
                    picture.set_paintable(Some(texture));
                }
                let spinner = gtk::Spinner::new();
                spinner.set_halign(gtk::Align::Center);
                spinner.set_valign(gtk::Align::Center);
                let icon = gtk::Image::from_icon_name("image-x-generic-symbolic");
                icon.set_halign(gtk::Align::Center);
                icon.set_valign(gtk::Align::Center);
                icon.set_visible(entry.phase != Phase::Ready);
                for child in [
                    picture.upcast_ref::<gtk::Widget>(),
                    icon.upcast_ref(),
                    spinner.upcast_ref(),
                ] {
                    content.add_overlay(child);
                    content.set_measure_overlay(child, false);
                    content.set_clip_overlay(child, true);
                }
                content.add_overlay(&selection);
                content.set_measure_overlay(&selection, false);
                content.set_clip_overlay(&selection, true);
                button.set_child(Some(&content));
                let weak = Rc::downgrade(&i);
                let index = entry.index.clone();
                button.connect_clicked(move |_| {
                    if let Some(i) = weak.upgrade() {
                        if let Some(callback) = i.select.borrow().as_ref() {
                            callback(index.get());
                        }
                    }
                });
                i.items.append(&button);
                entry.widgets = Some(CellWidgets {
                    button,
                    selection,
                    picture,
                    spinner,
                    icon,
                });
            }
            if entries.iter().any(|e| e.widgets.is_none()) {
                glib::ControlFlow::Continue
            } else {
                i.building.borrow_mut().take();
                let mut previous: Option<gtk::Button> = None;
                for entry in entries.iter() {
                    if let Some(w) = &entry.widgets {
                        i.items.reorder_child_after(&w.button, previous.as_ref());
                        previous = Some(w.button.clone());
                    }
                }
                i.scroller.queue_draw();
                glib::ControlFlow::Break
            }
        });
        self.building.replace(Some(id));
    }
    fn align(&self) {
        if !self.scroller.is_mapped() || self.building.borrow().is_some() {
            return;
        }
        let adj = self.scroller.hadjustment();
        let page = adj.page_size();
        if page <= 0. || page > f64::from(self.scroller.width()) + 1. {
            return;
        }
        let entries = self.entries.borrow();
        // The measured content must include the final cell, not a provisional
        // hidden-strip adjustment range. This is a layout certificate, not a sleep.
        let Some(last) = entries
            .last()
            .and_then(|e| e.widgets.as_ref())
            .and_then(|w| w.button.compute_bounds(&self.items))
        else {
            self.initial.set(None);
            return;
        };
        if last.width() <= 0. || adj.upper() + 1. < f64::from(last.x() + last.width()) {
            return;
        }
        if let Some(index) = self.initial.get() {
            if let Some(bounds) = entries
                .iter()
                .find(|e| e.index.get() == index)
                .and_then(|e| e.widgets.as_ref())
                .and_then(|w| w.button.compute_bounds(&self.items))
            {
                let x = f64::from(bounds.x());
                let right = x + f64::from(bounds.width());
                let target = if x < adj.value() {
                    x
                } else if right > adj.value() + page {
                    right - page
                } else {
                    adj.value()
                };
                self.initial.set(None);
                drop(entries);
                adj.set_value(target.clamp(adj.lower(), (adj.upper() - page).max(adj.lower())));
            } else {
                self.initial.set(None);
            }
        } else if let Some(offset) = self.restore.take() {
            drop(entries);
            adj.set_value(offset.clamp(adj.lower(), (adj.upper() - page).max(adj.lower())));
        }
    }
    fn visible(&self, entries: &[Entry]) -> std::ops::Range<usize> {
        let adj = self.scroller.hadjustment();
        // Actual allocations include theme padding and fractional HiDPI layout;
        // a guessed fixed pitch can omit a still-visible edge cell.
        let mut shown = entries.iter().enumerate().filter_map(|(n, e)| {
            let b = e.widgets.as_ref()?.button.compute_bounds(&self.items)?;
            (f64::from(b.x()) < adj.value() + adj.page_size()
                && f64::from(b.x() + b.width()) > adj.value())
            .then_some(n)
        });
        let Some(first) = shown.next() else {
            return 0..0;
        };
        let last = shown.last().unwrap_or(first);
        first..last + 1
    }

    fn drive(self: &Rc<Self>) {
        if self.closed.get()
            || !self.scroller.is_mapped()
            || self.building.borrow().is_some()
            || self.initial.get().is_some()
            || self.retry.borrow().is_some()
            || self.validation.borrow().is_some()
        {
            return;
        }
        let mut entries = self.entries.borrow_mut();
        let visible = self.visible(&entries);
        let selected = entries
            .iter()
            .position(|e| Some(e.index.get()) == self.selected.get());
        reconsider_evicted(&mut entries, &visible, &self.last_visible, selected);
        let near = visible.start.saturating_sub(2)..(visible.end + 2).min(entries.len());
        self.clock.set(self.clock.get().wrapping_add(1));
        for (n, e) in entries.iter_mut().enumerate() {
            if visible.contains(&n) {
                e.used = self.clock.get();
            }
            if let Some(w) = &e.widgets {
                if visible.contains(&n) && matches!(e.phase, Phase::Pending | Phase::Loading) {
                    w.spinner.start();
                    w.icon.set_visible(false);
                } else {
                    w.spinner.stop();
                }
            }
        }
        let next = selected
            .filter(|&n| entries[n].phase == Phase::Pending)
            .or_else(|| {
                visible
                    .clone()
                    .find(|&n| entries[n].phase == Phase::Pending)
            })
            .or_else(|| near.clone().find(|&n| entries[n].phase == Phase::Pending))
            .or_else(|| entries.iter().position(|e| e.phase == Phase::Pending));
        if let Some(active) = self.active.borrow().as_ref() {
            if let Some(next) = next {
                if !active.token.is_cancelled()
                    && visible.contains(&next)
                    && !entries
                        .iter()
                        .position(|e| e.path == active.path)
                        .is_some_and(|n| near.contains(&n))
                {
                    trace("cancel-visible-priority", &active.path);
                    active.token.cancel();
                }
            }
            return;
        }
        let Some(n) = next else {
            return;
        };
        let path = entries[n].path.clone();
        trace(
            if Some(n) == selected {
                "admit-selected"
            } else if visible.contains(&n) {
                "admit-visible"
            } else {
                "admit-background"
            },
            &path,
        );
        entries[n].phase = Phase::Loading;
        drop(entries);
        let token = gio::Cancellable::new();
        let epoch = self.epoch.get();
        self.started.set(self.started.get().saturating_add(1));
        self.active.replace(Some(Active {
            path: path.clone(),
            token: token.clone(),
            epoch,
        }));
        let scale = self.scroller.scale_factor().clamp(1, 2);
        let worker_path = path.clone();
        let worker_token = token.clone();
        let job = gio::spawn_blocking(move || {
            if worker_token.is_cancelled() {
                return Work::Cancelled;
            }
            let response = match Manager::session().resolve(&worker_path, &worker_token) {
                Ok(response) => response,
                Err(Error::Busy) => return Work::Busy,
                Err(Error::Cancelled) => return Work::Cancelled,
                Err(_) => return Work::Failed("Thumbnail unavailable".into()),
            };
            if let Some(candidate) = response.candidate() {
                let cached = cache_work(candidate, &worker_token, response.deadline(), scale);
                if matches!(&cached, Work::Hit(..)) {
                    return cached;
                }
                if let Some(generated) = response.memory() {
                    return memory_work(generated, &worker_token, response.deadline(), scale);
                }
                if Manager::session().id() == pivra::thumbnail_provider::ProviderId::Freedesktop {
                    return match Manager::session().recover_cache(
                        &response,
                        &worker_path,
                        &worker_token,
                    ) {
                        Ok(generated) => {
                            memory_work(&generated, &worker_token, response.deadline(), scale)
                        }
                        Err(Error::Cancelled) => Work::Cancelled,
                        Err(Error::Busy) => Work::Busy,
                        Err(_) => Work::Failed("Thumbnail unavailable".into()),
                    };
                }
                return cached;
            }
            if let Some(generated) = response.memory() {
                return memory_work(generated, &worker_token, response.deadline(), scale);
            }
            Work::Failed("Thumbnail unavailable".into())
        });
        let weak = Rc::downgrade(self);
        glib::MainContext::default().spawn_local(async move {
            let result = job
                .await
                .unwrap_or_else(|_| Work::Failed("thumbnail worker failed".into()));
            let Some(i) = weak.upgrade() else {
                return;
            };
            let valid = i
                .active
                .borrow()
                .as_ref()
                .is_some_and(|a| a.token == token && a.epoch == epoch);
            if !valid {
                return;
            }
            i.active.borrow_mut().take();
            if i.closed.get() {
                return;
            }
            let mut entries = i.entries.borrow_mut();
            if let Some(entry) = entries.iter_mut().find(|e| e.path == path) {
                let stale = token.is_cancelled() || epoch != i.epoch.get();
                entry.phase = Phase::Failed;
                match result {
                    _ if stale => {
                        trace("discard-stale-or-cancelled", &path);
                        entry.phase = Phase::Pending;
                    }
                    Work::Hit(texture, source, cache, decoded_cost) => {
                        i.decoded_max.set(i.decoded_max.get().max(decoded_cost));
                        // Identity was verified on the worker before publication;
                        // source rescans invalidate the window epoch as well.
                        let cost =
                            display_cost(texture.width(), texture.height()).unwrap_or(usize::MAX);
                        if cost <= BUDGET {
                            entry.cost = cost;
                            entry.texture = Some(texture.clone());
                            entry.phase = Phase::Ready;
                            entry.used = i.clock.get();
                            if let Some(w) = &entry.widgets {
                                let ratio = (CELL_WIDTH as f64 / texture.width() as f64)
                                    .min(CELL_HEIGHT as f64 / texture.height() as f64);
                                let width = (texture.width() as f64 * ratio).round() as i32;
                                let height = (texture.height() as f64 * ratio).round() as i32;
                                w.selection.set_size_request(width.max(1), height.max(1));
                                w.picture.set_paintable(Some(&texture));
                                w.icon.set_visible(false);
                            }
                        }
                        entry.keys = Some((source, cache));
                    }
                    Work::Busy => entry.phase = Phase::Pending,
                    Work::Cancelled => entry.phase = Phase::Failed,
                    Work::Failed(_error) => {
                        trace("thumbnail-failed", &path);
                    }
                }
                if let Some(w) = &entry.widgets {
                    w.spinner.stop();
                    w.icon.set_visible(entry.phase == Phase::Failed);
                }
            }
            // LRU bounds apply to widget paintables as well, not just a side map.
            while entries.iter().map(|e| e.cost).sum::<usize>() > BUDGET
                || entries.iter().filter(|e| e.texture.is_some()).count() > LIMIT
            {
                let visible = i.visible(&entries);
                let victim = entries
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.texture.is_some())
                    .min_by_key(|(n, e)| {
                        (
                            Some(e.index.get()) == i.selected.get(),
                            visible.contains(n),
                            e.used,
                        )
                    })
                    .map(|(n, _)| n);
                let Some(n) = victim else {
                    break;
                };
                let e = &mut entries[n];
                trace("evict", &e.path);
                e.texture = None;
                e.cost = 0;
                e.phase = Phase::Evicted;
                if let Some(w) = &e.widgets {
                    w.picture.set_paintable(None::<&gdk::Texture>);
                    w.icon.set_visible(true);
                }
            }
            drop(entries);
            // Yield even on Busy; no unbounded idle retry loop or queued decoders.
            let weak = Rc::downgrade(&i);
            let id =
                glib::timeout_add_local_once(std::time::Duration::from_millis(30), move || {
                    if let Some(i) = weak.upgrade() {
                        i.retry.borrow_mut().take();
                        i.drive();
                    }
                });
            i.retry.replace(Some(id));
        });
    }
}
// A viewport wider than the RAM budget must not endlessly decode/evict the
// same visible images. Budget-evicted entries are reconsidered on viewport or
// catalog change, not on every paint/worker completion.
fn cache_work(
    candidate: &pivra::thumbnail_provider::Candidate,
    worker_token: &gio::Cancellable,
    deadline: std::time::Instant,
    scale: i32,
) -> Work {
    if !candidate.still_current(worker_token) {
        return if worker_token.is_cancelled() {
            Work::Cancelled
        } else {
            Work::Failed("thumbnail identity changed before decode".into())
        };
    }
    let context = glib::MainContext::new();
    let request = pivra::thumbnail_provider::Request {
        source: &candidate.source,
        cancel: worker_token,
        deadline,
    };
    let mut control = pivra::thumbnail_manager::CacheControl::new(&request, &context);
    let frame = context.block_on(crate::image_loader::preload(
        &candidate.thumbnail.path,
        control.cancellable(),
        BUDGET,
    ));
    if frame.is_ok() {
        control.complete();
    }
    if let Err(error) = request.check_control(std::time::Instant::now()) {
        return match error {
            Error::Cancelled => Work::Cancelled,
            _ => Work::Failed("Thumbnail timed out".into()),
        };
    }
    match frame {
        Ok(frame) if candidate.still_current(worker_token) => {
            if worker_token.is_cancelled() {
                return Work::Cancelled;
            }
            let texture = frame.texture();
            let decoded_cost =
                texture_cost(texture.width() as u32, texture.height() as u32).unwrap_or(usize::MAX);
            match display_texture(&texture, scale) {
                Ok(texture)
                    if !worker_token.is_cancelled() && std::time::Instant::now() < deadline =>
                {
                    Work::Hit(
                        texture,
                        candidate.source.clone(),
                        Some(candidate.thumbnail.clone()),
                        decoded_cost,
                    )
                }
                Ok(_) => Work::Cancelled,
                Err(error) => Work::Failed(error),
            }
        }
        Ok(_) if worker_token.is_cancelled() => Work::Cancelled,
        Ok(_) => Work::Failed("thumbnail identity changed during decode".into()),
        Err(_) => Work::Failed("Thumbnail decode failed".into()),
    }
}
fn memory_work(
    generated: &pivra::freedesktop_generator::GeneratedThumbnail,
    worker_token: &gio::Cancellable,
    deadline: std::time::Instant,
    scale: i32,
) -> Work {
    if worker_token.is_cancelled() || std::time::Instant::now() >= deadline {
        return Work::Cancelled;
    }
    if FileKey::read(&generated.source.path).ok().as_ref() != Some(&generated.source) {
        return Work::Failed("Source changed".into());
    }
    let texture = gdk::MemoryTexture::new(
        generated.width as i32,
        generated.height as i32,
        gdk::MemoryFormat::R8g8b8a8,
        &generated.pixels,
        generated.stride,
    )
    .upcast();
    match display_texture(&texture, scale) {
        Ok(texture)
            if !worker_token.is_cancelled()
                && std::time::Instant::now() < deadline
                && FileKey::read(&generated.source.path).ok().as_ref()
                    == Some(&generated.source) =>
        {
            Work::Hit(
                texture,
                generated.source.clone(),
                None,
                generated.pixels.len(),
            )
        }
        Ok(_) => Work::Cancelled,
        Err(e) => Work::Failed(e),
    }
}

fn reconsider_evicted(
    entries: &mut [Entry],
    visible: &std::ops::Range<usize>,
    last: &Cell<Option<(usize, usize)>>,
    selected: Option<usize>,
) {
    let range = Some((visible.start, visible.end));
    if last.replace(range) != range {
        for (n, e) in entries.iter_mut().enumerate() {
            if (visible.contains(&n) || Some(n) == selected) && e.phase == Phase::Evicted {
                trace("rehydrate-after-viewport-or-selection", &e.path);
                e.phase = Phase::Pending;
            }
        }
    }
}
// Only resample already sandbox-decoded pixels. No file parsing or new decoder.
// Normalize to RGBA8 and at most 2x cell density. Charge 8 bytes/pixel:
// owned RGBA8 pixels + a renderer upload. 64 * 192*160*8 < 16 MiB.
// The main photo's conservative 32-byte estimate remains unchanged.
fn display_texture(texture: &gdk::Texture, scale: i32) -> Result<gdk::Texture, String> {
    let (width, height) = display_size(texture.width(), texture.height(), scale);
    let mut download = gdk::TextureDownloader::new(texture);
    download.set_format(gdk::MemoryFormat::R8g8b8a8);
    let (bytes, stride) = download.download_bytes();
    if (width, height) == (texture.width(), texture.height()) {
        return Ok(gdk::MemoryTexture::new(
            width,
            height,
            gdk::MemoryFormat::R8g8b8a8,
            &bytes,
            stride,
        )
        .upcast());
    }
    let stride = i32::try_from(stride).map_err(|_| "Thumbnail stride exceeds limit")?;
    let pixels = gdk::gdk_pixbuf::Pixbuf::from_bytes(
        &bytes,
        gdk::gdk_pixbuf::Colorspace::Rgb,
        true,
        8,
        texture.width(),
        texture.height(),
        stride,
    );
    let small = pixels
        .scale_simple(width, height, gdk::gdk_pixbuf::InterpType::Bilinear)
        .ok_or("Cannot scale decoded thumbnail pixels")?;
    Ok(gdk::Texture::for_pixbuf(&small))
}
fn display_cost(width: i32, height: i32) -> Option<usize> {
    usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?
        .checked_mul(8)
}

fn display_size(width: i32, height: i32, scale: i32) -> (i32, i32) {
    let scale = scale.clamp(1, 2);
    let ratio = (f64::from(CELL_WIDTH * scale) / f64::from(width))
        .min(f64::from(CELL_HEIGHT * scale) / f64::from(height))
        .min(1.);
    (
        (f64::from(width) * ratio).floor().max(1.) as i32,
        (f64::from(height) * ratio).floor().max(1.) as i32,
    )
}

// Explicit opt-in diagnostics only; never an unbounded in-memory event history.
fn trace(event: &str, path: &std::path::Path) {
    if std::env::var_os("PIVRA_TRACE_THUMBNAILS").is_some_and(|v| v == "1") {
        eprintln!("Pivra thumbnail {event}: {}", path.display());
    }
}

enum Work {
    Hit(gdk::Texture, FileKey, Option<FileKey>, usize),
    Failed(String),
    Cancelled,
    Busy,
}
impl Drop for Inner {
    fn drop(&mut self) {
        self.pause();
        self.disconnect();
        if let Some(id) = self.building.borrow_mut().take() {
            id.remove();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn over_budget_visible_entries_do_not_spin_until_viewport_changes() {
        let mut entries: Vec<_> = (0..65)
            .map(|index| Entry {
                index: Rc::new(Cell::new(index)),
                path: PathBuf::from(index.to_string()),
                widgets: None,
                phase: Phase::Evicted,
                texture: None,
                keys: None,
                cost: 0,
                used: 0,
            })
            .collect();
        let last = Cell::new(None);
        reconsider_evicted(&mut entries, &(0..64), &last, None);
        assert!(entries[..64].iter().all(|e| e.phase == Phase::Pending));
        for e in &mut entries[..64] {
            e.phase = Phase::Evicted;
        } // RAM eviction after admission
        reconsider_evicted(&mut entries, &(0..64), &last, None);
        assert!(entries[..64].iter().all(|e| e.phase == Phase::Evicted));
        reconsider_evicted(&mut entries, &(1..65), &last, None);
        assert!(entries[1..].iter().all(|e| e.phase == Phase::Pending));
        assert!(entries[0].phase == Phase::Evicted);
    }
    #[test]
    fn rgba8_density_and_conservative_upload_charge_fit_both_caps() {
        for scale in 1..=4 {
            for (w, h) in [(512, 512), (4096, 3072), (64, 32), (1, 32768)] {
                let (width, height) = display_size(w, h, scale);
                assert!(width > 0 && height > 0 && width <= w && height <= h);
                assert!(width <= CELL_WIDTH * 2 && height <= CELL_HEIGHT * 2);
                assert!(display_cost(width, height).unwrap() * LIMIT <= BUDGET);
            }
        }
        assert_eq!(texture_cost(512, 512), Some(8 * 1024 * 1024));
        assert_eq!(display_cost(160, 160), Some(204800));
    }
    #[test]
    fn evicted_offscreen_selection_is_reconsidered_without_paint_retry_loop() {
        let mut entries: Vec<_> = (0..5)
            .map(|index| Entry {
                index: Rc::new(Cell::new(index)),
                path: PathBuf::new(),
                widgets: None,
                phase: Phase::Evicted,
                texture: None,
                keys: None,
                cost: 0,
                used: 0,
            })
            .collect();
        let last = Cell::new(None);
        reconsider_evicted(&mut entries, &(0..1), &last, Some(4));
        assert!(entries[0].phase == Phase::Pending && entries[4].phase == Phase::Pending);
        assert!(entries[1..4].iter().all(|e| e.phase == Phase::Evicted));
        entries[4].phase = Phase::Evicted;
        reconsider_evicted(&mut entries, &(0..1), &last, Some(4));
        assert!(entries[4].phase == Phase::Evicted);
    }
    #[test]
    fn generated_memory_texture_owns_pixels_and_never_invents_cache_identity() {
        use pivra::freedesktop_generator::{GeneratedThumbnail, Metrics};
        use pivra::freedesktop_thumbnail::SizeClass;
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("source.png");
        std::fs::write(&path, b"controlled source identity only").unwrap();
        let key = FileKey::read(&path).unwrap();
        let t = GeneratedThumbnail {
            source: key.clone(),
            class: SizeClass::Normal,
            width: 4,
            height: 2,
            stride: 16,
            pixels: glib::Bytes::from_owned([20u8, 40, 60, 128].repeat(8)),
            metrics: Metrics {
                decoded_width: 4,
                decoded_height: 2,
                frame_bytes: 32,
                download_bytes: 32,
                output_bytes: 32,
                decode_time: std::time::Duration::ZERO,
                resize_time: std::time::Duration::ZERO,
            },
        };
        let cancel = gio::Cancellable::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let Work::Hit(texture, source, cache, _) = memory_work(&t, &cancel, deadline, 1) else {
            panic!("memory pixels");
        };
        assert_eq!(source, key);
        assert!(cache.is_none());
        assert_eq!((texture.width(), texture.height()), (4, 2));
        assert!(!matches!(
            memory_work(&t, &cancel, std::time::Instant::now(), 1),
            Work::Hit(..)
        ));
        std::fs::write(&path, b"changed").unwrap();
        assert!(!matches!(
            memory_work(&t, &cancel, deadline, 1),
            Work::Hit(..)
        ));
        cancel.cancel();
        assert!(!matches!(
            memory_work(&t, &cancel, deadline, 1),
            Work::Hit(..)
        ));
        drop(t);
        let mut downloader = gdk::TextureDownloader::new(&texture);
        downloader.set_format(gdk::MemoryFormat::R8g8b8a8);
        let (bytes, _) = downloader.download_bytes();
        for pixel in bytes.chunks_exact(4) {
            for (actual, expected) in pixel.iter().zip([20u8, 40, 60, 128]) {
                assert!(actual.abs_diff(expected) <= 1);
            }
        }
    }
    #[test]
    fn global_worker_bound_deduplicates_and_releases_paths() {
        let root = tempfile::tempdir().unwrap();
        let a = root.path().join("a");
        let b = root.path().join("b");
        let c = root.path().join("c");
        let first = Permit::take(&a).unwrap();
        assert!(Permit::take(&a).is_none());
        let second = Permit::take(&b).unwrap();
        assert!(Permit::take(&c).is_none());
        drop(first);
        assert!(Permit::take(&c).is_some());
        drop(second);
        assert_eq!(Permit::active_count(), 0);
    }
}
