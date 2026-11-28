//! Desktop clipboard actions. No filesystem transfer or image decode.
use super::{FileContext, undo::Undo};
use crate::image_view::ImageView;
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

pub(super) const PATH: &str = "copy-file-path";
pub(super) const FILES: &str = "copy-files";
pub(super) const IMAGE: &str = "copy-image";

fn payloads(files: &[gio::File]) -> (String, String) {
    let uris: Vec<_> = files.iter().map(|file| file.uri()).collect();
    (
        format!("{}\r\n", uris.join("\r\n")),
        format!("copy\n{}", uris.join("\n")),
    )
}

fn files_provider(paths: &[PathBuf]) -> gdk::ContentProvider {
    let files: Vec<_> = paths.iter().map(gio::File::for_path).collect();
    let (standard, gnome) = payloads(&files);
    gdk::ContentProvider::new_union(&[
        gdk::ContentProvider::for_bytes("text/uri-list", &glib::Bytes::from_owned(standard)),
        gdk::ContentProvider::for_bytes(
            "x-special/gnome-copied-files",
            &glib::Bytes::from_owned(gnome),
        ),
        gdk::ContentProvider::for_value(&gdk::FileList::from_array(&files).to_value()),
    ])
}

pub(super) fn update_image_action(window: &adw::ApplicationWindow, view: &ImageView, path: &Path) {
    if let Some(action) = window
        .lookup_action(IMAGE)
        .and_downcast::<gio::SimpleAction>()
    {
        action.set_enabled(view.clipboard_texture(path).is_some());
    }
}

pub(super) fn install(
    window: &adw::ApplicationWindow,
    context: Rc<RefCell<FileContext>>,
    view: &ImageView,
    notifications: &Rc<Undo>,
) {
    for name in [PATH, FILES, IMAGE] {
        let action = gio::SimpleAction::new(name, None);
        action.set_enabled(name != IMAGE);
        let window_weak = window.downgrade();
        let context = context.clone();
        let notifications = notifications.clone();
        let view = view.clone();
        action.connect_activate(move |_, _| {
            let Some(window) = window_weak.upgrade() else {
                return;
            };
            let Some(path) = context
                .borrow()
                .collection
                .current()
                .map(|item| item.path().to_owned())
            else {
                return;
            };
            let clipboard = window.clipboard();
            let (result, message) = match name {
                PATH => {
                    clipboard.set_text(&path.to_string_lossy());
                    (Ok(()), "File path copied".into())
                }
                FILES => {
                    let mut paths = view.marked_thumbnails();
                    if paths.is_empty() {
                        paths.push(path);
                    }
                    let message = if paths.len() == 1 {
                        "File copied to clipboard".into()
                    } else {
                        format!("{} files copied to clipboard", paths.len())
                    };
                    (
                        clipboard.set_content(Some(&files_provider(&paths))),
                        message,
                    )
                }
                IMAGE => {
                    let Some(texture) = view.clipboard_texture(&path) else {
                        return;
                    };
                    let content = gdk::ContentProvider::for_value(&texture.to_value());
                    (
                        clipboard.set_content(Some(&content)),
                        "Image copied to clipboard".into(),
                    )
                }
                _ => unreachable!(),
            };
            match result {
                Ok(()) => notifications.notify(&message),
                Err(error) => {
                    notifications.notify(&format!("Could not copy to clipboard: {error}"))
                }
            }
        });
        window.add_action(&action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_clipboard_payloads_keep_uri_encoding_order_and_copy_intent() {
        let files = [
            gio::File::for_path("/tmp/çekim #1.png"),
            gio::File::for_path("/tmp/second.mp4"),
        ];
        let (standard, gnome) = payloads(&files);
        assert_eq!(
            standard,
            "file:///tmp/%C3%A7ekim%20%231.png\r\nfile:///tmp/second.mp4\r\n"
        );
        assert_eq!(
            gnome,
            "copy\nfile:///tmp/%C3%A7ekim%20%231.png\nfile:///tmp/second.mp4"
        );
    }
}
