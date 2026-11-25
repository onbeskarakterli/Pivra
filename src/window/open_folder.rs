//! Existing last-open-folder persistence, shared by the Open action.
use gtk::{gio, glib};
use std::path::PathBuf;

pub(super) fn load_last_open_folder() -> Option<gio::File> {
    let path = glib::user_config_dir().join("pivra/last-open-dir.conf");
    let folder = PathBuf::from(std::fs::read_to_string(path).ok()?.trim());
    folder.is_dir().then(|| gio::File::for_path(folder))
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
