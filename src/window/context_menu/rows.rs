//! One native button/label/indicator layout for every menu row.
use super::submenu::Menus;
use adw::prelude::*;
use gtk::gio;
use std::{cell::RefCell, rc::Rc};

pub(super) fn row(label: &str, role: gtk::AccessibleRole) -> (gtk::Button, gtk::Label, gtk::Image) {
    let button = gtk::Button::builder()
        .has_frame(false)
        .accessible_role(role)
        .build();
    button.add_css_class("flat");
    // GTK's documented menu-row class, shared by action and submenu rows.
    button.add_css_class("model");
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let text = gtk::Label::builder()
        .label(label)
        .xalign(0.)
        .hexpand(true)
        .build();
    // Keep the theme's family/size, using normal menu typography for every
    // row rather than GtkButton's heading-like weight on some Adwaita themes.
    let attributes = gtk::pango::AttrList::new();
    attributes.insert(gtk::pango::AttrInt::new_weight(gtk::pango::Weight::Normal));
    text.set_attributes(Some(&attributes));
    text.set_widget_name("context-menu-label");
    let indicator = gtk::Image::from_icon_name("object-select-symbolic");
    indicator.set_widget_name("context-menu-indicator");
    indicator.set_pixel_size(16);
    indicator.set_opacity(0.);
    content.append(&text);
    content.append(&indicator);
    button.set_child(Some(&content));
    button.update_property(&[gtk::accessible::Property::Label(label)]);
    (button, text, indicator)
}

pub(super) fn action(
    window: &adw::ApplicationWindow,
    menus: &Rc<Menus>,
    label: &str,
    name: &str,
    target: Option<&str>,
    root_row: bool,
) -> gtk::Button {
    let source = window.lookup_action(name).expect("existing menu action");
    let checked = target.is_some() || source.state().is_some_and(|v| v.get::<bool>().is_some());
    let role = if target.is_some() {
        gtk::AccessibleRole::MenuItemRadio
    } else if checked {
        gtk::AccessibleRole::MenuItemCheckbox
    } else {
        gtk::AccessibleRole::MenuItem
    };
    let (button, _, indicator) = row(label, role);
    button.set_widget_name(&format!("context-action-{}", target.unwrap_or(name)));
    if let Some(target) = target {
        button.set_action_target_value(Some(&target.to_variant()));
    }
    button.set_action_name(Some(&format!("win.{name}")));
    if checked {
        let weak = button.downgrade();
        let icon = indicator.downgrade();
        let target = target.map(str::to_owned);
        let update = move |source: &gio::Action| {
            let active = source.state().is_some_and(|v| {
                target
                    .as_ref()
                    .map_or_else(|| v.get::<bool>().unwrap_or(false), |t| v.str() == Some(t))
            });
            if let (Some(button), Some(icon)) = (weak.upgrade(), icon.upgrade()) {
                // Opacity preserves the same compact trailing allocation in
                // both states. No shared leading-indicator size group exists.
                icon.set_opacity(if active { 1. } else { 0. });
                button.update_state(&[gtk::accessible::State::Checked(if active {
                    gtk::AccessibleTristate::True
                } else {
                    gtk::AccessibleTristate::False
                })]);
            }
        };
        update(&source);
        let id = source.connect_notify_local(Some("state"), move |a, _| update(a));
        let source = source.downgrade();
        let id = RefCell::new(Some(id));
        button.connect_destroy(move |_| {
            if let (Some(source), Some(id)) = (source.upgrade(), id.borrow_mut().take()) {
                source.disconnect(id);
            }
        });
    }
    let weak = Rc::downgrade(menus);
    button.connect_clicked(move |_| {
        if let Some(menus) = weak.upgrade() {
            menus.close();
        }
    });
    if root_row {
        menus.watch_other(&button);
    }
    button
}
