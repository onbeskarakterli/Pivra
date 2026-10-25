//! Actual app::launch/GIO-open path. No mock decoder, no automatic MIME override.
#[path = "../src/app.rs"]
mod app;
#[path = "../src/image_loader.rs"]
mod image_loader;
#[path = "../src/image_view.rs"]
mod image_view;
#[path = "../src/input.rs"]
mod input;
#[path = "../src/window.rs"]
mod window;
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};
fn descendants(w: &gtk::Widget) -> Vec<gtk::Widget> {
    let mut result = vec![w.clone()];
    let mut child = w.first_child();
    while let Some(widget) = child {
        result.extend(descendants(&widget));
        child = widget.next_sibling();
    }
    result
}
fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images");
    for (name, size) in [
        ("wide.png", (64, 32)),
        ("large.png", (1600, 1200)),
        ("no-exif.jpg", (64, 32)),
    ] {
        let path = root.join(name);
        let original = std::fs::read(&path).unwrap();
        // Exercise both the original fixture directory and URI admission.
        for uri in [false, true] {
            let argument = if uri {
                gio::File::for_path(&path).uri().to_string().into()
            } else {
                path.clone().into_os_string()
            };
            let opened = input::OpenedFile::from_argument(&argument).unwrap();
            assert_eq!(opened.file.path().as_ref(), Some(&path));
            let result = Rc::new(Cell::new(None));
            let observed = result.clone();
            let end = Instant::now() + Duration::from_secs(30);
            glib::timeout_add_local(Duration::from_millis(10), move || {
                let application = gio::Application::default().and_downcast::<adw::Application>();
                if let Some(application) = application {
                    for w in application.windows() {
                        for widget in descendants(w.upcast_ref()) {
                            if let Ok(p) = widget.downcast::<gtk::Picture>() {
                                // Thumbnail cells also contain Pictures. Only the
                                // main image is the direct child of its Overlay;
                                // never treat a 64x32 thumbnail as the opened file.
                                let is_main = p
                                    .parent()
                                    .and_downcast::<gtk::Overlay>()
                                    .and_then(|overlay| overlay.child())
                                    .is_some_and(|child| child == p.clone().upcast::<gtk::Widget>());
                                if !is_main {
                                    continue;
                                }
                                if let Some(texture) = p.paintable().and_downcast::<gdk::Texture>()
                                {
                                    assert_eq!((texture.width(), texture.height()), size);
                                    let mut pixels = vec![0; (size.0 * size.1 * 4) as usize];
                                    texture.download(&mut pixels, size.0 as usize * 4);
                                    assert!(pixels.chunks_exact(4).any(|p| p[3] != 0));
                                    observed.set(Some(0));
                                    w.close();
                                    return glib::ControlFlow::Break;
                                }
                                if let Some(label) = p.next_sibling().and_downcast::<gtk::Label>() {
                                    if label.text().starts_with("Unable") {
                                        let message = label.text();
                                        eprintln!("FAIL normal application entry: {message}");
                                        observed.set(Some(
                                            if message.contains("Loader process exited early") {
                                                77
                                            } else {
                                                1
                                            },
                                        ));
                                        w.close();
                                        return glib::ControlFlow::Break;
                                    }
                                }
                            }
                        }
                    }
                }
                assert!(Instant::now() < end, "normal application open timeout");
                glib::ControlFlow::Continue
            });
            let status = app::launch(Some(opened));
            assert_eq!(status, glib::ExitCode::SUCCESS);
            let code = result
                .get()
                .expect("application exited before image observation");
            if code != 0 {
                std::process::exit(code);
            }
            assert_eq!(std::fs::read(&path).unwrap(), original);
            println!(
                "PASS normal app entry {name}, URI={uri}: dimensions, nonempty rendered texture, source integrity"
            );
        }
    }
}
