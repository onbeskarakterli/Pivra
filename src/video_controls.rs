//! Compact native video controls, separate from animated-image playback.
use gtk::{glib, prelude::*};
use std::{cell::Cell, rc::Rc, time::Duration};

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
}

fn clock(us: i64) -> String {
    let secs = us.max(0) as u64 / 1_000_000;
    format!("{}:{:02}", secs / 60, secs % 60)
}

impl Controls {
    pub fn new() -> Self {
        let stream: Rc<std::cell::RefCell<Option<gtk::MediaStream>>> = Rc::default();
        let updating = Rc::new(Cell::new(false));
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
            play.upcast_ref::<gtk::Widget>(),
            position.upcast_ref(),
            time.upcast_ref(),
            mute.upcast_ref(),
            volume.upcast_ref(),
        ] {
            bar.append(child);
        }
        let result = Self { bar, play, position, time, volume, mute, stream, updating };
        let refresh = result.clone();
        glib::MainContext::default().spawn_local(async move {
            loop {
                glib::timeout_future(Duration::from_millis(150)).await;
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
                    refresh.updating.set(true);
                    refresh.position.set_value(if duration > 0 {
                        (media.timestamp() as f64 / duration as f64).clamp(0.0, 1.0)
                    } else { 0.0 });
                    refresh.updating.set(false);
                }
                if refresh.bar.root().is_none() && refresh.bar.is_visible() {
                    break;
                }
            }
        });
        result
    }

    pub fn attach(&self, media: Option<gtk::MediaStream>) {
        *self.stream.borrow_mut() = media;
        self.bar.set_visible(self.stream.borrow().is_some());
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
