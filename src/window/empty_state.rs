//! Empty-window entry point; file choosing stays in win.choose-media.
use adw::prelude::*;
use gtk::gio;

pub(super) fn install(window: &adw::ApplicationWindow, viewport: &gtk::Box, viewer: &gtk::Box) {
    let button = gtk::Button::with_label("Open File");
    button.set_widget_name("open-file-button");
    button.set_action_name(Some("win.choose-media"));
    button.set_halign(gtk::Align::Center);
    button.add_css_class("suggested-action");
    button.add_css_class("pill");
    let page = adw::StatusPage::builder()
        .icon_name("folder-open-symbolic")
        .title("Open a photo or video to get started")
        .child(&button)
        .hexpand(true)
        .vexpand(true)
        .build();
    page.set_widget_name("empty-media-state");
    viewer.set_visible(false);
    viewport.append(&page);

    // Read-only presentation state, set from the existing catalog/title update.
    // The empty page never owns another viewer, catalog or chooser callback.
    let present = gio::SimpleAction::new_stateful("media-present", None, &false.to_variant());
    present.set_enabled(false);
    let viewer = viewer.downgrade();
    let page = page.downgrade();
    present.connect_notify_local(Some("state"), move |action, _| {
        let has_media = action
            .state()
            .and_then(|v| v.get::<bool>())
            .unwrap_or(false);
        if let Some(viewer) = viewer.upgrade() {
            viewer.set_visible(has_media);
        }
        if let Some(page) = page.upgrade() {
            page.set_visible(!has_media);
        }
    });
    window.add_action(&present);
}

pub(super) fn update(window: &adw::ApplicationWindow, has_media: bool) {
    if let Some(action) = window
        .lookup_action("media-present")
        .and_downcast::<gio::SimpleAction>()
    {
        action.set_state(&has_media.to_variant());
    }
}
