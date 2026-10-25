//! FD-04 worker-only in-memory generation. No cache writes or production wiring.
//! This is full secure decode followed by resampling, NOT early/reduced decode.
use crate::{
    freedesktop_thumbnail::SizeClass,
    image_cache::FileKey,
    thumbnail_provider::{Error, Request},
};
use gtk::{gdk, gio, glib, prelude::*};
use std::{
    future::Future,
    os::unix::fs::MetadataExt,
    path::Path,
    pin::Pin,
    sync::Mutex,
    time::{Duration, Instant},
};

// Compile the exact existing boundary, including mandatory Bwrap, transforms and
// limits. Its foreground entry is intentionally unused in this bounded adapter.
#[allow(dead_code)]
#[path = "image_loader.rs"]
mod secure_loader;

pub const DECODE_BUDGET: usize = 512 * 1024 * 1024;
const MAX_FILE: u64 = 64 * 1024 * 1024;
const MAX_DOWNLOAD: usize = 224 * 1024 * 1024;
static ACTIVE: Mutex<Vec<(u64, u64)>> = Mutex::new(Vec::new());

#[derive(Debug)]
pub struct Metrics {
    pub decoded_width: u32,
    pub decoded_height: u32,
    pub frame_bytes: usize,
    pub download_bytes: usize,
    pub output_bytes: usize,
    pub decode_time: Duration,
    pub resize_time: Duration,
}
/// Tight, unpremultiplied RGBA8 in sRGB, ready for a future PNG writer.
/// Not a cache Candidate: there is no cache path, metadata PNG or failure marker.
#[derive(Debug)]
pub struct GeneratedThumbnail {
    pub source: FileKey,
    pub class: SizeClass,
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub pixels: glib::Bytes,
    pub metrics: Metrics,
}

/// Invoke only on an already admitted thumbnail worker, never the GTK main thread.
/// Creates no thread/pool/queue. Standalone admission is fail-fast, max two and
/// deduplicated by dev/inode (including hardlinks). FD-06 must share the existing
/// application worker permit; this separate adapter is not wired into the strip.
pub fn generate(request: &Request<'_>, class: SizeClass) -> Result<GeneratedThumbnail, Error> {
    generate_with(request, class, &SecureDecoder::default())
}

struct Admission((u64, u64));
impl Admission {
    fn acquire(identity: (u64, u64)) -> Result<Self, Error> {
        let mut jobs = ACTIVE
            .lock()
            .map_err(|_| Error::GenerationFailed("admission poisoned".into()))?;
        if jobs.len() >= 2 || jobs.contains(&identity) {
            return Err(Error::Busy);
        }
        jobs.push(identity);
        Ok(Self(identity))
    }
}
impl Drop for Admission {
    fn drop(&mut self) {
        ACTIVE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .retain(|key| *key != self.0);
    }
}
fn source_current(request: &Request<'_>) -> Result<(), Error> {
    request.check_control(Instant::now())?;
    let current =
        FileKey::read(&request.source.path).map_err(|e| Error::SourceUnavailable(e.to_string()))?;
    if current != *request.source {
        return Err(Error::SourceChanged);
    }
    Ok(())
}
fn preflight(request: &Request<'_>) -> Result<(u64, u64), Error> {
    source_current(request)?;
    let m = std::fs::metadata(&request.source.path)
        .map_err(|e| Error::SourceUnavailable(e.to_string()))?;
    if !m.is_file() || m.len() > MAX_FILE {
        return Err(Error::LimitExceeded(
            "regular source <=64MiB required".into(),
        ));
    }
    let result = (|| {
        let file = gio::File::for_path(&request.source.path);
        let stream = file.read(Some(request.cancel))?;
        stream.read_bytes(8, Some(request.cancel))
    })();
    source_current(request)?;
    let bytes = result.map_err(|e| Error::SourceUnavailable(e.to_string()))?;
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") && !bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Err(Error::InvalidSource(
            "not a JPEG/PNG signature (or truncated input)".into(),
        ));
    }
    Ok((m.dev(), m.ino()))
}

struct DecodeControl<'a> {
    child: gio::Cancellable,
    parent: &'a gio::Cancellable,
    handler: Option<gio::CancelledHandlerId>,
    timer: glib::Source,
}
impl<'a> DecodeControl<'a> {
    fn new(request: &Request<'a>, context: &glib::MainContext) -> Self {
        let child = gio::Cancellable::new();
        let cancel = child.clone();
        let handler = request.cancel.connect_cancelled(move |_| cancel.cancel());
        let cancel = child.clone();
        let remaining = request.deadline.saturating_duration_since(Instant::now());
        // GLib timers use integer milliseconds; round UP, never cancel early.
        let millis = remaining
            .as_millis()
            .saturating_add(1)
            .min(u128::from(u32::MAX)) as u64;
        let deadline = request.deadline;
        let timer = glib::timeout_source_new(
            Duration::from_millis(millis),
            Some("fd04-decode-deadline"),
            glib::Priority::DEFAULT,
            move || {
                if Instant::now() < deadline {
                    return glib::ControlFlow::Continue;
                }
                cancel.cancel();
                glib::ControlFlow::Break
            },
        );
        timer.attach(Some(context));
        Self {
            child,
            parent: request.cancel,
            handler,
            timer,
        }
    }
}
impl Drop for DecodeControl<'_> {
    fn drop(&mut self) {
        self.timer.destroy();
        if let Some(handler) = self.handler.take() {
            self.parent.disconnect_cancelled(handler);
        }
        self.child.cancel();
    }
}
struct Decoded {
    texture: gdk::Texture,
    frame_bytes: usize,
}
type DecodeFuture<'a> = Pin<Box<dyn Future<Output = Result<Decoded, Error>> + 'a>>;
// Private seam for controlled tests only; public generate always uses SecureDecoder.
trait Decoder {
    fn decode<'a>(&'a self, path: &'a Path, cancel: &'a gio::Cancellable) -> DecodeFuture<'a>;
}
#[derive(Default)]
struct SecureDecoder {
    reservation: std::cell::RefCell<Option<Reservation>>,
}
static MEMORY: Mutex<(usize, bool)> = Mutex::new((0, false));
struct Reservation {
    cost: usize,
    large: bool,
    pixels: u64,
}
impl Reservation {
    fn acquire(pixels: u64, available: usize) -> Result<Self, Error> {
        if pixels == 0 || pixels > 50_000_000 {
            return Err(Error::LimitExceeded("experimental ceiling 50 MP".into()));
        }
        let cost = usize::try_from(pixels)
            .ok()
            .and_then(|p| p.checked_mul(32))
            .and_then(|v| v.checked_add(128 * 1024 * 1024))
            .ok_or_else(|| Error::LimitExceeded("memory accounting overflow".into()))?;
        let large = pixels > 8_388_608;
        let mut budget = MEMORY
            .lock()
            .map_err(|_| Error::GenerationFailed("memory admission poisoned".into()))?;
        if large && budget.1 {
            return Err(Error::Busy);
        }
        let total = budget
            .0
            .checked_add(cost)
            .ok_or_else(|| Error::LimitExceeded("global memory accounting overflow".into()))?;
        if total > 2 * 1024 * 1024 * 1024 || available < total.saturating_add(512 * 1024 * 1024) {
            return Err(Error::LimitExceeded(
                "insufficient memory headroom for thumbnail reservation".into(),
            ));
        }
        budget.0 = total;
        if large {
            budget.1 = true;
        }
        Ok(Self {
            cost,
            large,
            pixels,
        })
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        let mut b = MEMORY.lock().unwrap_or_else(|e| e.into_inner());
        b.0 -= self.cost;
        if self.large {
            b.1 = false;
        }
    }
}
/// Observed host/visible-cgroup headroom; not an RSS hard cap or reservation.
pub fn memory_headroom() -> Result<usize, Error> {
    let text = std::fs::read_to_string("/proc/meminfo").map_err(|e| Error::Io(e.to_string()))?;
    let mut available = text
        .lines()
        .find_map(|line| {
            line.strip_prefix("MemAvailable:")
                .and_then(|v| v.split_whitespace().next())
                .and_then(|v| v.parse::<usize>().ok())
        })
        .and_then(|v| v.checked_mul(1024))
        .ok_or_else(|| Error::LimitExceeded("MemAvailable unavailable".into()))?;
    // Include every visible finite v2 ancestor; never modify cgroup settings.
    let base = std::path::Path::new("/sys/fs/cgroup");
    let mut paths = vec![base.to_path_buf()];
    if let Ok(groups) = std::fs::read_to_string("/proc/self/cgroup") {
        if let Some(relative) = groups.lines().find_map(|s| s.strip_prefix("0::")) {
            let path = base.join(relative.trim_start_matches('/'));
            if !path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                let mut current = path.as_path();
                while current.starts_with(base) {
                    paths.push(current.to_path_buf());
                    let Some(parent) = current.parent() else {
                        break;
                    };
                    current = parent;
                }
            }
        }
    }
    for path in paths {
        match std::fs::read_to_string(path.join("memory.max")) {
            Ok(max) if max.trim() != "max" => {
                let max = max
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| Error::LimitExceeded("invalid cgroup memory.max".into()))?;
                let current = std::fs::read_to_string(path.join("memory.current"))
                    .map_err(|e| Error::Io(e.to_string()))?
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| Error::LimitExceeded("invalid cgroup memory.current".into()))?;
                available = available.min(max.saturating_sub(current));
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(Error::Io(e.to_string())),
        }
    }
    Ok(available)
}
impl Decoder for SecureDecoder {
    fn decode<'a>(&'a self, path: &'a Path, cancel: &'a gio::Cancellable) -> DecodeFuture<'a> {
        Box::pin(async move {
            let failure = std::cell::RefCell::new(None);
            let admit = |w, h| {
                let result = memory_headroom().and_then(|available| {
                    Reservation::acquire(u64::from(w) * u64::from(h), available)
                });
                match result {
                    Ok(r) => {
                        self.reservation.replace(Some(r));
                        Ok(())
                    }
                    Err(e) => {
                        failure.replace(Some(e));
                        Err("Thumbnail memory admission refused".into())
                    }
                }
            };
            let result = secure_loader::load_thumbnail(path, cancel, &admit).await;
            if let Some(error) = failure.into_inner() {
                return Err(error);
            }
            let frame = result.map_err(loader_error)?;
            let min_len = (frame.height() as usize - 1)
                .checked_mul(frame.stride() as usize)
                .and_then(|v| v.checked_add(frame.width() as usize))
                .ok_or_else(|| Error::LimitExceeded("frame stride overflow".into()))?;
            if frame.stride() < frame.width() || frame.buf_slice().len() < min_len {
                return Err(Error::GenerationFailed(
                    "invalid actual frame stride/buffer".into(),
                ));
            }
            if std::env::var_os("PIVRA_TRACE_FD05A").is_some_and(|v| v == "1") {
                eprintln!(
                    "FD05A_FRAME width={} height={} stride={} format={:?} bytes={}",
                    frame.width(),
                    frame.height(),
                    frame.stride(),
                    frame.memory_format(),
                    frame.buf_slice().len()
                );
            }
            if self
                .reservation
                .borrow()
                .as_ref()
                .is_none_or(|r| u64::from(frame.width()) * u64::from(frame.height()) > r.pixels)
            {
                return Err(Error::LimitExceeded(
                    "actual frame exceeds reserved pixels".into(),
                ));
            }
            if frame.buf_slice().len() > DECODE_BUDGET {
                return Err(Error::LimitExceeded("actual frame buffer >512MiB".into()));
            }
            fitted_size(frame.width(), frame.height(), SizeClass::XxLarge)?;
            Ok(Decoded {
                texture: frame.texture(),
                frame_bytes: frame.buf_slice().len(),
            })
        })
    }
}
fn loader_error(message: String) -> Error {
    // Exact messages owned by our unchanged boundary, never filename/content matching.
    match message.as_str() {
        "Thumbnail exceeds experimental 50 MP / 32768 edge ceiling"
        | "Preload exceeds cache entry budget"
        | "Decoded frame exceeds the 512 MiB buffer limit"
        | "Image exceeds the 32 MP / 32768-pixel edge limit or has invalid dimensions" => {
            Error::LimitExceeded(message)
        }
        "Decoded content is not a JPEG or PNG image" => Error::InvalidSource(message),
        _ => Error::GenerationFailed(message),
    }
}

fn generate_with(
    request: &Request<'_>,
    class: SizeClass,
    decoder: &impl Decoder,
) -> Result<GeneratedThumbnail, Error> {
    let identity = preflight(request)?;
    let _admission = Admission::acquire(identity)?;
    source_current(request)?;
    let context = glib::MainContext::new();
    context
        .with_thread_default(|| {
            let control = DecodeControl::new(request, &context);
            source_current(request)?;
            let begin = Instant::now();
            let decoded = context.block_on(decoder.decode(&request.source.path, &control.child));
            // Explicit user cancel wins over deadline; both win over opaque loader errors.
            source_current(request)?;
            let decoded = decoded?;
            if decoded.frame_bytes > DECODE_BUDGET {
                return Err(Error::LimitExceeded("actual frame buffer >512MiB".into()));
            }
            let decode_time = begin.elapsed();
            let start = Instant::now();
            let (width, height, pixels, download_bytes) =
                resample(&decoded.texture, class, request)?;
            source_current(request)?;
            Ok(GeneratedThumbnail {
                source: request.source.clone(),
                class,
                width,
                height,
                stride: width as usize * 4,
                metrics: Metrics {
                    decoded_width: decoded.texture.width() as u32,
                    decoded_height: decoded.texture.height() as u32,
                    frame_bytes: decoded.frame_bytes,
                    download_bytes,
                    output_bytes: pixels.len(),
                    decode_time,
                    resize_time: start.elapsed(),
                },
                pixels,
            })
        })
        .map_err(|e| Error::GenerationFailed(format!("worker context unavailable: {e}")))?
}

fn fitted_size(width: u32, height: u32, class: SizeClass) -> Result<(u32, u32), Error> {
    if width == 0
        || height == 0
        || width > 32768
        || height > 32768
        || u64::from(width) * u64::from(height) > 50_000_000
    {
        return Err(Error::LimitExceeded(
            "experimental thumbnail ceiling 50 MP / 32768 edge".into(),
        ));
    }
    let edge = class.max_edge();
    if width.max(height) <= edge {
        return Ok((width, height));
    }
    if width >= height {
        Ok((
            edge,
            (u64::from(height) * u64::from(edge) / u64::from(width)).max(1) as u32,
        ))
    } else {
        Ok((
            (u64::from(width) * u64::from(edge) / u64::from(height)).max(1) as u32,
            edge,
        ))
    }
}
fn resample(
    texture: &gdk::Texture,
    class: SizeClass,
    request: &Request<'_>,
) -> Result<(u32, u32, glib::Bytes, usize), Error> {
    source_current(request)?;
    let (width, height) = fitted_size(texture.width() as u32, texture.height() as u32, class)?;
    let mut downloader = gdk::TextureDownloader::new(texture);
    downloader.set_format(gdk::MemoryFormat::R8g8b8a8);
    downloader.set_color_state(&gdk::ColorState::srgb());
    let (bytes, stride) = downloader.download_bytes();
    source_current(request)?;
    let download_bytes = bytes.len();
    if download_bytes > MAX_DOWNLOAD
        || stride < texture.width() as usize * 4
        || stride > i32::MAX as usize
    {
        return Err(Error::LimitExceeded("RGBA download bound/stride".into()));
    }
    let required = (texture.height() as usize - 1)
        .checked_mul(stride)
        .and_then(|x| x.checked_add(texture.width() as usize * 4))
        .ok_or_else(|| Error::LimitExceeded("RGBA stride overflow".into()))?;
    if bytes.len() < required {
        return Err(Error::GenerationFailed("short decoded RGBA buffer".into()));
    }
    let (small, small_stride) =
        if (width as i32, height as i32) == (texture.width(), texture.height()) {
            (bytes, stride)
        } else {
            let pixbuf = gdk::gdk_pixbuf::Pixbuf::from_bytes(
                &bytes,
                gdk::gdk_pixbuf::Colorspace::Rgb,
                true,
                8,
                texture.width(),
                texture.height(),
                stride as i32,
            );
            let scaled = pixbuf
                .scale_simple(
                    width as i32,
                    height as i32,
                    gdk::gdk_pixbuf::InterpType::Bilinear,
                )
                .ok_or_else(|| Error::GenerationFailed("thumbnail resampling failed".into()))?;
            (scaled.read_pixel_bytes(), scaled.rowstride() as usize)
        };
    source_current(request)?;
    let row = width as usize * 4;
    let len = row
        .checked_mul(height as usize)
        .ok_or_else(|| Error::LimitExceeded("output size overflow".into()))?;
    if len > class.max_edge() as usize * class.max_edge() as usize * 4 {
        return Err(Error::LimitExceeded("output class bound".into()));
    }
    let mut pixels = vec![0; len];
    for y in 0..height as usize {
        request.check_control(Instant::now())?;
        let input = small
            .get(y * small_stride..y * small_stride + row)
            .ok_or_else(|| Error::GenerationFailed("short resampled RGBA buffer".into()))?;
        pixels[y * row..(y + 1) * row].copy_from_slice(input);
    }
    source_current(request)?;
    Ok((
        width,
        height,
        glib::Bytes::from_owned(pixels),
        download_bytes,
    ))
}

#[cfg(test)]
#[path = "../tests/freedesktop/generator.rs"]
mod tests;
