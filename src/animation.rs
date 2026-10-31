//! Animated foreground images; thumbnails stay on their static first frame.
use super::{present_filtered, ViewMode};
use gtk::{gio, glib, prelude::*};
use pivra::viewing::{Resampling, Transform};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Clone)]
pub(super) struct Controls {
    pub paused: Rc<Cell<bool>>,
    pub speed: Rc<Cell<f64>>,
    pub looping: Rc<Cell<bool>>,
    pub bar: gtk::Box,
    pub play: gtk::Button,
    pub progress: gtk::Scale,
    time_label: gtk::Label,
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
        loop_button.add_css_class("pivra-loop-active");
        let css = gtk::CssProvider::new();
        css.load_from_string(
            ".pivra-animation-controls button.pivra-loop-active { background-color: @accent_bg_color; color: @accent_fg_color; }"
        );
        gtk::style_context_add_provider_for_display(
            &loop_button.display(),
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        let loop_state = looping.clone();
        loop_button.connect_toggled(move |button| {
            loop_state.set(button.is_active());
            if button.is_active() {
                button.add_css_class("pivra-loop-active");
            } else {
                button.remove_css_class("pivra-loop-active");
            }
            button.set_tooltip_text(Some(if button.is_active() { "Loop on" } else { "Loop off" }));
        });

        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        bar.add_css_class("pivra-animation-controls");
        bar.set_halign(gtk::Align::Fill);
        bar.set_valign(gtk::Align::End);
        // Reserve side gutters for the navigation panels on every viewport.
        bar.set_margin_start(96);
        bar.set_margin_end(96);
        bar.set_margin_bottom(0);
        let time_label = gtk::Label::new(Some("0:00 / --:--"));
        time_label.add_css_class("dim-label");
        bar.append(&play);
        bar.append(&progress);
        bar.append(&time_label);
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
            time_label,
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
        self.time_label.set_text("0:00 / --:--");
        self.show_progress(0.0);
        self.speed_button.set_label("1×");
        self.loop_button.set_active(true);
        self.bar.set_visible(false);
        self.play.set_icon_name("media-playback-pause-symbolic");
    }
    fn show_time(&self, current: f64, total: f64) {
        let timestamp = |seconds: f64| {
            let seconds = seconds.max(0.0).floor() as u64;
            format!("{}:{:02}", seconds / 60, seconds % 60)
        };
        self.time_label.set_text(&format!("{} / {}", timestamp(current), timestamp(total)));
    }
    fn show_progress(&self, value: f64) {
        self.internal_update.set(true);
        self.progress.set_value(value.clamp(0.0, 1.0));
        self.internal_update.set(false);
    }
}

#[allow(clippy::too_many_arguments)]
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
    if !is_animated_file(&path) {
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

        // Start foreground playback immediately. Index frame durations in a
        // separate cooperative task so a long GIF never blocks first playback.
        type AnimationTimeline = Rc<RefCell<Option<(Rc<Vec<f64>>, f64)>>>;
        let timeline: AnimationTimeline = Rc::new(RefCell::new(None));
        let probe_timeline = timeline.clone();
        let probe_controls = controls.clone();
        let probe_generation = generation.clone();
        let probe_path = path.clone();
        let probe_cancel = cancelled.clone();
        glib::MainContext::default().spawn_local(async move {
            let mut probe_loader = glycin::Loader::new(gio::File::for_path(&probe_path));
            probe_loader.sandbox_selector(glycin::SandboxSelector::Bwrap);
            probe_loader.cancellable(probe_cancel.clone());
            probe_loader.apply_transformations(true);
            probe_loader.use_expose_base_dir(false);
            let Ok(probe) = probe_loader.load().await else { return; };
            let mut durations = Vec::new();
            let mut reached_end = false;
            for index in 0..2048 {
                if probe_generation.get() != expected || probe_cancel.is_cancelled() {
                    probe_cancel.cancel();
                    return;
                }
                match probe.specific_frame(glycin::FrameRequest::new().loop_animation(false)).await {
                    Ok(frame) => match frame.delay() {
                        Some(delay) => durations.push(delay.as_secs_f64().clamp(0.02, 10.0)),
                        None => { reached_end = true; break; }
                    },
                    Err(_) => { reached_end = true; break; }
                }
                // Give the foreground renderer and controls event-loop time.
                if index % 8 == 7 {
                    glib::timeout_future(Duration::from_millis(1)).await;
                }
            }
            if probe_generation.get() == expected && reached_end && durations.len() > 1 {
                let total: f64 = durations.iter().sum();
                probe_timeline.replace(Some((Rc::new(durations), total)));
                probe_controls.progress.set_sensitive(true);
                probe_controls.show_time(0.0, total);
            }
        });

        let Ok(mut image) = open_image().load().await else { return; };
        if generation.get() != expected { cancelled.cancel(); return; }
        let Ok(first_frame) = image.next_frame().await else { return; };
        if generation.get() != expected { cancelled.cancel(); return; }
        let canvas_width = image.details().width();
        let canvas_height = image.details().height();
        let webp = path.extension().and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("webp"));
        let Some(mut delay) = first_frame.delay() else {
            controls.bar.set_visible(false);
            return;
        };
        // The initial frame is already displayed by the foreground loader.
        // Drop its decoded buffer instead of retaining it for the entire animation.
        drop(first_frame);
        let mut frame_index = 0usize;
        let mut elapsed = 0.0f64;
        loop {
            if generation.get() != expected { cancelled.cancel(); return; }

            // Clone just the Rc pointer, not the complete frame-duration index.
            // The total is computed once after indexing, not at every frame.
            let timeline_snapshot = timeline.borrow().clone();
            let durations = timeline_snapshot.as_ref().map(|(frames, _)| frames.as_slice()).unwrap_or(&[]);
            let timeline_ready = !durations.is_empty();
            let total = timeline_snapshot.as_ref().map_or(0.0, |(_, total)| *total);
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
                    // If the requested frame is ahead of the current decoder
                    // position, advance in-place instead of reopening and
                    // re-decoding from frame zero on every forward seek.
                    let forward = target_index >= frame_index;
                    if forward || target_index < frame_index {
                        let ready = if forward {
                            true
                        } else if let Ok(reloaded) = open_image().load().await {
                            image = reloaded;
                            true
                        } else {
                            false
                        };
                        if generation.get() != expected { cancelled.cancel(); return; }
                        if ready {
                            let steps = if forward {
                                target_index - frame_index
                            } else {
                                target_index + 1
                            };
                            let mut found = None;
                            for step in 0..steps {
                                if generation.get() != expected { cancelled.cancel(); return; }
                                match image.next_frame().await {
                                    Ok(next) => found = Some(next),
                                    Err(_) => break,
                                }
                                if step % 16 == 15 {
                                    glib::timeout_future(Duration::from_millis(1)).await;
                                }
                            }
                            if generation.get() != expected { cancelled.cancel(); return; }
                            {
                                if let Some(next) = found {
                                    if frame_is_valid(&next)
                                        && (!webp || (next.width() == canvas_width
                                            && next.height() == canvas_height))
                                    {
                                        frame_index = target_index;
                                        elapsed = offset;
                                        delay = next.delay().unwrap_or(Duration::from_millis(100));
                                        if let Some(picture) = picture.upgrade() {
                                            let next = next.texture();
                                            let mode = mode_action.state()
                                                .and_then(|v| v.str().and_then(ViewMode::from_id))
                                                .unwrap_or_default();
                                            texture.replace(Some(next.clone()));
                                            present_filtered(&picture, Some(&next), mode.id(), transform.clone(), filter.clone());
                                        }
                                        controls.show_progress(elapsed / total);
                                        controls.show_time(elapsed, total);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let mut remaining = delay.as_secs_f64().clamp(0.02, 10.0);
            while remaining > 0.0 || controls.paused.get() {
                if generation.get() != expected { cancelled.cancel(); return; }
                if controls.seek.get().is_some() { break; }
                // Sleep close to the next frame deadline instead of polling
                // every 5 ms. Bound the sleep so pause/speed/seek controls
                // remain responsive even on long-duration frames.
                let rate = controls.speed.get().clamp(0.25, 4.0);
                let sleep = if controls.paused.get() {
                    Duration::from_millis(40)
                } else {
                    Duration::from_secs_f64((remaining / rate).clamp(0.001, 0.04))
                };
                let started = Instant::now();
                glib::timeout_future(sleep).await;
                if !controls.paused.get() {
                    remaining -= started.elapsed().as_secs_f64() * controls.speed.get();
                    if timeline_ready {
                        let current = elapsed + (delay.as_secs_f64() - remaining.max(0.0)).min(delay.as_secs_f64());
                        controls.show_progress(current / total);
                        controls.show_time(current, total);
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
                    controls.show_time(total, total);
                    controls.paused.set(true);
                    controls.play.set_icon_name("media-playback-start-symbolic");
                    controls.play.set_tooltip_text(Some("Replay animation"));
                    while controls.paused.get()
                        && !controls.looping.get()
                        && controls.seek.get().is_none()
                    {
                        if generation.get() != expected { cancelled.cancel(); return; }
                        glib::timeout_future(Duration::from_millis(20)).await;
                    }
                    // Enabling loop after the final frame resumes playback.
                    if controls.looping.get() && controls.paused.get() {
                        controls.paused.set(false);
                        controls.play.set_icon_name("media-playback-pause-symbolic");
                        controls.play.set_tooltip_text(Some("Pause animation"));
                    }
                    if controls.seek.get().is_some() {
                        frame_index = frame_index.saturating_sub(1);
                        continue;
                    }
                }
                let Ok(reloaded) = open_image().load().await else { break; };
                if generation.get() != expected { cancelled.cancel(); return; }
                image = reloaded;
                frame_index = 0;
                elapsed = 0.0;
            }
            let Ok(next_frame) = image.next_frame().await else { break; };
            if generation.get() != expected { cancelled.cancel(); return; }
            if !frame_is_valid(&next_frame) { break; }
            if webp && (next_frame.width() != canvas_width
                || next_frame.height() != canvas_height)
            {
                eprintln!(
                    "Pivra: WebP returned a partial frame ({}x{}, canvas {}x{});                      refusing to stretch a partial frame across the image",
                    next_frame.width(), next_frame.height(), canvas_width, canvas_height
                );
                break;
            }
            let Some(next_delay) = next_frame.delay() else { break; };
            let Some(picture) = picture.upgrade() else { break; };
            let next = next_frame.texture();
            let mode = mode_action.state()
                .and_then(|v| v.str().and_then(ViewMode::from_id))
                .unwrap_or_default();
            texture.replace(Some(next.clone()));
            present_filtered(&picture, Some(&next), mode.id(), transform.clone(), filter.clone());
            delay = next_delay;
            if timeline_ready {
                controls.show_progress(elapsed / total);
                controls.show_time(elapsed, total);
            }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimationKind {
    Gif,
    Webp,
    Apng,
}

/// Check animation-specific header information without decoding the media or
/// allocating unbounded file data. GIF always uses the frame-based path;
/// Glycin's first frame delay still determines whether playback is possible.
fn animation_kind(path: &std::path::Path) -> Option<AnimationKind> {
    use std::io::Read;
    let ext = path.extension()?.to_str()?;
    if ext.eq_ignore_ascii_case("gif") {
        return Some(AnimationKind::Gif);
    }
    if ext.eq_ignore_ascii_case("webp") {
        let mut file = std::fs::File::open(path).ok()?;
        let mut header = [0u8; 21];
        file.read_exact(&mut header).ok()?;
        // Extended WebP declares animation in VP8X feature bit 1.
        if &header[0..4] == b"RIFF"
            && &header[8..12] == b"WEBP"
            && &header[12..16] == b"VP8X"
            && header[20] & 0x02 != 0
        {
            return Some(AnimationKind::Webp);
        }
        return None;
    }
    if !ext.eq_ignore_ascii_case("png") && !ext.eq_ignore_ascii_case("apng") {
        return None;
    }
    let mut file = std::fs::File::open(path).ok()?;
    let mut signature = [0u8; 8];
    file.read_exact(&mut signature).ok()?;
    if signature != [137, 80, 78, 71, 13, 10, 26, 10] {
        return None;
    }
    // APNG's acTL chunk must precede the first IDAT. Limit the amount of
    // metadata examined so malformed PNGs cannot trigger an unbounded scan.
    let mut examined = 8u64;
    while examined < 1024 * 1024 {
        let mut chunk_header = [0u8; 8];
        file.read_exact(&mut chunk_header).ok()?;
        let len = u32::from_be_bytes(chunk_header[0..4].try_into().ok()?) as u64;
        let kind = &chunk_header[4..8];
        if kind == b"acTL" && len == 8 {
            return Some(AnimationKind::Apng);
        }
        if kind == b"IDAT" || kind == b"IEND" {
            return None;
        }
        examined = examined.checked_add(12)?.checked_add(len)?;
        if examined > 1024 * 1024 {
            return None;
        }
        std::io::copy(&mut std::io::Read::by_ref(&mut file).take(len + 4), &mut std::io::sink()).ok()?;
    }
    None
}

fn is_animated_file(path: &std::path::Path) -> bool {
    animation_kind(path).is_some()
}
