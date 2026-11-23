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
    // Prefer XWayland until the GTK EGL teardown issue is fixed.
    // An explicit GDK_BACKEND (including "wayland") always takes precedence.
    let prefer_x11 = std::env::var_os("GDK_BACKEND").is_none()
        && std::env::var_os("DISPLAY").is_some();
    let select_gl = cfg!(pivra_gpu_backend)
        && std::env::var_os("GSK_RENDERER").is_none()
        && std::env::var("PIVRA_VIDEO_BACKEND").as_deref() != Ok("stock");
    if prefer_x11 || select_gl {
        use std::os::unix::process::CommandExt;
        match std::env::current_exe() {
            Ok(exe) => {
                let mut command = std::process::Command::new(exe);
                command.args(std::env::args_os().skip(1));
                if prefer_x11 {
                    command.env("GDK_BACKEND", "x11");
                }
                if select_gl {
                    command.env(
                        "GSK_RENDERER",
                        if gtk::minor_version() < 20 {
                            "ngl"
                        } else {
                            "gl"
                        },
                    );
                }
                let error = command.exec();
                eprintln!("Pivra: backend re-exec failed: {error}");
            }
            Err(error) => eprintln!("Pivra: backend selection unavailable: {error}"),
        }
    }
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
