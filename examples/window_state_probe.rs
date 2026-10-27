//! Real production window lifecycle; private XDG_CONFIG_HOME is mandatory.
#[path = "../src/image_loader.rs"]
mod image_loader;
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/input.rs"]
mod input;
#[path = "../src/window.rs"]
mod window;

use adw::prelude::*;
use gtk::{gio, glib};
use std::time::{Duration, Instant};

fn wait(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(8);
    while !condition() {
        assert!(Instant::now() < deadline, "window-state configure deadline");
        for _ in 0..100 {
            if !glib::MainContext::default().pending() {
                break;
            }
            glib::MainContext::default().iteration(false);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn record() -> String {
    std::fs::read_to_string(glib::user_config_dir().join("pivra/window.conf")).unwrap()
}

fn main() {
    assert!(
        std::env::var_os("XDG_CONFIG_HOME").is_some(),
        "use an isolated XDG_CONFIG_HOME"
    );
    let phase = std::env::args().nth(1);
    let normal_only = phase.as_deref() == Some("--normal-only");
    let external_resize = phase.as_deref() == Some("--external-resize");
    if phase.is_none() || normal_only || external_resize {
        // Every phase is a fresh application process: no cached in-process state
        // can make restart restoration pass.
        let phases: &[&str] = if normal_only {
            &["resize", "restart-normal", "fullscreen", "verify-normal"]
        } else {
            &[
                "resize",
                "maximize",
                "restore",
                "fullscreen",
                "max-fullscreen",
                "verify",
            ]
        };
        for phase in phases {
            assert!(
                std::process::Command::new(std::env::current_exe().unwrap())
                    .arg(phase)
                    .env(
                        "PIVRA_WINDOW_TEST_EXTERNAL_RESIZE",
                        if external_resize { "1" } else { "0" }
                    )
                    .status()
                    .unwrap()
                    .success(),
                "phase {phase}"
            );
        }
        if normal_only {
            println!(
                "PASS window state: process restarts, resize, fullscreen, normal-size preservation"
            );
            println!(
                "NOT RUN maximize/restore: explicitly excluded for compositors without maximize support"
            );
        } else {
            println!(
                "PASS window state: process restarts, resize, maximize/restore, fullscreen, normal-size preservation"
            );
        }
        return;
    }
    let phase = phase.unwrap();
    adw::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.onbeskarakterli.Pivra")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    window::build(&app, &pivra::settings::Settings::default(), None);
    let window = app.windows().into_iter().next().unwrap();
    wait(|| window.is_mapped() && window.width() > 0);
    // allocation includes CSS borders on X11, but excludes the external
    // Wayland shadow. Content width / raw surface width are not portable sizes.
    match phase.as_str() {
        "resize" => {
            assert_eq!(window.default_size(), (960, 640));
            if std::env::var("PIVRA_WINDOW_TEST_EXTERNAL_RESIZE").as_deref() == Ok("1") {
                assert_eq!(std::env::var("GDK_BACKEND").as_deref(), Ok("x11"));
                let result = std::process::Command::new("xdotool")
                    .args([
                        "search",
                        "--onlyvisible",
                        "--pid",
                        &std::process::id().to_string(),
                        "--name",
                        "^Pivra$",
                    ])
                    .output()
                    .unwrap();
                assert!(result.status.success());
                let ids = String::from_utf8(result.stdout).unwrap();
                let id = ids.lines().next().expect("production window XID");
                assert!(
                    std::process::Command::new("xdotool")
                        .args(["windowsize", id, "880", "620"])
                        .status()
                        .unwrap()
                        .success()
                );
            } else {
                window.set_default_size(880, 620);
            }
            wait(|| window.allocation().width() == 880 && window.allocation().height() == 620);
            assert_eq!(window.default_size(), (880, 620));
        }
        "restart-normal" => {
            assert_eq!(window.default_size(), (880, 620));
            assert!(!window.is_maximized());
            window.set_default_size(900, 660);
            wait(|| window.allocation().width() == 900 && window.allocation().height() == 660);
        }
        "verify-normal" => {
            assert!(!window.is_maximized());
            assert!(!window.is_fullscreen());
            assert_eq!(window.default_size(), (900, 660));
        }
        "maximize" => {
            assert_eq!(window.default_size(), (880, 620));
            assert!(!window.is_maximized());
            window.maximize();
            wait(|| window.is_maximized());
        }
        "restore" => {
            wait(|| window.is_maximized());
            assert_eq!(window.default_size(), (880, 620));
            window.unmaximize();
            wait(|| {
                !window.is_maximized()
                    && (window.allocation().width() == 880 && window.allocation().height() == 620)
            });
            window.set_default_size(900, 660);
            wait(|| window.allocation().width() == 900 && window.allocation().height() == 660);
        }
        "fullscreen" => {
            assert!(!window.is_maximized());
            assert_eq!(window.default_size(), (900, 660));
            window.fullscreen();
            wait(|| window.is_fullscreen());
            assert_eq!(window.default_size(), (900, 660));
            window.unfullscreen();
            wait(|| {
                !window.is_fullscreen()
                    && (window.allocation().width() == 900 && window.allocation().height() == 660)
            });
            window.fullscreen();
            wait(|| window.is_fullscreen());
        }
        "max-fullscreen" => {
            assert!(
                !window.is_fullscreen(),
                "fullscreen must not persist across launches"
            );
            assert!(!window.is_maximized());
            assert_eq!(window.default_size(), (900, 660));
            window.maximize();
            wait(|| window.is_maximized());
            window.fullscreen();
            wait(|| window.is_fullscreen());
            assert_eq!(window.default_size(), (900, 660));
        }
        "verify" => {
            wait(|| window.is_maximized());
            assert!(!window.is_fullscreen());
            assert_eq!(window.default_size(), (900, 660));
            window.unmaximize();
            wait(|| {
                !window.is_maximized()
                    && (window.allocation().width() == 900 && window.allocation().height() == 660)
            });
        }
        _ => panic!("unknown phase"),
    }
    let size = window.default_size();
    let maximized = window.is_maximized();
    window.close();
    wait(|| !window.is_visible());
    let expected = format!(
        "version=1\nwidth={}\nheight={}\nmaximized={}\n",
        size.0, size.1, maximized
    );
    assert_eq!(record(), expected);
    println!(
        "PASS {phase}: normal={}x{}, maximized={maximized}",
        size.0, size.1
    );
}
