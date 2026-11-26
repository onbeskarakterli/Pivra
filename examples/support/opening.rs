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

pub(super) fn popup(window: &adw::ApplicationWindow) -> gtk::Popover {
    wait("opening menu owner active", || {
        window
            .surface()
            .and_downcast::<gtk::gdk::Toplevel>()
            .is_some_and(|s| s.state().contains(gtk::gdk::ToplevelState::FOCUSED))
    });
    let picture = widgets(window.upcast_ref())
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Picture>().ok())
        .find(|p| p.widget_name() != "video-content")
        .unwrap();
    let click = picture
        .observe_controllers()
        .iter::<glib::Object>()
        .filter_map(Result::ok)
        .filter_map(|c| c.downcast::<gtk::GestureClick>().ok())
        .find(|c| c.button() == 3)
        .unwrap();
    click.emit_by_name::<()>("pressed", &[&1i32, &30f64, &30f64]);
    let menu = widgets(picture.upcast_ref())
        .into_iter()
        .find(|w| w.widget_name() == "media-context-menu")
        .unwrap()
        .downcast::<gtk::Popover>()
        .unwrap();
    wait("opening menu mapped", || menu.is_mapped());
    menu
}

pub(super) fn reveal(app: &adw::Application) {
    use std::cell::RefCell;
    let main = glib::MainContext::default();
    let bus = main
        .block_on(gio::bus_get_future(gio::BusType::Session))
        .unwrap();
    let reply = main
        .block_on(bus.call_future(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "RequestName",
            Some(&("org.freedesktop.FileManager1", 0u32).to_variant()),
            None,
            gio::DBusCallFlags::NONE,
            2000,
        ))
        .unwrap();
    assert_eq!(
        reply.get::<(u32,)>(),
        Some((1,)),
        "private FileManager1 name"
    );
    let info = gio::DBusNodeInfo::for_xml(
        "<node><interface name='org.freedesktop.FileManager1'><method name='ShowItems'><arg type='as' direction='in'/><arg type='s' direction='in'/></method></interface></node>"
    ).unwrap();
    let invocation = Rc::new(RefCell::new(None::<gio::DBusMethodInvocation>));
    let args = Rc::new(RefCell::new(Vec::<String>::new()));
    let held = invocation.clone();
    let received = args.clone();
    let registration = bus
        .register_object("/org/freedesktop/FileManager1", &info.interfaces()[0])
        .method_call(move |_, _, _, _, name, params, call| {
            assert_eq!(name, "ShowItems");
            let (uris, startup): (Vec<String>, String) = params.get().unwrap();
            assert!(startup.is_empty());
            *received.borrow_mut() = uris;
            assert!(
                held.borrow_mut().replace(call).is_none(),
                "duplicate request"
            );
        })
        .build()
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Å current photo.png");
    std::fs::copy("tests/fixtures/images/wide.png", &path).unwrap();
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
    let menu = popup(&window);
    let row = widgets(menu.upcast_ref())
        .into_iter()
        .find(|w| w.widget_name() == "context-action-show-in-files")
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap();
    let action = window.lookup_action("show-in-files").unwrap();
    let pulses = Rc::new(Cell::new(0));
    let count = pulses.clone();
    let heartbeat = glib::timeout_add_local(Duration::from_millis(10), move || {
        count.set(count.get() + 1);
        glib::ControlFlow::Continue
    });
    let start = Instant::now();
    row.emit_clicked();
    let returned = start.elapsed().as_micros();
    assert!(!menu.is_visible(), "popup grab survived action dispatch");
    assert!(!action.is_enabled());
    action.activate(None);
    wait("mock file manager receives request", || {
        invocation.borrow().is_some()
    });
    assert_eq!(
        *args.borrow(),
        vec![gio::File::for_path(&path).uri().to_string()],
        "reveal current file, not parent directory"
    );
    gio::prelude::ActionGroupExt::activate_action(&window, "media-info", None);
    assert!(
        state(&window, "media-info"),
        "UI action blocked by FileManager1 wait"
    );
    wait("delayed desktop reply with responsive UI", || {
        start.elapsed() >= Duration::from_millis(250) && pulses.get() >= 5
    });
    assert!(!action.is_enabled(), "pending desktop reply lost admission");
    invocation
        .borrow_mut()
        .take()
        .unwrap()
        .return_value(Some(&().to_variant()));
    wait("async show-items completion", || action.is_enabled());
    assert_eq!(window.title(), title);
    action.activate(None);
    wait("second reveal request", || invocation.borrow().is_some());
    invocation.borrow_mut().take().unwrap().return_dbus_error(
        "org.freedesktop.DBus.Error.NotSupported",
        "controlled unavailable file selection",
    );
    wait("file manager error handled", || action.is_enabled());
    assert!(window.is_visible());
    action.activate(None);
    wait("close with pending file manager", || {
        invocation.borrow().is_some()
    });
    window.close();
    invocation
        .borrow_mut()
        .take()
        .unwrap()
        .return_value(Some(&().to_variant()));
    pump();
    assert!(!window.is_visible());
    heartbeat.remove();
    bus.unregister_object(registration).unwrap();
    eprintln!(
        "PASS reveal: exact Unicode current URI, popup handoff, single cancellable request, delayed service/UI pulses={}, error and close cleanup; action returned in {returned}us; controlled FileManager1, native Nautilus pending",
        pulses.get()
    );
}
