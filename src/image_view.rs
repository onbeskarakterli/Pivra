//! GTK presentation of an asynchronously decoded frame. No orientation transform here.

use adw::prelude::*;
use gtk::{gio, glib};
use std::path::PathBuf;

pub(super) fn build(window: &adw::ApplicationWindow, path: PathBuf) -> gtk::Overlay {
    let picture = gtk::Picture::builder()
        .content_fit(gtk::ContentFit::ScaleDown)
        .can_shrink(true)
        .hexpand(true)
        .vexpand(true)
        .build();
    let status = gtk::Label::new(Some("Loading image securely…"));
    status.set_wrap(true);
    status.set_halign(gtk::Align::Center);
    status.set_valign(gtk::Align::Center);
    let overlay = gtk::Overlay::new();
    overlay.set_hexpand(true);
    overlay.set_vexpand(true);
    overlay.set_child(Some(&picture));
    overlay.add_overlay(&status);
    let cancellable = gio::Cancellable::new();
    let closing = cancellable.clone();
    window.connect_close_request(move |_| {
        closing.cancel();
        glib::Propagation::Proceed
    });
    let destroying = cancellable.clone();
    window.connect_destroy(move |_| destroying.cancel());
    let loading = cancellable.clone();
    let job = gio::spawn_blocking(move || {
        glib::MainContext::new().block_on(crate::image_loader::load(&path, &loading))
    });
    let window = window.downgrade();
    let picture = picture.downgrade();
    let status = status.downgrade();
    glib::MainContext::default().spawn_local(async move {
        let result = job
            .await
            .unwrap_or_else(|_| Err("Image worker failed".into()));
        if cancellable.is_cancelled() {
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
        match result {
            Ok(frame) => {
                // Texture creation and GTK work stay on the main thread.
                // Frame pixels have already been oriented exactly once by Glycin.
                picture.set_paintable(Some(&frame.texture()));
                status.set_visible(false);
            }
            Err(error) => {
                eprintln!("Pivra: image load failed: {error}");
                let summary = error.lines().next().unwrap_or("Secure loading failed.");
                status.set_text(&format!("Unable to display image.\n{summary}"));
            }
        }
    });
    overlay
}
