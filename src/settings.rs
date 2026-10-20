//! In-memory defaults consumed by the window; no GSettings, disk I/O or globals.
//! Theme behavior remains inherited from GTK/libadwaita and the desktop.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub window_width: i32,
    pub window_height: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            window_width: 960,
            window_height: 640,
        }
    }
}
