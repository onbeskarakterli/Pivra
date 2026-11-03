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
    // Select GL before native threads exist; respect an explicit renderer override.
    if cfg!(pivra_gpu_backend)
        && std::env::var_os("GSK_RENDERER").is_none()
        && std::env::var("PIVRA_VIDEO_BACKEND").as_deref() != Ok("stock")
    {
        use std::os::unix::process::CommandExt;
        match std::env::current_exe() {
            Ok(exe) => {
                let error = std::process::Command::new(exe)
                    .args(std::env::args_os().skip(1))
                    .env(
                        "GSK_RENDERER",
                        if gtk::minor_version() < 20 {
                            "ngl"
                        } else {
                            "gl"
                        },
                    )
                    .exec();
                eprintln!("Pivra: GL re-exec failed: {error}; stock video retained");
            }
            Err(error) => eprintln!("Pivra: renderer selection unavailable: {error}"),
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
