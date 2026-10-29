//! Configuration discovery only: never starts a loader or weakens its sandbox.
use gtk::glib;

fn main() {
    assert_eq!(glycin::COMPAT_VERSION, 2);
    let data = std::env::var_os("GLYCIN_DATA_DIR")
        .expect("activate the isolated PIV-09 image environment");
    let config =
        std::path::PathBuf::from(data).join("glycin-loaders/2+/conf.d/pivra-jpeg-png.conf");
    assert!(config.is_file(), "missing isolated loader config");
    let key = glib::KeyFile::new();
    key.load_from_file(&config, glib::KeyFileFlags::NONE)
        .expect("parse loader config");
    assert_eq!(
        key.groups().iter().map(|g| g.as_str()).collect::<Vec<_>>(),
        ["loader:image/jpeg", "loader:image/png"]
    );
    for group in key.groups() {
        let executable = key.string(&group, "Exec").expect("loader executable");
        let path = std::path::Path::new(executable.as_str());
        let metadata = std::fs::metadata(path).expect("installed loader executable");
        use std::os::unix::fs::PermissionsExt;
        assert!(metadata.is_file() && metadata.permissions().mode() & 0o111 != 0);
    }
    let mut types: Vec<_> = glib::MainContext::new()
        .block_on(glycin::Loader::supported_mime_types())
        .into_iter()
        .map(|mime| mime.to_string())
        .collect();
    types.sort();
    assert_eq!(types, ["image/jpeg", "image/png"]);
    println!(
        "PASS client 2+ configuration discovery: {types:?}; loader handshake/decode NOT TESTED"
    );
}
