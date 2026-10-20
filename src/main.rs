//! Pivra bootstrap entry point.
//!
//! The GTK4/libadwaita application shell is scheduled for PIV-02.

const APP_NAME: &str = "Pivra";

fn main() {
    match parse_args(std::env::args().skip(1)) {
        Command::Version => println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION")),
        Command::Help => print_help(),
        Command::Launch => {
            // PIV-02 will initialize the GNOME application here.
            println!("{APP_NAME}: graphical application is not yet implemented");
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

fn print_help() {
    println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION"));
    println!("Usage: pivra [--help | --version]");
    println!("GNOME image viewing will be available in a later development stage.");
}

#[cfg(test)]
mod tests {
    use super::{Command, parse_args};

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
