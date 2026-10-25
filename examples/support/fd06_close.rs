//! Native thumbnail (not foreground) close. External pidfd partial-frame certificate required.
use super::*;
use std::{
    io::{self, Write},
    path::Path,
    sync::mpsc,
};
fn until(mut condition: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(30);
    while !condition() {
        assert!(Instant::now() < end, "native window setup deadline");
        pump();
    }
}
fn build(app: &adw::Application, path: &Path) -> adw::ApplicationWindow {
    let before = app.windows();
    window::build(
        app,
        &pivra::settings::Settings::default(),
        Some(input::OpenedFile::from_argument(path.as_os_str()).unwrap()),
    );
    app.windows()
        .into_iter()
        .find(|w| !before.contains(w))
        .unwrap()
        .downcast()
        .unwrap()
}
fn photo(w: &adw::ApplicationWindow) -> gtk::Picture {
    let strip: gtk::ScrolledWindow = find(w.upcast_ref(), "thumbnail-strip");
    strip
        .parent()
        .unwrap()
        .downcast::<gtk::Overlay>()
        .unwrap()
        .child()
        .unwrap()
        .downcast()
        .unwrap()
}
fn reveal(w: &adw::ApplicationWindow) {
    let strip: gtk::ScrolledWindow = find(w.upcast_ref(), "thumbnail-strip");
    let overlay = strip.parent().unwrap().downcast::<gtk::Overlay>().unwrap();
    let motion = controller::<gtk::EventControllerMotion>(&overlay);
    overlay.remove_controller(&motion);
    motion.emit_by_name::<()>("motion", &[&20f64, &(overlay.height() as f64 - 4.)]);
}
pub(super) fn run(app: &adw::Application) {
    assert!(std::env::args().any(|s| s == "--native-thumbnails"));
    assert_eq!(
        pivra::thumbnail_manager::Manager::session().id(),
        pivra::thumbnail_provider::ProviderId::Freedesktop
    );
    let args: Vec<_> = std::env::args_os().collect();
    let at = args
        .iter()
        .position(|s| s == "--fd06-active-close")
        .unwrap();
    let source = std::path::PathBuf::from(args.get(at + 1).expect("private main source"));
    let big = source.parent().unwrap().join("02-large.png");
    let key = pivra::image_cache::FileKey::read(&big).unwrap();
    let other = tempfile::tempdir().unwrap();
    let other_source = other.path().join("other.png");
    std::fs::write(
        &other_source,
        include_bytes!("../../tests/fixtures/images/wide.png"),
    )
    .unwrap();
    pivra::settings::thumbnail_preference().set_enabled(false);
    let first = build(app, &source);
    let second = build(app, &other_source);
    let first_photo = photo(&first);
    let second_photo = photo(&second);
    until(|| {
        first_photo.paintable().is_some()
            && second_photo.paintable().is_some()
            && first.is_mapped()
            && second.is_mapped()
    });
    let second_pixels = image_bytes(&second, &second_photo);
    let title = second.title();
    let weak = first.downgrade();
    let weak_photo = first_photo.downgrade();
    drop(first_photo);
    pivra::settings::thumbnail_preference().set_enabled(true);
    reveal(&first);
    reveal(&second);
    let (send, receive) = mpsc::channel();
    std::thread::spawn(move || {
        loop {
            let mut line = String::new();
            let n = io::stdin().read_line(&mut line).unwrap();
            if n == 0 {
                let _ = send.send("eof".to_owned());
                break;
            }
            if send.send(line.trim().to_owned()).is_err() {
                break;
            }
        }
    });
    let loop_ = glib::MainLoop::new(None, false);
    let quit = loop_.clone();
    let mut first = Some(first);
    let mut ready = false;
    let end = Instant::now() + Duration::from_secs(95);
    println!("READY native thumbnail window close; requires external partial-frame checkpoint");
    io::stdout().flush().unwrap();
    glib::timeout_add_local(Duration::from_millis(5), move || {
        assert!(Instant::now() < end, "native close deadline");
        if let Ok(command) = receive.try_recv() {
            match command.as_str() {
                "cancel" => {
                    first.take().expect("single close").close();
                    println!("CANCEL_ISSUED");
                    io::stdout().flush().unwrap();
                }
                "finish" => {
                    assert!(ready);
                    assert!(second.is_visible());
                    second.close();
                    println!(
                        "PASS CANCELLED future_completed thumbnail_window_closed other_window_stable source_unchanged=true"
                    );
                    quit.quit();
                    return glib::ControlFlow::Break;
                }
                "eof" => {
                    eprintln!("NOT RUN no certified active thumbnail close checkpoint");
                    std::process::exit(77);
                }
                _ => panic!("unknown test command"),
            }
        }
        if first.is_none()
            && !ready
            && weak.upgrade().is_none()
            && weak_photo.upgrade().is_none()
            && pivra::thumbnail_manager::Permit::active_count() == 0
        {
            assert_eq!(
                pivra::freedesktop_generator::reservation_state(),
                (0, false)
            );
            assert!(second.is_visible());
            assert_eq!(second.title(), title);
            assert_eq!(image_bytes(&second, &second_photo), second_pixels);
            assert_eq!(pivra::image_cache::FileKey::read(&big).unwrap(), key);
            ready = true;
            println!("WINDOW_CLOSED_OTHER_ALIVE");
            io::stdout().flush().unwrap();
        }
        glib::ControlFlow::Continue
    });
    loop_.run();
    drop(other);
}
