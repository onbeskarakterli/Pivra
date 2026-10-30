//! Compact native video controls, separate from animated-image playback.
use gtk::{glib, prelude::*};
use std::{cell::Cell, rc::Rc, time::{Duration, Instant}};

#[derive(Clone)]
pub(super) struct Controls {
    pub bar: gtk::Box,
    play: gtk::Button,
    position: gtk::Scale,
    time: gtk::Label,
    volume: gtk::Scale,
    mute: gtk::ToggleButton,
    stream: Rc<std::cell::RefCell<Option<gtk::MediaStream>>>,
    updating: Rc<Cell<bool>>,
    scrubbing: Rc<Cell<bool>>,
    alive: Rc<Cell<bool>>,
    last_activity: Rc<Cell<Instant>>,
}

fn clock(us: i64) -> String {
    let secs = us.max(0) as u64 / 1_000_000;
    format!("{}:{:02}", secs / 60, secs % 60)
}

impl Controls {
    pub fn new() -> Self {
        let stream: Rc<std::cell::RefCell<Option<gtk::MediaStream>>> = Rc::default();
        let updating = Rc::new(Cell::new(false));
        let scrubbing = Rc::new(Cell::new(false));
        let alive = Rc::new(Cell::new(true));
        let last_activity = Rc::new(Cell::new(Instant::now()));
        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        bar.add_css_class("pivra-animation-controls");
        bar.set_halign(gtk::Align::Fill);
        bar.set_valign(gtk::Align::End);
        bar.set_margin_start(96);
        bar.set_margin_end(96);
        bar.set_visible(false);

        let play = gtk::Button::from_icon_name("media-playback-pause-symbolic");
        play.add_css_class("flat");
        let current = stream.clone();
        play.connect_clicked(move |_| {
            if let Some(media) = current.borrow().as_ref() {
                if media.is_playing() { media.pause(); } else { media.play(); }
            }
        });

        // Seek buttons share the same five-second step as Ctrl+Arrow.
        let rewind = gtk::Button::from_icon_name("media-seek-backward-symbolic");
        rewind.add_css_class("flat");
        rewind.set_tooltip_text(Some("Back 5 seconds"));
        let current = stream.clone();
        rewind.connect_clicked(move |_| {
            if let Some(media) = current.borrow().as_ref() {
                if media.is_seekable() {
                    media.seek(media.timestamp().saturating_sub(5_000_000).max(0));
                }
            }
        });

        let forward = gtk::Button::from_icon_name("media-seek-forward-symbolic");
        forward.add_css_class("flat");
        forward.set_tooltip_text(Some("Forward 5 seconds"));
        let current = stream.clone();
        forward.connect_clicked(move |_| {
            if let Some(media) = current.borrow().as_ref() {
                if media.is_seekable() {
                    media.seek(media.timestamp().saturating_add(5_000_000)
                        .min(media.duration().max(0)));
                }
            }
        });

        let fullscreen = gtk::Button::from_icon_name("view-fullscreen-symbolic");
        fullscreen.add_css_class("flat");
        fullscreen.set_tooltip_text(Some("Toggle fullscreen"));
        fullscreen.set_action_name(Some("win.fullscreen"));

        let position = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.001);
        position.set_hexpand(true);
        position.set_draw_value(false);
        position.set_sensitive(false);
        let current = stream.clone();
        let guard = updating.clone();
        position.connect_value_changed(move |scale| {
            if guard.get() { return; }
            if let Some(media) = current.borrow().as_ref() {
                let duration = media.duration();
                if duration > 0 && media.is_seekable() {
                    media.seek((duration as f64 * scale.value()) as i64);
                }
            }
        });

        // Capture the drag at the slider, then keep receiving relative motion
        // even after the pointer leaves its bounds. GTK's gesture sequence
        // remains owned until the button is released.
        let drag = gtk::GestureDrag::new();
        drag.set_button(1);
        drag.set_propagation_phase(gtk::PropagationPhase::Capture);
        let start_x = Rc::new(Cell::new(0.0_f64));
        let initial = start_x.clone();
        let active = scrubbing.clone();
        let slider = position.clone();
        let media = stream.clone();
        drag.connect_drag_begin(move |gesture, x, _| {
            if !slider.is_sensitive() || media.borrow().is_none() {
                return;
            }
            active.set(true);
            initial.set(x);
            gesture.set_state(gtk::EventSequenceState::Claimed);
            slider.set_value((x / f64::from(slider.width().max(1))).clamp(0.0, 1.0));
        });
        let initial = start_x.clone();
        let slider = position.clone();
        let active = scrubbing.clone();
        drag.connect_drag_update(move |_, dx, _| {
            if active.get() {
                slider.set_value(((initial.get() + dx) / f64::from(slider.width().max(1)))
                    .clamp(0.0, 1.0));
            }
        });
        let slider = position.clone();
        let initial = start_x.clone();
        let active = scrubbing.clone();
        drag.connect_drag_end(move |_, dx, _| {
            if active.replace(false) {
                slider.set_value(((initial.get() + dx) / f64::from(slider.width().max(1)))
                    .clamp(0.0, 1.0));
            }
        });
        let active = scrubbing.clone();
        drag.connect_cancel(move |_, _| active.set(false));
        position.add_controller(drag);

        let time = gtk::Label::new(Some("0:00 / --:--"));
        time.add_css_class("dim-label");
        let mute = gtk::ToggleButton::new();
        mute.set_icon_name("audio-volume-high-symbolic");
        mute.add_css_class("flat");
        let current = stream.clone();
        mute.connect_toggled(move |button| {
            if let Some(media) = current.borrow().as_ref() {
                media.set_muted(button.is_active());
            }
            button.set_icon_name(if button.is_active() {
                "audio-volume-muted-symbolic"
            } else {
                "audio-volume-high-symbolic"
            });
        });
        let volume = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
        volume.set_size_request(96, -1);
        volume.set_draw_value(false);
        volume.set_value(1.0);
        let current = stream.clone();
        volume.connect_value_changed(move |scale| {
            if let Some(media) = current.borrow().as_ref() {
                media.set_volume(scale.value());
            }
        });
        for child in [
            rewind.upcast_ref::<gtk::Widget>(),
            play.upcast_ref(),
            forward.upcast_ref(),
            position.upcast_ref(),
            time.upcast_ref(),
            mute.upcast_ref(),
            volume.upcast_ref(),
            fullscreen.upcast_ref(),
        ] {
            bar.append(child);
        }
        let result = Self { bar, play, position, time, volume, mute, stream, updating, scrubbing, alive, last_activity };
        let refresh = result.clone();
        glib::MainContext::default().spawn_local(async move {
            while refresh.alive.get() {
                glib::timeout_future(Duration::from_millis(150)).await;
                if refresh.stream.borrow().is_some() && !refresh.scrubbing.get() && refresh.last_activity.get().elapsed() >= Duration::from_secs(2) {
                    refresh.bar.set_visible(false);
                }
                if let Some(media) = refresh.stream.borrow().as_ref() {
                    refresh.play.set_icon_name(if media.is_playing() {
                        "media-playback-pause-symbolic"
                    } else {
                        "media-playback-start-symbolic"
                    });
                    let duration = media.duration();
                    refresh.time.set_text(&format!(
                        "{} / {}",
                        clock(media.timestamp()),
                        if duration > 0 { clock(duration) } else { "--:--".into() }
                    ));
                    refresh.position.set_sensitive(duration > 0 && media.is_seekable());
                    if !refresh.scrubbing.get() {
                    refresh.updating.set(true);
                    refresh.position.set_value(if duration > 0 {
                        (media.timestamp() as f64 / duration as f64).clamp(0.0, 1.0)
                    } else { 0.0 });
                    refresh.updating.set(false);
                    }
                }
            }
        });
        result
    }

    pub fn reveal(&self) {
        if self.stream.borrow().is_some() {
            self.last_activity.set(Instant::now());
            self.bar.set_visible(true);
        }
    }

    pub fn handle_key(&self, key: gtk::gdk::Key, ctrl: bool) -> bool {
        let media = self.stream.borrow().clone();
        let Some(media) = media else { return false; };
        match (key, ctrl) {
            (gtk::gdk::Key::space, false) => {
                if media.is_playing() { media.pause(); } else { media.play(); }
            }
            (gtk::gdk::Key::Left, true) | (gtk::gdk::Key::Right, true) => {
                if !media.is_seekable() { return false; }
                let offset = if key == gtk::gdk::Key::Left { -5_000_000 } else { 5_000_000 };
                media.seek((media.timestamp() + offset).clamp(0, media.duration().max(0)));
            }
            (gtk::gdk::Key::m, false) | (gtk::gdk::Key::M, false) => {
                self.mute.set_active(!self.mute.is_active());
            }
            _ => return false,
        }
        self.reveal();
        true
    }

    pub fn shutdown(&self) {
        self.alive.set(false);
        self.attach(None);
    }

    pub fn attach(&self, media: Option<gtk::MediaStream>) {
        *self.stream.borrow_mut() = media;
        self.bar.set_visible(self.stream.borrow().is_some());
        self.last_activity.set(Instant::now());
        self.scrubbing.set(false);
        self.position.set_sensitive(false);
        self.updating.set(true);
        self.position.set_value(0.0);
        self.updating.set(false);
        if let Some(media) = self.stream.borrow().as_ref() {
            media.set_volume(self.volume.value());
            media.set_muted(self.mute.is_active());
        } else {
            self.time.set_text("0:00 / --:--");
        }
    }
}
