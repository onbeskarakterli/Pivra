//! Dispatch validated CLI commands or start the graphical application.

mod app;
mod cli;
mod window;

use pivra::APP_NAME;

fn main() {
    match cli::parse_args(std::env::args().skip(1)) {
        cli::Command::Version => println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION")),
        cli::Command::Help => cli::print_help(),
        cli::Command::Launch => {
            std::process::exit(app::launch().into());
        }
        cli::Command::Invalid => {
            eprintln!("{APP_NAME}: invalid arguments; use --help");
            std::process::exit(2);
        }
    }
}
