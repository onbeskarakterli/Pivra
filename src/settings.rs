//! Window defaults and the shared, optional persistent thumbnail preference.
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

// Only the tiny thumbnail preference is persisted. No schema installation or
// new dependency is required by native source builds. UI visibility is not saved.
use gtk::{gio, glib, prelude::*};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::{Rc, Weak},
};
type Listener = dyn Fn(bool);
pub struct ThumbnailPreference {
    enabled: Cell<bool>,
    path: PathBuf,
    listeners: RefCell<Vec<Weak<Listener>>>,
}
impl ThumbnailPreference {
    fn load(path: PathBuf) -> Self {
        use std::io::Read;
        let mut bytes = Vec::new();
        let enabled = std::fs::File::open(&path)
            .and_then(|file| file.take(32).read_to_end(&mut bytes))
            .is_ok()
            && bytes == b"enabled=true\n";
        Self {
            enabled: Cell::new(enabled),
            path,
            listeners: RefCell::new(Vec::new()),
        }
    }
    pub fn enabled(&self) -> bool {
        self.enabled.get()
    }
    pub fn subscribe(&self, listener: &Rc<Listener>) {
        let mut listeners = self.listeners.borrow_mut();
        listeners.retain(|l| l.strong_count() > 0);
        listeners.push(Rc::downgrade(listener));
    }
    pub fn set_enabled(&self, enabled: bool) {
        if self.enabled.replace(enabled) == enabled {
            return;
        }
        if let Err(error) = self.save(enabled) {
            eprintln!("Pivra: could not save thumbnail preference: {error}");
        }
        // Release the list borrow before callbacks; expired windows are pruned.
        let listeners: Vec<_> = self
            .listeners
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .collect();
        self.listeners.borrow_mut().retain(|l| l.strong_count() > 0);
        for listener in listeners {
            listener(enabled);
        }
    }
    fn save(&self, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(self.path.parent().ok_or("missing settings directory")?)?;
        let contents = if enabled {
            b"enabled=true\n".as_slice()
        } else {
            b"enabled=false\n".as_slice()
        };
        gio::File::for_path(&self.path).replace_contents(
            contents,
            None,
            false,
            gio::FileCreateFlags::PRIVATE | gio::FileCreateFlags::REPLACE_DESTINATION,
            None::<&gio::Cancellable>,
        )?;
        Ok(())
    }
}
thread_local! {
    static THUMBNAILS: Rc<ThumbnailPreference> = Rc::new(ThumbnailPreference::load(
        glib::user_config_dir().join("pivra").join("thumbnails.conf")));
}
pub fn thumbnail_preference() -> Rc<ThumbnailPreference> {
    THUMBNAILS.with(Rc::clone)
}

/// Persistent thumbnail strip height, independent from whether it is visible.
pub fn load_thumbnail_height() -> i32 {
    let path = glib::user_config_dir().join("pivra/thumbnail-height.conf");
    std::fs::read_to_string(path).ok()
        .and_then(|value| value.trim().parse::<i32>().ok())
        .filter(|&height| (128..=560).contains(&height))
        .unwrap_or(128)
}

pub fn save_thumbnail_height(height: i32) {
    let dir = glib::user_config_dir().join("pivra");
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&dir)?;
        gio::File::for_path(dir.join("thumbnail-height.conf")).replace_contents(
            format!("{}\n", height.clamp(128, 560)).as_bytes(),
            None, false,
            gio::FileCreateFlags::PRIVATE | gio::FileCreateFlags::REPLACE_DESTINATION,
            None::<&gio::Cancellable>,
        )?;
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("Pivra: could not save thumbnail height: {error}");
    }
}

/// Pin state is distinct from the thumbnail enabled preference.
pub fn load_thumbnail_pinned() -> bool {
    let path = glib::user_config_dir().join("pivra/thumbnail-pinned.conf");
    std::fs::read(path).ok().as_deref() == Some(b"true\n")
}

pub fn save_thumbnail_pinned(pinned: bool) {
    let dir = glib::user_config_dir().join("pivra");
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&dir)?;
        let bytes: &[u8] = if pinned { b"true\n" } else { b"false\n" };
        gio::File::for_path(dir.join("thumbnail-pinned.conf")).replace_contents(
            bytes, None, false,
            gio::FileCreateFlags::PRIVATE | gio::FileCreateFlags::REPLACE_DESTINATION,
            None::<&gio::Cancellable>,
        )?;
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("Pivra: could not save thumbnail pin state: {error}");
    }
}

/// Remember the last chosen Fit / Fill / 100% preset between launches.
pub fn load_view_mode() -> crate::viewing::ViewMode {
    use std::io::Read;
    let path = glib::user_config_dir().join("pivra/view-mode.conf");
    let mut content = Vec::new();
    if std::fs::File::open(path)
        .and_then(|f| f.take(32).read_to_end(&mut content))
        .is_ok()
    {
        match content.as_slice() {
            b"fill\n" => return crate::viewing::ViewMode::Fill,
            b"actual\n" => return crate::viewing::ViewMode::Actual,
            _ => {}
        }
    }
    crate::viewing::ViewMode::Fit
}

pub fn save_view_mode(mode: crate::viewing::ViewMode) {
    let dir = glib::user_config_dir().join("pivra");
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&dir)?;
        let value: &[u8] = match mode {
            crate::viewing::ViewMode::Fit => b"fit\n",
            crate::viewing::ViewMode::Fill => b"fill\n",
            crate::viewing::ViewMode::Actual => b"actual\n",
        };
        gio::File::for_path(dir.join("view-mode.conf")).replace_contents(
            value,
            None,
            false,
            gio::FileCreateFlags::PRIVATE | gio::FileCreateFlags::REPLACE_DESTINATION,
            None::<&gio::Cancellable>,
        )?;
        Ok(())
    })();
    if let Err(err) = result {
        eprintln!("Pivra: could not save view mode: {err}");
    }
}

#[cfg(test)]
mod thumbnail_tests {
    use super::*;
    #[test]
    fn default_persist_restart_and_corruption() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("pivra/thumbnails.conf");
        let p = ThumbnailPreference::load(path.clone());
        assert!(!p.enabled());
        p.set_enabled(true);
        assert!(ThumbnailPreference::load(path.clone()).enabled());
        p.set_enabled(false);
        assert!(!ThumbnailPreference::load(path.clone()).enabled());
        std::fs::write(&path, b"invalid or partial data").unwrap();
        assert!(!ThumbnailPreference::load(path).enabled());
    }
    #[test]
    fn shared_listeners_and_unwritable_storage_are_nonfatal() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("not-a-directory");
        std::fs::write(&path, b"file").unwrap();
        let p = ThumbnailPreference::load(path.join("setting"));
        let observed = Rc::new(Cell::new(false));
        let c = observed.clone();
        let listener: Rc<Listener> = Rc::new(move |value| c.set(value));
        p.subscribe(&listener);
        p.set_enabled(true);
        assert!(p.enabled() && observed.get());
        drop(listener);
        p.set_enabled(false);
        assert!(p.listeners.borrow().is_empty());
    }
}
