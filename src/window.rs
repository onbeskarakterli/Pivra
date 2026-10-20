//! Native window construction; no argument parsing or media I/O.

use adw::prelude::*;
use pivra::{APP_NAME, settings::Settings};

pub(super) fn build(app: &adw::Application, settings: &Settings) {
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&adw::WindowTitle::new(APP_NAME, "")));

    let viewport = gtk::Box::new(gtk::Orientation::Vertical, 0);
    viewport.set_hexpand(true);
    viewport.set_vexpand(true);

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
    // GTK owns the widget tree; closing the last window releases the app hold.
    window.present();
}
