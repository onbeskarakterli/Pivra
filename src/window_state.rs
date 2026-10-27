//! Small atomic window-state record, alongside the existing XDG preferences.
//! GTK's default_size is its remembered normal size, not fullscreen allocation.
use gtk::{gio, glib, prelude::*};
use std::{io::Read, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State {
    width: i32,
    height: i32,
    maximized: bool,
}

impl State {
    fn parse(bytes: &[u8]) -> Option<Self> {
        let text = std::str::from_utf8(bytes).ok()?;
        let mut lines = text.lines();
        if lines.next()? != "version=1" {
            return None;
        }
        let width = lines.next()?.strip_prefix("width=")?.parse().ok()?;
        let height = lines.next()?.strip_prefix("height=")?.parse().ok()?;
        let maximized = match lines.next()? {
            "maximized=true" => true,
            "maximized=false" => false,
            _ => return None,
        };
        if lines.next().is_some()
            || !(320..=16384).contains(&width)
            || !(240..=16384).contains(&height)
        {
            return None;
        }
        Some(Self {
            width,
            height,
            maximized,
        })
    }

    fn load(path: &Path) -> Option<Self> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .ok()?
            .take(257)
            .read_to_end(&mut bytes)
            .ok()?;
        (bytes.len() <= 256).then(|| Self::parse(&bytes)).flatten()
    }

    fn save(self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(path.parent().ok_or("missing config directory")?)?;
        gio::File::for_path(path).replace_contents(
            format!(
                "version=1\nwidth={}\nheight={}\nmaximized={}\n",
                self.width, self.height, self.maximized
            )
            .as_bytes(),
            None,
            false,
            gio::FileCreateFlags::PRIVATE | gio::FileCreateFlags::REPLACE_DESTINATION,
            None::<&gio::Cancellable>,
        )?;
        Ok(())
    }

    fn fitted(self, fallback: Self, bounds: (i32, i32)) -> Self {
        // Wayland has no portable workarea API. Leave a conservative margin for
        // desktop chrome; the compositor still decides placement and final size.
        let width = (bounds.0 - 96).max(1);
        let height = (bounds.1 - 96).max(1);
        if self.width <= width && self.height <= height {
            self
        } else {
            Self {
                width: fallback.width.min(width),
                height: fallback.height.min(height),
                ..self
            }
        }
    }
}

/// Install before presenting the window. Last closed window wins, matching the
/// existing app-wide preferences. No position, fullscreen state, or monitor ID
/// is persisted. GTK default_size is the documented restart-state API.
pub fn install(window: &impl IsA<gtk::Window>, defaults: &super::Settings) {
    let window = window.as_ref();
    let path = glib::user_config_dir().join("pivra/window.conf");
    let fallback = State {
        width: defaults.window_width,
        height: defaults.window_height,
        maximized: false,
    };
    let mut state = State::load(&path).unwrap_or(fallback);
    if let Some(monitor) = gtk::prelude::WidgetExt::display(window)
        .monitors()
        .item(0)
        .and_downcast::<gtk::gdk::Monitor>()
    {
        let geometry = monitor.geometry();
        state = state.fitted(fallback, (geometry.width(), geometry.height()));
    } else {
        // No monitor information: do not restore potentially off-screen sizes.
        state = fallback;
    }
    window.set_default_size(state.width, state.height);
    window.set_maximized(state.maximized);
    window.connect_close_request(move |window| {
        let (width, height) = window.default_size();
        let saved = State {
            width,
            height,
            maximized: window.is_maximized(),
        };
        // Refuse invalid dimensions rather than replacing a good record.
        let encoded = format!(
            "version=1\nwidth={width}\nheight={height}\nmaximized={}\n",
            saved.maximized
        );
        if State::parse(encoded.as_bytes()).is_some() {
            if let Err(error) = saved.save(&path) {
                eprintln!("Pivra: could not save window state: {error}");
            }
        }
        glib::Propagation::Proceed
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    const NORMAL: State = State {
        width: 960,
        height: 640,
        maximized: false,
    };

    #[test]
    fn restart_preserves_size_and_maximized_separately() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("pivra/window.conf");
        assert_eq!(State::load(&path), None);
        for state in [
            NORMAL,
            State {
                width: 1100,
                height: 780,
                maximized: true,
            },
            NORMAL,
        ] {
            state.save(&path).unwrap();
            assert_eq!(State::load(&path), Some(state));
        }
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(path).unwrap().permissions().mode() & 0o077,
            0
        );
    }

    #[test]
    fn corrupt_partial_overflow_and_oversized_records_are_rejected() {
        for bytes in [
            b"".as_slice(),
            b"version=2\nwidth=960\nheight=640\nmaximized=false\n",
            b"version=1\nwidth=-1\nheight=640\nmaximized=true\n",
            b"version=1\nwidth=99999999999999\nheight=640\nmaximized=true\n",
            b"version=1\nwidth=960\nheight=0\nmaximized=true\n",
            b"version=1\nwidth=960\nheight=640\nmaximized=maybe\n",
            b"version=1\nwidth=960\nheight=640\nmaximized=false\nextra=true\n",
            b"\xff",
        ] {
            assert_eq!(State::parse(bytes), None);
        }
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("window.conf");
        std::fs::write(&path, vec![b'x'; 10000]).unwrap();
        assert_eq!(State::load(&path), None);
    }

    #[test]
    fn changed_monitor_uses_safe_defaults_preserving_maximize_intent() {
        let large = State {
            width: 3000,
            height: 2000,
            maximized: true,
        };
        assert_eq!(
            large.fitted(NORMAL, (1280, 1024)),
            State {
                maximized: true,
                ..NORMAL
            }
        );
        let small = large.fitted(NORMAL, (800, 600));
        assert_eq!((small.width, small.height), (704, 504));
        assert_eq!(NORMAL.fitted(NORMAL, (1920, 1080)), NORMAL);
    }

    #[test]
    fn write_failure_is_reported_without_replacing_other_settings() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("not-directory");
        std::fs::write(&path, b"keep").unwrap();
        assert!(NORMAL.save(&path.join("window.conf")).is_err());
        assert_eq!(std::fs::read(path).unwrap(), b"keep");
    }
}
