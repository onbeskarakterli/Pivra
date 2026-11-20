//! Probe-only capability barrier. A rendered frame is not a preparation signal.
use gtk::{glib, prelude::*};
use std::{cell::Cell, rc::Rc, time::Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct State {
    prepared: bool,
    seekable: bool,
    duration_us: i64,
}
impl State {
    fn read(media: &impl IsA<gtk::MediaStream>) -> Self {
        let media = media.as_ref();
        Self {
            prepared: media.is_prepared(),
            seekable: media.is_seekable(),
            duration_us: media.duration(),
        }
    }
    fn ready(self) -> bool {
        self.prepared && self.seekable && self.duration_us > 0
    }
    fn describe(self) -> String {
        format!(
            "prepared={} seekable={} duration_us={}",
            self.prepared, self.seekable, self.duration_us
        )
    }
}

pub fn wait(media: &impl IsA<gtk::MediaStream>, deadline: Instant) -> Result<(), String> {
    let initial = State::read(media);
    eprintln!("CONTROL_PREPARATION initial {}", initial.describe());
    if let Some(error) = media.as_ref().error() {
        return Err(format!(
            "control preparation error: {error}; {}",
            initial.describe()
        ));
    }
    if initial.ready() {
        return Ok(());
    }
    if initial.prepared {
        return Err(format!(
            "control gate requires a prepared, seekable finite video; prepared capabilities: {}",
            initial.describe()
        ));
    }
    let expired = Rc::new(Cell::new(false));
    let timeout = expired.clone();
    // Use the caller's existing frame deadline. A source wakes a blocking GLib
    // iteration even if preparation never completes. GTK publishes seekability
    // and duration atomically with prepared; invalid prepared capabilities fail.
    let source = glib::timeout_add_local_once(
        deadline.saturating_duration_since(Instant::now()),
        move || timeout.set(true),
    );
    let mut previous = initial;
    let result = loop {
        let state = State::read(media);
        if state != previous {
            eprintln!("CONTROL_PREPARATION transition {}", state.describe());
            previous = state;
        }
        if let Some(error) = media.as_ref().error() {
            break Err(format!(
                "control preparation error: {error}; {}",
                state.describe()
            ));
        }
        if state.ready() {
            eprintln!("CONTROL_PREPARATION ready {}", state.describe());
            break Ok(());
        }
        if state.prepared {
            break Err(format!(
                "control gate requires a prepared, seekable finite video; prepared capabilities: {}",
                state.describe()
            ));
        }
        if expired.get() || Instant::now() >= deadline {
            break Err(format!(
                "control gate requires a prepared, seekable finite video; preparation deadline: {}",
                state.describe()
            ));
        }
        glib::MainContext::default().iteration(true);
    };
    if !expired.get() {
        source.remove();
    }
    result
}
