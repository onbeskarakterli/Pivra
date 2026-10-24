//! Per-window presentation and cancellation. Glycin alone applies orientation.

#[path = "actual_paintable.rs"]
mod actual_paintable;
use adw::prelude::*;
use gtk::{gio, glib};
use pivra::viewing::{Transform, ViewMode};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
};

#[derive(Default)]
struct LoadSlot(RefCell<Option<gio::Cancellable>>);

impl LoadSlot {
    fn begin(&self) -> gio::Cancellable {
        self.cancel();
        let token = gio::Cancellable::new();
        self.0.replace(Some(token.clone()));
        token
    }

    fn cancel(&self) {
        let previous = self.0.borrow_mut().take();
        if let Some(previous) = previous {
            previous.cancel();
        }
    }

    fn accepts(&self, token: &gio::Cancellable) -> bool {
        !token.is_cancelled() && self.0.borrow().as_ref() == Some(token)
    }
}

#[derive(Clone)]
pub(super) struct ImageView {
    overlay: gtk::Overlay,
    picture: gtk::Picture,
    status: gtk::Label,
    slot: Rc<LoadSlot>,
    window: glib::WeakRef<adw::ApplicationWindow>,
    mode_action: gio::SimpleAction,
    texture: Rc<RefCell<Option<gtk::gdk::Texture>>>,
    transform: Rc<RefCell<Transform>>,
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
        overlay.add_overlay(&status);
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
            &"fit".to_variant(),
        );
        mode_action.set_enabled(false);
        let texture: Rc<RefCell<Option<gtk::gdk::Texture>>> = Rc::default();
        let transform = Rc::new(RefCell::new(Transform::default()));
        install_interaction(&picture, texture.clone(), transform.clone(), &mode_action);
        let mode_transform = transform.clone();
        let weak_picture = picture.downgrade();
        let current_texture = texture.clone();
        mode_action.connect_activate(move |action, parameter| {
            let Some(mode) = parameter.and_then(|v| v.str()).and_then(ViewMode::from_id) else {
                return;
            };
            mode_transform.borrow_mut().reset();
            action.set_state(&mode.id().to_variant());
            if let Some(picture) = weak_picture.upgrade() {
                present(&picture, current_texture.borrow().as_ref(), mode);
            }
        });
        window.add_action(&mode_action);
        Self {
            mode_action,
            transform,
            texture,
            overlay,
            picture,
            status,
            slot,
            window: window.downgrade(),
        }
    }

    pub(super) fn widget(&self) -> &gtk::Overlay {
        &self.overlay
    }

    pub(super) fn show_message(&self, message: &str) {
        self.slot.cancel();
        self.transform.borrow_mut().reset();
        self.mode_action.set_enabled(false);
        self.texture.borrow_mut().take();
        self.mode_action.set_state(&"fit".to_variant());
        self.picture.set_paintable(None::<&gtk::gdk::Paintable>);
        self.picture.set_content_fit(gtk::ContentFit::ScaleDown);
        self.status.set_text(message);
        self.status.set_visible(true);
    }

    pub(super) fn load(&self, path: PathBuf) {
        self.load_with_completion(path, || {});
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
        self.show_message("Loading image securely…");
        self.mode_action.set_enabled(true);
        let cancellable = self.slot.begin();
        let loading = cancellable.clone();
        let job = gio::spawn_blocking(move || {
            glib::MainContext::new().block_on(crate::image_loader::load(&path, &loading))
        });
        let window = self.window.clone();
        let picture = self.picture.downgrade();
        let status = self.status.downgrade();
        let slot = self.slot.clone();
        let mode_action = self.mode_action.clone();
        let texture = self.texture.clone();
        glib::MainContext::default().spawn_local(async move {
            let result = job
                .await
                .unwrap_or_else(|_| Err("Image worker failed".into()));
            on_completion();
            // A completed old request may already be queued on the GTK context.
            // Cancellation and identity both gate every mutation/diagnostic.
            if !slot.accepts(&cancellable) {
                return;
            }
            let (Some(window), Some(picture), Some(status)) =
                (window.upgrade(), picture.upgrade(), status.upgrade())
            else {
                return;
            };
            if !window.is_visible() {
                return;
            }
            on_result(result.is_ok());
            match result {
                Ok(frame) => {
                    let decoded = frame.texture();
                    let mode = mode_action
                        .state()
                        .and_then(|v| v.str().and_then(ViewMode::from_id))
                        .unwrap_or_default();
                    present(&picture, Some(&decoded), mode);
                    texture.replace(Some(decoded));
                    status.set_visible(false);
                }
                Err(error) => {
                    eprintln!("Pivra: image load failed: {error}");
                    let summary = error.lines().next().unwrap_or("Secure loading failed.");
                    status.set_text(&format!("Unable to display image.\n{summary}"));
                }
            }
        });
    }
}

/// Attached only to the photo widget: header/menu/banner events never enter here.
pub(super) fn install_interaction(
    picture: &gtk::Picture,
    texture: Rc<RefCell<Option<gtk::gdk::Texture>>>,
    transform: Rc<RefCell<Transform>>,
    action: &gio::SimpleAction,
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
    scroll.connect_scroll(move |_, _, dy| {
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
            manual(&picture, texture, state.clone(), mode, &mode_action);
        }
        glib::Propagation::Stop
    });
    picture.add_controller(scroll);
    let drag = gtk::GestureDrag::new();
    drag.set_button(1);
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
            offsets.set(None);
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
            manual(&picture, texture, state.clone(), mode, &mode_action);
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
) {
    picture.set_overflow(gtk::Overflow::Hidden);
    picture.set_content_fit(gtk::ContentFit::Fill);
    if let Some(current) = picture
        .paintable()
        .and_then(|p| p.downcast::<actual_paintable::ActualPaintable>().ok())
        .filter(|p| p.has_transform(&transform))
    {
        current.invalidate_contents();
    } else {
        picture.set_paintable(Some(&actual_paintable::ActualPaintable::transformed(
            texture, transform, mode,
        )));
    }
    action.set_state(&"manual".to_variant());
}

pub(super) fn present(picture: &gtk::Picture, texture: Option<&gtk::gdk::Texture>, mode: ViewMode) {
    picture.set_overflow(gtk::Overflow::Hidden);
    match mode {
        ViewMode::Fit | ViewMode::Fill => {
            picture.set_content_fit(if mode == ViewMode::Fit {
                gtk::ContentFit::ScaleDown
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
