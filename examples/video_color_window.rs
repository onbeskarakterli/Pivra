//! Production application/window/media code with a test-only, explicit GIO loader.
//! No installed Pivra binary, global GTK configuration or default backend change.
#[path = "../src/app.rs"]
mod app;
#[path = "../src/image_loader.rs"]
mod image_loader;
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/input.rs"]
mod input;
#[path = "../src/raw_loader.rs"]
mod raw_loader;
#[path = "../src/raw_ppm.rs"]
mod raw_ppm;
#[path = "support/video_backend.rs"]
mod video_backend;
#[path = "../src/window.rs"]
mod window;
use gtk::prelude::*;

fn main() {
    let Some(argument) = std::env::args_os().nth(1) else {
        eprintln!("Usage: video_color_window /absolute/path/to/video");
        std::process::exit(2);
    };
    let opened = input::OpenedFile::from_argument(&argument).unwrap_or_else(|error| {
        eprintln!("FAIL: {error}");
        std::process::exit(2);
    });
    if let Err(error) = adw::init() {
        eprintln!("NOT RUN GTK: {error}");
        std::process::exit(77);
    }
    let expected = video_backend::load().unwrap_or_else(|error| {
        eprintln!("NOT RUN backend loading: {error}");
        std::process::exit(77);
    });
    if expected.is_none() {
        eprintln!("NOT RUN: explicit test module required");
        std::process::exit(77);
    }
    // Verify the actual factory-selected implementation before launching UI.
    // This empty object starts no decoder; normal window code opens the video.
    let sentinel = gtk::MediaFile::new();
    if let Err(error) = video_backend::verify(&sentinel, expected) {
        eprintln!("NOT RUN: {error}");
        std::process::exit(77);
    }
    sentinel.clear();
    drop(sentinel);
    std::process::exit(app::launch(Some(opened)).into());
}
