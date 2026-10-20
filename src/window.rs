//! Native window construction; no argument parsing or media I/O.

use crate::input::OpenedFile;
use adw::prelude::*;
use pivra::{APP_NAME, settings::Settings};

pub(super) fn build(app: &adw::Application, settings: &Settings, file: Option<OpenedFile>) {
    let header = adw::HeaderBar::new();
    let title = adw::WindowTitle::new(APP_NAME, "");
    header.set_title_widget(Some(&title));

    let viewport = gtk::Box::new(gtk::Orientation::Vertical, 0);
    viewport.set_hexpand(true);
    viewport.set_vexpand(true);
    if file.is_some() {
        let placeholder = gtk::Label::new(Some("Image decoding is not implemented yet."));
        placeholder.set_wrap(true);
        placeholder.set_valign(gtk::Align::Center);
        placeholder.set_vexpand(true);
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
    // The map callback owns the association for this window's lifetime.
    // No global/current file is shared between independent launches.
    window.connect_map(move |window| {
        if let Some(file) = &file {
            let name = file
                .item
                .path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            window.set_title(Some(&format!("{name} — {APP_NAME}")));
            title.set_title(&name);
            title.set_subtitle("Image decoding is not implemented yet.");
        }
    });
    // GTK owns the widget tree; closing the last window releases the app hold.
    window.present();
}
