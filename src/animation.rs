//! Animated foreground images; thumbnails stay on their static first frame.
use super::{present_filtered, ViewMode};
use gtk::{gio, glib, prelude::*};
use pivra::viewing::{Resampling, Transform};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    time::Duration,
};

#[derive(Clone)]
pub(super) struct Controls {
    pub paused: Rc<Cell<bool>>,
    pub speed: Rc<Cell<f64>>,
    pub looping: Rc<Cell<bool>>,
    pub bar: gtk::Box,
    pub play: gtk::Button,
    pub progress: gtk::Scale,
}

impl Controls {
    pub fn new() -> Self {
        let paused = Rc::new(Cell::new(false));
        let speed = Rc::new(Cell::new(1.0));
        let looping = Rc::new(Cell::new(true));
        let play = gtk::Button::from_icon_name("media-playback-pause-symbolic");
        play.set_tooltip_text(Some("Pause animation"));
        play.add_css_class("flat");
        let pause_state = paused.clone();
        play.connect_clicked(move |button| {
            let pause = !pause_state.get();
            pause_state.set(pause);
            button.set_icon_name(if pause {
                "media-playback-start-symbolic"
            } else {
                "media-playback-pause-symbolic"
            });
            button.set_tooltip_text(Some(if pause { "Play animation" } else { "Pause animation" }));
        });

        let progress = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.001);
        progress.set_draw_value(false);
        progress.set_hexpand(true);
        progress.set_sensitive(false); // Enable only after an accurate frame timeline exists.
        progress.set_tooltip_text(Some("Seeking requires a decoded frame timeline"));

        let speed_button = gtk::MenuButton::new();
        speed_button.set_label("1×");
        speed_button.add_css_class("flat");
        let speed_popover = gtk::Popover::new();
        let speeds = gtk::Box::new(gtk::Orientation::Vertical, 2);
        for (label, value) in [("0.25×", 0.25), ("0.5×", 0.5), ("1×", 1.0), ("2×", 2.0), ("4×", 4.0)] {
            let option = gtk::Button::with_label(label);
            option.add_css_class("flat");
            let selected = speed.clone();
            let button = speed_button.clone();
            let popover = speed_popover.clone();
            option.connect_clicked(move |_| {
                selected.set(value);
                button.set_label(label);
                popover.popdown();
            });
            speeds.append(&option);
        }
        speeds.set_margin_top(6);
        speeds.set_margin_bottom(6);
        speeds.set_margin_start(6);
        speeds.set_margin_end(6);
        speed_popover.set_child(Some(&speeds));
        speed_button.set_popover(Some(&speed_popover));

        let loop_button = gtk::ToggleButton::new();
        loop_button.set_icon_name("media-playlist-repeat-symbolic");
        loop_button.set_active(true);
        loop_button.set_tooltip_text(Some("Loop on"));
        loop_button.add_css_class("flat");
        let loop_state = looping.clone();
        loop_button.connect_toggled(move |button| {
            loop_state.set(button.is_active());
            button.set_tooltip_text(Some(if button.is_active() { "Loop on" } else { "Loop off" }));
        });

        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        bar.add_css_class("pivra-animation-controls");
        bar.set_halign(gtk::Align::Fill);
        bar.set_valign(gtk::Align::End);
        bar.set_margin_start(16);
        bar.set_margin_end(16);
        bar.set_margin_bottom(12);
        bar.append(&play);
        bar.append(&progress);
        bar.append(&speed_button);
        bar.append(&loop_button);
        bar.set_visible(false);
        Self { paused, speed, looping, bar, play, progress }
    }
    pub fn reset(&self) {
        self.paused.set(false);
        self.speed.set(1.0);
        self.looping.set(true);
        self.progress.set_value(0.0);
        self.bar.set_visible(false);
        self.play.set_icon_name("media-playback-pause-symbolic");
    }
}

pub(super) fn start(
    path: PathBuf,
    generation: Rc<Cell<u64>>,
    expected: u64,
    controls: Controls,
    picture: glib::WeakRef<gtk::Picture>,
    texture: Rc<RefCell<Option<gtk::gdk::Texture>>>,
    mode_action: gio::SimpleAction,
    transform: Rc<RefCell<Transform>>,
    filter: Rc<Cell<Resampling>>,
) {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !["gif", "webp", "png", "apng"]
        .iter()
        .any(|s| ext.eq_ignore_ascii_case(s))
    {
        return;
    }
    controls.bar.set_visible(true);
    glib::MainContext::default().spawn_local(async move {
        let cancelled = gio::Cancellable::new();
        let mut loader = glycin::Loader::new(gio::File::for_path(&path));
        loader.sandbox_selector(glycin::SandboxSelector::Bwrap);
        loader.cancellable(cancelled.clone());
        loader.apply_transformations(true);
        loader.use_expose_base_dir(false);
        let Ok(image) = loader.load().await else {
            return;
        };
        if generation.get() != expected {
            cancelled.cancel();
            return;
        }
        let Ok(first) = image.next_frame().await else {
            return;
        };
        let Some(mut delay) = first.delay() else {
            return;
        };
        loop {
            if generation.get() != expected {
                cancelled.cancel();
                break;
            }
            let mut left = delay.as_secs_f64() / controls.speed.get();
            while left > 0.0 || controls.paused.get() {
                if generation.get() != expected { cancelled.cancel(); return; }
                glib::timeout_future(Duration::from_millis(20)).await;
                if !controls.paused.get() { left -= 0.020; }
            }
            if generation.get() != expected {
                cancelled.cancel();
                break;
            }
            let Ok(frame) = image.next_frame().await else {
                break;
            };
            if generation.get() != expected {
                cancelled.cancel();
                break;
            }
            let width = frame.width();
            let height = frame.height();
            if width == 0
                || height == 0
                || width > 32_768
                || height > 32_768
                || u64::from(width) * u64::from(height) > 32_000_000
                || frame.buf_slice().len() > 512 * 1024 * 1024
            {
                break;
            }
            let Some(next_delay) = frame.delay() else {
                break;
            };
            let Some(picture) = picture.upgrade() else {
                break;
            };
            let next = frame.texture();
            let mode = mode_action
                .state()
                .and_then(|v| v.str().and_then(ViewMode::from_id))
                .unwrap_or_default();
            texture.replace(Some(next.clone()));
            present_filtered(
                &picture,
                Some(&next),
                mode.id(),
                transform.clone(),
                filter.clone(),
            );
            delay = next_delay;
        }
    });
}
