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

// Small preferences are persisted. No schema installation or
// new dependency is required by native source builds. UI visibility is not saved.
#[path = "window_state.rs"]
pub mod window_state;

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
    std::fs::read_to_string(path)
        .ok()
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
            None,
            false,
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
            bytes,
            None,
            false,
            gio::FileCreateFlags::PRIVATE | gio::FileCreateFlags::REPLACE_DESTINATION,
            None::<&gio::Cancellable>,
        )?;
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("Pivra: could not save thumbnail pin state: {error}");
    }
}

/// Read the existing view settings, including one-line legacy files.
fn view_preferences() -> (crate::viewing::ViewMode, crate::media_sort::Order) {
    let value = load_toolbar_preference("view-mode.conf").unwrap_or_default();
    let mut lines = value.lines();
    let view = lines
        .next()
        .and_then(crate::viewing::ViewMode::from_id)
        .unwrap_or(crate::viewing::ViewMode::Fit);
    let sort = lines
        .next()
        .and_then(|line| line.strip_prefix("sort="))
        .and_then(crate::media_sort::Order::from_id)
        .unwrap_or_default();
    (view, sort)
}

fn save_view_preferences(view: crate::viewing::ViewMode, sort: crate::media_sort::Order) {
    let value = if sort == crate::media_sort::Order::default() {
        view.id().to_owned()
    } else {
        format!("{}\nsort={}", view.id(), sort.id())
    };
    save_toolbar_preference("view-mode.conf", &value);
}

pub fn load_view_mode() -> crate::viewing::ViewMode {
    view_preferences().0
}

pub fn save_view_mode(view: crate::viewing::ViewMode) {
    save_view_preferences(view, view_preferences().1);
}

pub fn load_media_sort() -> crate::media_sort::Order {
    view_preferences().1
}

pub fn save_media_sort(order: crate::media_sort::Order) {
    save_view_preferences(view_preferences().0, order);
}

/// Saved video audio controls shared by subsequent windows and launches.
pub fn load_video_audio() -> (f64, bool) {
    let path = glib::user_config_dir().join("pivra/video-audio.conf");
    let Ok(content) = std::fs::read_to_string(path) else {
        return (1.0, false);
    };
    let mut lines = content.lines();
    let volume = lines
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite() && (0.0..=1.0).contains(v));
    let muted = match lines.next() {
        Some("true") => Some(true),
        Some("false") => Some(false),
        _ => None,
    };
    match (volume, muted) {
        (Some(volume), Some(muted)) => (volume, muted),
        _ => (1.0, false),
    }
}

pub fn save_video_audio(volume: f64, muted: bool) {
    if !volume.is_finite() {
        return;
    }
    let dir = glib::user_config_dir().join("pivra");
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&dir)?;
        gio::File::for_path(dir.join("video-audio.conf")).replace_contents(
            format!("{:.4}\n{}\n", volume.clamp(0.0, 1.0), muted).as_bytes(),
            None,
            false,
            gio::FileCreateFlags::PRIVATE | gio::FileCreateFlags::REPLACE_DESTINATION,
            None::<&gio::Cancellable>,
        )?;
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("Pivra: could not save video audio settings: {error}");
    }
}

/// Bounded slideshow photo interval; the preference is independent of playback.
pub fn load_slideshow_interval() -> u32 {
    use std::io::Read;
    let mut bytes = String::new();
    std::fs::File::open(glib::user_config_dir().join("pivra/slideshow-interval.conf"))
        .and_then(|f| f.take(32).read_to_string(&mut bytes))
        .ok();
    parse_slideshow_interval(&bytes)
}
fn parse_slideshow_interval(bytes: &str) -> u32 {
    crate::slideshow::parse_interval(bytes).unwrap_or(crate::slideshow::DEFAULT_SECONDS)
}
pub fn save_slideshow_interval(seconds: u32) {
    if !crate::slideshow::valid_interval(seconds) {
        return;
    }
    let dir = glib::user_config_dir().join("pivra");
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&dir)?;
        gio::File::for_path(dir.join("slideshow-interval.conf")).replace_contents(
            format!("{seconds}\n").as_bytes(),
            None,
            false,
            gio::FileCreateFlags::PRIVATE | gio::FileCreateFlags::REPLACE_DESTINATION,
            None::<&gio::Cancellable>,
        )?;
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("Pivra: could not save slideshow interval: {error}");
    }
}

fn load_toolbar_preference(name: &str) -> Option<String> {
    use std::io::Read;
    let mut text = String::new();
    std::fs::File::open(glib::user_config_dir().join("pivra").join(name))
        .ok()?
        .take(32)
        .read_to_string(&mut text)
        .ok()?;
    Some(text)
}
fn save_toolbar_preference(name: &str, value: &str) {
    let dir = glib::user_config_dir().join("pivra");
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&dir)?;
        gio::File::for_path(dir.join(name)).replace_contents(
            format!("{value}\n").as_bytes(),
            None,
            false,
            gio::FileCreateFlags::PRIVATE | gio::FileCreateFlags::REPLACE_DESTINATION,
            None::<&gio::Cancellable>,
        )?;
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("Pivra: could not save {name}: {error}");
    }
}
pub fn load_slideshow_effect() -> crate::slideshow::Effect {
    load_toolbar_preference("slideshow-transition.conf")
        .as_deref()
        .and_then(|s| crate::slideshow::Effect::from_id(s.trim()))
        .unwrap_or_default()
}
pub fn save_slideshow_effect(effect: crate::slideshow::Effect) {
    save_toolbar_preference("slideshow-transition.conf", effect.id());
}
pub fn load_wheel_zoom() -> bool {
    load_toolbar_preference("wheel-zoom.conf").as_deref() == Some("true\n")
}
pub fn save_wheel_zoom(enabled: bool) {
    save_toolbar_preference("wheel-zoom.conf", if enabled { "true" } else { "false" });
}
pub fn load_resampling() -> crate::viewing::Resampling {
    load_toolbar_preference("resampling.conf")
        .as_deref()
        .and_then(|s| crate::viewing::Resampling::from_id(s.trim()))
        .unwrap_or_default()
}
pub fn save_resampling(filter: crate::viewing::Resampling) {
    save_toolbar_preference("resampling.conf", filter.id());
}

#[cfg(test)]
mod thumbnail_tests {
    #[test]
    fn slideshow_interval_rejects_invalid_preferences() {
        for value in ["", "0", "-5", "3601", "not-a-number", "5\n10"] {
            assert_eq!(super::parse_slideshow_interval(value), 5);
        }
        for value in ["1", "2", "3", "5", "7", "10", "15", "30\n", "3600"] {
            assert_eq!(
                super::parse_slideshow_interval(value),
                value.trim().parse::<u32>().unwrap()
            );
        }
    }

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
