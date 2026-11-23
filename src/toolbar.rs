//! Compact slideshow controls; actions remain owned by the per-window Session.
use adw::prelude::*;
use gtk::gio;
use pivra::slideshow::{Effect, INTERVALS};
use std::rc::Rc;

pub(super) fn slideshow_group(window: &adw::ApplicationWindow) -> gtk::Box {
    let group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    group.set_widget_name("slideshow-controls");
    group.add_css_class("linked");
    let play = gtk::Button::from_icon_name("media-playback-start-symbolic");
    play.set_widget_name("slideshow-play");
    play.set_tooltip_text(Some("Start slideshow (F5)"));
    play.set_action_name(Some("win.slideshow"));
    group.append(&play);

    let interval = window
        .lookup_action("slideshow-interval")
        .expect("interval action");
    let seconds = interval.state().and_then(|v| v.get::<u32>()).unwrap_or(5);
    let button = gtk::MenuButton::builder()
        .label(format!("{seconds} s"))
        .tooltip_text("Photo interval; videos play to completion")
        .build();
    button.set_widget_name("slideshow-interval-selector");
    let popover = gtk::Popover::new();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    let presets = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    for seconds in INTERVALS {
        let preset = gtk::Button::with_label(&format!("{seconds} s"));
        preset.set_widget_name(&format!("slideshow-interval-{seconds}"));
        preset.set_action_name(Some("win.slideshow-interval"));
        preset.set_action_target_value(Some(&seconds.to_variant()));
        let weak = popover.downgrade();
        preset.connect_clicked(move |_| {
            if let Some(p) = weak.upgrade() {
                p.popdown();
            }
        });
        presets.append(&preset);
    }
    content.append(&presets);
    let custom = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let entry = gtk::Entry::builder()
        .text(seconds.to_string())
        .width_chars(8)
        .max_length(16)
        .input_purpose(gtk::InputPurpose::Digits)
        .placeholder_text("Seconds")
        .tooltip_text("Whole seconds, 1–3600")
        .build();
    entry.set_widget_name("slideshow-custom-interval");
    let apply = gtk::Button::with_label("Apply");
    apply.set_widget_name("slideshow-interval-apply");
    custom.append(&entry);
    custom.append(&apply);
    content.append(&custom);
    let error = gtk::Label::new(None);
    error.set_widget_name("slideshow-interval-error");
    error.add_css_class("error");
    error.set_wrap(true);
    error.set_max_width_chars(28);
    error.set_visible(false);
    content.append(&error);
    popover.set_child(Some(&content));
    button.set_popover(Some(&popover));
    let (weak_window, weak_entry, weak_error, weak_popover) = (
        window.downgrade(),
        entry.downgrade(),
        error.downgrade(),
        popover.downgrade(),
    );
    let submit = Rc::new(move || {
        let (Some(w), Some(entry), Some(error)) = (
            weak_window.upgrade(),
            weak_entry.upgrade(),
            weak_error.upgrade(),
        ) else {
            return;
        };
        let Some(seconds) = pivra::slideshow::parse_interval(&entry.text()) else {
            entry.add_css_class("error");
            error.set_text("Enter whole seconds from 1 to 3600.");
            error.set_visible(true);
            return;
        };
        gio::prelude::ActionGroupExt::activate_action(
            &w,
            "slideshow-interval",
            Some(&seconds.to_variant()),
        );
        if let Some(p) = weak_popover.upgrade() {
            p.popdown();
        }
    });
    let pressed = submit.clone();
    apply.connect_clicked(move |_| pressed());
    entry.connect_activate(move |_| submit());
    let weak = error.downgrade();
    entry.connect_changed(move |entry| {
        entry.remove_css_class("error");
        if let Some(error) = weak.upgrade() {
            error.set_visible(false);
        }
    });
    let (weak_button, weak_entry) = (button.downgrade(), entry.downgrade());
    interval.connect_notify_local(Some("state"), move |action, _| {
        if let Some(seconds) = action.state().and_then(|v| v.get::<u32>()) {
            if let Some(button) = weak_button.upgrade() {
                button.set_label(&format!("{seconds} s"));
            }
            if let Some(entry) = weak_entry.upgrade() {
                entry.set_text(&seconds.to_string());
            }
        }
    });
    group.append(&button);

    let model = gio::Menu::new();
    for effect in [Effect::Fade, Effect::None] {
        let item = gio::MenuItem::new(Some(effect.label()), None);
        item.set_action_and_target_value(
            Some("win.slideshow-transition"),
            Some(&effect.id().to_variant()),
        );
        model.append_item(&item);
    }
    let effect = window
        .lookup_action("slideshow-transition")
        .expect("transition action");
    let selected = effect
        .state()
        .and_then(|v| v.str().and_then(Effect::from_id))
        .unwrap_or_default();
    let effect_button = gtk::MenuButton::builder()
        .label(selected.label())
        .tooltip_text("Photo transition")
        .menu_model(&model)
        .build();
    effect_button.set_widget_name("slideshow-transition-selector");
    let weak = effect_button.downgrade();
    effect.connect_notify_local(Some("state"), move |action, _| {
        if let (Some(button), Some(effect)) = (
            weak.upgrade(),
            action
                .state()
                .and_then(|v| v.str().and_then(Effect::from_id)),
        ) {
            button.set_label(effect.label());
        }
    });
    group.append(&effect_button);
    group
}
