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
    speed_button: gtk::MenuButton,
    loop_button: gtk::ToggleButton,
    seek: Rc<Cell<Option<f64>>>,
    internal_update: Rc<Cell<bool>>,
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

        let seek = Rc::new(Cell::new(None));
        let internal_update = Rc::new(Cell::new(false));
        let progress = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.001);
        progress.set_draw_value(false);
        progress.set_hexpand(true);
        progress.set_sensitive(false);
        progress.set_tooltip_text(Some("Seek animation"));
        let seek_requested = seek.clone();
        let updating = internal_update.clone();
        progress.connect_value_changed(move |scale| {
            if !updating.get() {
                seek_requested.set(Some(scale.value()));
            }
        });

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
        Self {
            paused,
            speed,
            looping,
            bar,
            play,
            progress,
            speed_button,
            loop_button,
            seek,
            internal_update,
        }
    }
    pub fn reset(&self) {
        self.paused.set(false);
        self.speed.set(1.0);
        self.looping.set(true);
        self.seek.set(None);
        self.progress.set_sensitive(false);
        self.show_progress(0.0);
        self.speed_button.set_label("1×");
        self.loop_button.set_active(true);
        self.bar.set_visible(false);
        self.play.set_icon_name("media-playback-pause-symbolic");
    }
    fn show_progress(&self, value: f64) {
        self.internal_update.set(true);
        self.progress.set_value(value.clamp(0.0, 1.0));
        self.internal_update.set(false);
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
    // Inspect frame durations without retaining frame textures. Seeking
    // re-decodes frames rather than caching an unbounded animation in memory.
    controls.bar.set_visible(true);
    glib::MainContext::default().spawn_local(async move {
        let cancelled = gio::Cancellable::new();
        let open_image = || {
            let mut loader = glycin::Loader::new(gio::File::for_path(&path));
            loader.sandbox_selector(glycin::SandboxSelector::Bwrap);
            loader.cancellable(cancelled.clone());
            loader.apply_transformations(true);
            loader.use_expose_base_dir(false);
            loader
        };

        // A bounded timeline pass is required for real percentage-based seeking.
        // If the decoder exposes an endlessly repeating stream or more than
        // 2048 frames, keep playback but leave seeking disabled.
        let mut durations = Vec::new();
        if let Ok(probe) = open_image().load().await {
            for _ in 0..2048 {
                if generation.get() != expected { cancelled.cancel(); return; }
                match probe.next_frame().await {
                    Ok(frame) => match frame.delay() {
                        Some(delay) => durations.push(delay.as_secs_f64().clamp(0.02, 10.0)),
                        None => break,
                    },
                    Err(_) => break,
                }
            }
        }
        if generation.get() != expected { cancelled.cancel(); return; }
        let timeline_ready = !durations.is_empty() && durations.len() < 2048;
        let total: f64 = durations.iter().sum();
        if timeline_ready && total > 0.0 {
            controls.progress.set_sensitive(true);
        }

        let Ok(mut image) = open_image().load().await else { return; };
        let Ok(mut frame) = image.next_frame().await else { return; };
        let Some(mut delay) = frame.delay() else {
            controls.bar.set_visible(false);
            return;
        };
        let mut frame_index = 0usize;
        let mut elapsed = 0.0f64;
        loop {
            if generation.get() != expected { cancelled.cancel(); return; }

            // Seeking is a decoder restart and bounded forward scan, never
            // an arbitrary seek in the sandboxed compressed byte stream.
            if let Some(fraction) = controls.seek.take() {
                if timeline_ready {
                    let target = total * fraction.clamp(0.0, 1.0);
                    let mut offset = 0.0;
                    let mut target_index = 0;
                    for (index, seconds) in durations.iter().enumerate() {
                        if offset + seconds > target {
                            target_index = index;
                            break;
                        }
                        offset += seconds;
                        target_index = index;
                    }
                    if let Ok(reloaded) = open_image().load().await {
                        image = reloaded;
                        let mut found = None;
                        for _ in 0..=target_index {
                            if generation.get() != expected { cancelled.cancel(); return; }
                            match image.next_frame().await {
                                Ok(next) => found = Some(next),
                                Err(_) => break,
                            }
                        }
                        if let Some(next) = found {
                            frame = next;
                            frame_index = target_index;
                            elapsed = offset;
                            delay = frame.delay().unwrap_or(Duration::from_millis(100));
                            if let Some(picture) = picture.upgrade() {
                                if frame_is_valid(&frame) {
                                    let next = frame.texture();
                                    let mode = mode_action.state()
                                        .and_then(|v| v.str().and_then(ViewMode::from_id))
                                        .unwrap_or_default();
                                    texture.replace(Some(next.clone()));
                                    present_filtered(&picture, Some(&next), mode.id(), transform.clone(), filter.clone());
                                }
                            }
                            controls.show_progress(elapsed / total);
                        }
                    }
                }
            }

            let mut remaining = delay.as_secs_f64().clamp(0.02, 10.0);
            while remaining > 0.0 || controls.paused.get() {
                if generation.get() != expected { cancelled.cancel(); return; }
                if controls.seek.get().is_some() { break; }
                glib::timeout_future(Duration::from_millis(20)).await;
                if !controls.paused.get() {
                    let rate = controls.speed.get();
                    remaining -= 0.020 * rate;
                    if timeline_ready {
                        controls.show_progress((elapsed + (delay.as_secs_f64() - remaining.max(0.0)).min(delay.as_secs_f64())) / total);
                    }
                }
            }
            if controls.seek.get().is_some() { continue; }
            if generation.get() != expected { cancelled.cancel(); return; }
            elapsed += delay.as_secs_f64().clamp(0.02, 10.0);
            frame_index += 1;

            if timeline_ready && frame_index >= durations.len() {
                if !controls.looping.get() {
                    controls.show_progress(1.0);
                    controls.paused.set(true);
                    controls.play.set_icon_name("media-playback-start-symbolic");
                    controls.play.set_tooltip_text(Some("Replay animation"));
                    while controls.paused.get() && controls.seek.get().is_none() {
                        if generation.get() != expected { cancelled.cancel(); return; }
                        glib::timeout_future(Duration::from_millis(20)).await;
                    }
                    if controls.seek.get().is_some() {
                        frame_index = frame_index.saturating_sub(1);
                        continue;
                    }
                }
                let Ok(reloaded) = open_image().load().await else { break; };
                image = reloaded;
                frame_index = 0;
                elapsed = 0.0;
            }
            let Ok(next_frame) = image.next_frame().await else { break; };
            if !frame_is_valid(&next_frame) { break; }
            let Some(next_delay) = next_frame.delay() else { break; };
            let Some(picture) = picture.upgrade() else { break; };
            let next = next_frame.texture();
            let mode = mode_action.state()
                .and_then(|v| v.str().and_then(ViewMode::from_id))
                .unwrap_or_default();
            texture.replace(Some(next.clone()));
            present_filtered(&picture, Some(&next), mode.id(), transform.clone(), filter.clone());
            delay = next_delay;
            if timeline_ready { controls.show_progress(elapsed / total); }
        }
        cancelled.cancel();
    });
}

fn frame_is_valid(frame: &glycin::Frame) -> bool {
    let width = frame.width();
    let height = frame.height();
    width > 0 && height > 0
        && width <= 32_768 && height <= 32_768
        && u64::from(width) * u64::from(height) <= 32_000_000
        && frame.buf_slice().len() <= 512 * 1024 * 1024
}
