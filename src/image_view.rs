//! Per-window presentation and cancellation. Glycin alone applies orientation.

#[path = "actual_paintable.rs"]
mod actual_paintable;
use adw::prelude::*;
use gtk::{gio, glib};
use pivra::viewing::ViewMode;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

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
        let weak_picture = picture.downgrade();
        let current_texture = texture.clone();
        mode_action.connect_activate(move |action, parameter| {
            let Some(mode) = parameter.and_then(|v| v.str()).and_then(ViewMode::from_id) else {
                return;
            };
            action.set_state(&mode.id().to_variant());
            if let Some(picture) = weak_picture.upgrade() {
                present(&picture, current_texture.borrow().as_ref(), mode);
            }
        });
        window.add_action(&mode_action);
        Self {
            mode_action,
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
