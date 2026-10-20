//! GTK initialization and application lifecycle; CLI dispatch stays in main.

use adw::prelude::*;
use gtk::{gio, glib};
use pivra::{APP_NAME, settings::Settings};

use crate::window;

const APP_ID: &str = "io.github.onbeskarakterli.Pivra";

pub(super) fn launch() -> glib::ExitCode {
    if let Err(error) = adw::init() {
        eprintln!("{APP_NAME}: unable to initialize GTK; a graphical session is required: {error}");
        return glib::ExitCode::FAILURE;
    }

    let app = adw::Application::builder()
        .application_id(APP_ID)
        // Separate launches may own independent windows. File opening comes later.
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    let settings = Settings::default();
    app.connect_activate(move |app| window::build(app, &settings));
    // CLI arguments have already been validated; do not parse them again in GIO.
    app.run_with_args::<&str>(&[])
}

#[cfg(test)]
mod tests {
    #[test]
    fn application_id_is_valid() {
        assert!(gtk::gio::Application::id_is_valid(super::APP_ID));
    }
}
