//! Pivra CLI and minimal GTK4/libadwaita application shell.

use adw::prelude::*;
use gtk::{gio, glib};

const APP_NAME: &str = "Pivra";
const APP_ID: &str = "io.github.onbeskarakterli.Pivra";

fn main() {
    match parse_args(std::env::args().skip(1)) {
        Command::Version => println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION")),
        Command::Help => print_help(),
        Command::Launch => {
            std::process::exit(launch().into());
        }
        Command::Invalid => {
            eprintln!("{APP_NAME}: invalid arguments; use --help");
            std::process::exit(2);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Launch,
    Help,
    Version,
    Invalid,
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Command {
    match (args.next().as_deref(), args.next()) {
        (None, None) => Command::Launch,
        (Some("--help" | "-h"), None) => Command::Help,
        (Some("--version" | "-V"), None) => Command::Version,
        _ => Command::Invalid,
    }
}

fn launch() -> glib::ExitCode {
    if let Err(error) = adw::init() {
        eprintln!("{APP_NAME}: unable to initialize GTK; a graphical session is required: {error}");
        return glib::ExitCode::FAILURE;
    }

    let app = adw::Application::builder()
        .application_id(APP_ID)
        // Separate launches may own independent windows. File opening comes later.
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(build_window);
    // CLI arguments have already been validated; do not parse them again in GIO.
    app.run_with_args::<&str>(&[])
}

fn build_window(app: &adw::Application) {
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
        .default_width(960)
        .default_height(640)
        .content(&content)
        .build();
    // GTK owns the widget tree; closing the last window releases the app hold.
    window.present();
}

fn print_help() {
    println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION"));
    println!("Usage: pivra [--help | --version]");
    println!("GNOME image viewing will be available in a later development stage.");
}

#[cfg(test)]
mod tests {
    use super::{Command, parse_args};

    #[test]
    fn application_id_is_valid() {
        assert!(gtk::gio::Application::id_is_valid(super::APP_ID));
    }

    fn command(args: &[&str]) -> Command {
        parse_args(args.iter().map(|s| (*s).to_owned()))
    }

    #[test]
    fn no_arguments_starts_app() {
        assert_eq!(command(&[]), Command::Launch);
    }

    #[test]
    fn help_and_version_flags() {
        assert_eq!(command(&["--help"]), Command::Help);
        assert_eq!(command(&["-h"]), Command::Help);
        assert_eq!(command(&["--version"]), Command::Version);
        assert_eq!(command(&["-V"]), Command::Version);
    }

    #[test]
    fn invalid_arguments_are_rejected() {
        assert_eq!(command(&["--unknown"]), Command::Invalid);
        assert_eq!(command(&["one", "two"]), Command::Invalid);
        assert_eq!(command(&["--help", "--version"]), Command::Invalid);
    }
}
