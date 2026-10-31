//! Per-window prefetch orchestration. Background completions never mutate GTK.
use super::image_requests::LoadSlot;
use gtk::{gdk, gio, glib, prelude::*};
use pivra::image_cache::{Cache, CacheStats, DEFAULT_ENTRY_LIMIT, FileKey, texture_cost};
use std::{
    cell::RefCell,
    collections::VecDeque,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
    time::Instant,
};
pub(super) enum Output {
    Decoded(crate::image_loader::LoadedFrame, Option<FileKey>),
    Cached(gdk::Texture, FileKey),
    Raw(gdk::Texture, Option<FileKey>),
}
pub(super) type Completion = Box<dyn FnOnce(Result<Output, String>, bool)>;
pub(super) struct Background {
    pub path: PathBuf,
    pub token: gio::Cancellable,
    pub consumer: Rc<RefCell<Option<Completion>>>,
}
#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub(crate) struct PreloadStats {
    pub cache: CacheStats,
    pub started: u64,
    pub completed: u64,
    pub cancelled: u64,
    pub decoded: u64,
    pub promoted: u64,
    pub foreground_decoded: u64,
    pub admission_max_us: u64,
    pub preload_active: bool,
    pub enabled: bool,
    pub memory_backend: String,
}
pub(super) struct State {
    pub cache: Arc<Mutex<Cache<gdk::Texture>>>,
    pub epoch: Rc<()>,
    pub background: Option<Background>,
    pub current_path: Option<PathBuf>,
    pub neighbors: Vec<PathBuf>,
    pub queue: VecDeque<PathBuf>,
    pub failed: VecDeque<PathBuf>,
    pub enabled: bool,
    pub closed: bool,
    pub active_large: bool,
    stats: PreloadStats,
    monitor: gio::MemoryMonitor,
    handler: Option<glib::SignalHandlerId>,
}
impl State {
    pub fn new() -> Rc<RefCell<Self>> {
        let monitor = gio::MemoryMonitor::dup_default();
        let memory_backend = monitor.type_().name().to_owned();
        let state = Rc::new(RefCell::new(Self {
            cache: Arc::new(Mutex::new(Cache::default())),
            epoch: Rc::new(()),
            background: None,
            current_path: None,
            neighbors: vec![],
            queue: VecDeque::new(),
            failed: VecDeque::new(),
            enabled: true,
            closed: false,
            active_large: false,
            stats: PreloadStats {
                memory_backend,
                ..Default::default()
            },
            monitor: monitor.clone(),
            handler: None,
        }));
        let weak = Rc::downgrade(&state);
        let id = monitor.connect_low_memory_warning(move |_, _| {
            if let Some(state) = weak.upgrade() {
                state.borrow_mut().pressure();
            }
        });
        state.borrow_mut().handler = Some(id);
        state
    }
    pub fn pressure(&mut self) {
        self.enabled = false;
        self.queue.clear();
        if let Some(bg) = self
            .background
            .as_ref()
            .filter(|b| b.consumer.borrow().is_none())
        {
            bg.token.cancel();
        }
        self.cache.lock().unwrap().pressure();
    }
    pub fn invalidate(&mut self) {
        self.invalidate_related(true);
    }
    pub fn invalidate_paths(&mut self, paths: &[PathBuf]) {
        let related = self
            .current_path
            .as_ref()
            .is_some_and(|p| paths.contains(p));
        self.invalidate_related(related);
    }
    fn invalidate_related(&mut self, related: bool) {
        if related {
            self.epoch = Rc::new(());
        }
        self.cache.lock().unwrap().clear();
        self.failed.clear();
        self.queue.clear();
        if let Some(bg) = &self.background {
            if related || bg.consumer.borrow().is_none() {
                bg.token.cancel();
            }
        }
    }
    pub fn close(&mut self) {
        self.closed = true;
        self.invalidate();
        if let Some(id) = self.handler.take() {
            self.monitor.disconnect(id);
        }
    }
    pub fn stats(&self) -> PreloadStats {
        PreloadStats {
            cache: self.cache.lock().unwrap().stats(),
            preload_active: self.background.is_some(),
            enabled: self.enabled,
            ..self.stats.clone()
        }
    }
    pub fn admission(&mut self, start: Instant) {
        self.stats.admission_max_us = self
            .stats
            .admission_max_us
            .max(start.elapsed().as_micros().try_into().unwrap_or(u64::MAX));
    }
    pub fn promoted(&mut self) {
        self.stats.promoted = self.stats.promoted.saturating_add(1);
    }
    pub fn foreground_decode(&mut self) {
        self.stats.foreground_decoded = self.stats.foreground_decoded.saturating_add(1);
    }
}
impl Drop for State {
    fn drop(&mut self) {
        if let Some(id) = self.handler.take() {
            self.monitor.disconnect(id);
        }
    }
}
/// File stat and decoder work run only inside a blocking worker; mutex holds no I/O.
pub(super) fn fetch(
    path: &Path,
    token: &gio::Cancellable,
    cache: &Arc<Mutex<Cache<gdk::Texture>>>,
    allow_cache: bool,
    budget: Option<usize>,
) -> (Result<Output, String>, bool) {
    let before = FileKey::read(path).ok();
    if before.is_none() {
        cache.lock().unwrap().invalidate_path(path);
    }
    if allow_cache {
        if let Some(key) = &before {
            let cached = { cache.lock().unwrap().lookup(key) };
            if let Some(texture) = cached {
                return (
                    if FileKey::read(path).ok().as_ref() == Some(key) {
                        Ok(Output::Cached(texture, key.clone()))
                    } else {
                        Err("Image changed during cache lookup; refresh to reload.".into())
                    },
                    false,
                );
            }
        }
    }
    let result = if crate::raw_loader::is_dng(path) {
        if budget.is_some() {
            return (Err("DNG preloading is not enabled".into()), true);
        }
        crate::raw_loader::load(path, token).map(|texture| Output::Raw(texture, before.clone()))
    } else {
        let decoded = glib::MainContext::new().block_on(async {
            if let Some(budget) = budget {
                crate::image_loader::preload(path, token, budget).await
            } else {
                crate::image_loader::load(path, token).await
            }
        });
        let after = FileKey::read(path).ok();
        let key = if before == after { before } else { None };
        decoded.map(|frame| Output::Decoded(frame, key))
    };
    (result, true)
}
pub(super) fn maybe_start(state: &Rc<RefCell<State>>, slot: &Rc<LoadSlot>, has_image: bool) {
    if !has_image || slot.stats().running != 0 || slot.stats().pending != 0 {
        return;
    }
    let choice = {
        let mut s = state.borrow_mut();
        if !s.enabled || s.closed || s.active_large || s.background.is_some() {
            return;
        }
        let mut found = None;
        while let Some(path) = s.queue.pop_front() {
            if s.current_path.as_ref() == Some(&path)
                || s.failed.contains(&path)
                || s.cache.lock().unwrap().contains_path(&path)
            {
                continue;
            }
            let index = s.neighbors.iter().position(|p| p == &path).unwrap_or(0);
            let budget = s.cache.lock().unwrap().available(&s.neighbors[..index]);
            if budget == 0 {
                continue;
            }
            found = Some((path, budget.min(DEFAULT_ENTRY_LIMIT)));
            break;
        }
        found
    };
    let Some((path, budget)) = choice else {
        return;
    };
    let token = gio::Cancellable::new();
    let consumer = Rc::new(RefCell::new(None::<Completion>));
    let cache = state.borrow().cache.clone();
    let epoch = state.borrow().epoch.clone();
    let weak = Rc::downgrade(state);
    let running = slot.clone();
    let target = token.clone();
    let name = path.clone();
    let background_path = path.clone();
    let deliver = consumer.clone();
    // Reserve before starting: one background job, never a foreground pending slot.
    let admitted = slot.try_background(&token, move || {
        let cancel = target.clone();
        let job = gio::spawn_blocking(move || fetch(&name, &cancel, &cache, true, Some(budget)));
        glib::MainContext::default().spawn_local(async move {
            let (result, decoded) = job
                .await
                .unwrap_or_else(|_| (Err("Preload worker failed".into()), true));
            running.finish(&target);
            let Some(state) = weak.upgrade() else {
                return;
            };
            {
                let mut s = state.borrow_mut();
                s.stats.completed = s.stats.completed.saturating_add(1);
                if decoded {
                    s.stats.decoded = s.stats.decoded.saturating_add(1);
                }
                if target.is_cancelled() {
                    s.stats.cancelled = s.stats.cancelled.saturating_add(1);
                }
                s.background.take();
            }
            let callback = deliver.borrow_mut().take();
            if let Some(callback) = callback {
                callback(
                    if !Rc::ptr_eq(&epoch, &state.borrow().epoch) {
                        Err("Image changed during preload; refresh to reload.".into())
                    } else {
                        result
                    },
                    true,
                );
            } else if !target.is_cancelled()
                && Rc::ptr_eq(&epoch, &state.borrow().epoch)
                && !state.borrow().closed
            {
                match result {
                    Ok(output) => {
                        let (texture, key) = match output {
                            Output::Decoded(frame, key) => (frame.texture(), key),
                            Output::Cached(texture, key) => (texture, Some(key)),
                            Output::Raw(texture, key) => (texture, key),
                        };
                        if let (Some(key), Some(cost)) = (
                            key,
                            texture_cost(texture.width() as u32, texture.height() as u32),
                        ) {
                            let s = state.borrow();
                            if let Some(index) = s.neighbors.iter().position(|p| p == &path) {
                                s.cache.lock().unwrap().insert(
                                    key,
                                    texture,
                                    cost,
                                    &s.neighbors[..index],
                                );
                            }
                        }
                    }
                    Err(_) => {
                        let mut s = state.borrow_mut();
                        if s.failed.len() == 4 {
                            s.failed.pop_front();
                        }
                        s.failed.push_back(path);
                    }
                }
            }
            // Foreground completion owns the next scheduling decision when promoted.
            let live = !state.borrow().closed;
            maybe_start(&state, &running, live);
        });
    });
    if admitted {
        let mut s = state.borrow_mut();
        s.stats.started = s.stats.started.saturating_add(1);
        s.background = Some(Background {
            path: background_path,
            token,
            consumer,
        });
    }
}
