//! Native viewport menu; existing window actions remain the operation owners.
use super::{FileContext, slideshow};
use crate::image_view::ImageView;
use adw::prelude::*;
use gtk::{gdk, gio};
use pivra::media::MediaKind;
use std::{cell::RefCell, path::Path, rc::Rc};

#[path = "context_menu/rows.rs"]
mod rows;
#[path = "context_menu/submenu.rs"]
mod submenu;

// Selection identity, not title notifications, determines whether an open menu
// has become obsolete. Folder reconciliation also updates an unchanged title.
pub(super) fn selection_changed(window: &adw::ApplicationWindow, path: Option<&Path>) {
    if let Some(action) = window
        .lookup_action("context-media")
        .and_downcast::<gio::SimpleAction>()
    {
        let uri = path
            .map(|p| gio::File::for_path(p).uri().to_string())
            .unwrap_or_default()
            .to_variant();
        if action.state().as_ref() != Some(&uri) {
            action.set_state(&uri);
        }
    }
}

fn populate(
    root: &gtk::Popover,
    menus: &Rc<submenu::Menus>,
    window: &adw::ApplicationWindow,
    video: bool,
) {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let add =
        |label, action| content.append(&rows::action(window, menus, label, action, None, true));
    for (label, action) in [
        ("Open With…", "open-with"),
        ("Show in Files", "show-in-files"),
        ("Rename", "rename-media"),
    ] {
        add(label, action);
    }
    content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    if video {
        add("Play / Pause", "context-playback");
    }
    add("Zoom", "context-zoom");
    let (button, _, arrow) = rows::row("Display Mode", gtk::AccessibleRole::MenuItem);
    button.set_widget_name("display-mode-submenu-button");
    arrow.set_icon_name(Some("pan-end-symbolic"));
    arrow.set_opacity(1.);
    let child = gtk::Popover::builder()
        .has_arrow(false)
        .accessible_role(gtk::AccessibleRole::Menu)
        .build();
    child.set_widget_name("display-mode-submenu");
    child.add_css_class("menu");
    let modes = gtk::Box::new(gtk::Orientation::Vertical, 0);
    for mode in pivra::viewing::ViewMode::ALL {
        modes.append(&rows::action(
            window,
            menus,
            mode.label(),
            "image-mode",
            Some(mode.id()),
            false,
        ));
    }
    child.set_child(Some(&modes));
    menus.watch_parent(&button, &child);
    content.append(&button);
    add("Fullscreen", "context-fullscreen");
    let (label, name) = (
        if slideshow::active(window) {
            "Stop Slideshow"
        } else {
            "Start Slideshow"
        },
        "context-slideshow",
    );
    let slide = rows::action(window, menus, label, name, None, true);
    slide.set_widget_name("context-slideshow-row");
    content.append(&slide);
    content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    add("Thumbnail Strip", "context-thumbnails");
    add("Media Info", "media-info");
    content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    add("Move to Trash", "trash-selected");
    root.set_child(Some(&content));
}

pub(super) fn dismiss(window: &adw::ApplicationWindow) {
    gio::prelude::ActionGroupExt::activate_action(window, "dismiss-context-menu", None);
}

pub(super) fn install(
    window: &adw::ApplicationWindow,
    context: Rc<RefCell<FileContext>>,
    view: &ImageView,
    zoom: &gtk::ToggleButton,
    toasts: &adw::ToastOverlay,
    current_toast: Rc<RefCell<Option<adw::Toast>>>,
) {
    let dismiss = gio::SimpleAction::new("dismiss-context-menu", None);
    window.add_action(&dismiss);
    super::application_chooser::install(window, context.clone(), toasts, current_toast.clone());
    super::reveal::install(window, context.clone(), toasts, current_toast.clone());
    let zoom_action =
        gio::SimpleAction::new_stateful("context-zoom", None, &zoom.is_active().to_variant());
    let button = zoom.downgrade();
    zoom_action.connect_activate(move |_, _| {
        if let Some(button) = button.upgrade() {
            button.emit_clicked();
        }
    });
    let state = zoom_action.downgrade();
    zoom.connect_active_notify(move |button| {
        if let Some(action) = state.upgrade() {
            action.set_state(&button.is_active().to_variant());
        }
    });
    window.add_action(&zoom_action);
    let fullscreen = gio::SimpleAction::new_stateful(
        "context-fullscreen",
        None,
        &window.is_fullscreen().to_variant(),
    );
    let weak = window.downgrade();
    fullscreen.connect_activate(move |_, _| {
        if let Some(window) = weak.upgrade() {
            gio::prelude::ActionGroupExt::activate_action(&window, "fullscreen", None);
        }
    });
    let state = fullscreen.downgrade();
    window.connect_fullscreened_notify(move |window| {
        if let Some(action) = state.upgrade() {
            action.set_state(&window.is_fullscreen().to_variant());
        }
    });
    window.add_action(&fullscreen);
    let slides = gio::SimpleAction::new("context-slideshow", None);
    let weak = window.downgrade();
    slides.connect_activate(move |_, _| {
        if let Some(window) = weak.upgrade() {
            gio::prelude::ActionGroupExt::activate_action(
                &window,
                if slideshow::active(&window) {
                    "slideshow-stop"
                } else {
                    "slideshow"
                },
                None,
            );
        }
    });
    window.add_action(&slides);
    let preference = pivra::settings::thumbnail_preference();
    let thumbs = gio::SimpleAction::new_stateful(
        "context-thumbnails",
        None,
        &preference.enabled().to_variant(),
    );
    let setting = preference.clone();
    thumbs.connect_activate(move |_, _| setting.set_enabled(!setting.enabled()));
    let state = thumbs.downgrade();
    let listener: Rc<dyn Fn(bool)> = Rc::new(move |enabled| {
        if let Some(action) = state.upgrade() {
            action.set_state(&enabled.to_variant());
        }
    });
    preference.subscribe(&listener);
    let listener = RefCell::new(Some(listener));
    window.connect_destroy(move |_| {
        listener.borrow_mut().take();
    });
    window.add_action(&thumbs);
    let playback = gio::SimpleAction::new("context-playback", None);
    let player = Rc::new(view.clone());
    let controls = player.clone();
    playback.connect_activate(move |_, _| {
        controls.video_key(gdk::Key::space, false);
    });
    window.add_action(&playback);

    let target = gio::SimpleAction::new_stateful("context-media", None, &"".to_variant());
    target.set_enabled(false);
    window.add_action(&target);
    selection_changed(
        window,
        context.borrow().collection.current().map(|i| i.path()),
    );
    let photo = view.widget().child().expect("photo viewport");
    for widget in [photo, view.video_widget().clone().upcast()] {
        let popover = gtk::Popover::builder()
            .has_arrow(false)
            .accessible_role(gtk::AccessibleRole::Menu)
            .build();
        popover.set_widget_name("media-context-menu");
        popover.add_css_class("menu");
        popover.set_parent(&widget);
        let menus = submenu::Menus::new(&popover, window);
        let closing = Rc::downgrade(&menus);
        dismiss.connect_activate(move |_, _| {
            if let Some(menus) = closing.upgrade() {
                menus.close();
            }
        });
        let click = gtk::GestureClick::new();
        click.set_button(3);
        click.set_propagation_phase(gtk::PropagationPhase::Capture);
        let menu = popover.downgrade();
        let state = Rc::downgrade(&menus);
        let selected = context.clone();
        let weak = window.downgrade();
        let player = Rc::downgrade(&player);
        let playback = playback.downgrade();
        click.connect_pressed(move |gesture, _, x, y| {
            let Some(window) = weak.upgrade().filter(|w| submenu::Menus::owner_active(w)) else {
                return;
            };
            let Some(video) = selected
                .borrow()
                .collection
                .current()
                .map(|i| i.kind() == MediaKind::Video)
            else {
                return;
            };
            let (Some(menu), Some(menus)) = (menu.upgrade(), state.upgrade()) else {
                return;
            };
            if menu.is_visible() {
                return;
            }
            if let Some(playback) = playback.upgrade() {
                playback.set_enabled(
                    video
                        && player
                            .upgrade()
                            .and_then(|p| p.current_video())
                            .is_some_and(|m| m.is_prepared() && m.error().is_none()),
                );
            }
            populate(&menu, &menus, &window, video);
            menu.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            gesture.set_state(gtk::EventSequenceState::Claimed);
            menu.popup();
        });
        widget.add_controller(click);
        let state = Rc::downgrade(&menus);
        target.connect_notify_local(Some("state"), move |_, _| {
            if let Some(menus) = state.upgrade() {
                menus.close();
            }
        });
        if let Some(action) = window.lookup_action("slideshow") {
            let menu = popover.downgrade();
            let weak = window.downgrade();
            action.connect_notify_local(Some("state"), move |_, _| {
                if let (Some(menu), Some(window)) = (menu.upgrade(), weak.upgrade()) {
                    if let Some(content) = menu.child() {
                        let mut row = content.first_child();
                        while let Some(widget) = row {
                            if widget.widget_name() == "context-slideshow-row" {
                                if let Some(label) = widget
                                    .first_child()
                                    .and_then(|content| content.first_child())
                                    .and_downcast::<gtk::Label>()
                                {
                                    let text = if slideshow::active(&window) {
                                        "Stop Slideshow"
                                    } else {
                                        "Start Slideshow"
                                    };
                                    label.set_label(text);
                                    widget
                                        .update_property(&[gtk::accessible::Property::Label(text)]);
                                }
                                break;
                            }
                            row = widget.next_sibling();
                        }
                    }
                }
            });
        }
    }
}
