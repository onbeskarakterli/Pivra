//! Each window owns its ordered catalog, keyboard actions and cancellable view.

use crate::{image_view::ImageView, input::OpenedFile};
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use pivra::{APP_NAME, catalog::MediaCollection, media::MediaKind, settings::Settings};
use std::{
    cell::RefCell,
    fs::{self, File, OpenOptions},
    os::fd::AsRawFd,
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
};

#[path = "fullscreen.rs"]
pub(super) mod fullscreen;
#[path = "rename_dialog.rs"]
mod rename_dialog;
#[path = "slideshow_window.rs"]
pub(super) mod slideshow;
#[path = "toolbar.rs"]
mod toolbar;
#[path = "undo.rs"]
mod undo;
#[path = "video_frame_export.rs"]
mod video_frame_export;

#[path = "window/batch_operations.rs"]
mod batch_operations;
#[path = "window/batch_rename.rs"]
mod batch_rename;
#[path = "window/compare_view.rs"]
mod compare_view;
#[path = "window/context_menu.rs"]
mod context_menu;
#[path = "window/empty_state.rs"]
mod empty_state;
#[path = "window/folder_watch.rs"]
mod folder_watch;
use folder_watch::{FolderWatch, watch};
#[path = "window/application_chooser.rs"]
mod application_chooser;
#[path = "window/file_chooser.rs"]
mod file_chooser;
#[path = "window/open_folder.rs"]
mod open_folder;
#[path = "window/reveal.rs"]
mod reveal;
#[path = "window/slideshow_interaction.rs"]
mod slideshow_interaction;
use open_folder::save_last_open_folder;

// Keep one current notification per window.
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
    own_batch_rename: bool,
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

// Keep the original inode pinned across the move into Trash. A URI or
// original pathname alone can be reused after an external removal.
struct TrashSource {
    original: PathBuf,
    source: File,
    directory: File,
}

struct TrashReceipt {
    original: PathBuf,
    source: File,
    directory: File,
    previous: Vec<gio::File>,
    staged: Mutex<Option<StagedRestore>>,
}

struct StagedRestore {
    directory: File,
    path: PathBuf,
}

impl Drop for StagedRestore {
    fn drop(&mut self) {
        // A failed publish may still need recovery. Remove only empty staging folders.
        if let (Ok(held), Ok(current)) =
            (self.directory.metadata(), fs::symlink_metadata(&self.path))
            && same_inode(&held, &current)
        {
            let _ = fs::remove_dir(&self.path);
        }
    }
}

impl TrashSource {
    fn capture(original: PathBuf) -> Result<Self, String> {
        // O_PATH | O_NOFOLLOW: identify the directory entry, including symlinks,
        // without opening the media payload.
        let source = OpenOptions::new()
            .read(true)
            .custom_flags(0x200000 | 0x20000)
            .open(&original)
            .map_err(|error| error.to_string())?;
        let directory = File::open(original.parent().ok_or("Missing parent folder")?)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            original,
            source,
            directory,
        })
    }

    fn trash(self) -> Result<TrashReceipt, String> {
        let Self {
            original,
            source,
            directory,
        } = self;
        let identity = source.metadata().map_err(|error| error.to_string())?;
        let previous = trash_entries(&original)?
            .into_iter()
            .filter(|(_, info)| trash_identity(info) == Some((identity.dev(), identity.ino())))
            .map(|(file, _)| file)
            .collect();

        check_trash_parent(&original, &directory)?;
        if !same_inode(
            &identity,
            &fs::symlink_metadata(&original).map_err(|error| error.to_string())?,
        ) {
            return Err("The source file changed".into());
        }
        gio::File::for_path(&original)
            .trash(None::<&gio::Cancellable>)
            .map_err(|error| error.to_string())?;
        Ok(TrashReceipt {
            original,
            source,
            directory,
            previous,
            staged: Mutex::new(None),
        })
    }
}

impl TrashReceipt {
    fn move_to_trash(original: PathBuf) -> Result<Self, String> {
        TrashSource::capture(original)?.trash()
    }

    fn path(&self) -> &Path {
        &self.original
    }

    fn restore(&self) -> Result<(), String> {
        let source = self.source.metadata().map_err(|error| error.to_string())?;
        let mut staged = self.staged.lock().map_err(|error| error.to_string())?;
        let stage = match staged.as_ref() {
            Some(stage) => stage,
            None => {
                check_trash_destination(&self.original, &self.directory)?;
                staged.insert(self.stage(&source)?)
            }
        };
        let publish = || -> Result<(), String> {
            let staged_file = fs::symlink_metadata(
                PathBuf::from(format!("/proc/self/fd/{}", stage.directory.as_raw_fd()))
                    .join("media"),
            )
            .map_err(|error| error.to_string())?;
            if !same_inode(&source, &staged_file) {
                return Err("The file being restored changed".into());
            }
            check_trash_destination(&self.original, &self.directory)?;
            pivra::rename::move_no_replace(
                &stage.directory,
                std::ffi::OsStr::new("media"),
                &self.directory,
                self.original.file_name().ok_or("Missing filename")?,
            )
            .map_err(|error| error.to_string())
        };
        if let Err(error) = publish() {
            return Err(format!(
                "{error}. File staged at {}",
                stage.path.join("media").display()
            ));
        }
        staged.take();
        Ok(())
    }

    fn stage(&self, source: &fs::Metadata) -> Result<StagedRestore, String> {
        let mut entries = trash_entries(&self.original)?
            .into_iter()
            .filter(|(file, info)| {
                !self.previous.iter().any(|previous| previous.equal(file))
                    && trash_identity(info) == Some((source.dev(), source.ino()))
            });
        let (entry, _) = entries
            .next()
            .ok_or("The original file was not found in Trash")?;
        if entries.next().is_some() {
            return Err("More than one Trash entry matches the original file".into());
        }
        // GVfs moves out of Trash using a destination check and then rename.
        // Restore there only inside a private directory, then publish atomically.
        let parent = self.original.parent().ok_or("Missing parent folder")?;
        let path = glib::mkdtemp(parent.join(".pivra-restore-XXXXXX")).ok_or_else(|| {
            format!(
                "Could not prepare restore: {}",
                std::io::Error::last_os_error()
            )
        })?;
        let directory = match File::open(&path) {
            Ok(directory) => directory,
            Err(error) => {
                let _ = fs::remove_dir(&path);
                return Err(error.to_string());
            }
        };
        let stage = StagedRestore { directory, path };
        entry
            .move_(
                &gio::File::for_path(stage.path.join("media")),
                gio::FileCopyFlags::NOFOLLOW_SYMLINKS | gio::FileCopyFlags::NO_FALLBACK_FOR_MOVE,
                None::<&gio::Cancellable>,
                None,
            )
            .map_err(|error| error.to_string())?;
        Ok(stage)
    }
}

fn same_inode(a: &fs::Metadata, b: &fs::Metadata) -> bool {
    (a.dev(), a.ino()) == (b.dev(), b.ino())
}

fn check_trash_parent(path: &Path, directory: &File) -> Result<(), String> {
    let held = directory.metadata().map_err(|error| error.to_string())?;
    let current = fs::metadata(path.parent().ok_or("Missing parent folder")?)
        .map_err(|error| error.to_string())?;
    if !same_inode(&held, &current) {
        return Err("The original folder changed".into());
    }
    Ok(())
}

fn check_trash_destination(path: &Path, directory: &File) -> Result<(), String> {
    check_trash_parent(path, directory)?;
    match fs::symlink_metadata(path) {
        Ok(_) => Err("The destination already exists".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn trash_identity(info: &gio::FileInfo) -> Option<(u64, u64)> {
    (info.has_attribute("unix::device") && info.has_attribute("unix::inode")).then(|| {
        (
            u64::from(info.attribute_uint32("unix::device")),
            info.attribute_uint64("unix::inode"),
        )
    })
}

fn trash_entries(original: &Path) -> Result<Vec<(gio::File, gio::FileInfo)>, String> {
    let root = gio::File::for_uri("trash:///");
    let enumerator = root
        .enumerate_children(
            "standard::name,trash::orig-path,unix::device,unix::inode",
            gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
            None::<&gio::Cancellable>,
        )
        .map_err(|error| format!("Could not read Trash: {error}"))?;
    let mut entries = vec![];
    while let Some(info) = enumerator
        .next_file(None::<&gio::Cancellable>)
        .map_err(|error| format!("Could not read Trash: {error}"))?
    {
        if info
            .attribute_byte_string("trash::orig-path")
            .is_some_and(|value| value.as_bytes() == original.as_os_str().as_bytes())
        {
            entries.push((enumerator.child(&info), info));
        }
    }
    Ok(entries)
}

fn update_title(window: &adw::ApplicationWindow, title: &adw::WindowTitle, context: &FileContext) {
    context_menu::selection_changed(window, context.collection.current().map(|item| item.path()));
    empty_state::update(window, context.collection.current().is_some());
    update_navigation_actions(window, context);
    let Some(selected) = context.collection.current() else {
        window.set_title(Some(APP_NAME));
        title.set_title(APP_NAME);
        title.set_subtitle("");
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
    let _timing = pivra::performance::Span::new("display_selected_ui");
    if let Some(item) = context.collection.current() {
        pivra::performance::media_event("media_selected", "request", item.path());
    }
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
    let toast_overlay = adw::ToastOverlay::new();
    let pages = gtk::Stack::new();
    pages.set_hhomogeneous(false);
    pages.set_vhomogeneous(false);
    pages.add_named(&viewport, Some("viewer"));
    toast_overlay.set_child(Some(&pages));
    toast_overlay.set_hexpand(true);
    toast_overlay.set_vexpand(true);
    content.append(&toast_overlay);
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_width(settings.window_width)
        .default_height(settings.window_height)
        .content(&content)
        .build();
    pivra::settings::window_state::install(&window, settings);
    {
        let context = Rc::new(RefCell::new(FileContext {
            own_rename: None,
            own_batch_rename: false,
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
        view.install_thumbnail_selection(&window);
        install_navigation_arrows(view.widget());
        let current_toast = Rc::new(RefCell::new(None::<adw::Toast>));
        let undo = undo::Undo::install(
            &window,
            &toast_overlay,
            current_toast.clone(),
            &title,
            &banner,
            context.clone(),
            view.clone(),
            folder_watch.clone(),
        );
        batch_rename::install(&window, &view, &undo);
        rename_dialog::install(&window, context.clone(), undo.clone(), &view);
        let viewer_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        viewer_row.set_vexpand(true);
        viewer_row.append(view.widget());
        viewer_row.append(view.information_widget());
        viewport.prepend(&viewer_row);
        empty_state::install(&window, &viewport, &viewer_row);
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
                let _timing = pivra::performance::Span::new(if forward {
                    "next_media_ui"
                } else {
                    "previous_media_ui"
                });
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
        let delete = gio::SimpleAction::new("trash-selected", None);
        let trash_context = context.clone();
        let trash_view = view.clone();
        let trash_title = title.clone();
        let trash_banner = banner.clone();
        let trash_window = window.downgrade();
        let trash_undo = undo.clone();
        let watches = folder_watch.clone();
        delete.connect_activate(move |_, _| {
            let marked = trash_view.marked_thumbnails();
            if marked.len() > 1 {
                if let Some(window) = trash_window.upgrade() {
                    batch_operations::trash(&window, marked, &trash_undo);
                }
                return;
            }
            let Some(path) = trash_context
                .borrow()
                .collection
                .current()
                .map(|item| item.path().to_owned())
            else {
                return;
            };
            let Some(permit) = trash_undo.begin() else {
                return;
            };
            let worker_path = path.clone();
            let job = gio::spawn_blocking(move || {
                undo::run_file_operation(|| TrashReceipt::move_to_trash(worker_path))
            });
            let context = trash_context.clone();
            let view = trash_view.clone();
            let title = trash_title.clone();
            let banner = trash_banner.clone();
            let window = trash_window.clone();
            let undo = trash_undo.clone();
            let watches = watches.clone();
            let original_watch = watches.borrow().clone();
            glib::MainContext::default().spawn_local(async move {
                let _permit = permit;
                let result = job.await;
                let Some(window) = window.upgrade().filter(|window| window.is_visible()) else {
                    return;
                };
                let receipt = match result {
                    Ok(Ok(receipt)) => receipt,
                    other => {
                        let error = match other {
                            Ok(Err(error)) => error,
                            _ => "worker failed".into(),
                        };
                        undo.notify(&format!("Could not move to Trash: {error}"));
                        return;
                    }
                };
                let current_watch = watches.borrow().clone();
                if let Some(watch) = current_watch.filter(|watch| {
                    original_watch
                        .as_ref()
                        .is_some_and(|original| Rc::ptr_eq(watch, original))
                }) {
                    let mut context = context.borrow_mut();
                    let replacement = context.collection.selected_index().filter(|_| {
                        context
                            .collection
                            .current()
                            .is_some_and(|item| item.path() == path)
                    });
                    let remaining = context
                        .collection
                        .items()
                        .iter()
                        .filter(|item| item.path() != path)
                        .cloned()
                        .collect();
                    context.collection.replace_preserving(remaining);
                    if let Some(index) = replacement {
                        let last_index = context.collection.items().len().saturating_sub(1);
                        context.collection.select(index.min(last_index));
                        context.revision = context.revision.wrapping_add(1);
                        banner.set_reveal_child(false);
                        update_title(&window, &title, &context);
                        display_selected(&view, &context);
                    } else {
                        update_title(&window, &title, &context);
                        update_thumbnail_catalog(&view, &context);
                        view.set_neighbors(context.collection.photo_neighbors());
                    }
                    watch.schedule();
                }
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                undo.record(
                    undo::Operation::Trash(receipt),
                    &format!("Moved to Trash: {name}"),
                );
            });
        });
        window.add_action(&delete);
        let frame_context = context.clone();
        let frame_toasts = toast_overlay.downgrade();
        let frame_current_toast = current_toast.clone();
        video_frame_export::install(
            &window,
            &view,
            move || {
                frame_context
                    .borrow()
                    .collection
                    .current()
                    .filter(|item| item.kind() == MediaKind::Video)
                    .map(|item| item.path().to_owned())
            },
            move |message| {
                if let Some(overlay) = frame_toasts.upgrade() {
                    show_latest_toast(
                        &overlay,
                        &frame_current_toast,
                        adw::Toast::new(&glib::markup_escape_text(message)),
                    );
                }
            },
        );
        let slides = slideshow::install(
            &window,
            &title,
            context.clone(),
            view.clone(),
            &toast_overlay,
            current_toast.clone(),
        );
        groups.append(&toolbar::slideshow_group(&window));
        groups.append(&info_button);
        context_menu::install(
            &window,
            context.clone(),
            &view,
            &zoom_toggle,
            &toast_overlay,
            current_toast.clone(),
        );
        batch_operations::install_transfers(&window, context.clone(), &view, &undo);
        compare_view::install(&window, &pages, &view, undo.clone());
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
            if weak_window
                .upgrade()
                .is_some_and(|window| compare_view::handle_key(&window, key, modifiers))
            {
                return glib::Propagation::Stop;
            }
            if key.to_lower() == gdk::Key::s
                && !modifiers.intersects(
                    gdk::ModifierType::CONTROL_MASK
                        | gdk::ModifierType::SHIFT_MASK
                        | gdk::ModifierType::ALT_MASK
                        | gdk::ModifierType::SUPER_MASK
                        | gdk::ModifierType::META_MASK
                        | gdk::ModifierType::HYPER_MASK,
                )
                && let Some(window) = weak_window.upgrade()
                && let Some(action) = window.lookup_action(video_frame_export::ACTION)
                && action.is_enabled()
            {
                action.activate(None);
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::z
                && modifiers.contains(gdk::ModifierType::CONTROL_MASK)
                && !modifiers.intersects(
                    gdk::ModifierType::SHIFT_MASK
                        | gdk::ModifierType::ALT_MASK
                        | gdk::ModifierType::SUPER_MASK
                        | gdk::ModifierType::META_MASK,
                )
            {
                if let Some(window) = weak_window.upgrade() {
                    gio::prelude::ActionGroupExt::activate_action(&window, "undo", None);
                }
                return glib::Propagation::Stop;
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
            let _timing = pivra::performance::Span::new("open_file_apply_ui");
            let Some(uri) = value.and_then(|v| v.str()) else {
                return;
            };
            let Some(window) = open_window.upgrade().filter(|w| w.is_visible()) else {
                return;
            };
            let file = gio::File::for_uri(uri);
            let opened = {
                let _timing = pivra::performance::Span::new("open_file_validate_ui");
                let Ok(opened) = OpenedFile::from_file(&file) else {
                    eprintln!("Pivra: selected file could not be opened");
                    return;
                };
                opened
            };
            if let Some(parent) = file.path().and_then(|p| p.parent().map(|p| p.to_owned())) {
                let _timing = pivra::performance::Span::new("open_file_save_folder_ui");
                save_last_open_folder(&parent);
            }
            gio::prelude::ActionGroupExt::activate_action(&window, "slideshow-stop", None);
            if let Some(previous) = open_watch.borrow_mut().take() {
                previous.stop();
            }
            {
                let mut state = open_context.borrow_mut();
                state.collection = MediaCollection::new(vec![opened.item.clone()]);
                state.own_rename = None;
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
        file_chooser::install(&window, app);
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
        }
    }
}
