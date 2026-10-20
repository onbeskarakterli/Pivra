mod support;

use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    run_on_backend(args, "x11")
}

fn run_on_backend(args: &[&str], backend: &str) -> Output {
    let runtime = support::temp_dir();
    Command::new(env!("CARGO_BIN_EXE_pivra"))
        .args(args)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env("XDG_RUNTIME_DIR", runtime.path())
        .env("GDK_BACKEND", backend)
        .output()
        .expect("run Pivra")
}

#[test]
fn help_and_alias_work_without_a_display() {
    let expected = b"Pivra 0.1.0\nUsage: pivra [--help | --version]\nGNOME image viewing will be available in a later development stage.\n";
    for flag in ["--help", "-h"] {
        let output = run(&[flag]);
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn version_and_alias_work_without_a_display() {
    for flag in ["--version", "-V"] {
        let output = run(&[flag]);
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, b"Pivra 0.1.0\n");
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn invalid_arguments_keep_exit_code_and_streams() {
    for args in [
        vec!["--unknown"],
        vec!["one", "two"],
        vec!["--help", "--version"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, b"Pivra: invalid arguments; use --help\n");
    }
}

#[test]
fn launch_without_a_display_fails_cleanly() {
    for backend in ["x11", "wayland"] {
        let output = run_on_backend(&[], backend);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).expect("UTF-8 error");
        assert!(
            error.starts_with("Pivra: unable to initialize GTK; a graphical session is required:")
        );
        assert!(!error.contains("panicked"));
    }
}
