//! Dispatch validated CLI commands or start the graphical application.

mod app;
mod cli;
mod image_loader;
mod image_view;
mod input;
mod raw_loader;
#[allow(dead_code)] // Staged RAW integration; no decoder enabled in UI yet.
mod raw_ppm;
mod window;

use pivra::APP_NAME;

fn main() {
    match cli::parse_args(std::env::args_os().skip(1)) {
        cli::Command::Version => println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION")),
        cli::Command::Help => cli::print_help(),
        cli::Command::Launch => {
            std::process::exit(app::launch(None).into());
        }
        cli::Command::Open(argument) => match input::OpenedFile::from_argument(&argument) {
            Ok(file) => std::process::exit(app::launch(Some(file)).into()),
            Err(error) => {
                eprintln!("{APP_NAME}: {error}");
                std::process::exit(2);
            }
        },
        cli::Command::Invalid => {
            eprintln!("{APP_NAME}: invalid arguments; use --help");
            std::process::exit(2);
        }
    }
}
