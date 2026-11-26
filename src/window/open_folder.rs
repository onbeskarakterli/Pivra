//! Existing last-open-folder persistence, shared by the Open action.
use gtk::glib;
use std::path::PathBuf;

pub(super) fn read_last_open_folder() -> Option<PathBuf> {
    let path = glib::user_config_dir().join("pivra/last-open-dir.conf");
    use std::io::Read;
    let mut text = String::new();
    std::fs::File::open(path)
        .ok()?
        .take(16 * 1024)
        .read_to_string(&mut text)
        .ok()?;
    let folder = PathBuf::from(text.trim());
    folder.is_dir().then_some(folder)
}

pub(super) fn save_last_open_folder(folder: &std::path::Path) {
    let config = glib::user_config_dir().join("pivra");
    if let Err(error) = std::fs::create_dir_all(&config).and_then(|_| {
        std::fs::write(
            config.join("last-open-dir.conf"),
            folder.as_os_str().as_encoded_bytes(),
        )
    }) {
        eprintln!("Pivra: could not save last open folder: {error}");
    }
}
