use gtk::{gio, glib, prelude::*};

#[test]
fn desktop_entry_has_single_uri_exec_and_only_the_admitted_image_types() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/io.github.onbeskarakterli.Pivra.desktop");
    let key = glib::KeyFile::new();
    key.load_from_file(path, glib::KeyFileFlags::NONE).unwrap();
    assert_eq!(key.string("Desktop Entry", "Exec").unwrap(), "pivra %u");
    // GDesktopAppInfo checks executable availability. The source entry targets
    // an installed PATH binary; this test explicitly substitutes our build.
    key.set_string(
        "Desktop Entry",
        "Exec",
        &format!("\"{}\" %u", env!("CARGO_BIN_EXE_pivra")),
    );
    let info = gio::DesktopAppInfo::from_keyfile(&key).expect("parse desktop entry with GIO");
    assert!(info.supports_uris());
    let types = info.supported_types();
    assert_eq!(
        types.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        ["image/jpeg", "image/png"]
    );
    assert!(!info.boolean("DBusActivatable"));
}
