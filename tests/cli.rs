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
    let expected = format!(
        "Pivra {}\nUsage: pivra [--help | --version]\n       pivra <local-media-path | directory | file://URI>\nPhoto decoding requires a working sandboxed Glycin loader.\nA directory opens its first supported media in natural filename order.\n",
        env!("CARGO_PKG_VERSION")
    );
    for flag in ["--help", "-h"] {
        let output = run(&[flag]);
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn version_and_alias_work_without_a_display() {
    for flag in ["--version", "-V"] {
        let output = run(&[flag]);
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(
            output.stdout,
            format!("Pivra {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
        );
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

#[test]
fn valid_local_path_and_uri_reach_graphical_initialization_without_decoding() {
    let directory = support::temp_dir();
    let path = directory.path().join("fotoğraf # % örneği.png");
    std::fs::write(&path, include_bytes!("fixtures/pixels-2x2.png")).unwrap();
    let uri = gtk::glib::filename_to_uri(&path, None).unwrap();
    for argument in [path.to_str().unwrap(), uri.as_str()] {
        let output = run(&[argument]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .starts_with("Pivra: unable to initialize GTK;")
        );
    }
}

#[test]
fn directory_with_media_passes_input_validation_before_graphical_initialization() {
    let directory = support::temp_dir();
    let path = directory.path().join("1.png");
    std::fs::write(&path, include_bytes!("fixtures/pixels-2x2.png")).unwrap();
    let output = run(&[directory.path().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .starts_with("Pivra: unable to initialize GTK;"));
}

#[test]
fn invalid_file_inputs_fail_before_graphical_initialization() {
    let directory = support::temp_dir();
    let unsupported = directory.path().join("unsupported.txt");
    std::fs::write(&unsupported, include_bytes!("fixtures/model-video.txt")).unwrap();
    let missing = directory.path().join("missing.png");
    for argument in [
        directory.path().to_str().unwrap(),
        unsupported.to_str().unwrap(),
        missing.to_str().unwrap(),
        "https://example.org/a.png",
        "file:///tmp/a%GG.png",
        "",
    ] {
        let output = run(&[argument]);
        assert_eq!(output.status.code(), Some(2), "{argument}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.starts_with("Pivra: "));
        assert!(!error.contains("initialize GTK") && !error.contains("panicked"));
    }
}
