//! GTK initialization and application lifecycle; CLI dispatch stays in main.

use adw::prelude::*;
use gtk::{gio, glib};
use pivra::{APP_NAME, settings::Settings};

use crate::input::OpenedFile;
use crate::window;
use std::{cell::Cell, rc::Rc};

const APP_ID: &str = "io.github.onbeskarakterli.Pivra";

pub(super) fn launch(input: Option<OpenedFile>) -> glib::ExitCode {
    if let Err(error) = adw::init() {
        eprintln!("{APP_NAME}: unable to initialize GTK; a graphical session is required: {error}");
        return glib::ExitCode::FAILURE;
    }

    let app = adw::Application::builder()
        .application_id(APP_ID)
        // Desktop Exec launches remain independent; GIO delivers files via open.
        .flags(gio::ApplicationFlags::NON_UNIQUE | gio::ApplicationFlags::HANDLES_OPEN)
        .build();
    let settings = Settings::default();
    let open_settings = settings.clone();
    app.connect_activate(move |app| match &input {
        Some(input) => app.open(std::slice::from_ref(&input.file), ""),
        None => window::build(app, &settings, None),
    });
    let failed = Rc::new(Cell::new(false));
    let open_failed = failed.clone();
    app.connect_open(move |app, files, _hint| {
        for file in files {
            match OpenedFile::from_file(file) {
                Ok(file) => window::build(app, &open_settings, Some(file)),
                Err(error) => {
                    eprintln!("{APP_NAME}: {error}");
                    open_failed.set(true);
                }
            }
        }
    });
    // CLI arguments have already been validated; do not parse them again in GIO.
    let status = app.run_with_args::<&str>(&[]);
    if failed.get() {
        glib::ExitCode::FAILURE
    } else {
        status
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn application_id_is_valid() {
        assert!(gtk::gio::Application::id_is_valid(super::APP_ID));
    }
}
