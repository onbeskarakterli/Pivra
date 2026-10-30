//! Per-window presentation and cancellation. Glycin alone applies orientation.

#[path = "animation.rs"]
mod animation;
#[path = "video_controls.rs"]
mod video_controls;
#[path = "actual_paintable.rs"]
mod actual_paintable;
#[path = "thumbnail_overlay.rs"]
pub(crate) mod thumbnail_overlay;
use adw::prelude::*;
use gtk::{gio, glib};
use pivra::viewing::{Resampling, Transform, ViewMode};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
};

#[path = "image_requests.rs"]
mod image_requests;
#[path = "preload.rs"]
mod preload;
use image_requests::LoadSlot;
use pivra::image_cache::texture_cost;
use pivra::image_pipeline::Phase;
#[derive(Clone)]
pub(super) struct ImageView {
    overlay: gtk::Overlay,
    thumbnail_strip: thumbnail_overlay::ThumbnailStrip,
    picture: gtk::Picture,
    video: gtk::Video,
    video_controls: video_controls::Controls,
    status: gtk::Label,
    loading_spinner: gtk::Spinner,
    slot: Rc<LoadSlot>,
    window: glib::WeakRef<adw::ApplicationWindow>,
    mode_action: gio::SimpleAction,
    texture: Rc<RefCell<Option<gtk::gdk::Texture>>>,
    transform: Rc<RefCell<Transform>>,
    filter: Rc<Cell<Resampling>>,
    zoom_wheel: Rc<Cell<bool>>,
    requested: Rc<RefCell<Option<PathBuf>>>,
    prefetch: Rc<RefCell<preload::State>>,
    phase: Rc<Cell<Phase>>,
    animation_generation: Rc<Cell<u64>>,
    animation_controls: animation::Controls,
}

impl ImageView {
    pub(super) fn new(window: &adw::ApplicationWindow) -> Self {
        let picture = gtk::Picture::builder()
            .content_fit(gtk::ContentFit::ScaleDown)
            .can_shrink(true)
            .hexpand(true)
            .vexpand(true)
            .build();
        let status = gtk::Label::new(None);
        status.set_wrap(true);
        status.set_halign(gtk::Align::Center);
        status.set_valign(gtk::Align::Center);
        let overlay = gtk::Overlay::new();
        overlay.set_hexpand(true);
        overlay.set_vexpand(true);
        overlay.set_child(Some(&picture));
        // Native GTK media playback is an intentionally small first video
        // milestone. The backend depends on installed media codecs.
        let video = gtk::Video::new();
        video.set_hexpand(true);
        video.set_vexpand(true);
        video.set_autoplay(true);
        video.set_loop(false);
        video.set_visible(false);
        overlay.add_overlay(&video);
        overlay.set_measure_overlay(&video, false);
        overlay.add_overlay(&status);
        let loading_spinner = gtk::Spinner::new();
        loading_spinner.set_widget_name("photo-loading-spinner");
        loading_spinner.set_halign(gtk::Align::Center);
        loading_spinner.set_valign(gtk::Align::Center);
        loading_spinner.set_can_target(false);
        loading_spinner.set_spinning(true);
        loading_spinner.set_visible(false);
        overlay.add_overlay(&loading_spinner);
        overlay.set_measure_overlay(&loading_spinner, false);
        // UI admission follows the same session provider as the strip worker.
        // Preserve GNOME's capability gate; Freedesktop does not require GNOME.
        let thumbnails_available = match pivra::thumbnail_manager::Manager::session().id() {
            pivra::thumbnail_provider::ProviderId::Gnome => {
                pivra::thumbnail::Provider::discover(None).capability()
                    != pivra::thumbnail::Capability::Unavailable
            }
            pivra::thumbnail_provider::ProviderId::Freedesktop => true,
        };
        let thumbnail_strip = thumbnail_overlay::install(window, &overlay, thumbnails_available);
        let phase = Rc::new(Cell::new(Phase::Requested));
        let closing_phase = phase.clone();
        window.connect_close_request(move |_| {
            closing_phase.set(Phase::Cancelled);
            glib::Propagation::Proceed
        });
        let destroyed_phase = phase.clone();
        window.connect_destroy(move |_| destroyed_phase.set(Phase::Cancelled));
        let prefetch = preload::State::new();
        let closing_prefetch = prefetch.clone();
        window.connect_close_request(move |_| {
            closing_prefetch.borrow_mut().close();
            glib::Propagation::Proceed
        });
        let destroyed_prefetch = prefetch.clone();
        window.connect_destroy(move |_| destroyed_prefetch.borrow_mut().close());
        let animation_controls = animation::Controls::new();
        overlay.add_overlay(&animation_controls.bar);
        overlay.set_measure_overlay(&animation_controls.bar, false);
        let video_controls = video_controls::Controls::new();
        overlay.add_overlay(&video_controls.bar);
        overlay.set_measure_overlay(&video_controls.bar, false);
        // The thumbnail controls are bottom-aligned overlay siblings. Their
        // bottom margin already accounts for the strip's visible height.
        // Keep the timeline directly above their allocated button height.
        let mut child = overlay.first_child();
        while let Some(widget) = child {
            if widget.widget_name() == "thumbnail-controls" {
                if let Ok(thumbnail_controls) = widget.downcast::<gtk::Box>() {
                    let bar = animation_controls.bar.downgrade();
                    let video_bar = video_controls.bar.downgrade();
                    let align = move |controls: &gtk::Box| {
                        if let Some(bar) = bar.upgrade() {
                            let button_height = controls.height().max(30);
                            let offset = if controls.is_visible() {
                                controls.margin_bottom() + button_height + 2
                            } else {
                                2
                            };
                            if bar.margin_bottom() != offset {
                                bar.set_margin_bottom(offset);
                            }
                            if let Some(video_bar) = video_bar.upgrade() {
                                video_bar.set_margin_bottom(offset);
                            }
                        }
                    };
                    align(&thumbnail_controls);
                    let align = Rc::new(align);
                    let changed = align.clone();
                    thumbnail_controls.connect_notify_local(Some("visible"), move |controls, _| {
                        changed(controls);
                    });
                    let changed = align.clone();
                    thumbnail_controls.connect_notify_local(Some("margin-bottom"), move |controls, _| {
                        changed(controls);
                    });
                    thumbnail_controls.connect_notify_local(Some("height"), move |controls, _| {
                        align(controls);
                    });
                }
                break;
            }
            child = widget.next_sibling();
        }
        let animation_generation = Rc::new(Cell::new(0_u64));
        for signal in [false, true] {
            let generation = animation_generation.clone();
            if signal {
                window.connect_destroy(move |_| generation.set(generation.get().wrapping_add(1)));
            } else {
                window.connect_close_request(move |_| {
                    generation.set(generation.get().wrapping_add(1));
                    glib::Propagation::Proceed
                });
            }
        }
        let slot = Rc::new(LoadSlot::default());
        let closing = slot.clone();
        window.connect_close_request(move |_| {
            closing.cancel();
            glib::Propagation::Proceed
        });
        let destroying = slot.clone();
        window.connect_destroy(move |_| destroying.cancel());
        let mode_action = gio::SimpleAction::new_stateful(
            "image-mode",
            Some(glib::VariantTy::STRING),
            &pivra::settings::load_view_mode().id().to_variant(),
        );
        mode_action.set_enabled(false);
        let texture: Rc<RefCell<Option<gtk::gdk::Texture>>> = Rc::default();
        let closing_texture = texture.clone();
        window.connect_close_request(move |_| {
            closing_texture.borrow_mut().take();
            glib::Propagation::Proceed
        });
        let destroyed_texture = texture.clone();
        window.connect_destroy(move |_| {
            destroyed_texture.borrow_mut().take();
        });
        let transform = Rc::new(RefCell::new(Transform::default()));
        let filter = Rc::new(Cell::new(Resampling::default()));
        let zoom_wheel = Rc::new(Cell::new(false));
        install_interaction(
            &picture,
            texture.clone(),
            transform.clone(),
            &mode_action,
            filter.clone(),
            zoom_wheel.clone(),
        );
        let mode_transform = transform.clone();
        let weak_picture = picture.downgrade();
        let current_texture = texture.clone();
        let mode_filter = filter.clone();
        mode_action.connect_activate(move |action, parameter| {
            let Some(mode) = parameter.and_then(|v| v.str()).and_then(ViewMode::from_id) else {
                return;
            };
            mode_transform.borrow_mut().reset();
            action.set_state(&mode.id().to_variant());
            pivra::settings::save_view_mode(mode);
            if let Some(picture) = weak_picture.upgrade() {
                present_filtered(
                    &picture,
                    current_texture.borrow().as_ref(),
                    mode.id(),
                    mode_transform.clone(),
                    mode_filter.clone(),
                );
            }
        });
        window.add_action(&mode_action);
        let quality_action = gio::SimpleAction::new_stateful(
            "image-resampling",
            Some(glib::VariantTy::STRING),
            &Resampling::default().id().to_variant(),
        );
        let weak_picture = picture.downgrade();
        let current_texture = texture.clone();
        let state = transform.clone();
        let chosen = filter.clone();
        let mode = mode_action.clone();
        quality_action.connect_activate(move |action, parameter| {
            let Some(value) = parameter
                .and_then(|v| v.str())
                .and_then(Resampling::from_id)
            else {
                return;
            };
            chosen.set(value);
            action.set_state(&value.id().to_variant());
            if let Some(picture) = weak_picture.upgrade() {
                let mode = mode.state();
                let id = mode.as_ref().and_then(|v| v.str()).unwrap_or("fit");
                present_filtered(
                    &picture,
                    current_texture.borrow().as_ref(),
                    id,
                    state.clone(),
                    chosen.clone(),
                );
            }
        });
        window.add_action(&quality_action);
        Self {
            animation_controls,
            video_controls,
            animation_generation,
            thumbnail_strip,
            video,
            mode_action,
            prefetch,
            phase,
            requested: Rc::default(),
            filter,
            zoom_wheel,
            transform,
            texture,
            overlay,
            picture,
            status,
            loading_spinner,
            slot,
            window: window.downgrade(),
        }
    }

    /// Bounded queue diagnostics for native/controlled probes; no product UI.
    #[allow(dead_code)]
    pub(super) fn set_zoom_wheel_enabled(&self, enabled: bool) {
        self.zoom_wheel.set(enabled);
    }

    // Lifecycle probes consume this diagnostic; production has no stats UI.
    #[allow(dead_code, reason = "Used by selected standalone lifecycle probes")]
    pub(super) fn load_stats(&self) -> image_requests::LoadStats {
        self.slot.stats()
    }

    #[allow(dead_code)] // Used by the production window, only a subset of standalone probes.
    pub(super) fn set_thumbnail_catalog(
        &self,
        photos: &[(usize, PathBuf)],
        selected: Option<usize>,
    ) {
        self.thumbnail_strip.set_catalog(photos, selected);
    }
    #[allow(dead_code)]
    pub(super) fn finish_initial_thumbnail_catalog(&self, selected: Option<usize>) {
        self.thumbnail_strip.finish_initial_catalog(selected);
    }
    #[allow(dead_code)]
    pub(super) fn thumbnail_stats(&self) -> thumbnail_overlay::ThumbnailStats {
        self.thumbnail_strip.stats()
    }
    #[allow(dead_code)]
    pub(super) fn invalidate_thumbnails(&self) {
        self.thumbnail_strip.invalidate();
    }
    #[allow(dead_code)]
    pub(super) fn on_thumbnail_select(&self, callback: impl Fn(usize) + 'static) {
        self.thumbnail_strip.on_select(callback);
    }
    #[allow(dead_code)]
    pub(super) fn clear_thumbnail_select(&self) {
        self.thumbnail_strip.clear_on_select();
    }
    pub(super) fn widget(&self) -> &gtk::Overlay {
        &self.overlay
    }

    #[allow(dead_code)] // Some standalone view probes never display a non-image selection.
    pub(super) fn show_message(&self, message: &str) {
        self.prepare(message, true);
    }
    pub(super) fn load_video(&self, path: &std::path::Path) {
        self.prepare("", true);
        self.video.set_filename(Some(path));
        self.video.set_visible(true);
        self.video_controls.attach(self.video.media_stream());
    }
    fn stop_video(&self) {
        self.video_controls.attach(None);
        if let Some(stream) = self.video.media_stream() {
            stream.pause();
        }
        self.video.set_visible(false);
        self.video.set_filename(None::<&std::path::Path>);
    }
    fn prepare(&self, message: &str, cancel: bool) {
        self.stop_video();
        self.animation_generation.set(self.animation_generation.get().wrapping_add(1));
        self.animation_controls.reset();
        if cancel {
            self.slot.cancel();
            self.phase.set(Phase::Cancelled);
        }
        {
            let mut state = self.prefetch.borrow_mut();
            state.current_path = None;
            state.cache.lock().unwrap().pin(None);
            state.queue = state.neighbors.clone().into();
        }
        self.requested.borrow_mut().take();
        self.transform.borrow_mut().reset();
        self.mode_action.set_enabled(false);
        self.texture.borrow_mut().take();
        self.mode_action.set_state(&"fit".to_variant());
        self.picture.set_paintable(None::<&gtk::gdk::Paintable>);
        self.loading_spinner.set_visible(false);
        self.picture.set_content_fit(gtk::ContentFit::ScaleDown);
        self.status.set_text(message);
        self.status.set_visible(!message.is_empty());
    }

    #[allow(dead_code)] // Standalone view probes do not own a catalog.
    pub(super) fn set_neighbors(&self, paths: Vec<PathBuf>) {
        let paths: Vec<_> = paths
            .into_iter()
            .filter(|p| pivra::discovery::supported_photo(p))
            .take(2)
            .collect();
        {
            let mut state = self.prefetch.borrow_mut();
            if state.neighbors != paths {
                if let Some(bg) = state
                    .background
                    .as_ref()
                    .filter(|b| b.consumer.borrow().is_none())
                {
                    bg.token.cancel();
                }
                state.neighbors = paths.clone();
            }
            state.queue = paths.into();
        }
        preload::maybe_start(&self.prefetch, &self.slot, self.texture.borrow().is_some());
    }
    #[allow(dead_code)]
    pub(super) fn invalidate_cache_paths(&self, paths: &[PathBuf]) {
        self.prefetch.borrow_mut().invalidate_paths(paths);
    }
    #[allow(dead_code)]
    pub(super) fn preload_stats(&self) -> preload::PreloadStats {
        self.prefetch.borrow().stats()
    }
    #[allow(dead_code)]
    pub(super) fn memory_pressure(&self) {
        self.prefetch.borrow_mut().pressure();
    }
    #[allow(dead_code)]
    pub(super) fn disable_preload(&self) {
        let mut state = self.prefetch.borrow_mut();
        state.enabled = false;
        state.queue.clear();
        if let Some(bg) = state
            .background
            .as_ref()
            .filter(|b| b.consumer.borrow().is_none())
        {
            bg.token.cancel();
        }
    }

    #[allow(dead_code)]
    pub(super) fn configure_cache(&self, budget: usize, entry: usize) {
        let mut state = self.prefetch.borrow_mut();
        state.invalidate();
        *state.cache.lock().unwrap() = pivra::image_cache::Cache::new(budget, entry);
    }
    #[allow(dead_code)]
    pub(super) fn cached_texture(&self, path: &std::path::Path) -> Option<gtk::gdk::Texture> {
        self.prefetch.borrow().cache.lock().unwrap().peek(path)
    }

    /// Retarget only an unfinished supported-image request after a paired folder rename.
    #[allow(dead_code)] // Standalone view probes omit the folder-monitor integration.
    pub(super) fn retarget_pending(&self, old: &std::path::Path, new: &std::path::Path) {
        let supported = pivra::discovery::supported_photo(new);
        if !supported
            || self.requested.borrow().as_deref() != Some(old)
            || !self.slot.current_in_flight()
        {
            return;
        }
        let mode = self.mode_action.state();
        self.load(new.to_owned());
        if let Some(mode) = mode {
            self.mode_action.activate(Some(&mode));
        }
    }

    #[allow(dead_code)] // Probe diagnostics, not a user-visible debug indicator.
    pub(super) fn phase(&self) -> Phase {
        self.phase.get()
    }

    pub(super) fn load(&self, path: PathBuf) {
        if self.requested.borrow().as_ref() == Some(&path) && self.slot.current_in_flight() {
            return;
        }
        self.load_request(path, true, || {}, |_| {});
    }

    pub(super) fn load_with_completion(
        &self,
        path: PathBuf,
        on_completion: impl FnOnce() + 'static,
    ) {
        self.load_observed(path, on_completion, |_| {});
    }

    pub(super) fn load_observed(
        &self,
        path: PathBuf,
        on_completion: impl FnOnce() + 'static,
        on_result: impl FnOnce(bool) + 'static,
    ) {
        self.load_request(path, false, on_completion, on_result);
    }
    fn load_request(
        &self,
        path: PathBuf,
        allow_cache: bool,
        on_completion: impl FnOnce() + 'static,
        on_result: impl FnOnce(bool) + 'static,
    ) {
        self.stop_video();
        self.animation_controls.reset();
        let animation_controls = self.animation_controls.clone();
        let generation = self.animation_generation.clone();
        let expected = generation.get().wrapping_add(1);
        generation.set(expected);
        let animation_path = path.clone();
        let animation_picture = self.picture.downgrade();
        let animation_texture = self.texture.clone();
        let animation_mode = self.mode_action.clone();
        let animation_transform = self.transform.clone();
        let animation_filter = self.filter.clone();
        if !allow_cache {
            self.prefetch.borrow_mut().invalidate();
        }
        let promoted = if allow_cache {
            self.prefetch
                .borrow()
                .background
                .as_ref()
                .filter(|b| {
                    b.path == path && !b.token.is_cancelled() && b.consumer.borrow().is_none()
                })
                .map(|b| (b.token.clone(), b.consumer.clone()))
        } else {
            None
        };
        let cancellable = if let Some((background, _)) = &promoted {
            self.slot.begin_promoting(background)
        } else {
            self.slot.begin()
        };
        // A request is not a presentation. Keep the last ready paintable,
        // transform and cache pin until a replacement texture is fully ready.
        // No loading text, blank placeholder or early geometry reset.
        self.status.set_text("");
        self.status.set_visible(false);
        // Retain the last ready frame under a non-interactive loading indicator.
        // Do not show a spinner on a blank first-load viewport.
        self.loading_spinner
            .set_visible(self.texture.borrow().is_some());
        self.phase.set(Phase::FullLoading);
        {
            let mut state = self.prefetch.borrow_mut();
            state.current_path = Some(path.clone());
            state.queue = state.neighbors.clone().into();
        }

        self.requested.replace(Some(path.clone()));
        self.mode_action.set_enabled(true);
        let window = self.window.clone();
        let picture = self.picture.downgrade();
        let status = self.status.downgrade();
        let loading_spinner = self.loading_spinner.downgrade();
        let slot = self.slot.clone();
        let mode_action = self.mode_action.clone();
        let texture = self.texture.clone();
        let filter = self.filter.clone();
        let transform = self.transform.clone();
        let prefetch = self.prefetch.clone();
        let epoch = prefetch.borrow().epoch.clone();
        let phase = self.phase.clone();
        let target_name = path
            .file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
            .into_owned();
        type Completion = preload::Completion;
        let completion: Rc<RefCell<Option<Completion>>> = Rc::new(RefCell::new(Some(Box::new({
            let slot = slot.clone();
            let cancellable = cancellable.clone();
            move |result, started| {
                if started {
                    on_completion();
                }
                // A completed old request may already be queued on the GTK context.
                // Cancellation and identity both gate every mutation/diagnostic.
                if !slot.accepts(&cancellable) {
                    slot.suppress();
                    return;
                }
                let (Some(window), Some(picture), Some(status), Some(loading_spinner)) = (
                    window.upgrade(),
                    picture.upgrade(),
                    status.upgrade(),
                    loading_spinner.upgrade(),
                ) else {
                    return;
                };
                if !window.is_visible() {
                    return;
                }
                let result = match result {
                    Ok(preload::Output::Cached(_, _))
                        if !Rc::ptr_eq(&epoch, &prefetch.borrow().epoch) =>
                    {
                        Err("Image changed during cache lookup; refresh to reload.".into())
                    }
                    other => other,
                };
                let success = result.is_ok();
                loading_spinner.set_visible(false);
                phase.set(phase.get().full(success, false));
                match result {
                    Ok(output) => {
                        let (decoded, key) = match output {
                            preload::Output::Decoded(frame, key) => (frame.texture(), key),
                            preload::Output::Cached(texture, key) => (texture, Some(key)),
                        };
                        let cost = texture_cost(decoded.width() as u32, decoded.height() as u32);
                        {
                            let mut state = prefetch.borrow_mut();
                            state.active_large =
                                cost.is_none_or(|c| c > state.cache.lock().unwrap().entry_limit());
                            let mut cache = state.cache.lock().unwrap();
                            if Rc::ptr_eq(&epoch, &state.epoch) {
                                cache.pin(key.clone());
                                if let (Some(key), Some(cost)) = (key, cost) {
                                    cache.insert(key, decoded.clone(), cost, &[]);
                                }
                            }
                        }

                        let mode = mode_action
                            .state()
                            .and_then(|v| v.str().and_then(ViewMode::from_id))
                            .unwrap_or_default();
                        // One main-context handoff, without yielding a GTK frame:
                        // old pixels stay in Picture until the new paintable is set.
                        transform.borrow_mut().reset();
                        texture.replace(Some(decoded.clone()));
                        mode_action.set_state(&mode.id().to_variant());
                        present_filtered(
                            &picture,
                            Some(&decoded),
                            mode.id(),
                            transform.clone(),
                            filter.clone(),
                        );
                        status.set_visible(false);
                        animation::start(
                            animation_path,
                            generation,
                            expected,
                            animation_controls,
                            animation_picture,
                            animation_texture,
                            animation_mode,
                            animation_transform,
                            animation_filter,
                        );
                    }
                    Err(error) => {
                        eprintln!("Pivra: image load failed: {error}");
                        let summary = error.lines().next().unwrap_or("Secure loading failed.");
                        let retained = if texture.borrow().is_some() {
                            "\nPrevious photo is still displayed."
                        } else {
                            ""
                        };
                        status.set_text(&format!(
                            "Unable to display image.\nFile: {target_name}\n{summary}{retained}"
                        ));
                        status.set_visible(true);
                    }
                }
                preload::maybe_start(&prefetch, &slot, texture.borrow().is_some());
                // User observers can reenter/close/start a request. Never mutate GTK after them.
                on_result(success);
            }
        }))));
        if let Some((_, consumer)) = promoted {
            self.prefetch.borrow_mut().promoted();
            let callback = completion.borrow_mut().take();
            *consumer.borrow_mut() = callback;
            return;
        }
        let deliver = completion.clone();
        let expired = completion;
        let running = slot.clone();
        let token = cancellable.clone();
        let prefetch = self.prefetch.clone();
        let requested = std::time::Instant::now();
        let cache = prefetch.borrow().cache.clone();
        slot.dispatch(
            &cancellable,
            move || {
                prefetch.borrow_mut().admission(requested);
                let loading = token.clone();
                let job = gio::spawn_blocking(move || {
                    preload::fetch(&path, &loading, &cache, allow_cache, None)
                });
                glib::MainContext::default().spawn_local(async move {
                    let (result, decoded) = job
                        .await
                        .unwrap_or_else(|_| (Err("Image worker failed".into()), true));
                    if decoded {
                        prefetch.borrow_mut().foreground_decode();
                    }
                    running.finish(&token);
                    let callback = deliver.borrow_mut().take();
                    if let Some(callback) = callback {
                        callback(result, true);
                    }
                });
            },
            move || {
                let callback = expired.borrow_mut().take();
                if let Some(callback) = callback {
                    callback(
                        Err("Image loading is busy. Please try again shortly.".into()),
                        false,
                    );
                }
            },
        );
    }
}

/// Attached only to the photo widget: header/menu/banner events never enter here.
pub(super) fn install_interaction(
    picture: &gtk::Picture,
    texture: Rc<RefCell<Option<gtk::gdk::Texture>>>,
    transform: Rc<RefCell<Transform>>,
    action: &gio::SimpleAction,
    filter: Rc<Cell<Resampling>>,
    zoom_wheel: Rc<Cell<bool>>,
) {
    let pointer = Rc::new(Cell::new((0., 0.)));
    let motion = gtk::EventControllerMotion::new();
    let entered = pointer.clone();
    motion.connect_enter(move |_, x, y| entered.set((x, y)));
    let position = pointer.clone();
    motion.connect_motion(move |_, x, y| position.set((x, y)));
    picture.add_controller(motion);
    let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
    let weak = picture.downgrade();
    let source = texture.clone();
    let state = transform.clone();
    let mode_action = action.clone();
    let scroll_filter = filter.clone();
    scroll.connect_scroll(move |controller, _, dy| {
        let zoom = zoom_wheel.get()
            || controller
                .current_event_state()
                .contains(gtk::gdk::ModifierType::BUTTON1_MASK);
        if !zoom {
            if let Some(picture) = weak.upgrade() {
                if let Some(window) = picture.root().and_downcast::<adw::ApplicationWindow>() {
                    let action = if dy > 0.0 {
                        "next-media"
                    } else {
                        "previous-media"
                    };
                    if dy != 0.0 {
                        gio::prelude::ActionGroupExt::activate_action(&window, action, None);
                    }
                }
            }
            return glib::Propagation::Stop;
        }
        let Some(picture) = weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        if !picture.is_mapped() {
            return glib::Propagation::Proceed;
        }
        let source = source.borrow();
        let Some(texture) = source.as_ref() else {
            return glib::Propagation::Proceed;
        };
        let mode = mode_action
            .state()
            .and_then(|v| v.str().and_then(ViewMode::from_id))
            .unwrap_or(ViewMode::Actual);
        let changed = state.borrow_mut().zoom(
            (texture.width() as f64, texture.height() as f64),
            (picture.width() as f64, picture.height() as f64),
            mode,
            pointer.get(),
            dy,
        );
        if changed {
            manual(
                &picture,
                texture,
                state.clone(),
                mode,
                &mode_action,
                scroll_filter.clone(),
            );
        }
        glib::Propagation::Stop
    });
    picture.add_controller(scroll);
    let drag = gtk::GestureDrag::new();
    drag.set_button(1);
    drag.set_propagation_phase(gtk::PropagationPhase::Capture);
    let last = Rc::new(Cell::new(None::<(f64, f64)>));
    let start = last.clone();
    let source = texture.clone();
    let drag_generation = Rc::new(Cell::new(0));
    let began_generation = drag_generation.clone();
    let began_transform = transform.clone();
    drag.connect_drag_begin(move |gesture, _, _| {
        if source.borrow().is_some() && gesture.widget().is_some_and(|w| w.is_mapped()) {
            began_generation.set(began_transform.borrow().generation());
            start.set(Some((0., 0.)));
            gesture.set_state(gtk::EventSequenceState::Claimed);
        }
    });
    let weak = picture.downgrade();
    let offsets = last.clone();
    let state = transform;
    let mode_action = action.clone();
    drag.connect_drag_update(move |_, x, y| {
        if drag_generation.get() != state.borrow().generation() {
            // Zoom while holding the left button changes the transform
            // generation; rebase the drag instead of disabling it permanently.
            drag_generation.set(state.borrow().generation());
            offsets.set(Some((x, y)));
            return;
        }
        let Some(previous) = offsets.get() else {
            return;
        };
        let Some(picture) = weak.upgrade() else {
            return;
        };
        if !picture.is_mapped() {
            offsets.set(None);
            return;
        }
        let source = texture.borrow();
        let Some(texture) = source.as_ref() else {
            offsets.set(None);
            return;
        };
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        offsets.set(Some((x, y)));
        let mode = mode_action
            .state()
            .and_then(|v| v.str().and_then(ViewMode::from_id))
            .unwrap_or(ViewMode::Actual);
        let changed = state.borrow_mut().pan(
            (texture.width() as f64, texture.height() as f64),
            (picture.width() as f64, picture.height() as f64),
            mode,
            (x - previous.0, y - previous.1),
        );
        if changed {
            manual(
                &picture,
                texture,
                state.clone(),
                mode,
                &mode_action,
                filter.clone(),
            );
        }
    });
    let end = last.clone();
    drag.connect_drag_end(move |_, _, _| end.set(None));
    drag.connect_cancel(move |_, _| last.set(None));
    picture.add_controller(drag);
}
fn manual(
    picture: &gtk::Picture,
    texture: &gtk::gdk::Texture,
    transform: Rc<RefCell<Transform>>,
    mode: ViewMode,
    action: &gio::SimpleAction,
    filter: Rc<Cell<Resampling>>,
) {
    picture.set_overflow(gtk::Overflow::Hidden);
    picture.set_content_fit(gtk::ContentFit::Fill);
    if let Some(current) = picture
        .paintable()
        .and_then(|p| p.downcast::<actual_paintable::ActualPaintable>().ok())
        .filter(|p| p.has_transform(&transform) && p.has_texture(texture))
    {
        current.invalidate_contents();
    } else {
        picture.set_paintable(Some(&actual_paintable::ActualPaintable::transformed(
            texture, transform, mode, filter,
        )));
    }
    action.set_state(&"manual".to_variant());
}

pub(super) fn present_filtered(
    picture: &gtk::Picture,
    texture: Option<&gtk::gdk::Texture>,
    id: &str,
    transform: Rc<RefCell<Transform>>,
    filter: Rc<Cell<Resampling>>,
) {
    let mode = ViewMode::from_id(id).unwrap_or(ViewMode::Actual);
    if id != "manual" && filter.get() == Resampling::Smooth && mode != ViewMode::Actual {
        present(picture, texture, mode);
        return;
    }
    let Some(texture) = texture else {
        return;
    };
    if let Some(current) = picture
        .paintable()
        .and_then(|p| p.downcast::<actual_paintable::ActualPaintable>().ok())
        .filter(|p| p.has_transform(&transform) && p.has_texture(texture))
    {
        current.set_mode(mode);
        current.invalidate_contents();
        return;
    }
    picture.set_overflow(gtk::Overflow::Hidden);
    picture.set_content_fit(gtk::ContentFit::Fill);
    picture.set_paintable(Some(&actual_paintable::ActualPaintable::transformed(
        texture, transform, mode, filter,
    )));
}

pub(super) fn present(picture: &gtk::Picture, texture: Option<&gtk::gdk::Texture>, mode: ViewMode) {
    picture.set_overflow(gtk::Overflow::Hidden);
    match mode {
        ViewMode::Fit | ViewMode::Fill => {
            picture.set_content_fit(if mode == ViewMode::Fit {
                gtk::ContentFit::Contain
            } else {
                gtk::ContentFit::Cover
            });
            picture.set_paintable(texture);
        }
        ViewMode::Actual => {
            picture.set_content_fit(gtk::ContentFit::Fill);
            picture.set_paintable(texture.map(actual_paintable::ActualPaintable::new).as_ref());
        }
    }
}

// Compatibility entry points used by standalone PIV-09 regression examples.
#[allow(dead_code)]
pub(super) fn build(window: &adw::ApplicationWindow, path: PathBuf) -> gtk::Overlay {
    let view = ImageView::new(window);
    view.load(path);
    view.widget().clone()
}

#[allow(dead_code)]
pub(super) fn build_with_completion(
    window: &adw::ApplicationWindow,
    path: PathBuf,
    on_completion: impl FnOnce() + 'static,
) -> gtk::Overlay {
    let view = ImageView::new(window);
    view.load_with_completion(path, on_completion);
    view.widget().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_completed_requests_are_rejected_after_replacement_or_video() {
        let slot = LoadSlot::default();
        let completed_but_callback_queued = slot.begin();
        assert!(slot.accepts(&completed_but_callback_queued));
        let current = slot.begin();
        assert!(completed_but_callback_queued.is_cancelled());
        assert!(!slot.accepts(&completed_but_callback_queued));
        assert!(slot.accepts(&current));
        slot.cancel(); // video/unsupported/close: no active image request
        assert!(current.is_cancelled());
        assert!(!slot.accepts(&current));
    }

    #[test]
    fn rapidly_replaced_loads_and_windows_have_independent_cancellation() {
        let first = LoadSlot::default();
        let second = LoadSlot::default();
        let other = second.begin();
        let mut previous = first.begin();
        for _ in 0..1000 {
            let next = first.begin();
            assert!(previous.is_cancelled());
            assert!(!first.accepts(&previous));
            assert!(first.accepts(&next));
            assert!(second.accepts(&other));
            previous = next;
        }
        first.cancel();
        assert!(second.accepts(&other));
        second.cancel();
        assert!(!second.accepts(&other));
    }
}
