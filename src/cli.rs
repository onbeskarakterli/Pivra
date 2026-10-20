//! CLI parsing and help text; no GTK initialization.

use pivra::APP_NAME;
use std::ffi::OsString;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Command {
    Launch,
    Open(OsString),
    Help,
    Version,
    Invalid,
}

pub(super) fn parse_args(mut args: impl Iterator<Item = OsString>) -> Command {
    let first = args.next();
    match (first.as_ref().and_then(|s| s.to_str()), args.next()) {
        (_, None) if first.is_none() => Command::Launch,
        (Some("--help" | "-h"), None) => Command::Help,
        (Some("--version" | "-V"), None) => Command::Version,
        (_, None)
            if first
                .as_ref()
                .is_some_and(|s| !s.is_empty() && !s.as_encoded_bytes().starts_with(b"-")) =>
        {
            Command::Open(first.expect("one argument"))
        }
        _ => Command::Invalid,
    }
}

pub(super) fn print_help() {
    println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION"));
    println!("Usage: pivra [--help | --version]");
    println!("       pivra <local-photo-path | file://URI>");
    println!("GNOME image viewing will be available in a later development stage.");
    println!("Accepts one local .jpg/.jpeg/.png file; opens a placeholder, without decoding.");
}

#[cfg(test)]
mod tests {
    use super::{Command, parse_args};
    use std::ffi::OsString;

    fn command(args: &[&str]) -> Command {
        parse_args(args.iter().map(OsString::from))
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
