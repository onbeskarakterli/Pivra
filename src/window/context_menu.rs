//! Native viewport menu; existing window actions remain the operation owners.
use super::{FileContext, show_latest_toast, slideshow};
use crate::image_view::ImageView;
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use pivra::{media::MediaKind, viewing::ViewMode};
use std::{cell::RefCell, rc::Rc};

fn model(window: &adw::ApplicationWindow, video: bool) -> gio::Menu {
    let menu = gio::Menu::new();
    let files = gio::Menu::new();
    for (label, action) in [
        ("Open With…", "win.open-with"),
        ("Show in Files", "win.show-in-files"),
        ("Rename", "win.rename-media"),
    ] {
        files.append(Some(label), Some(action));
    }
    menu.append_section(None, &files);
    let viewing = gio::Menu::new();
    if video {
        viewing.append(Some("Play / Pause"), Some("win.context-playback"));
    }
    viewing.append(Some("Zoom"), Some("win.context-zoom"));
    let modes = gio::Menu::new();
    for mode in ViewMode::ALL {
        let item = gio::MenuItem::new(Some(mode.label()), None);
        item.set_action_and_target_value(Some("win.image-mode"), Some(&mode.id().to_variant()));
        modes.append_item(&item);
    }
    viewing.append_submenu(Some("Display Mode"), &modes);
    viewing.append(Some("Fullscreen"), Some("win.context-fullscreen"));
    viewing.append(
        Some(if slideshow::active(window) {
            "Stop Slideshow"
        } else {
            "Start Slideshow"
        }),
        Some("win.context-slideshow"),
    );
    menu.append_section(None, &viewing);
    let interface = gio::Menu::new();
    interface.append(Some("Thumbnail Strip"), Some("win.context-thumbnails"));
    interface.append(Some("Media Info"), Some("win.media-info"));
    menu.append_section(None, &interface);
    let destructive = gio::Menu::new();
    destructive.append(Some("Move to Trash"), Some("win.trash-selected"));
    menu.append_section(None, &destructive);
    menu
}

pub(super) fn install(
    window: &adw::ApplicationWindow,
    context: Rc<RefCell<FileContext>>,
    view: &ImageView,
    zoom: &gtk::ToggleButton,
    toasts: &adw::ToastOverlay,
    current_toast: Rc<RefCell<Option<adw::Toast>>>,
) {
    // FileLauncher uses the platform's GIO/portal application chooser and
    // FileManager integration. It never changes the default association.
    for (name, open_with) in [("open-with", true), ("show-in-files", false)] {
        let action = gio::SimpleAction::new(name, None);
        let weak = window.downgrade();
        let selected = context.clone();
        let toasts = toasts.downgrade();
        let current = current_toast.clone();
        let pending = Rc::new(RefCell::new(None::<gio::Cancellable>));
        let tokens = pending.clone();
        action.connect_activate(move |action, _| {
            let Some(window) = weak.upgrade().filter(|w| w.is_visible()) else {
                return;
            };
            let Some(path) = selected
                .borrow()
                .collection
                .current()
                .map(|i| i.path().to_owned())
            else {
                return;
            };
            let launcher = gtk::FileLauncher::new(Some(&gio::File::for_path(path)));
            launcher.set_always_ask(open_with);
            let cancel = gio::Cancellable::new();
            tokens.replace(Some(cancel.clone()));
            action.set_enabled(false);
            let action = action.downgrade();
            let tokens = tokens.clone();
            let toasts = toasts.clone();
            let current = current.clone();
            let done = move |result: Result<(), glib::Error>| {
                tokens.borrow_mut().take();
                if let Some(action) = action.upgrade() {
                    action.set_enabled(true);
                }
                if let Err(error) = result {
                    if error.matches(gtk::DialogError::Dismissed)
                        || error.matches(gtk::DialogError::Cancelled)
                        || error.matches(gio::IOErrorEnum::Cancelled)
                    {
                        return;
                    }
                    if let Some(toasts) = toasts.upgrade().filter(|w| w.is_mapped()) {
                        show_latest_toast(
                            &toasts,
                            &current,
                            adw::Toast::new(&format!(
                                "Could not {}: {error}",
                                if open_with {
                                    "open with another application"
                                } else {
                                    "show in Files"
                                }
                            )),
                        );
                    }
                }
            };
            if open_with {
                launcher.launch(Some(&window), Some(&cancel), done);
            } else {
                launcher.open_containing_folder(Some(&window), Some(&cancel), done);
            }
        });
        window.add_action(&action);
        let closing = pending.clone();
        window.connect_close_request(move |_| {
            if let Some(cancel) = closing.borrow_mut().take() {
                cancel.cancel();
            }
            glib::Propagation::Proceed
        });
        window.connect_destroy(move |_| {
            if let Some(cancel) = pending.borrow_mut().take() {
                cancel.cancel();
            }
        });
    }
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

    let photo = view.widget().child().expect("photo viewport");
    for (widget, video) in [(photo, false), (view.video_widget().clone().upcast(), true)] {
        let popover = gtk::PopoverMenu::from_model(None::<&gio::Menu>);
        popover.set_widget_name("media-context-menu");
        popover.set_has_arrow(false);
        popover.set_parent(&widget);
        let click = gtk::GestureClick::new();
        click.set_button(3);
        click.set_propagation_phase(gtk::PropagationPhase::Capture);
        let menu = popover.downgrade();
        let selected = context.clone();
        let weak = window.downgrade();
        // A controller belongs to the Picture; it must not own its ImageView.
        let player = Rc::downgrade(&player);
        let playback = playback.downgrade();
        click.connect_pressed(move |gesture, _, x, y| {
            let Some(window) = weak.upgrade() else {
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
            let Some(menu) = menu.upgrade() else {
                return;
            };
            if let Some(playback) = playback.upgrade() {
                playback.set_enabled(
                    video
                        && player
                            .upgrade()
                            .and_then(|p| p.current_video())
                            .is_some_and(|m| m.is_prepared() && m.error().is_none()),
                );
            }
            menu.set_menu_model(Some(&model(&window, video)));
            menu.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            gesture.set_state(gtk::EventSequenceState::Claimed);
            menu.popup();
        });
        widget.add_controller(click);
        let menu = popover.downgrade();
        window.connect_notify_local(Some("title"), move |_, _| {
            if let Some(menu) = menu.upgrade() {
                menu.popdown();
            }
        });
        // Keep Start/Stop wording current even when F5/Escape changes the session.
        if let Some(action) = window.lookup_action("slideshow") {
            let menu = popover.downgrade();
            let weak = window.downgrade();
            action.connect_notify_local(Some("state"), move |_, _| {
                if let (Some(menu), Some(window)) = (menu.upgrade(), weak.upgrade()) {
                    if menu.is_visible() {
                        menu.set_menu_model(Some(&model(&window, video)));
                    }
                }
            });
        }
        let menu = popover.downgrade();
        window.connect_close_request(move |_| {
            if let Some(menu) = menu.upgrade() {
                menu.popdown();
            }
            glib::Propagation::Proceed
        });
        window.connect_destroy(move |_| {
            popover.unparent();
        });
    }
}
