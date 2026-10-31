//! Each window owns its ordered catalog, keyboard actions and cancellable view.

use crate::{image_view::ImageView, input::OpenedFile};
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use pivra::{APP_NAME, catalog::MediaCollection, discovery, media::MediaKind, settings::Settings};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

#[path = "fullscreen.rs"]
pub(super) mod fullscreen;

struct FileContext {
    collection: MediaCollection,
    suffix: Option<String>,
    warning: bool,
    revision: u64,
}

// Identify the exact newly trashed entry rather than restoring an unrelated
// older item with the same original path.
fn trash_entries(original: &std::path::Path) -> Vec<gio::File> {
    let root = gio::File::for_uri("trash:///");
    let Ok(enumerator) = root.enumerate_children(
        "standard::name,trash::orig-path",
        gio::FileQueryInfoFlags::NONE,
        None::<&gio::Cancellable>,
    ) else {
        return vec![];
    };
    let mut entries = vec![];
    while let Ok(Some(info)) = enumerator.next_file(None::<&gio::Cancellable>) {
        if info
            .attribute_byte_string("trash::orig-path")
            .is_some_and(|value| std::path::Path::new(value.as_str()) == original)
        {
            entries.push(enumerator.child(&info));
        }
    }
    entries
}

fn update_title(window: &adw::ApplicationWindow, title: &adw::WindowTitle, context: &FileContext) {
    update_navigation_actions(window, context);
    let Some(selected) = context.collection.current() else {
        window.set_title(Some("No media selected — Pivra"));
        title.set_title("No media selected");
        title.set_subtitle("Use arrow keys to select remaining media.");
        return;
    };
    let name = selected
        .path()
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let suffix = context
        .suffix
        .as_ref()
        .map(|s| format!(" ({s})"))
        .unwrap_or_default();
    window.set_title(Some(&format!("{name} — {APP_NAME}{suffix}")));
    title.set_title(&name);
    if selected.kind() == MediaKind::Image {
        let selected_index = context.collection.selected_index().unwrap_or(0);
        let mut photo_number = 0;
        let mut photo_total = 0;
        for (index, item) in context.collection.items().iter().enumerate() {
            if item.kind() == MediaKind::Image {
                photo_total += 1;
                if index <= selected_index {
                    photo_number += 1;
                }
            }
        }
        title.set_subtitle(&format!("{photo_number} / {photo_total}"));
    } else {
        title.set_subtitle("");
    }
}

// Keep the on-image arrow buttons in sync with the same catalog actions as
// the Left/Right keyboard shortcuts, including asynchronous folder updates.
fn update_navigation_actions(window: &adw::ApplicationWindow, context: &FileContext) {
    for (name, enabled) in [
        (
            "previous-media",
            context
                .collection
                .selected_index()
                .map_or(!context.collection.items().is_empty(), |i| i > 0),
        ),
        (
            "next-media",
            context
                .collection
                .selected_index()
                .map_or(!context.collection.items().is_empty(), |i| {
                    i + 1 < context.collection.items().len()
                }),
        ),
    ] {
        if let Some(action) = window
            .lookup_action(name)
            .and_then(|action| action.downcast::<gio::SimpleAction>().ok())
        {
            action.set_enabled(enabled);
        }
    }
}

fn install_navigation_arrows(overlay: &gtk::Overlay) {
    let css = gtk::CssProvider::new();
    css.load_from_string(
        ".pivra-navigation-glass { background-image: none; background-color: alpha(@window_bg_color, 0.50); border: 0; border-radius: 0; box-shadow: none; padding: 0; min-width: 0; min-height: 0; color: @window_fg_color; } .pivra-navigation-glass image { -gtk-icon-size: 48px; } .pivra-navigation-glass:hover { background-color: alpha(@window_bg_color, 0.56); }"
    );
    gtk::style_context_add_provider_for_display(
        &overlay.display(),
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    // Draw open-wing chevrons explicitly: GTK symbolic icons have a fixed,
    // narrower silhouette regardless of their requested icon size.
    let make_arrow = |left: bool, action: &str, align: gtk::Align| {
        let button = gtk::Button::new();
        button.add_css_class("pivra-navigation-glass");
        button.set_tooltip_text(Some(if left {
            "Önceki fotoğraf"
        } else {
            "Sonraki fotoğraf"
        }));
        let arrow = gtk::DrawingArea::new();
        arrow.set_content_width(32);
        arrow.set_content_height(48);
        arrow.set_can_target(false);
        arrow.set_draw_func(move |_, cr, width, height| {
            // Preserve the 4K/fullscreen silhouette; scale only down in a
            // smaller viewport, including the chevron stroke and wing length.
            let scale = (f64::from(width) / 112.0)
                .min(f64::from(height) / 900.0)
                .clamp(0.35, 1.0)
                * 0.5;
            let cx = f64::from(width) / 2.0;
            let cy = f64::from(height) / 2.0;
            let tip_x = if left {
                cx - 32.0 * scale
            } else {
                cx + 32.0 * scale
            };
            let ends_x = if left {
                cx + 32.0 * scale
            } else {
                cx - 32.0 * scale
            };
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.94);
            cr.set_line_width(5.0 * scale);
            cr.set_line_cap(gtk::cairo::LineCap::Round);
            cr.set_line_join(gtk::cairo::LineJoin::Round);
            cr.move_to(ends_x, cy - 105.0 * scale);
            cr.line_to(tip_x, cy);
            cr.line_to(ends_x, cy + 105.0 * scale);
            let _ = cr.stroke();
        });
        button.set_child(Some(&arrow));
        button.set_action_name(Some(action));
        button.set_halign(align);
        button.set_valign(gtk::Align::Fill);
        button.set_size_request(80, -1);
        button.set_focus_on_click(false);
        button.set_visible(false);
        overlay.add_overlay(&button);
        overlay.set_measure_overlay(&button, false);
        overlay.set_clip_overlay(&button, true);
        button
    };
    let previous = make_arrow(true, "win.previous-media", gtk::Align::Start);
    let next = make_arrow(false, "win.next-media", gtk::Align::End);

    // Keep full-height navigation panels out of the thumbnail strip. The strip
    // can be shown, hidden and resized independently after arrow creation.
    let mut sibling = overlay.first_child();
    while let Some(widget) = sibling {
        if widget.widget_name() == "thumbnail-strip" {
            if let Ok(strip) = widget.downcast::<gtk::ScrolledWindow>() {
                let left = previous.downgrade();
                let right = next.downgrade();
                let update = move |strip: &gtk::ScrolledWindow| {
                    let bottom = if strip.is_visible() {
                        strip.height_request().max(0)
                    } else {
                        0
                    };
                    for arrow in [&left, &right] {
                        if let Some(arrow) = arrow.upgrade() {
                            arrow.set_margin_bottom(bottom);
                        }
                    }
                };
                update(&strip);
                let update = std::rc::Rc::new(update);
                let visibility = update.clone();
                strip.connect_notify_local(Some("visible"), move |strip, _| visibility(strip));
                strip.connect_notify_local(Some("height-request"), move |strip, _| update(strip));
            }
            break;
        }
        sibling = widget.next_sibling();
    }
    // GTK reports logical viewport size after allocation (including HiDPI).
    // Resize the glass panels as one piece with their arrows; no increase
    // beyond the reduced 80px fullscreen width.
    let prev_size = previous.downgrade();
    let next_size = next.downgrade();
    let resize = move |viewport: &gtk::Overlay| {
        let scale = (f64::from(viewport.width()) / 1600.0)
            .min(f64::from(viewport.height()) / 900.0)
            .clamp(0.5, 1.0);
        let panel_width = (80.0 * scale).round() as i32;
        for panel in [&prev_size, &next_size] {
            if let Some(panel) = panel.upgrade() {
                if panel.width_request() != panel_width {
                    panel.set_width_request(panel_width);
                }
            }
        }
    };
    let resize = std::rc::Rc::new(resize);
    let on_width = resize.clone();
    overlay.connect_notify_local(Some("width"), move |viewport, _| on_width(viewport));
    overlay.connect_notify_local(Some("height"), move |viewport, _| resize(viewport));
    let motion = gtk::EventControllerMotion::new();
    let left = previous.downgrade();
    let right = next.downgrade();
    motion.connect_motion(move |controller, x, _| {
        let Some(widget) = controller.widget() else {
            return;
        };
        let width = widget.width() as f64;
        if let Some(button) = left.upgrade() {
            let edge = f64::from(button.width_request()) * 1.55;
            button.set_visible(x <= edge && button.is_sensitive());
        }
        if let Some(button) = right.upgrade() {
            let edge = f64::from(button.width_request()) * 1.55;
            button.set_visible(x >= width - edge && button.is_sensitive());
        }
    });
    motion.connect_leave(move |_| {
        previous.set_visible(false);
        next.set_visible(false);
    });
    overlay.add_controller(motion);
}

fn update_thumbnail_catalog(view: &ImageView, context: &FileContext) {
    let photos: Vec<_> = context
        .collection
        .items()
        .iter()
        .enumerate()
        .map(|(index, item)| (index, item.path().to_owned()))
        .collect();
    view.set_thumbnail_catalog(&photos, context.collection.selected_index());
}

fn display_selected(view: &ImageView, context: &FileContext) {
    update_thumbnail_catalog(view, context);
    let Some(item) = context.collection.current() else {
        view.set_neighbors(vec![]);
        view.show_message("No media selected.");
        return;
    };
    if item.kind() == MediaKind::Video {
        view.set_neighbors(vec![]);
        view.load_video(item.path());
    } else if pivra::discovery::supported_photo(item.path()) {
        view.load(item.path().to_owned());
        view.set_neighbors(context.collection.photo_neighbors());
    } else {
        view.set_neighbors(vec![]);
        view.show_message("This image format is not supported.");
    }
}

pub(super) fn build(app: &adw::Application, settings: &Settings, file: Option<OpenedFile>) {
    let header = adw::HeaderBar::new();
    header.set_widget_name("pivra-header");
    let title = adw::WindowTitle::new(APP_NAME, "");
    header.set_title_widget(Some(&title));
    let viewport = gtk::Box::new(gtk::Orientation::Vertical, 0);
    viewport.set_hexpand(true);
    viewport.set_vexpand(true);
    let placeholder = gtk::Label::new(None);
    placeholder.set_wrap(true);
    placeholder.set_valign(gtk::Align::Center);
    placeholder.set_vexpand(true);
    // Folder scan diagnostics are reflected in the window title and image errors.
    // Keep the label for FolderWatch callbacks, but never allocate a bottom status bar.
    placeholder.set_visible(false);
    if file.is_some() {
        placeholder.set_text("Discovering neighboring media…");
        viewport.append(&placeholder);
    }
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&header);
    content.append(&viewport);
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_width(settings.window_width)
        .default_height(settings.window_height)
        .content(&content)
        .build();
    pivra::settings::window_state::install(&window, settings);
    if let Some(opened) = file {
        let context = Rc::new(RefCell::new(FileContext {
            collection: MediaCollection::new(vec![opened.item.clone()]),
            suffix: None,
            warning: false,
            revision: 0,
        }));
        let banner = gtk::Revealer::new();
        let notice = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        notice.set_halign(gtk::Align::Center);
        notice.append(&gtk::Label::new(Some("Dosya değiştirildi — Yenile")));
        let refresh_button = gtk::Button::with_label("Yenile");
        notice.append(&refresh_button);
        banner.set_child(Some(&notice));
        viewport.prepend(&banner);
        let view = ImageView::new(&window);
        let picture = view
            .widget()
            .child()
            .unwrap()
            .downcast::<gtk::Picture>()
            .unwrap();
        fullscreen::install(
            &window,
            &picture,
            view.video_widget(),
            &[header.clone().upcast(), banner.clone().upcast()],
        );
        let menu = gio::Menu::new();
        for mode in pivra::viewing::ViewMode::ALL {
            let item = gio::MenuItem::new(Some(mode.label()), None);
            item.set_action_and_target_value(Some("win.image-mode"), Some(&mode.id().to_variant()));
            menu.append_item(&item);
        }
        let quality = gio::Menu::new();
        for filter in pivra::viewing::Resampling::ALL {
            let item = gio::MenuItem::new(Some(filter.label()), None);
            item.set_action_and_target_value(
                Some("win.image-resampling"),
                Some(&filter.id().to_variant()),
            );
            quality.append_item(&item);
        }
        menu.append_section(Some(pivra::viewing::Resampling::MENU), &quality);
        let mode_button = gtk::MenuButton::builder()
            .label(pivra::settings::load_view_mode().label())
            .tooltip_text(pivra::viewing::ViewMode::HELP)
            .menu_model(&menu)
            .build();
        header.pack_end(&mode_button);
        // Wheel mode is per-window; the lens is visibly active only for zoom.
        let zoom_toggle = gtk::ToggleButton::builder()
            .tooltip_text("Wheel zoom (off: wheel changes photos; hold left mouse button to zoom)")
            .build();
        // Draw a literal magnifying glass instead of relying on a theme glyph
        // which can appear as a plus sign.
        let lens = gtk::DrawingArea::new();
        lens.set_content_width(22);
        lens.set_content_height(22);
        lens.set_can_target(false);
        lens.set_draw_func(|_, cr, width, height| {
            let cx = f64::from(width) * 0.44;
            let cy = f64::from(height) * 0.42;
            let radius = f64::from(width.min(height)) * 0.24;
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.94);
            cr.set_line_width(2.1);
            cr.arc(cx, cy, radius, 0.0, std::f64::consts::TAU);
            let _ = cr.stroke();
            cr.move_to(cx + radius * 0.72, cy + radius * 0.72);
            cr.line_to(cx + radius * 1.65, cy + radius * 1.65);
            let _ = cr.stroke();
        });
        zoom_toggle.set_child(Some(&lens));
        zoom_toggle.set_active(false);
        // Use the GNOME accent color only while wheel zoom is enabled.
        zoom_toggle.connect_toggled(|button| {
            if button.is_active() {
                button.add_css_class("suggested-action");
            } else {
                button.remove_css_class("suggested-action");
            }
        });
        header.pack_end(&zoom_toggle);
        let zoom_for_key = zoom_toggle.clone();
        let zoom_key = gtk::EventControllerKey::new();
        zoom_key.set_propagation_phase(gtk::PropagationPhase::Capture);
        zoom_key.connect_key_pressed(move |_, key, _, modifiers| {
            if !matches!(key, gdk::Key::z | gdk::Key::Z)
                || modifiers.intersects(
                    gdk::ModifierType::CONTROL_MASK
                        | gdk::ModifierType::ALT_MASK
                        | gdk::ModifierType::SUPER_MASK
                        | gdk::ModifierType::META_MASK,
                )
            {
                return glib::Propagation::Proceed;
            }
            zoom_for_key.set_active(!zoom_for_key.is_active());
            glib::Propagation::Stop
        });
        window.add_controller(zoom_key);
        let zoom_view = view.clone();
        zoom_toggle.connect_toggled(move |button| {
            zoom_view.set_zoom_wheel_enabled(button.is_active());
            button.set_tooltip_text(Some(if button.is_active() {
                "Wheel zoom enabled"
            } else {
                "Wheel changes photos; hold left mouse button to zoom"
            }));
        });
        let weak_button = mode_button.downgrade();
        let action = window
            .lookup_action("image-mode")
            .expect("view mode action");
        action.connect_notify_local(Some("state"), move |action, _| {
            if let Some(button) = weak_button.upgrade() {
                let state = action.state();
                let id = state.as_ref().and_then(|v| v.str()).unwrap_or("fit");
                button.set_label(if id == "manual" {
                    pivra::viewing::ViewMode::MANUAL_LABEL
                } else {
                    pivra::viewing::ViewMode::from_id(id)
                        .unwrap_or_default()
                        .label()
                });
            }
        });
        // Thumbnail clicks select the same catalog indices used by keyboard navigation.
        let select_context = context.clone();
        let select_view = view.clone();
        let select_title = title.clone();
        let select_banner = banner.clone();
        let select_window = window.downgrade();
        let teardown_view = view.clone();
        window.connect_destroy(move |_| teardown_view.clear_thumbnail_select());
        let close_view = view.clone();
        window.connect_close_request(move |_| {
            close_view.clear_thumbnail_select();
            glib::Propagation::Proceed
        });
        view.on_thumbnail_select(move |index| {
            let diagnose = std::env::var_os("PIVRA_TRACE_CLICKS").is_some_and(|value| value == "1");
            if diagnose {
                eprintln!("[pivra-click] window-received index={index}");
            }
            let Some(window) = select_window.upgrade() else {
                if diagnose {
                    eprintln!("[pivra-click] window-gone");
                }
                return;
            };
            let mut context = select_context.borrow_mut();
            if context.collection.selected_index() == Some(index) {
                if diagnose {
                    eprintln!("[pivra-click] already-selected index={index}");
                }
                return;
            }
            if context.collection.select(index) {
                if diagnose {
                    eprintln!("[pivra-click] selected index={index}");
                }
                context.revision = context.revision.wrapping_add(1);
                select_banner.set_reveal_child(false);
                update_title(&window, &select_title, &context);
                display_selected(&select_view, &context);
            }
        });
        install_navigation_arrows(view.widget());
        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_child(Some(view.widget()));
        viewport.prepend(&toast_overlay);
        placeholder.set_vexpand(false);
        update_title(&window, &title, &context.borrow());
        display_selected(&view, &context.borrow());
        for (name, forward) in [("previous-media", false), ("next-media", true)] {
            let action = gio::SimpleAction::new(name, None);
            let weak_window = window.downgrade();
            let title = title.clone();
            let context = context.clone();
            let view = view.clone();
            let banner = banner.clone();
            action.connect_activate(move |_, _| {
                let Some(window) = weak_window.upgrade() else {
                    return;
                };
                let mut context = context.borrow_mut();
                let changed = if forward {
                    context.collection.select_next()
                } else {
                    context.collection.select_previous()
                };
                if changed {
                    context.revision = context.revision.wrapping_add(1);
                    banner.set_reveal_child(false);
                    update_title(&window, &title, &context);
                    display_selected(&view, &context);
                }
            });
            window.add_action(&action);
        }
        update_navigation_actions(&window, &context.borrow());

        // Delete moves the selected photo or video to the system Trash.
        // Undo is bound to the specific new trash entry, never a path guess.
        let delete = gio::SimpleAction::new("trash-selected", None);
        let trash_context = context.clone();
        let trash_view = view.clone();
        let trash_title = title.clone();
        let trash_banner = banner.clone();
        let trash_window = window.downgrade();
        let toasts = toast_overlay.clone();
        delete.connect_activate(move |_, _| {
            let Some(window) = trash_window.upgrade() else {
                return;
            };
            let mut context = trash_context.borrow_mut();
            let Some(selected) = context.collection.current().cloned() else {
                return;
            };
            let path = selected.path().to_owned();
            let old_index = context.collection.selected_index().unwrap_or(0);
            let old_entries = trash_entries(&path);
            let file = gio::File::for_path(&path);
            if let Err(error) = file.trash(None::<&gio::Cancellable>) {
                toasts.add_toast(adw::Toast::new(&format!(
                    "Could not move to Trash: {error}"
                )));
                return;
            }
            let remaining: Vec<_> = context
                .collection
                .items()
                .iter()
                .filter(|item| item.path() != path)
                .cloned()
                .collect();
            context.collection = MediaCollection::new(remaining);
            if !context.collection.items().is_empty() {
                let last_index = context.collection.items().len() - 1;
                context.collection.select(old_index.min(last_index));
            }
            context.revision = context.revision.wrapping_add(1);
            trash_banner.set_reveal_child(false);
            update_title(&window, &trash_title, &context);
            display_selected(&trash_view, &context);
            drop(context);

            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let toast = adw::Toast::new(&format!("Moved to Trash: {name}"));
            // The Trash backend may update its GIO directory asynchronously.
            // Always expose Undo and resolve the new entry when clicked, not
            // immediately after trash() returns.
            toast.set_button_label(Some("Undo"));
            let undo_toasts = toasts.clone();
            toast.connect_button_clicked(move |_| {
                if path.exists() {
                    undo_toasts
                        .add_toast(adw::Toast::new("Undo failed: destination already exists"));
                    return;
                }
                let trashed = trash_entries(&path)
                    .into_iter()
                    .find(|entry| !old_entries.iter().any(|old| old.equal(entry)));
                let Some(trashed) = trashed else {
                    undo_toasts.add_toast(adw::Toast::new("Undo failed: trashed file not found"));
                    return;
                };
                if let Err(error) = trashed.move_(
                    &gio::File::for_path(&path),
                    gio::FileCopyFlags::NONE,
                    None::<&gio::Cancellable>,
                    None,
                ) {
                    undo_toasts.add_toast(adw::Toast::new(&format!("Undo failed: {error}")));
                }
            });
            toasts.add_toast(toast);
        });
        window.add_action(&delete);
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak_window = window.downgrade();
        let video_keys = view.clone();
        keys.connect_key_pressed(move |_, key, _, modifiers| {
            if !modifiers.intersects(gdk::ModifierType::ALT_MASK | gdk::ModifierType::SUPER_MASK)
                && video_keys.video_key(key, modifiers.contains(gdk::ModifierType::CONTROL_MASK))
            {
                return glib::Propagation::Stop;
            }
            if modifiers.intersects(
                gdk::ModifierType::CONTROL_MASK
                    | gdk::ModifierType::ALT_MASK
                    | gdk::ModifierType::SUPER_MASK,
            ) {
                return glib::Propagation::Proceed;
            }
            if key == gdk::Key::Delete {
                if let Some(window) = weak_window.upgrade() {
                    gio::prelude::ActionGroupExt::activate_action(&window, "trash-selected", None);
                }
                return glib::Propagation::Stop;
            }
            let mode = match key {
                gdk::Key::F6 => Some("fit"),
                gdk::Key::F7 => Some("fill"),
                gdk::Key::F8 => Some("actual"),
                _ => None,
            };
            if let Some(mode) = mode {
                if let Some(window) = weak_window.upgrade() {
                    gio::prelude::ActionGroupExt::activate_action(
                        &window,
                        "image-mode",
                        Some(&mode.to_variant()),
                    );
                }
                return glib::Propagation::Stop;
            }
            let name = match key {
                gdk::Key::Left => "previous-media",
                gdk::Key::Right => "next-media",
                _ => return glib::Propagation::Proceed,
            };
            if let Some(window) = weak_window.upgrade() {
                gio::prelude::ActionGroupExt::activate_action(&window, name, None);
            }
            glib::Propagation::Stop
        });
        window.add_controller(keys);
        let reload = gio::SimpleAction::new("reload-media", None);
        let refresh_context = context.clone();
        let refresh_view = view.clone();
        let refresh_banner = banner.clone();
        reload.connect_activate(move |_, _| {
            let context = refresh_context.borrow();
            let Some(item) = context.collection.current() else {
                return;
            };
            if item.kind() == MediaKind::Video || !pivra::discovery::supported_photo(item.path()) {
                refresh_banner.set_reveal_child(false);
                display_selected(&refresh_view, &context);
                return;
            }
            let path = item.path().to_owned();
            let revision = context.revision;
            drop(context);
            let context = refresh_context.clone();
            let banner = refresh_banner.downgrade();
            refresh_view.load_observed(
                path,
                || {},
                move |success| {
                    if success && context.borrow().revision == revision {
                        if let Some(banner) = banner.upgrade() {
                            banner.set_reveal_child(false);
                        }
                    }
                },
            );
        });
        window.add_action(&reload);
        let weak_window = window.downgrade();
        refresh_button.connect_clicked(move |_| {
            if let Some(window) = weak_window.upgrade() {
                gio::prelude::ActionGroupExt::activate_action(&window, "reload-media", None);
            }
        });
        window.present();
        watch(
            &window,
            &title,
            &placeholder,
            &banner,
            view,
            context,
            opened,
        );
    } else {
        window.present();
    }
}

// Only main-context callbacks own this state. Blocking scans receive paths and
// an atomic cancellation flag, never GTK objects. One scan and one timer maximum.
struct FolderWatch {
    window: glib::WeakRef<adw::ApplicationWindow>,
    title: glib::WeakRef<adw::WindowTitle>,
    status: glib::WeakRef<gtk::Label>,
    banner: glib::WeakRef<gtk::Revealer>,
    context: Rc<RefCell<FileContext>>,
    view: ImageView,
    directory: PathBuf,
    cancelled: Arc<AtomicBool>,
    monitor: RefCell<Option<gio::FileMonitor>>,
    timer: RefCell<Option<glib::SourceId>>,
    fallback_poll: RefCell<Option<glib::SourceId>>,
    running: Cell<bool>,
    dirty: Cell<bool>,
    initialized: Cell<bool>,
    link_monitors: RefCell<Vec<gio::FileMonitor>>,
    link_targets: RefCell<Vec<(PathBuf, PathBuf)>>,
}

impl FolderWatch {
    fn stop(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
        if let Some(timer) = self.timer.borrow_mut().take() {
            timer.remove();
        }
        if let Some(poll) = self.fallback_poll.borrow_mut().take() {
            poll.remove();
        }
        for monitor in self.link_monitors.borrow_mut().drain(..) {
            monitor.cancel();
        }
        if let Some(monitor) = self.monitor.borrow_mut().take() {
            monitor.cancel();
        }
    }

    fn watch_links(self: &Rc<Self>, targets: Vec<(PathBuf, PathBuf)>) {
        if *self.link_targets.borrow() == targets {
            return;
        }
        for monitor in self.link_monitors.borrow_mut().drain(..) {
            monitor.cancel();
        }
        self.link_targets.replace(targets.clone());
        // Directory events refer to target names, not aliases. Follow only file
        // candidate symlinks in the same lexical folder, including dangling aliases.
        for (alias, target) in targets {
            match gio::File::for_path(target).monitor_file(
                gio::FileMonitorFlags::WATCH_MOVES,
                None::<&gio::Cancellable>,
            ) {
                Ok(monitor) => {
                    let weak = Rc::downgrade(self);
                    monitor.connect_changed(move |_, _, _, event| {
                        let Some(state) = weak.upgrade() else {
                            return;
                        };
                        if state.cancelled.load(Ordering::Relaxed) {
                            return;
                        }
                        state
                            .view
                            .invalidate_cache_paths(std::slice::from_ref(&alias));
                        let mut context = state.context.borrow_mut();
                        if context
                            .collection
                            .current()
                            .is_some_and(|item| item.path() == alias)
                            && !matches!(
                                event,
                                gio::FileMonitorEvent::Deleted
                                    | gio::FileMonitorEvent::MovedOut
                                    | gio::FileMonitorEvent::Unmounted
                                    | gio::FileMonitorEvent::PreUnmount
                            )
                        {
                            context.revision = context.revision.wrapping_add(1);
                            if let Some(banner) = state.banner.upgrade() {
                                banner.set_reveal_child(true);
                            }
                        }
                        drop(context);
                        state.schedule();
                    });
                    self.link_monitors.borrow_mut().push(monitor);
                }
                Err(error) => eprintln!("{APP_NAME}: symlink target monitor unavailable: {error}"),
            }
        }
    }

    fn schedule(self: &Rc<Self>) {
        if self.cancelled.load(Ordering::Relaxed) {
            return;
        }
        self.dirty.set(true);
        if self.running.get() || self.timer.borrow().is_some() {
            return;
        }
        let state = self.clone();
        // Fixed trailing window: sustained events cannot postpone scans forever.
        let timer = glib::timeout_add_local_once(Duration::from_millis(120), move || {
            state.timer.borrow_mut().take();
            state.scan();
        });
        self.timer.replace(Some(timer));
    }

    fn scan(self: &Rc<Self>) {
        if self.cancelled.load(Ordering::Relaxed) {
            return;
        }
        self.running.set(true);
        self.dirty.set(false);
        let directory = self.directory.clone();
        let cancelled = self.cancelled.clone();
        let job = gio::spawn_blocking(move || {
            discovery::scan_directory(&directory, &cancelled).map(|report| {
                let mut links: Vec<_> = std::fs::read_dir(&directory)
                    .into_iter()
                    .flatten()
                    .take_while(|_| !cancelled.load(Ordering::Relaxed))
                    .filter_map(|entry| {
                        let path = entry.ok()?.path();
                        discovery::candidate_kind(&path)?;
                        if !std::fs::symlink_metadata(&path).ok()?.is_symlink() {
                            return None;
                        }
                        let target = std::fs::read_link(&path).ok()?;
                        let target = if target.is_absolute() {
                            target
                        } else {
                            directory.join(target)
                        };
                        let target = std::fs::canonicalize(&target).unwrap_or(target);
                        Some((path, target))
                    })
                    .collect();
                links.sort();
                (report, links)
            })
        });
        let state = self.clone();
        glib::MainContext::default().spawn_local(async move {
            let result = job.await.unwrap_or_else(|_| Err(std::io::Error::other("folder worker failed")));
            state.running.set(false);
            if state.cancelled.load(Ordering::Relaxed) { return; }
            if state.dirty.get() { state.schedule(); return; }
            let (Some(window), Some(title), Some(status)) =
                (state.window.upgrade(), state.title.upgrade(), state.status.upgrade()) else { return; };
            let mut context = state.context.borrow_mut();
            match result {
                Ok((report, links)) => {
                    state.watch_links(links);
                    let warnings = report.issues.len() + usize::from(state.monitor.borrow().is_none());
                    for issue in &report.issues {
                        eprintln!("{APP_NAME}: folder scan incomplete: {:?}: {}", issue.path, issue.error);
                    }
                    // A periodic fallback scan must not cancel thumbnail work
                    // when the directory's media catalog is unchanged.
                    let catalog_changed = context.collection.items().iter().map(|item| item.path())
                        .ne(report.collection.items().iter().map(|item| item.path()));
                    let missing = if catalog_changed {
                        context.collection.replace_preserving(report.collection.items().to_vec())
                    } else {
                        false
                    };
                    if missing {
                        context.revision = context.revision.wrapping_add(1);
                        state.view.show_message("Selected media was removed or is inaccessible. Use arrow keys to continue.");
                        if let Some(banner) = state.banner.upgrade() { banner.set_reveal_child(false); }
                    }
                    state.view.set_neighbors(context.collection.photo_neighbors());
                    if catalog_changed {
                        update_thumbnail_catalog(&state.view, &context);
                        state.view.invalidate_thumbnails();
                    }
                    state.view.finish_initial_thumbnail_catalog(context.collection.selected_index());
                    let count = context.collection.items().len();
                    let suffix = if warnings == 0 {
                        format!("{count} {}", if count == 1 { "candidate" } else { "candidates" })
                    } else { format!("{count} {}, {warnings} {}", if count == 1 { "candidate" } else { "candidates" }, if warnings == 1 { "warning" } else { "warnings" }) };
                    status.set_text(&if state.monitor.borrow().is_none() {
                        format!("{suffix}. Folder monitoring unavailable; reopen to update.")
                    } else { format!("{suffix}.") });
                    context.suffix = Some(suffix);
                    context.warning = warnings != 0;
                }
                Err(error) => {
                    eprintln!("{APP_NAME}: folder scan failed: {error}");
                    if state.initialized.get() {
                        context.collection.replace_preserving(vec![]);
                    }
                    context.suffix = Some("folder scan failed".into());
                    context.warning = true;
                    update_thumbnail_catalog(&state.view, &context);
                    // Revalidate the reconciled catalog: rebuilding the model may
                    // cancel an older metadata epoch (e.g. replace + delete).
                    state.view.invalidate_thumbnails();
                    if state.initialized.get() {
                        state.view.show_message("Folder unavailable. Media cannot be accessed.");
                    }
                    if let Some(banner) = state.banner.upgrade() { banner.set_reveal_child(false); }
                    status.set_text("Folder scan failed. Folder unavailable.");
                }
            }
            state.initialized.set(true);
            update_title(&window, &title, &context);
        });
    }
}

fn watch(
    window: &adw::ApplicationWindow,
    title: &adw::WindowTitle,
    status: &gtk::Label,
    banner: &gtk::Revealer,
    view: ImageView,
    context: Rc<RefCell<FileContext>>,
    opened: OpenedFile,
) {
    let directory = opened
        .item
        .path()
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_owned();
    let state = Rc::new(FolderWatch {
        window: window.downgrade(),
        title: title.downgrade(),
        status: status.downgrade(),
        banner: banner.downgrade(),
        context,
        view,
        directory: directory.clone(),
        cancelled: Arc::new(AtomicBool::new(false)),
        monitor: RefCell::new(None),
        timer: RefCell::new(None),
        fallback_poll: RefCell::new(None),
        running: Cell::new(false),
        dirty: Cell::new(false),
        initialized: Cell::new(false),
        link_monitors: RefCell::new(vec![]),
        link_targets: RefCell::new(vec![]),
    });
    match gio::File::for_path(&directory).monitor_directory(
        gio::FileMonitorFlags::WATCH_MOVES,
        None::<&gio::Cancellable>,
    ) {
        Ok(monitor) => {
            let weak = Rc::downgrade(&state);
            monitor.connect_changed(move |_, file, other, event| {
                let Some(state) = weak.upgrade() else {
                    return;
                };
                if state.cancelled.load(Ordering::Relaxed) {
                    return;
                }
                let path = file.path();
                let affected: Vec<_> = path
                    .iter()
                    .cloned()
                    .chain(other.and_then(|f| f.path()))
                    .collect();
                state.view.invalidate_cache_paths(&affected);
                let mut retarget = None;
                let mut context = state.context.borrow_mut();
                let selected = context
                    .collection
                    .current()
                    .map(|item| item.path().to_owned());
                if matches!(
                    event,
                    gio::FileMonitorEvent::Renamed | gio::FileMonitorEvent::Moved
                ) {
                    if let (Some(old), Some(new)) =
                        (path.as_ref(), other.and_then(|file| file.path()))
                    {
                        if selected.as_ref() == Some(old) {
                            retarget = Some((old.clone(), new.clone()));
                        }
                        context.collection.rename_selected(old, &new);
                    }
                }
                let changed = matches!(
                    event,
                    gio::FileMonitorEvent::Changed
                        | gio::FileMonitorEvent::ChangesDoneHint
                        | gio::FileMonitorEvent::AttributeChanged
                        | gio::FileMonitorEvent::Created
                        | gio::FileMonitorEvent::MovedIn
                        | gio::FileMonitorEvent::Renamed
                        | gio::FileMonitorEvent::Moved
                );
                if changed
                    && selected.as_ref().is_some_and(|selected| {
                        path.as_ref() == Some(selected)
                            || other.and_then(|file| file.path()).as_ref() == Some(selected)
                    })
                {
                    if let Some(banner) = state.banner.upgrade() {
                        context.revision = context.revision.wrapping_add(1);
                        banner.set_reveal_child(true);
                    }
                }
                drop(context);
                if let Some((old, new)) = retarget {
                    state.view.retarget_pending(&old, &new);
                }
                state.schedule();
            });
            state.monitor.replace(Some(monitor));
        }
        Err(error) => {
            eprintln!("{APP_NAME}: folder monitor unavailable: {error}");
            status
                .set_text("Folder monitoring unavailable; reopen the file to update its catalog.");
        }
    }
    let closing = state.clone();
    window.connect_close_request(move |_| {
        closing.stop();
        glib::Propagation::Proceed
    });
    let destroying = state.clone();
    window.connect_destroy(move |_| destroying.stop());
    // Some backends do not deliver directory notifications reliably.
    // Periodically reconcile the catalog as a fallback while this window lives.
    let weak = Rc::downgrade(&state);
    let poll = glib::timeout_add_local(Duration::from_secs(5), move || {
        let Some(state) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        if state.cancelled.load(Ordering::Relaxed) {
            return glib::ControlFlow::Break;
        }
        if !state.running.get() {
            state.schedule();
        }
        glib::ControlFlow::Continue
    });
    state.fallback_poll.replace(Some(poll));
    state.scan();
}
