//! Each window owns its ordered catalog, keyboard actions and cancellable view.

use crate::{image_view::ImageView, input::OpenedFile};
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use pivra::{APP_NAME, catalog::MediaCollection, media::MediaKind, settings::Settings};
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc};

#[path = "fullscreen.rs"]
pub(super) mod fullscreen;
#[path = "rename_dialog.rs"]
mod rename_dialog;
#[path = "slideshow_window.rs"]
pub(super) mod slideshow;
#[path = "toolbar.rs"]
mod toolbar;

#[path = "window/folder_watch.rs"]
mod folder_watch;
use folder_watch::{FolderWatch, watch};

// Keep one current notification, shared by Trash and slideshow status.
fn show_latest_toast(
    overlay: &adw::ToastOverlay,
    current: &Rc<RefCell<Option<adw::Toast>>>,
    toast: adw::Toast,
) {
    let previous = current.borrow_mut().take();
    if let Some(previous) = previous {
        previous.dismiss();
    }
    // NORMAL priority can enqueue a new toast behind an outgoing animation.
    // HIGH displays the newest result immediately instead of waiting.
    toast.set_priority(adw::ToastPriority::High);
    *current.borrow_mut() = Some(toast.clone());
    let weak = Rc::downgrade(current);
    toast.connect_dismissed(move |toast| {
        if let Some(current) = weak.upgrade() {
            let same = current.borrow().as_ref() == Some(toast);
            if same {
                current.borrow_mut().take();
            }
        }
    });
    overlay.add_toast(toast);
}

struct FileContext {
    own_rename: Option<(PathBuf, PathBuf)>,
    collection: MediaCollection,
    suffix: Option<String>,
    warning: bool,
    revision: u64,
}
fn editing(window: &adw::ApplicationWindow) -> bool {
    let mut focus = gtk::prelude::GtkWindowExt::focus(window);
    while let Some(widget) = focus {
        if widget.is_mapped() && (widget.is::<gtk::Editable>() || widget.is::<gtk::TextView>()) {
            return true;
        }
        focus = widget.parent();
    }
    false
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

fn load_last_open_folder() -> Option<gio::File> {
    let path = glib::user_config_dir().join("pivra/last-open-dir.conf");
    let folder = PathBuf::from(std::fs::read_to_string(path).ok()?.trim());
    folder.is_dir().then(|| gio::File::for_path(folder))
}

fn save_last_open_folder(folder: &std::path::Path) {
    let config = glib::user_config_dir().join("pivra");
    if let Err(error) = std::fs::create_dir_all(&config).and_then(|_| {
        std::fs::write(
            config.join("last-open-dir.conf"),
            folder.as_os_str().as_encoded_bytes(),
        )
    }) {
        eprintln!("Pivra: could not save last open folder: {error}");
    }
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
    let slideshow = slideshow::active(window) && !context.collection.items().is_empty();
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
            action.set_enabled(enabled || slideshow);
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
    // Always keep Open at the far left of the header bar, including empty windows.
    let open_button = gtk::Button::from_icon_name("document-open-symbolic");
    open_button.set_tooltip_text(Some("Open media (Ctrl+O)"));
    open_button.set_widget_name("open-media-button");
    open_button.set_action_name(Some("win.choose-media"));
    header.pack_start(&open_button);
    {
        let context = Rc::new(RefCell::new(FileContext {
            own_rename: None,
            collection: MediaCollection::new(
                file.iter().map(|opened| opened.item.clone()).collect(),
            ),
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
        let folder_watch = Rc::new(RefCell::new(None::<Rc<FolderWatch>>));
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
        mode_button.set_widget_name("display-mode-selector");
        let groups = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        groups.set_widget_name("viewer-controls");
        groups.set_visible(file.is_some());
        let display_group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        display_group.set_widget_name("display-controls");
        display_group.add_css_class("linked");
        display_group.append(&mode_button);
        groups.append(&display_group);
        header.pack_end(&groups);
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
        zoom_toggle.set_widget_name("wheel-zoom-toggle");
        // The same GNOME accent represents explicit wheel mode or manual zoom.
        zoom_toggle.connect_toggled(|button| {
            if button.is_active() {
                button.add_css_class("suggested-action");
            } else {
                button.remove_css_class("suggested-action");
            }
        });
        let zoom_group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        zoom_group.set_widget_name("zoom-controls");
        zoom_group.append(&zoom_toggle);
        groups.append(&zoom_group);
        let wheel_zoom = view.zoom_wheel_state();
        let action = window
            .lookup_action("image-mode")
            .expect("view mode action");
        let input_handler = Rc::new(std::cell::OnceCell::<glib::SignalHandlerId>::new());
        let sync_zoom = {
            let button = zoom_toggle.downgrade();
            let mode = action.downgrade();
            let wheel = wheel_zoom.clone();
            let handler = input_handler.clone();
            Rc::new(move || {
                let Some(button) = button.upgrade() else {
                    return;
                };
                let manual = mode
                    .upgrade()
                    .and_then(|a| a.state())
                    .is_some_and(|v| v.str() == Some("manual"));
                // Feedback from the image state must not enable/persist wheel
                // mode. Ctrl+wheel remains temporary and cannot leak to a photo.
                let handler = handler.get().expect("zoom input binding");
                button.block_signal(handler);
                button.set_active(wheel.get() || manual);
                button.unblock_signal(handler);
                button.set_tooltip_text(Some(if wheel.get() {
                    "Wheel zoom enabled"
                } else {
                    "Wheel changes photos; hold Ctrl or left mouse button to zoom"
                }));
            })
        };
        let choose_zoom = {
            let wheel = wheel_zoom.clone();
            let sync = sync_zoom.clone();
            Rc::new(move |enabled| {
                wheel.set(enabled);
                pivra::settings::save_wheel_zoom(enabled);
                sync();
            })
        };
        let changed = choose_zoom.clone();
        input_handler
            .set(zoom_toggle.connect_toggled(move |button| changed(button.is_active())))
            .expect("single zoom input binding");
        wheel_zoom.set(pivra::settings::load_wheel_zoom());
        sync_zoom();
        let zoom_window = window.downgrade();
        let zoom_key = gtk::EventControllerKey::new();
        zoom_key.set_propagation_phase(gtk::PropagationPhase::Capture);
        zoom_key.connect_key_pressed(move |_, key, _, modifiers| {
            if zoom_window.upgrade().is_some_and(|w| editing(&w)) {
                return glib::Propagation::Proceed;
            }
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
            choose_zoom(!wheel_zoom.get());
            glib::Propagation::Stop
        });
        window.add_controller(zoom_key);
        let weak_button = mode_button.downgrade();
        action.connect_notify_local(Some("state"), move |action, _| {
            sync_zoom();
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
        let current_delete_toast = Rc::new(RefCell::new(None::<adw::Toast>));
        rename_dialog::install(
            &window,
            &title,
            context.clone(),
            view.clone(),
            &banner,
            folder_watch.clone(),
            &toast_overlay,
            current_delete_toast.clone(),
        );
        toast_overlay.set_child(Some(view.widget()));
        let viewer_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        toast_overlay.set_hexpand(true);
        toast_overlay.set_vexpand(true);
        viewer_row.set_vexpand(true);
        viewer_row.append(&toast_overlay);
        viewer_row.append(view.information_widget());
        viewport.prepend(&viewer_row);
        let info_button = gtk::ToggleButton::builder()
            .icon_name("help-about-symbolic")
            .tooltip_text("Media information (I)")
            .build();
        info_button.set_widget_name("media-information-button");
        info_button.set_action_name(Some("win.media-info"));
        let info = gio::SimpleAction::new_stateful("media-info", None, &false.to_variant());
        let info_view = view.clone();
        let weak_button = info_button.downgrade();
        info.connect_activate(move |action, _| {
            let show = !action
                .state()
                .and_then(|v| v.get::<bool>())
                .unwrap_or(false);
            action.set_state(&show.to_variant());
            info_view.show_information(show);
            if let Some(button) = weak_button.upgrade() {
                button.set_active(show);
            }
        });
        window.add_action(&info);
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
                if slideshow::active(&window) {
                    gio::prelude::ActionGroupExt::activate_action(
                        &window,
                        if forward {
                            "slideshow-next"
                        } else {
                            "slideshow-previous"
                        },
                        None,
                    );
                    return;
                }
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
        let current_toast = current_delete_toast.clone();
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
                show_latest_toast(
                    &toasts,
                    &current_toast,
                    adw::Toast::new(&format!("Could not move to Trash: {error}")),
                );
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
            let undo_current_toast = current_toast.clone();
            toast.connect_button_clicked(move |_| {
                if path.exists() {
                    show_latest_toast(
                        &undo_toasts,
                        &undo_current_toast,
                        adw::Toast::new("Undo failed: destination already exists"),
                    );
                    return;
                }
                let trashed = trash_entries(&path)
                    .into_iter()
                    .find(|entry| !old_entries.iter().any(|old| old.equal(entry)));
                let Some(trashed) = trashed else {
                    show_latest_toast(
                        &undo_toasts,
                        &undo_current_toast,
                        adw::Toast::new("Undo failed: trashed file not found"),
                    );
                    return;
                };
                if let Err(error) = trashed.move_(
                    &gio::File::for_path(&path),
                    gio::FileCopyFlags::NONE,
                    None::<&gio::Cancellable>,
                    None,
                ) {
                    show_latest_toast(
                        &undo_toasts,
                        &undo_current_toast,
                        adw::Toast::new(&format!("Undo failed: {error}")),
                    );
                }
            });
            show_latest_toast(&toasts, &current_toast, toast);
        });
        window.add_action(&delete);
        // S saves the currently displayed video or animated-image frame beside its source.
        let save_frame = gio::SimpleAction::new("save-current-frame", None);
        let frame_context = context.clone();
        let frame_view = view.clone();
        let frame_toasts = toast_overlay.clone();
        save_frame.connect_activate(move |_, _| {
            let source = {
                let state = frame_context.borrow();
                state
                    .collection
                    .current()
                    .map(|item| item.path().to_owned())
            };
            let Some(source) = source else {
                frame_toasts.add_toast(adw::Toast::new("Could not save frame: no media selected"));
                return;
            };
            let result = (|| -> Result<std::path::PathBuf, String> {
                let frame = frame_view.capture_frame(&source)?;
                let parent = source.parent().ok_or("Source folder is unavailable")?;
                let stem = source
                    .file_stem()
                    .ok_or("Source filename is unavailable")?
                    .to_string_lossy();
                for index in 1..=9999 {
                    let target = parent.join(format!("{stem}_frame_{index:03}.png"));
                    if target.exists() {
                        continue;
                    }
                    // Reserve each name without overwriting existing files.
                    let mut file = match std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&target)
                    {
                        Ok(file) => file,
                        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                        Err(error) => return Err(error.to_string()),
                    };
                    use std::io::Write;
                    let bytes = frame.save_to_png_bytes();
                    if let Err(error) = file.write_all(bytes.as_ref()) {
                        drop(file);
                        let _ = std::fs::remove_file(&target);
                        return Err(error.to_string());
                    }
                    return Ok(target);
                }
                Err("Frame filename limit reached".into())
            })();
            match result {
                Ok(target) => frame_toasts.add_toast(adw::Toast::new(&format!(
                    "Frame saved: {}",
                    target.file_name().unwrap_or_default().to_string_lossy()
                ))),
                Err(error) => frame_toasts
                    .add_toast(adw::Toast::new(&format!("Could not save frame: {error}"))),
            }
        });
        window.add_action(&save_frame);
        let slides = slideshow::install(
            &window,
            &title,
            context.clone(),
            view.clone(),
            &toast_overlay,
            current_delete_toast.clone(),
        );
        groups.append(&toolbar::slideshow_group(&window));
        groups.append(&info_button);
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak_window = window.downgrade();
        let video_keys = view.clone();
        keys.connect_key_pressed(move |_, key, _, modifiers| {
            if !matches!(key, gdk::Key::F5 | gdk::Key::Escape)
                && weak_window.upgrade().is_some_and(|w| editing(&w))
            {
                return glib::Propagation::Proceed;
            }
            if !modifiers.intersects(
                gdk::ModifierType::CONTROL_MASK
                    | gdk::ModifierType::ALT_MASK
                    | gdk::ModifierType::SUPER_MASK,
            ) {
                if key == gdk::Key::F5 {
                    slides.start();
                    return glib::Propagation::Stop;
                }
                if matches!(key, gdk::Key::i | gdk::Key::I) {
                    if let Some(window) = weak_window.upgrade() {
                        gio::prelude::ActionGroupExt::activate_action(&window, "media-info", None);
                    }
                    return glib::Propagation::Stop;
                }
                if weak_window.upgrade().is_some_and(|w| slideshow::active(&w)) {
                    match key {
                        gdk::Key::space => slides.pause(),
                        gdk::Key::Escape => slides.stop(),
                        gdk::Key::Left => slides.navigate(false),
                        gdk::Key::Right => slides.navigate(true),
                        _ => return glib::Propagation::Proceed,
                    }
                    return glib::Propagation::Stop;
                }
            }
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
            if matches!(key, gdk::Key::s | gdk::Key::S) {
                if let Some(window) = weak_window.upgrade() {
                    gio::prelude::ActionGroupExt::activate_action(
                        &window,
                        "save-current-frame",
                        None,
                    );
                }
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::Delete {
                if let Some(window) = weak_window.upgrade() {
                    gio::prelude::ActionGroupExt::activate_action(&window, "trash-selected", None);
                }
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::F2 {
                if let Some(window) = weak_window.upgrade() {
                    gio::prelude::ActionGroupExt::activate_action(&window, "rename-media", None);
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
        // Open changes the media in this window; cancel the prior folder watcher.
        let open_window = window.downgrade();
        let open_context = context.clone();
        let open_view = view.clone();
        let open_title = title.clone();
        let open_status = placeholder.clone();
        let open_banner = banner.clone();
        let open_watch = folder_watch.clone();
        let controls = groups.downgrade();
        let open_file = gio::SimpleAction::new("open-file", Some(glib::VariantTy::STRING));
        open_file.connect_activate(move |_, value| {
            let Some(uri) = value.and_then(|v| v.str()) else {
                return;
            };
            let Some(window) = open_window.upgrade().filter(|w| w.is_visible()) else {
                return;
            };
            let file = gio::File::for_uri(uri);
            let Ok(opened) = OpenedFile::from_file(&file) else {
                eprintln!("Pivra: selected file could not be opened");
                return;
            };
            if let Some(parent) = file.path().and_then(|p| p.parent().map(|p| p.to_owned())) {
                save_last_open_folder(&parent);
            }
            gio::prelude::ActionGroupExt::activate_action(&window, "slideshow-stop", None);
            if let Some(previous) = open_watch.borrow_mut().take() {
                previous.stop();
            }
            {
                let mut state = open_context.borrow_mut();
                state.collection = MediaCollection::new(vec![opened.item.clone()]);
                state.suffix = None;
                state.warning = false;
                state.revision = state.revision.wrapping_add(1);
                open_banner.set_reveal_child(false);
                update_title(&window, &open_title, &state);
                display_selected(&open_view, &state);
            }
            if let Some(controls) = controls.upgrade() {
                controls.set_visible(true);
            }
            *open_watch.borrow_mut() = Some(watch(
                &window,
                &open_title,
                &open_status,
                &open_banner,
                open_view.clone(),
                open_context.clone(),
                opened,
            ));
        });
        window.add_action(&open_file);
        let dialog_token = Rc::new(RefCell::new(None::<gio::Cancellable>));
        let choose = gio::SimpleAction::new("choose-media", None);
        let weak_window = window.downgrade();
        let tokens = dialog_token.clone();
        choose.connect_activate(move |action, _| {
            let Some(window) = weak_window.upgrade() else {
                return;
            };
            let dialog = gtk::FileDialog::new();
            dialog.set_title("Open media");
            if let Some(folder) = load_last_open_folder() {
                dialog.set_initial_folder(Some(&folder));
            }
            let token = gio::Cancellable::new();
            tokens.replace(Some(token.clone()));
            action.set_enabled(false);
            let action = action.downgrade();
            let weak = window.downgrade();
            let tokens = tokens.clone();
            dialog.open(Some(&window), Some(&token), move |result| {
                tokens.borrow_mut().take();
                if let Some(action) = action.upgrade() {
                    action.set_enabled(true);
                }
                if let (Ok(file), Some(window)) =
                    (result, weak.upgrade().filter(|w| w.is_visible()))
                {
                    gio::prelude::ActionGroupExt::activate_action(
                        &window,
                        "open-file",
                        Some(&file.uri().to_variant()),
                    );
                }
            });
        });
        window.add_action(&choose);
        app.set_accels_for_action("win.choose-media", &["<Control>o"]);
        let destroy_token = dialog_token.clone();
        window.connect_destroy(move |_| {
            if let Some(token) = destroy_token.borrow_mut().take() {
                token.cancel();
            }
        });
        window.connect_close_request(move |_| {
            if let Some(token) = dialog_token.borrow_mut().take() {
                token.cancel();
            }
            glib::Propagation::Proceed
        });
        let closing_watch = folder_watch.clone();
        window.connect_close_request(move |_| {
            if let Some(watch) = closing_watch.borrow_mut().take() {
                watch.stop();
            }
            glib::Propagation::Proceed
        });
        let destroy_watch = folder_watch.clone();
        window.connect_destroy(move |_| {
            if let Some(watch) = destroy_watch.borrow_mut().take() {
                watch.stop();
            }
        });
        window.present();
        if let Some(opened) = file {
            *folder_watch.borrow_mut() = Some(watch(
                &window,
                &title,
                &placeholder,
                &banner,
                view,
                context,
                opened,
            ));
        } else {
            view.show_message("Open a photo or video to start.");
        }
    }
}
