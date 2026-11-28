//! On-demand export from the current video presentation, independent of playback.

use crate::image_view::ImageView;
use adw::prelude::*;
use gtk::{gdk, gdk_pixbuf, gio, glib};
use std::{
    cell::RefCell,
    fs::{self, File, Metadata, OpenOptions},
    io,
    os::{
        fd::AsRawFd,
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

pub(super) const ACTION: &str = "save-video-frame";
const MAX_EDGE: i32 = 8192;
const MAX_PIXELS: i64 = 8192 * 4320;
static EXPORTING: AtomicBool = AtomicBool::new(false);

struct Export {
    action: gio::SimpleAction,
    window: glib::WeakRef<adw::ApplicationWindow>,
    stream: RefCell<Option<(gtk::MediaFile, Vec<glib::SignalHandlerId>)>>,
    pending: RefCell<Option<gio::Cancellable>>,
    notify: Box<dyn Fn(&str)>,
}

impl Export {
    fn ready(&self) -> bool {
        self.window
            .upgrade()
            .is_some_and(|window| window.renderer().is_some())
            && self.stream.borrow().as_ref().is_some_and(|(media, _)| {
                matches!(
                    media.type_().name(),
                    "GtkGstMediaFile" | "PivraGpuMediaFile"
                ) && media.is_prepared()
                    && media.has_video()
                    && media.error().is_none()
                    && supported_size(media.intrinsic_width(), media.intrinsic_height())
            })
    }

    fn refresh(&self) {
        self.action
            .set_enabled(self.pending.borrow().is_none() && self.ready());
    }

    fn bind(self: &Rc<Self>, media: Option<gtk::MediaFile>) {
        if let Some((previous, handlers)) = self.stream.borrow_mut().take() {
            for id in handlers {
                previous.disconnect(id);
            }
        }
        if let Some(media) = media {
            let mut handlers = Vec::new();
            for property in ["prepared", "has-video", "error"] {
                let weak = Rc::downgrade(self);
                handlers.push(media.connect_notify_local(Some(property), move |_, _| {
                    if let Some(export) = weak.upgrade() {
                        export.refresh();
                    }
                }));
            }
            let weak = Rc::downgrade(self);
            handlers.push(media.connect_invalidate_size(move |_| {
                if let Some(export) = weak.upgrade() {
                    export.refresh();
                }
            }));
            self.stream.replace(Some((media, handlers)));
        }
        self.refresh();
    }

    fn cancel(&self) {
        let token = self.pending.borrow().clone();
        if let Some(token) = token {
            token.cancel();
        }
    }

    fn failure(&self, error: &str) {
        (self.notify)(&format!("Could not save video frame: {error}"));
    }
}

struct Pending(Rc<Export>, gio::Cancellable);

impl Drop for Pending {
    fn drop(&mut self) {
        self.0.pending.borrow_mut().take();
        EXPORTING.store(false, Ordering::Release);
        self.0.refresh();
    }
}

pub(super) fn install(
    window: &adw::ApplicationWindow,
    view: &ImageView,
    selected_video: impl Fn() -> Option<PathBuf> + 'static,
    notify: impl Fn(&str) + 'static,
) {
    let action = gio::SimpleAction::new(ACTION, None);
    action.set_enabled(false);
    let export = Rc::new(Export {
        action: action.clone(),
        window: window.downgrade(),
        stream: RefCell::new(None),
        pending: RefCell::new(None),
        notify: Box::new(notify),
    });
    // The picture becomes visible after ImageView attaches its stream and is
    // hidden before the stream is retired, including automatic stock fallback.
    let observer = Rc::downgrade(&export);
    view.on_video_visibility(move |media| {
        if let Some(export) = observer.upgrade() {
            export.bind(media);
        }
    });
    let closing = export.clone();
    window.connect_close_request(move |_| {
        closing.cancel();
        glib::Propagation::Proceed
    });
    let closing = export.clone();
    window.connect_destroy(move |_| {
        closing.cancel();
        closing.bind(None);
    });
    let weak = Rc::downgrade(&export);
    let view = view.clone();
    action.connect_activate(move |_, _| {
        let (Some(export), Some(path)) = (weak.upgrade(), selected_video()) else {
            return;
        };
        let Some(window) = export.window.upgrade().filter(|w| w.is_visible()) else {
            return;
        };
        if !export.ready() || export.pending.borrow().is_some() {
            return;
        }
        if EXPORTING.swap(true, Ordering::AcqRel) {
            export.failure("another frame export is in progress");
            return;
        }
        let cancel = gio::Cancellable::new();
        export.pending.replace(Some(cancel.clone()));
        export.refresh();
        let pending = Pending(export, cancel);
        let timestamp = view.current_video().map_or(0, |media| media.timestamp());
        let pixels = match view.capture_frame(&path).and_then(Pixels::from_texture) {
            Ok(pixels) => pixels,
            Err(error) => {
                pending.0.failure(&error);
                return;
            }
        };
        choose_file(&window, &path, timestamp, pixels, pending);
    });
    window.add_action(&action);
}

fn choose_file(
    window: &adw::ApplicationWindow,
    source: &Path,
    timestamp: i64,
    pixels: Pixels,
    pending: Pending,
) {
    let dialog = gtk::FileDialog::builder()
        .title("Save Video Frame")
        .accept_label("Save")
        .initial_name(suggested_name(source, timestamp))
        .modal(true)
        .build();
    if let Some(parent) = source.parent() {
        dialog.set_initial_folder(Some(&gio::File::for_path(parent)));
    }
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("PNG or JPEG images (choose by filename extension)"));
    for suffix in ["png", "jpg", "jpeg"] {
        filter.add_suffix(suffix);
    }
    dialog.set_filters(Some(&gio::ListStore::from_iter([filter])));
    let cancel = pending.1.clone();
    dialog.save(Some(window), Some(&cancel), move |result| match result {
        Ok(file) if !pending.1.is_cancelled() => {
            let Some(path) = file.path() else {
                pending.0.failure("choose a local file");
                return;
            };
            let job = gio::spawn_blocking(move || Destination::inspect(path));
            glib::MainContext::default().spawn_local(async move {
                match job.await {
                    Ok(Ok(destination)) => confirm_or_write(destination, pixels, pending),
                    Ok(Err(error)) if !pending.1.is_cancelled() => pending.0.failure(&error),
                    Err(_) if !pending.1.is_cancelled() => pending.0.failure("file worker failed"),
                    _ => {}
                }
            });
        }
        Err(error)
            if !pending.1.is_cancelled()
                && !error.matches(gtk::DialogError::Dismissed)
                && !error.matches(gtk::DialogError::Cancelled) =>
        {
            pending.0.failure(&error.to_string());
        }
        _ => {}
    });
}

fn confirm_or_write(destination: Destination, pixels: Pixels, pending: Pending) {
    let Some(window) = pending.0.window.upgrade().filter(|w| w.is_visible()) else {
        return;
    };
    if pending.1.is_cancelled() {
        return;
    }
    if destination.existing.is_none() {
        write_in_background(&window, destination, pixels, pending);
        return;
    }
    // A native chooser returns a path, not what it saw at that path. Confirm
    // the existing destination explicitly, including files created meanwhile.
    let dialog = gtk::AlertDialog::builder()
        .message("Replace the existing file?")
        .detail(destination.path.to_string_lossy())
        .buttons(["Cancel", "Replace"])
        .cancel_button(0)
        .default_button(0)
        .modal(true)
        .build();
    let weak = window.downgrade();
    let cancel = pending.1.clone();
    dialog.choose(Some(&window), Some(&cancel), move |response| {
        if response == Ok(1)
            && !pending.1.is_cancelled()
            && let Some(window) = weak.upgrade().filter(|w| w.is_visible())
        {
            write_in_background(&window, destination, pixels, pending);
        }
    });
}

fn write_in_background(
    window: &adw::ApplicationWindow,
    destination: Destination,
    pixels: Pixels,
    pending: Pending,
) {
    let hold = window.application().map(|app| app.hold());
    let cancel = pending.1.clone();
    let job = gio::spawn_blocking(move || destination.write(pixels, &cancel));
    glib::MainContext::default().spawn_local(async move {
        let result = job.await;
        if !pending.1.is_cancelled() {
            match result {
                Ok(Ok(())) => (pending.0.notify)("Video frame saved"),
                Ok(Err(error)) => pending.0.failure(&error.to_string()),
                Err(_) => pending.0.failure("file worker failed"),
            }
        }
        drop(hold);
    });
}

fn suggested_name(source: &Path, timestamp: i64) -> String {
    let stem: String = source
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .chars()
        .take(48)
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let stem = stem.trim_matches(|c: char| c == '.' || c.is_whitespace());
    let stem = if stem.is_empty() { "video" } else { stem };
    let ms = timestamp.max(0) / 1000;
    format!(
        "{stem}_{:02}-{:02}-{:02}-{:03}.png",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1000 % 60,
        ms % 1000
    )
}

fn supported_size(width: i32, height: i32) -> bool {
    (1..=MAX_EDGE).contains(&width)
        && (1..=MAX_EDGE).contains(&height)
        && i64::from(width) * i64::from(height) <= MAX_PIXELS
}

struct Pixels {
    bytes: glib::Bytes,
    width: i32,
    height: i32,
    stride: i32,
}

impl Pixels {
    fn from_texture(texture: gdk::Texture) -> Result<Self, String> {
        let (width, height) = (texture.width(), texture.height());
        if !supported_size(width, height) {
            return Err("unsupported frame size".into());
        }
        // Read back from the renderer's output once. The worker and chooser
        // retain only sRGB pixels, never a live video or GL/DMA-BUF texture.
        let mut download = gdk::TextureDownloader::new(&texture);
        download.set_format(gdk::MemoryFormat::R8g8b8);
        download.set_color_state(&gdk::ColorState::srgb());
        let (bytes, stride) = download.download_bytes();
        let row = width as usize * 3;
        let size = stride
            .checked_mul(height as usize - 1)
            .and_then(|n| n.checked_add(row));
        if stride < row || size.is_none_or(|n| n > bytes.len()) || stride > i32::MAX as usize {
            return Err("could not read the rendered frame".into());
        }
        Ok(Self {
            bytes,
            width,
            height,
            stride: stride as i32,
        })
    }

    fn encode(self, file: File, format: Format, cancel: &gio::Cancellable) -> io::Result<()> {
        let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(
            &self.bytes,
            gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            self.width,
            self.height,
            self.stride,
        );
        let stream = gio::UnixOutputStream::take_fd(file.into());
        let (kind, options): (&str, &[(&str, &str)]) = match format {
            Format::Png => ("png", &[]),
            Format::Jpeg => ("jpeg", &[("quality", "95")]),
        };
        pixbuf
            .save_to_streamv(&stream, kind, options, Some(cancel))
            .map_err(io::Error::other)?;
        stream.close(Some(cancel)).map_err(io::Error::other)
    }
}

#[derive(Clone, Copy)]
enum Format {
    Png,
    Jpeg,
}

impl Format {
    fn from_path(path: &Path) -> Result<Self, &'static str> {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("png") => Ok(Self::Png),
            Some("jpg" | "jpeg") => Ok(Self::Jpeg),
            _ => Err("use a .png, .jpg or .jpeg filename"),
        }
    }
}

struct Destination {
    path: PathBuf,
    directory: File,
    format: Format,
    existing: Option<Metadata>,
}

impl Destination {
    fn inspect(path: PathBuf) -> Result<Self, String> {
        let format = Format::from_path(&path)?;
        let parent = path.parent().ok_or("missing output folder")?;
        let directory = File::open(parent).map_err(|err| err.to_string())?;
        let existing = current_file(&path).map_err(|err| err.to_string())?;
        Ok(Self {
            path,
            directory,
            format,
            existing,
        })
    }

    fn write(self, pixels: Pixels, cancel: &gio::Cancellable) -> io::Result<()> {
        check_cancel(cancel)?;
        let parent = self
            .path
            .parent()
            .ok_or(io::Error::other("missing output folder"))?;
        let target = self
            .path
            .file_name()
            .ok_or(io::Error::other("missing output filename"))?;
        let parent_fd = PathBuf::from(format!("/proc/self/fd/{}", self.directory.as_raw_fd()));
        let temporary = Temporary(
            glib::mkdtemp(parent_fd.join(".pivra-frame-XXXXXX"))
                .ok_or_else(io::Error::last_os_error)?,
        );
        let output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(temporary.path())?;
        pixels.encode(output, self.format, cancel)?;
        check_cancel(cancel)?;

        let (held, now) = (self.directory.metadata()?, fs::metadata(parent)?);
        if (held.dev(), held.ino()) != (now.dev(), now.ino()) {
            return Err(io::Error::other("the output folder changed"));
        }
        if let Some(expected) = self.existing {
            let current = current_file(&parent_fd.join(target))?;
            if !current.is_some_and(|current| stamp(&current) == stamp(&expected)) {
                return Err(io::Error::other(
                    "the existing file changed after confirmation",
                ));
            }
            fs::rename(temporary.path(), parent_fd.join(target))
        } else {
            pivra::rename::move_no_replace(
                &File::open(&temporary.0)?,
                std::ffi::OsStr::new("frame"),
                &self.directory,
                target,
            )
        }
    }
}

fn current_file(path: &Path) -> io::Result<Option<Metadata>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => Ok(Some(metadata)),
        Ok(_) => Err(io::Error::other("the output path is not a regular file")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn stamp(meta: &Metadata) -> (u64, u64, u64, i64, i64, i64, i64) {
    (
        meta.dev(),
        meta.ino(),
        meta.len(),
        meta.mtime(),
        meta.mtime_nsec(),
        meta.ctime(),
        meta.ctime_nsec(),
    )
}

fn check_cancel(cancel: &gio::Cancellable) -> io::Result<()> {
    cancel.set_error_if_cancelled().map_err(io::Error::other)
}

struct Temporary(PathBuf);

impl Temporary {
    fn path(&self) -> PathBuf {
        self.0.join("frame")
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path());
        let _ = fs::remove_dir(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red_frame() -> Pixels {
        Pixels {
            bytes: glib::Bytes::from_owned([242, 12, 17].repeat(192 * 96)),
            width: 192,
            height: 96,
            stride: 192 * 3,
        }
    }

    #[test]
    fn safe_suggested_name_and_frame_budget() {
        assert_eq!(
            suggested_name(Path::new("/media/video.mp4"), 84_350_000),
            "video_00-01-24-350.png"
        );
        assert_eq!(
            suggested_name(Path::new("odd\nname.mp4"), -1),
            "odd_name_00-00-00-000.png"
        );
        assert!(suggested_name(Path::new(&"📽".repeat(100)), 0).len() < 255);
        assert!(supported_size(7680, 4320) && supported_size(4320, 7680));
        assert!(!supported_size(8192, 8192) && !supported_size(0, 144));
    }

    #[test]
    fn writes_real_png_and_jpeg_at_source_size() {
        let dir = tempfile::tempdir().unwrap();
        for (extension, magic) in [
            ("png", &b"\x89PNG\r\n\x1a\n"[..]),
            ("jpeg", &b"\xff\xd8"[..]),
        ] {
            let path = dir.path().join(format!("capture.{extension}"));
            Destination::inspect(path.clone())
                .unwrap()
                .write(red_frame(), &gio::Cancellable::new())
                .unwrap();
            assert!(fs::read(&path).unwrap().starts_with(magic));
            let image = gdk_pixbuf::Pixbuf::from_file(path).unwrap();
            assert_eq!((image.width(), image.height()), (192, 96));
            let bytes = image.read_pixel_bytes();
            if extension == "png" {
                assert_eq!(&bytes.as_ref()[0..3], &[242, 12, 17]);
            } else {
                assert!(bytes[0] >= 235 && bytes[1] < 25 && bytes[2] < 25);
            }
        }
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    }

    #[test]
    fn rejects_late_collision_changed_target_symlink_and_cancel() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("frame.png");
        let cancel = gio::Cancellable::new();
        let fresh = Destination::inspect(path.clone()).unwrap();
        fs::write(&path, b"new arrival").unwrap();
        assert_eq!(
            fresh.write(red_frame(), &cancel).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read(&path).unwrap(), b"new arrival");
        let approved = Destination::inspect(path.clone()).unwrap();
        fs::write(&path, b"external edit").unwrap();
        assert!(approved.write(red_frame(), &cancel).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"external edit");
        let approved = Destination::inspect(path.clone()).unwrap();
        approved.write(red_frame(), &cancel).unwrap();
        assert!(fs::read(&path).unwrap().starts_with(b"\x89PNG"));

        let link = dir.path().join("unsafe.png");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(Destination::inspect(link).is_err());
        let absent = dir.path().join("cancel.png");
        cancel.cancel();
        assert!(
            Destination::inspect(absent.clone())
                .unwrap()
                .write(red_frame(), &cancel)
                .is_err()
        );
        assert!(!absent.exists());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    }
}
