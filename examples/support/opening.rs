//! Native chooser cancellation and a private installed GIO application fixture.
use super::*;
use adw::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::{cell::Cell, rc::Rc};
fn dialog() -> adw::MessageDialog {
    gtk::Window::list_toplevels()
        .into_iter()
        .find(|w| w.widget_name() == "open-with-dialog" && w.is_visible())
        .unwrap()
        .downcast()
        .unwrap()
}
pub(super) fn run(app: &adw::Application) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Å photo.png");
    std::fs::copy("tests/fixtures/images/wide.png", &path).unwrap();
    let output = dir.path().join("launched.txt");
    let script = dir.path().join("application");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n",
            output.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
    let apps =
        std::path::PathBuf::from(std::env::var_os("XDG_DATA_HOME").unwrap()).join("applications");
    std::fs::create_dir_all(&apps).unwrap();
    let desktop = apps.join("pivra-opening-probe.desktop");
    std::fs::write(&desktop,format!("[Desktop Entry]\nType=Application\nName=Pivra Opening Probe\nExec={} %u\nMimeType=image/png;\nNoDisplay=false\n",script.display())).unwrap();
    window::build(
        app,
        &pivra::settings::Settings::default(),
        Some(input::OpenedFile::from_file(&gio::File::for_path(&path)).unwrap()),
    );
    let window = app
        .active_window()
        .unwrap()
        .downcast::<adw::ApplicationWindow>()
        .unwrap();
    photo_presented(&window);
    let title = window.title();
    let action = window.lookup_action("open-with").unwrap();
    let choose = window.lookup_action("choose-media").unwrap();
    let pulses = Rc::new(Cell::new(0));
    let count = pulses.clone();
    let heartbeat = glib::timeout_add_local(Duration::from_millis(10), move || {
        count.set(count.get() + 1);
        glib::ControlFlow::Continue
    });
    let start = Instant::now();
    action.activate(None);
    let call_us = start.elapsed().as_micros();
    assert!(!action.is_enabled() && !choose.is_enabled());
    let first = dialog();
    action.activate(None);
    choose.activate(None);
    assert_eq!(
        dialog(),
        first,
        "overlapping dialog request escaped admission"
    );
    let selector = widgets(first.upcast_ref())
        .into_iter()
        .find(|w| w.widget_name() == "open-with-applications")
        .unwrap()
        .downcast::<gtk::DropDown>()
        .unwrap();
    wait("private installed application appears", || {
        selector.model().is_some_and(|m| m.n_items() > 0)
    });
    let model = selector.model().unwrap();
    let index = (0..model.n_items())
        .find(|&i| {
            model
                .item(i)
                .and_downcast::<gtk::StringObject>()
                .is_some_and(|s| s.string() == "Pivra Opening Probe")
        })
        .expect("GIO installed app fixture");
    selector.set_selected(index);
    first.response("open");
    wait("GIO application launch completes", || {
        action.is_enabled() && choose.is_enabled() && output.is_file()
    });
    let argument = std::fs::read_to_string(&output).unwrap();
    // GIO may pass a native filename for a local %u URI. Certify the actual
    // file identity and quoting, rather than requiring one equivalent encoding.
    assert!(gio::File::for_commandline_arg(argument.trim()).equal(&gio::File::for_path(&path)));
    assert_eq!(
        window.title(),
        title,
        "external opening changed viewer selection"
    );
    action.activate(None);
    let cancelled = dialog();
    cancelled.response("cancel");
    wait("application chooser cancel", || {
        action.is_enabled() && choose.is_enabled()
    });
    assert_eq!(window.title(), title);
    assert!(pulses.get() > 0, "GTK main context made no progress");
    heartbeat.remove();
    std::fs::remove_file(desktop).unwrap();
    window.close();
    eprintln!(
        "PASS opening: native installed-app selection/URI launch, overlap admission, cancel, same viewer, GTK pulses={}; first action returned in {call_us}us; private app fixture, physical GNOME pending",
        pulses.get()
    );
}
