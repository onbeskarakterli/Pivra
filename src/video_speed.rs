//! One per-window speed action for the transport and context menu.
use adw::prelude::*;
use gtk::{gio, glib};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub(crate) const OPTIONS: [(&str, &str); 8] = [
    ("0.25", "0.25×"),
    ("0.5", "0.5×"),
    ("0.75", "0.75×"),
    ("1", "1×"),
    ("1.25", "1.25×"),
    ("1.5", "1.5×"),
    ("1.75", "1.75×"),
    ("2", "2×"),
];
const NORMAL: u32 = 4;

#[cfg(pivra_gpu_backend)]
unsafe extern "C" {
    fn pivra_gpu_has_pitch_filter(stream: *mut gtk::ffi::GtkMediaStream) -> glib::ffi::gboolean;
    fn pivra_gpu_set_playback_rate(
        stream: *mut gtk::ffi::GtkMediaStream,
        quarters: u32,
    ) -> glib::ffi::gboolean;
}

fn supports_rate(stream: &gtk::MediaStream) -> bool {
    #[cfg(pivra_gpu_backend)]
    {
        unsafe { pivra_gpu_has_pitch_filter(stream.as_ptr()) != 0 }
    }
    #[cfg(not(pivra_gpu_backend))]
    {
        let _ = stream;
        false
    }
}

fn set_rate(stream: &gtk::MediaStream, quarters: u32) -> bool {
    #[cfg(pivra_gpu_backend)]
    {
        unsafe { pivra_gpu_set_playback_rate(stream.as_ptr(), quarters) != 0 }
    }
    #[cfg(not(pivra_gpu_backend))]
    {
        let _ = (stream, quarters);
        false
    }
}

type CurrentStream = Rc<RefCell<Option<gtk::MediaStream>>>;

#[derive(Clone)]
pub(super) struct SpeedControl {
    pub(super) button: gtk::MenuButton,
    action: gio::SimpleAction,
    current: CurrentStream,
    preferred: Rc<Cell<u32>>,
}

impl SpeedControl {
    pub(super) fn new(window: &adw::ApplicationWindow, current: CurrentStream) -> Self {
        let action = gio::SimpleAction::new_stateful(
            "video-speed",
            Some(glib::VariantTy::STRING),
            &"1".to_variant(),
        );
        action.set_enabled(false);
        let weak_current = Rc::downgrade(&current);
        action.connect_change_state(move |_, target| {
            let Some(quarters) = target
                .and_then(glib::Variant::str)
                .and_then(|id| OPTIONS.iter().position(|option| option.0 == id))
                .map(|index| index as u32 + 1)
            else {
                return;
            };
            let Some(stream) = weak_current
                .upgrade()
                .and_then(|slot| slot.borrow().clone())
            else {
                return;
            };
            if !set_rate(&stream, quarters) {
                eprintln!(
                    "Pivra: the current video rejected playback speed {}",
                    quarters as f64 / 4.
                );
            }
        });
        action.connect_activate(|action, target| {
            if let Some(target) = target {
                action.change_state(target);
            }
        });
        window.add_action(&action);

        let button = gtk::MenuButton::builder()
            .label("1×")
            .tooltip_text("Playback speed")
            .sensitive(false)
            .build();
        button.set_widget_name("video-speed-button");
        button.add_css_class("flat");
        let weak = button.downgrade();
        action.connect_notify_local(Some("state"), move |action, _| {
            let state = action.state();
            let id = state.as_ref().and_then(glib::Variant::str);
            if let (Some(button), Some((_, label))) = (
                weak.upgrade(),
                OPTIONS.iter().find(|option| Some(option.0) == id),
            ) {
                button.set_label(label);
            }
        });
        button.set_popover(Some(&speed_menu(&action)));
        Self {
            button,
            action,
            current,
            preferred: Rc::new(Cell::new(NORMAL)),
        }
    }

    pub(super) fn is_open(&self) -> bool {
        self.button.popover().is_some_and(|menu| menu.is_visible())
    }

    pub(super) fn attach(&self) {
        self.button.popdown();
        self.action.set_state(&"1".to_variant());
        self.action.set_enabled(false);
        self.button.set_sensitive(false);
        let stream = self.current.borrow().clone();
        let Some(stream) = stream.filter(supports_rate) else {
            self.button
                .set_tooltip_text(Some("Playback speed is unavailable for this video"));
            return;
        };
        self.button.set_tooltip_text(Some("Playback speed"));

        let current = Rc::downgrade(&self.current);
        let action = self.action.downgrade();
        let button = self.button.downgrade();
        let preferred = self.preferred.clone();
        let restored = Cell::new(false);
        let update = Rc::new(move |stream: &gtk::MediaStream, confirmed: bool| {
            let (Some(slot), Some(action), Some(button)) =
                (current.upgrade(), action.upgrade(), button.upgrade())
            else {
                return;
            };
            if slot.borrow().as_ref() != Some(stream) {
                return;
            }
            let ready = stream.is_prepared()
                && stream.is_seekable()
                && !stream.is_seeking()
                && !stream.is_ended()
                && stream.error().is_none();
            let rate = stream.property::<u32>("playback-rate-quarters");
            if confirmed {
                preferred.set(rate);
            }
            let value = OPTIONS[rate as usize - 1].0.to_variant();
            if action.state().as_ref() != Some(&value) {
                action.set_state(&value);
            }
            if ready
                && !restored.replace(true)
                && rate != preferred.get()
                && !set_rate(stream, preferred.get())
            {
                eprintln!("Pivra: could not restore playback speed on the selected video");
            }
            let enabled = ready && !stream.property::<bool>("playback-rate-pending");
            action.set_enabled(enabled);
            button.set_sensitive(enabled);
        });
        update(&stream, false);
        for name in [
            "prepared",
            "seekable",
            "seeking",
            "ended",
            "error",
            "playback-rate-quarters",
            "playback-rate-pending",
        ] {
            let update = update.clone();
            stream.connect_notify_local(Some(name), move |stream, _| {
                update(stream, name == "playback-rate-quarters");
            });
        }
    }
}

fn speed_menu(action: &gio::SimpleAction) -> gtk::Popover {
    let popover = gtk::Popover::builder()
        .accessible_role(gtk::AccessibleRole::Menu)
        .has_arrow(false)
        .build();
    popover.add_css_class("menu");
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    for (id, label) in OPTIONS {
        let row = gtk::Button::builder()
            .accessible_role(gtk::AccessibleRole::MenuItemRadio)
            .has_frame(false)
            .action_name("win.video-speed")
            .action_target(&id.to_variant())
            .build();
        row.set_widget_name(&format!("video-speed-{id}"));
        row.add_css_class("flat");
        row.add_css_class("model");
        row.update_property(&[gtk::accessible::Property::Label(label)]);
        let line = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let text = gtk::Label::builder()
            .label(label)
            .xalign(0.)
            .hexpand(true)
            .build();
        let check = gtk::Image::from_icon_name("object-select-symbolic");
        check.set_pixel_size(16);
        line.append(&text);
        line.append(&check);
        row.set_child(Some(&line));
        let check = check.downgrade();
        let button = row.downgrade();
        let update = move |action: &gio::SimpleAction| {
            if let (Some(button), Some(check)) = (button.upgrade(), check.upgrade()) {
                let active = action.state().is_some_and(|state| state.str() == Some(id));
                check.set_opacity(if active { 1. } else { 0. });
                let checked = if active {
                    gtk::AccessibleTristate::True
                } else {
                    gtk::AccessibleTristate::False
                };
                button.update_state(&[gtk::accessible::State::Checked(checked)]);
            }
        };
        update(action);
        let handler = action.connect_notify_local(Some("state"), move |action, _| update(action));
        let action = action.downgrade();
        let handler = RefCell::new(Some(handler));
        row.connect_destroy(move |_| {
            if let (Some(action), Some(id)) = (action.upgrade(), handler.borrow_mut().take()) {
                action.disconnect(id);
            }
        });
        let weak = popover.downgrade();
        row.connect_clicked(move |_| {
            if let Some(popover) = weak.upgrade() {
                popover.popdown();
            }
        });
        content.append(&row);
    }
    popover.set_child(Some(&content));
    popover
}
