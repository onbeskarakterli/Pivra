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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CloseSnapshot {
    close_requested: bool,
    unmapped: bool,
    destroyed: bool,
    surface_destroyed: bool,
    visible: bool,
    mapped: bool,
    window_referenced: bool,
    photo_referenced: bool,
    workers: usize,
    reserved: usize,
    large: bool,
}
impl CloseSnapshot {
    fn clean(&self) -> bool {
        self.close_requested
            && self.unmapped
            && self.surface_destroyed
            && !self.visible
            && !self.mapped
            && self.workers == 0
            && self.reserved == 0
            && !self.large
    }
}
#[derive(Default)]
struct CloseEvents {
    requested: std::cell::Cell<bool>,
    unmapped: std::cell::Cell<bool>,
    destroyed: std::cell::Cell<bool>,
}
fn watch(window: &adw::ApplicationWindow) -> std::rc::Rc<CloseEvents> {
    let events = std::rc::Rc::new(CloseEvents::default());
    let observed = events.clone();
    window.connect_close_request(move |_| {
        observed.requested.set(true);
        println!("CLOSE_EVENT close-request");
        glib::Propagation::Proceed
    });
    let observed = events.clone();
    window.connect_unmap(move |_| {
        observed.unmapped.set(true);
        println!("CLOSE_EVENT unmap");
    });
    let observed = events.clone();
    window.connect_destroy(move |_| {
        observed.destroyed.set(true);
        println!("CLOSE_EVENT destroy");
    });
    events
}
fn snapshot(
    events: &CloseEvents,
    window: &glib::WeakRef<adw::ApplicationWindow>,
    picture: &glib::WeakRef<gtk::Picture>,
    surface: &gdk::Surface,
) -> CloseSnapshot {
    let w = window.upgrade();
    let (reserved, large) = pivra::freedesktop_generator::reservation_state();
    CloseSnapshot {
        close_requested: events.requested.get(),
        unmapped: events.unmapped.get(),
        destroyed: events.destroyed.get(),
        surface_destroyed: surface.is_destroyed(),
        visible: w.as_ref().is_some_and(|w| w.is_visible()),
        mapped: w.as_ref().is_some_and(|w| w.is_mapped()),
        window_referenced: w.is_some(),
        photo_referenced: picture.upgrade().is_some(),
        workers: pivra::thumbnail_manager::Permit::active_count(),
        reserved,
        large,
    }
}
fn flush() {
    let _ = io::stdout().flush();
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
    let events = watch(&first);
    let surface = first.surface().expect("mapped first window surface");
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
            let command = match io::stdin().read_line(&mut line) {
                Ok(0) => "eof".to_owned(),
                Ok(_) => line.trim().to_owned(),
                Err(_) => "stdin-error".to_owned(),
            };
            let done = command == "eof" || command == "stdin-error";
            if send.send(command).is_err() || done {
                break;
            }
        }
    });
    let loop_ = glib::MainLoop::new(None, false);
    let quit = loop_.clone();
    // Callback records protocol/cleanup only. Render validation and exit run outside GLib.
    let result = std::rc::Rc::new(std::cell::RefCell::new(None));
    let outcome = result.clone();
    let observer_window = second.clone();
    let mut first = Some(first);
    let mut ready = false;
    let mut last = None;
    let end = Instant::now() + Duration::from_secs(95);
    println!(
        "READY native thumbnail window close; requires external partial-frame checkpoint; DO NOT close windows manually"
    );
    flush();
    glib::timeout_add_local(Duration::from_millis(5), move || {
        let state = snapshot(&events, &weak, &weak_photo, &surface);
        if last != Some(state) {
            println!("CLOSE_STATE {state:?}");
            flush();
            last = Some(state);
        }
        let mut completed = None;
        if !observer_window.is_visible() || !observer_window.is_mapped() {
            completed = Some(Err((
                1,
                "other window closed before certified finish".to_owned(),
            )));
        } else if state.close_requested && first.is_some() {
            completed = Some(Err((
                1,
                "first window closed before certified cancel command".to_owned(),
            )));
        } else if Instant::now() >= end {
            completed = Some(Err((
                1,
                format!(
                    "native close deadline: {state:?}; ready={ready}; waiting_for={}",
                    if ready {
                        "observer loader-exit/finish acknowledgement"
                    } else {
                        "window close events / worker / reservation cleanup"
                    }
                ),
            )));
        } else if let Ok(command) = receive.try_recv() {
            match command.as_str() {
                "cancel" => {
                    if let Some(window) = first.take() {
                        window.close();
                        println!("CANCEL_ISSUED");
                        flush();
                    } else {
                        completed = Some(Err((1, "duplicate cancel command".to_owned())));
                    }
                }
                "finish" if ready && state.clean() => completed = Some(Ok(())),
                "finish" => {
                    completed = Some(Err((
                        1,
                        format!("finish before window/worker cleanup: {state:?}"),
                    )))
                }
                "eof" if first.is_some() => {
                    completed = Some(Err((
                        77,
                        "no certified active thumbnail close checkpoint".to_owned(),
                    )))
                }
                "eof" => {
                    completed = Some(Err((
                        1,
                        "observer disconnected after cancel without loader-exit certificate"
                            .to_owned(),
                    )))
                }
                _ => completed = Some(Err((1, format!("invalid test command: {command}")))),
            }
        }
        if let Some(value) = completed {
            *outcome.borrow_mut() = Some(value);
            quit.quit();
            return glib::ControlFlow::Break;
        }
        if first.is_none() && !ready && state.clean() {
            ready = true;
            println!("WINDOW_CLOSED_OTHER_ALIVE");
            flush();
        }
        glib::ControlFlow::Continue
    });
    loop_.run();
    let outcome = result
        .borrow_mut()
        .take()
        .unwrap_or_else(|| Err((1, "main loop ended without protocol result".to_owned())));
    let outcome = outcome.and_then(|()| {
        if !second.is_visible() || !second.is_mapped() || second.title() != title {
            return Err((1, "other window visibility/title changed".to_owned()));
        }
        println!(
            "PASS other window visible/mapped/title unchanged after loader-exit acknowledgement"
        );
        if image_bytes(&second, &second_photo) != second_pixels {
            return Err((1, "other window render pixels changed".to_owned()));
        }
        println!("PASS other window render pixels unchanged");
        if pivra::image_cache::FileKey::read(&big).ok().as_ref() != Some(&key) {
            return Err((1, "large source identity changed".to_owned()));
        }
        if pivra::thumbnail_manager::Permit::active_count() != 0
            || pivra::freedesktop_generator::reservation_state() != (0, false)
        {
            return Err((
                1,
                "resources reacquired after cleanup acknowledgement".to_owned(),
            ));
        }
        println!(
            "PASS source identity and zero workers/reservations after final render verification"
        );
        Ok(())
    });
    second.close();
    drop(other);
    match outcome {
        Ok(()) => println!(
            "PASS CANCELLED future_completed thumbnail_window_closed other_window_stable source_unchanged=true"
        ),
        Err((code, reason)) => {
            eprintln!(
                "{} FD06 active close: {reason}",
                if code == 77 { "NOT RUN" } else { "FAIL" }
            );
            std::process::exit(code);
        }
    }
}

// Real GTK close with deliberately retained GObjects, but no decoder. This proves
// finalization is not a window-close/worker-completion signal.
pub(super) fn harness_regression(app: &adw::Application) {
    pivra::settings::thumbnail_preference().set_enabled(false);
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("private.png");
    std::fs::write(
        &source,
        include_bytes!("../../tests/fixtures/images/wide.png"),
    )
    .unwrap();
    let window = build(app, &source);
    let picture = photo(&window);
    until(|| window.is_mapped() && picture.paintable().is_some());
    let events = watch(&window);
    let surface = window.surface().expect("mapped regression surface");
    let weak = window.downgrade();
    let weak_photo = picture.downgrade();
    window.close();
    until(|| snapshot(&events, &weak, &weak_photo, &surface).clean());
    let state = snapshot(&events, &weak, &weak_photo, &surface);
    assert!(state.window_referenced && state.photo_referenced);
    assert!(state.clean());
    println!(
        "PASS controlled GTK close with retained window/photo references: {state:?}; no active decoder claim"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    fn closed() -> CloseSnapshot {
        CloseSnapshot {
            close_requested: true,
            unmapped: true,
            surface_destroyed: true,
            window_referenced: true,
            photo_referenced: true,
            ..Default::default()
        }
    }
    #[test]
    fn retained_references_are_not_pending_decoder_work() {
        let state = closed();
        assert!(
            !state.destroyed,
            "Widget destroy may follow final reference release"
        );
        assert!(state.clean());
    }
    #[test]
    fn every_close_and_cleanup_condition_is_required() {
        for state in [
            CloseSnapshot {
                close_requested: false,
                ..closed()
            },
            CloseSnapshot {
                unmapped: false,
                ..closed()
            },
            CloseSnapshot {
                surface_destroyed: false,
                ..closed()
            },
            CloseSnapshot {
                visible: true,
                ..closed()
            },
            CloseSnapshot {
                mapped: true,
                ..closed()
            },
            CloseSnapshot {
                workers: 1,
                ..closed()
            },
            CloseSnapshot {
                reserved: 1,
                ..closed()
            },
            CloseSnapshot {
                large: true,
                ..closed()
            },
        ] {
            assert!(!state.clean(), "{state:?}");
        }
    }
}
