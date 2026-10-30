//! Worker-only GIO cache validation and a bounded native GNOME factory adapter.
//! Cache pixels remain untrusted; callers must use secure Glycin decoding.
use crate::image_cache::FileKey;
use gtk::{gio, prelude::*};
use std::{
    fs,
    io::Read,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
const MAX_CACHE: u64 = 2 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    Unavailable,
    CacheOnly,
    CacheAndGenerate,
    TemporarilyFailed,
}
#[derive(Debug)]
pub enum Outcome {
    Hit(Candidate),
    Missing,
    FailedCached,
    Stale,
    Invalid(String),
    SourceUnavailable(String),
    Cancelled,
    Busy,
    /// A completed helper attempt exceeded its bounded deadline, not admission.
    TimedOut,
    UnsupportedDesktop,
}
#[derive(Debug)]
pub struct Candidate {
    pub uri: String,
    pub source: FileKey,
    pub thumbnail: FileKey,
    /// Container/metadata validated only. Still untrusted pixels: secure decode required.
    pub png: Vec<u8>,
}
impl Candidate {
    pub fn still_current(&self, cancel: &gio::Cancellable) -> bool {
        !cancel.is_cancelled()
            && FileKey::read(&self.source.path).ok().as_ref() == Some(&self.source)
            && FileKey::read(&self.thumbnail.path).ok().as_ref() == Some(&self.thumbnail)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Generator {
    Absent,
    Available,
    Transient,
}
fn capability(gnome: bool, generator: Generator) -> Capability {
    if !gnome {
        Capability::Unavailable
    } else {
        match generator {
            Generator::Absent => Capability::CacheOnly,
            Generator::Available => Capability::CacheAndGenerate,
            Generator::Transient => Capability::TemporarilyFailed,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FactoryObservation {
    NotConfigured,
    NotAttempted,
    MimeUnavailable,
    Eligible { mime: String },
    Ineligible { mime: String },
    Failed { mime: String, error: String },
}
pub struct Provider {
    gnome: bool,
    /// Explicit trusted metadata-only factory helper; never a decoder executable.
    factory: Option<PathBuf>,
    generator: Mutex<Generator>,
    factory_observation: Mutex<FactoryObservation>,
    busy: AtomicBool,
    eligible: AtomicBool,
}
impl Provider {
    pub fn new(gnome: bool, factory: Option<PathBuf>) -> Self {
        let observation = if factory.is_some() {
            FactoryObservation::NotAttempted
        } else {
            FactoryObservation::NotConfigured
        };
        Self {
            gnome,
            factory,
            factory_observation: Mutex::new(observation),
            generator: Mutex::new(Generator::Absent),
            busy: AtomicBool::new(false),
            eligible: AtomicBool::new(false),
        }
    }
    pub fn discover(factory: Option<PathBuf>) -> Self {
        let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        Self::new(
            desktop.split(':').any(|s| s.eq_ignore_ascii_case("GNOME")),
            factory,
        )
    }
    pub fn capability(&self) -> Capability {
        capability(self.gnome, *self.generator.lock().unwrap())
    }
    pub fn factory_observation(&self) -> FactoryObservation {
        self.factory_observation.lock().unwrap().clone()
    }
    pub fn generation_eligible(&self) -> bool {
        self.capability() == Capability::CacheAndGenerate && self.eligible.load(Ordering::Acquire)
    }
    /// UI-facing entry point: reuse GIO's blocking worker pool; no GTK mutation.
    pub async fn query_on_worker(
        self: std::sync::Arc<Self>,
        path: PathBuf,
        cancel: gio::Cancellable,
    ) -> Outcome {
        gio::spawn_blocking(move || self.query(&path, &cancel))
            .await
            .unwrap_or_else(|_| Outcome::Invalid("thumbnail metadata worker failed".into()))
    }
    /// One request, no pending queue; worker-only. Factory probe bounded to one second.
    pub fn query(&self, path: &Path, cancel: &gio::Cancellable) -> Outcome {
        if !self.gnome {
            return Outcome::UnsupportedDesktop;
        }
        if self.busy.swap(true, Ordering::AcqRel) {
            return Outcome::Busy;
        }
        struct Release<'a>(&'a AtomicBool);
        impl Drop for Release<'_> {
            fn drop(&mut self) {
                self.0.store(false, Ordering::Release);
            }
        }
        let _release = Release(&self.busy);
        self.eligible.store(false, Ordering::Release);
        *self.factory_observation.lock().unwrap() = if self.factory.is_some() {
            FactoryObservation::NotAttempted
        } else {
            FactoryObservation::NotConfigured
        };
        if cancel.is_cancelled() {
            return Outcome::Cancelled;
        }
        let source = match FileKey::read(path) {
            Ok(s) => s,
            Err(e) => return Outcome::SourceUnavailable(e.to_string()),
        };
        let file = gio::File::for_path(path);
        let uri = file.uri().to_string();
        // GIO validates thumbnails even for a path-only query and memory-maps them.
        // Preflight standard locations before invoking that native metadata parser.
        if let Err(error) = cache_preflight(&uri) {
            return Outcome::Invalid(error);
        }

        let info = match file.query_info(
            "thumbnail::path,thumbnail::failed,thumbnail::is-valid,standard::content-type,time::modified",
            gio::FileQueryInfoFlags::NONE,
            Some(cancel),
        ) {
            Ok(i) => i,
            Err(e) => {
                return if cancel.is_cancelled() {
                    Outcome::Cancelled
                } else {
                    Outcome::SourceUnavailable(e.to_string())
                };
            }
        };
        if let Some(helper) = &self.factory {
            let mime = info
                .content_type()
                .and_then(|c| gio::content_type_get_mime_type(&c));
            if let Some(mime) = mime {
                let state = factory_probe(
                    helper,
                    &uri,
                    &mime,
                    info.attribute_uint64("time::modified"),
                    cancel,
                );
                *self.factory_observation.lock().unwrap() = match &state {
                    Ok(true) => FactoryObservation::Eligible {
                        mime: mime.to_string(),
                    },
                    Ok(false) => FactoryObservation::Ineligible {
                        mime: mime.to_string(),
                    },
                    Err(error) => FactoryObservation::Failed {
                        mime: mime.to_string(),
                        error: error.clone(),
                    },
                };
                self.eligible
                    .store(matches!(state, Ok(true)), Ordering::Release);
                let mut previous = self.generator.lock().unwrap();
                *previous = match state {
                    Ok(true) => Generator::Available,
                    Ok(false) if *previous == Generator::Available => Generator::Available,
                    Ok(false) => Generator::Absent,
                    Err(_)
                        if *previous == Generator::Available
                            || *previous == Generator::Transient =>
                    {
                        Generator::Transient
                    }
                    Err(_) => Generator::Absent,
                };
            } else {
                *self.factory_observation.lock().unwrap() = FactoryObservation::MimeUnavailable;
            }
        }
        if cancel.is_cancelled() {
            return Outcome::Cancelled;
        }
        // Ask GIO for the byte-string as an OsString, preserving non-UTF8 paths.
        let Some(cache) = info
            .attribute_byte_string("thumbnail::path")
            .map(|p| PathBuf::from(std::ffi::OsStr::from_bytes(p.as_bytes())))
        else {
            return if info.boolean("thumbnail::failed") && info.boolean("thumbnail::is-valid") {
                Outcome::FailedCached
            } else {
                Outcome::Missing
            };
        };
        use std::os::unix::ffi::OsStrExt;
        let metadata = match fs::symlink_metadata(&cache) {
            Ok(m) => m,
            Err(_) => return Outcome::Stale,
        };
        if !metadata.is_file() || metadata.len() > MAX_CACHE {
            return Outcome::Invalid("cache must be regular, <=2MiB".into());
        }
        let cache_key = match FileKey::read(&cache) {
            Ok(k) => k,
            Err(_) => return Outcome::Stale,
        };
        let mut bytes = Vec::new();
        let result =
            fs::File::open(&cache).and_then(|f| f.take(MAX_CACHE + 1).read_to_end(&mut bytes));
        if result.is_err() || bytes.len() as u64 > MAX_CACHE {
            return Outcome::Invalid("cache read bound/error".into());
        }
        if let Err(e) = png_container(&bytes, cancel) {
            return if cancel.is_cancelled() {
                Outcome::Cancelled
            } else {
                Outcome::Invalid(e)
            };
        }
        // GIO is the standard authority for Thumb::URI/MTime/(optional)Size.
        let valid = match file.query_info(
            "thumbnail::is-valid,thumbnail::path",
            gio::FileQueryInfoFlags::NONE,
            Some(cancel),
        ) {
            Ok(i) => {
                i.boolean("thumbnail::is-valid")
                    && i.attribute_byte_string("thumbnail::path")
                        .is_some_and(|p| p.as_bytes() == cache.as_os_str().as_bytes())
            }
            Err(_) => false,
        };
        if cancel.is_cancelled() {
            return Outcome::Cancelled;
        }
        if !valid
            || FileKey::read(path).ok().as_ref() != Some(&source)
            || FileKey::read(&cache).ok().as_ref() != Some(&cache_key)
        {
            crate::thumbnail_diagnostics::trace("cache-stale-validation", Some(path),
                format_args!("gio_valid={valid} source_unchanged={} cache_unchanged={}",
                    FileKey::read(path).ok().as_ref() == Some(&source),
                    FileKey::read(&cache).ok().as_ref() == Some(&cache_key)));
            return Outcome::Stale;
        }
        // Reject same-path replacement even when seconds-resolution Thumb::MTime survives.
        let src = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return Outcome::Stale,
        };
        if (src.ctime(), src.ctime_nsec()) > (metadata.mtime(), metadata.mtime_nsec()) {
            crate::thumbnail_diagnostics::trace("cache-stale-ctime", Some(path),
                format_args!("source_ctime={}.{} cache_mtime={}.{}",
                    src.ctime(), src.ctime_nsec(), metadata.mtime(), metadata.mtime_nsec()));
            return Outcome::Stale;
        }
        Outcome::Hit(Candidate {
            uri,
            source,
            thumbnail: cache_key,
            png: bytes,
        })
    }
}
/// Build-time API-checked GNOME helper; an installed sidecar may be supplied by
/// the native package. Never take a decoder executable from media metadata.
pub fn generation_helper() -> Option<PathBuf> {
    std::env::var_os("PIVRA_THUMBNAIL_GENERATOR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .map(|p| p.with_file_name("pivra-thumbnail-factory"))
                .filter(|p| p.is_file())
        })
        .or_else(|| {
            option_env!("PIVRA_BUILT_THUMBNAIL_FACTORY")
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
        })
}
/// Worker-only, cache first. A failed attempt is terminal until source/catalog
/// invalidation; no implicit retries and no persistent failure marker on sandbox errors.
pub fn query_or_generate(provider: &Provider, path: &Path, cancel: &gio::Cancellable) -> Outcome {
    crate::thumbnail_diagnostics::trace("gnome-query", Some(path), "cache lookup");
    let outcome = provider.query(path, cancel);
    match &outcome {
        Outcome::Hit(_) => {
            crate::thumbnail_diagnostics::trace("gnome-query-result", Some(path), "cache-hit")
        }
        _ => crate::thumbnail_diagnostics::trace(
            "gnome-query-result",
            Some(path),
            format_args!("{outcome:?}"),
        ),
    }
    // Corrupt PNG caches are recoverable via the GNOME factory. Do not
    // regenerate for unrelated validation failures (unsafe path, oversized
    // cache, unavailable source, etc.).
    let corrupt_png = matches!(&outcome, Outcome::Invalid(error) if
        matches!(error.as_str(),
            "PNG signature" | "truncated PNG chunk" | "PNG chunk bounds" |
            "PNG CRC bounds" | "PNG CRC" | "PNG IHDR" | "thumbnail edge bound" |
            "duplicate IHDR" | "PNG ending" | "PNG chunk count" | "PNG overflow"));
    if !matches!(outcome, Outcome::Missing | Outcome::Stale) && !corrupt_png {
        return outcome;
    }
    // GIO may classify malformed existing normal-cache PNGs as Missing.
    // Force regeneration rather than letting GNOME's cached lookup short-circuit.
    let existing_normal = {
        let uri = gio::File::for_path(path).uri();
        gtk::glib::Checksum::new(gtk::glib::ChecksumType::Md5)
            .and_then(|mut hash| {
                hash.update(uri.as_bytes());
                hash.string().map(|digest| {
                    gtk::glib::user_cache_dir()
                        .join("thumbnails/normal")
                        .join(format!("{digest}.png"))
                        .is_file()
                })
            })
            .unwrap_or(false)
    };
    let force = matches!(outcome, Outcome::Stale) || corrupt_png || existing_normal;
    let Some(helper) = generation_helper() else {
        crate::thumbnail_diagnostics::trace(
            "helper-missing",
            Some(path),
            format_args!("outcome={outcome:?}"),
        );
        return outcome;
    };
    crate::thumbnail_diagnostics::trace(
        "helper-invoke",
        Some(path),
        format_args!("helper={} force={force}", helper.display()),
    );
    let source = match FileKey::read(path) {
        Ok(key) => key,
        Err(e) => return Outcome::SourceUnavailable(e.to_string()),
    };
    let file = gio::File::for_path(path);
    let info = match file.query_info(
        "standard::content-type,time::modified,standard::size",
        gio::FileQueryInfoFlags::NONE,
        Some(cancel),
    ) {
        Ok(info) => info,
        Err(e) => return Outcome::SourceUnavailable(e.to_string()),
    };
    // The desktop video thumbnailer reads compressed streams, not decoded image
    // pixels. Keep the image limit unchanged and bound videos separately.
    let is_video = crate::discovery::candidate_kind(path) == Some(crate::media::MediaKind::Video);
    let max_size = if is_video { 8_i64 * 1024 * 1024 * 1024 } else { 64_i64 * 1024 * 1024 };
    if info.size() > max_size {
        return Outcome::Invalid("source exceeds thumbnail generation size limit".into());
    }
    let Some(mime) = info
        .content_type()
        .and_then(|t| gio::content_type_get_mime_type(&t))
    else {
        return Outcome::Missing;
    };
    crate::thumbnail_diagnostics::trace(
        "helper-input",
        Some(path),
        format_args!(
            "mime={mime} bytes={} mtime={}",
            info.size(),
            info.attribute_uint64("time::modified")
        ),
    );
    let result = generate(
        &helper,
        file.uri().as_str(),
        &mime,
        info.attribute_uint64("time::modified"),
        force,
        cancel,
    );
    crate::thumbnail_diagnostics::trace("helper-result", Some(path), format_args!("{result:?}"));
    match result {
        Ok(()) if FileKey::read(path).ok().as_ref() == Some(&source) => {
            provider.query(path, cancel)
        }
        Ok(()) => Outcome::Stale,
        Err(_) if cancel.is_cancelled() => Outcome::Cancelled,
        Err(error) if error == "GNOME factory timed out" => Outcome::TimedOut,
        Err(error) => Outcome::Invalid(error),
    }
}
struct FactoryScratch(PathBuf);
impl FactoryScratch {
    fn new() -> Result<Self, String> {
        use std::os::unix::fs::DirBuilderExt;
        let path = std::env::temp_dir().join(format!(
            "pivra-thumbnail-{}",
            gtk::glib::uuid_string_random()
        ));
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .map_err(|e| e.to_string())?;
        Ok(Self(path))
    }
}
impl Drop for FactoryScratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn generate(
    helper: &Path,
    uri: &str,
    mime: &str,
    mtime: u64,
    force: bool,
    cancel: &gio::Cancellable,
) -> Result<(), String> {
    use std::os::unix::process::CommandExt;
    if uri.len() > 16384 || mime.len() > 256 {
        return Err("factory argument bound".into());
    }
    if cancel.is_cancelled() {
        return Err("cancelled".into());
    }
    let scratch = FactoryScratch::new()?;
    let diagnostic = crate::thumbnail_diagnostics::enabled();
    // Write to a private file instead of a pipe to avoid child stderr backpressure.
    let stderr_path = scratch.0.join("factory-stderr.log");
    let stderr = if diagnostic {
        Stdio::from(fs::File::create(&stderr_path).map_err(|e| e.to_string())?)
    } else {
        Stdio::null()
    };
    let mut child = Command::new(helper)
        .env("TMPDIR", &scratch.0)
        .args([
            uri,
            mime,
            &mtime.to_string(),
            if force { "stale" } else { "missing" },
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(stderr)
        .process_group(0)
        .spawn()
        .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if cancel.is_cancelled() || Instant::now() >= deadline {
            // The unreaped direct child's PID reserves this group identity.
            // GNOME bwrap also uses --die-with-parent. Kill/reap before returning.
            let _ = Command::new("/bin/kill")
                .args(["-KILL", "--", &format!("-{}", child.id())])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = child.kill();
            let _ = child.wait();
            return Err(if cancel.is_cancelled() {
                "GNOME factory cancelled"
            } else {
                "GNOME factory timed out"
            }
            .into());
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut text = String::new();
                if let Some(stdout) = child.stdout.take() {
                    stdout
                        .take(512)
                        .read_to_string(&mut text)
                        .map_err(|e| e.to_string())?;
                }
                return if status.success() && matches!(text.as_str(), "cached\n" | "generated\n") {
                    Ok(())
                } else {
                    let details = if diagnostic {
                        let details = read_factory_diagnostics(&stderr_path);
                        let source_path = gio::File::for_uri(uri).path();
                        // The regular helper-result event is capped at 512 chars;
                        // emit bounded chunks so the actual child error is not
                        // silently lost behind the expanded bwrap command.
                        for (index, chunk) in
                            details.chars().collect::<Vec<_>>().chunks(384).enumerate()
                        {
                            crate::thumbnail_diagnostics::trace(
                                "helper-stderr",
                                source_path.as_deref(),
                                format_args!("part={index} {}", chunk.iter().collect::<String>()),
                            );
                        }
                        details
                    } else {
                        String::new()
                    };
                    Err(format!(
                        "GNOME factory exit={status}: {text}; stderr={details}"
                    ))
                };
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.to_string());
            }
        }
    }
}

// Native log handler emits at most sixteen bounded lines; cap arbitrary helper
// stderr as well. Invalid UTF-8 is diagnostic data, not a protocol failure.
fn read_factory_diagnostics(path: &Path) -> String {
    use std::io::{Seek, SeekFrom};
    fs::File::open(path)
        .and_then(|mut file| {
            let mut bytes = Vec::new();
            if file.metadata()?.len() > 16384 {
                (&mut file).take(8192).read_to_end(&mut bytes)?;
                bytes.extend_from_slice(b"\n[stderr middle omitted]\n");
                file.seek(SeekFrom::End(-8192))?;
                file.take(8192).read_to_end(&mut bytes)?;
            } else {
                file.take(16384).read_to_end(&mut bytes)?;
            }
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        })
        .unwrap_or_default()
}

fn cache_preflight(uri: &str) -> Result<(), String> {
    let mut hash =
        gtk::glib::Checksum::new(gtk::glib::ChecksumType::Md5).ok_or("MD5 unavailable")?;
    hash.update(uri.as_bytes());
    let name = format!("{}.png", hash.string().ok_or("MD5 string")?);
    let root = gtk::glib::user_cache_dir().join("thumbnails");
    for size in [
        "normal",
        "large",
        "x-large",
        "xx-large",
        "fail/gnome-thumbnail-factory",
    ] {
        match fs::symlink_metadata(root.join(size).join(&name)) {
            Ok(m) if !m.is_file() || m.len() > MAX_CACHE => {
                return Err("cache preflight: regular file <=2MiB required".into());
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("cache preflight: {e}")),
        }
    }
    Ok(())
}
fn factory_probe(
    helper: &Path,
    uri: &str,
    mime: &str,
    mtime: u64,
    cancel: &gio::Cancellable,
) -> Result<bool, String> {
    if uri.len() > 16384 || mime.len() > 256 {
        return Err("probe argument bound".into());
    }
    let mut child = Command::new(helper)
        .args([uri, mime, &mtime.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let end = Instant::now() + Duration::from_secs(1);
    loop {
        if cancel.is_cancelled() || Instant::now() >= end {
            let _ = child.kill();
            let _ = child.wait();
            return Err("factory metadata probe cancelled/timed out".into());
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut output = String::new();
                if let Some(stdout) = child.stdout.take() {
                    stdout
                        .take(32)
                        .read_to_string(&mut output)
                        .map_err(|e| e.to_string())?;
                }
                return if status.success() && output == "1\n" {
                    Ok(true)
                } else if status.success() && output == "0\n" {
                    Ok(false)
                } else {
                    Err(format!("factory probe exit={status}, stdout={output:?}"))
                };
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.to_string());
            }
        }
    }
}
/// Structural/CRC bounds, not a PNG decoder or replacement for GIO metadata validation.
fn png_container(bytes: &[u8], cancel: &gio::Cancellable) -> Result<(), String> {
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("PNG signature".into());
    }
    let mut at = 8usize;
    let mut idat = false;
    let mut ihdr = false;
    for _ in 0..2048 {
        if cancel.is_cancelled() {
            return Err("cancelled".into());
        }
        let header = bytes.get(at..at + 8).ok_or("truncated PNG chunk")?;
        let n = u32::from_be_bytes(header[..4].try_into().unwrap()) as usize;
        let end = at
            .checked_add(n)
            .and_then(|x| x.checked_add(12))
            .ok_or("PNG overflow")?;
        let chunk = bytes.get(at + 4..end - 4).ok_or("PNG chunk bounds")?;
        let expected = bytes.get(end - 4..end).ok_or("PNG CRC bounds")?;
        let mut crc = !0u32;
        for b in chunk {
            crc ^= u32::from(*b);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb88320u32.wrapping_mul(crc & 1));
            }
        }
        if (!crc).to_be_bytes().as_slice() != expected {
            return Err("PNG CRC".into());
        }
        if !ihdr {
            if &header[4..] != b"IHDR" || n != 13 {
                return Err("PNG IHDR".into());
            }
            let w = u32::from_be_bytes(chunk[4..8].try_into().unwrap());
            let h = u32::from_be_bytes(chunk[8..12].try_into().unwrap());
            if w == 0 || h == 0 || w > 1024 || h > 1024 {
                return Err("thumbnail edge bound".into());
            }
            ihdr = true;
        } else if &header[4..] == b"IHDR" {
            return Err("duplicate IHDR".into());
        }
        if &header[4..] == b"IDAT" {
            idat = true;
        }
        if &header[4..] == b"IEND" {
            return if n == 0 && idat && end == bytes.len() {
                Ok(())
            } else {
                Err("PNG ending".into())
            };
        }
        at = end;
    }
    Err("PNG chunk count".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn four_capabilities_are_distinct() {
        assert_eq!(
            capability(false, Generator::Absent),
            Capability::Unavailable
        );
        assert_eq!(capability(true, Generator::Absent), Capability::CacheOnly);
        assert_eq!(
            capability(true, Generator::Available),
            Capability::CacheAndGenerate
        );
        assert_eq!(
            capability(true, Generator::Transient),
            Capability::TemporarilyFailed
        );
    }
    #[test]
    fn png_bounds_crc_corruption_and_cancel() {
        let bytes = include_bytes!("../tests/fixtures/pixels-2x2.png");
        let token = gio::Cancellable::new();
        assert!(png_container(bytes, &token).is_ok());
        for bad in [b"text".as_slice(), &bytes[..bytes.len() - 1]] {
            assert!(png_container(bad, &token).is_err());
        }
        let mut bad = bytes.to_vec();
        bad[30] ^= 1;
        assert!(png_container(&bad, &token).is_err());
        token.cancel();
        assert!(png_container(bytes, &token).is_err());
    }
    #[test]
    fn candidate_identity_catches_replace_rename_and_cache_delete() {
        let d = tempfile::tempdir().unwrap();
        let source = d.path().join("source.png");
        let cache = d.path().join("cache.png");
        fs::write(&source, b"original").unwrap();
        fs::write(&cache, b"cached").unwrap();
        let candidate = Candidate {
            uri: gio::File::for_path(&source).uri().to_string(),
            source: FileKey::read(&source).unwrap(),
            thumbnail: FileKey::read(&cache).unwrap(),
            png: vec![],
        };
        let token = gio::Cancellable::new();
        assert!(candidate.still_current(&token));
        let temp = d.path().join("save");
        fs::write(&temp, b"original").unwrap();
        fs::rename(&temp, &source).unwrap();
        assert!(!candidate.still_current(&token));
        fs::rename(&source, d.path().join("renamed")).unwrap();
        assert!(!candidate.still_current(&token));
        fs::remove_file(cache).unwrap();
        assert!(!candidate.still_current(&token));
    }
    #[test]
    fn factory_success_then_loss_is_transient_without_affecting_cache_query() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        let source = d.path().join("input.png");
        fs::write(&source, include_bytes!("../tests/fixtures/pixels-2x2.png")).unwrap();
        let helper = d.path().join("factory-mock");
        fs::write(&helper, b"#!/bin/sh\nprintf '1\\n'\n").unwrap();
        fs::set_permissions(&helper, fs::Permissions::from_mode(0o700)).unwrap();
        let provider = Provider::new(true, Some(helper.clone()));
        let token = gio::Cancellable::new();
        assert!(matches!(provider.query(&source, &token), Outcome::Missing));
        assert_eq!(provider.capability(), Capability::CacheAndGenerate);
        assert!(provider.generation_eligible());
        assert!(matches!(
            provider.factory_observation(),
            FactoryObservation::Eligible { .. }
        ));
        fs::write(&helper, b"#!/bin/sh\nprintf '0\\n'\n").unwrap();
        assert!(matches!(provider.query(&source, &token), Outcome::Missing));
        assert_eq!(provider.capability(), Capability::CacheAndGenerate);
        assert!(!provider.generation_eligible());
        assert!(matches!(
            provider.factory_observation(),
            FactoryObservation::Ineligible { .. }
        ));
        fs::remove_file(helper).unwrap();
        assert!(matches!(provider.query(&source, &token), Outcome::Missing));
        assert_eq!(provider.capability(), Capability::TemporarilyFailed);
        assert!(!provider.generation_eligible());
        assert!(matches!(
            provider.factory_observation(),
            FactoryObservation::Failed { .. }
        ));
    }
    #[test]
    fn no_queue_and_cancel_do_not_change_capability() {
        let provider = Provider::new(true, None);
        let token = gio::Cancellable::new();
        provider.busy.store(true, Ordering::Release);
        assert!(matches!(
            provider.query(Path::new("unused"), &token),
            Outcome::Busy
        ));
        provider.busy.store(false, Ordering::Release);
        token.cancel();
        assert!(matches!(
            provider.query(Path::new("unused"), &token),
            Outcome::Cancelled
        ));
        assert_eq!(provider.capability(), Capability::CacheOnly);
    }
}

#[cfg(test)]
mod generation_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    #[test]
    fn factory_stderr_is_bounded_and_accepts_non_utf8() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("stderr");
        fs::write(&path, b"GNOME child failed: \xff").unwrap();
        assert!(read_factory_diagnostics(&path).contains("GNOME child failed:"));
        fs::write(&path, vec![b'x'; 20000]).unwrap();
        let details = read_factory_diagnostics(&path);
        assert!(details.len() <= 16448);
        assert!(details.contains("[stderr middle omitted]"));
        let mut bytes = vec![b'x'; 20000];
        bytes.extend_from_slice(b"final underlying failure");
        fs::write(&path, bytes).unwrap();
        assert!(read_factory_diagnostics(&path).ends_with("final underlying failure"));
        assert_eq!(read_factory_diagnostics(&root.path().join("absent")), "");
    }
    #[test]
    fn helper_protocol_failure_and_cancelled_process_group() {
        let root = tempfile::tempdir().unwrap();
        let helper = root.path().join("helper");
        fs::write(&helper, b"#!/bin/sh\nprintf 'generated\\n'\n").unwrap();
        fs::set_permissions(&helper, fs::Permissions::from_mode(0o700)).unwrap();
        let token = gio::Cancellable::new();
        assert!(generate(&helper, "file:///unused", "image/png", 0, false, &token).is_ok());
        fs::write(&helper, b"#!/bin/sh\nprintf 'ineligible\\n'\nexit 3\n").unwrap();
        assert!(generate(&helper, "file:///unused", "image/png", 0, false, &token).is_err());
        let pidfile = root.path().join("child.pid");
        let scratchfile = root.path().join("scratch.path");
        fs::write(&helper,format!("#!/bin/sh\nprintf '%s' \"$TMPDIR\" > '{scratch}'\ntouch \"$TMPDIR/artifact\"\nsleep 30 &\nprintf '%s' \"$!\" > '{pid}.tmp'\nmv '{pid}.tmp' '{pid}'\nwait\n",scratch=scratchfile.display(),pid=pidfile.display())).unwrap();
        let worker_token = token.clone();
        let worker = std::thread::spawn(move || {
            generate(
                &helper,
                "file:///unused",
                "image/png",
                0,
                false,
                &worker_token,
            )
        });
        let deadline = Instant::now() + Duration::from_secs(3);
        while !pidfile.exists() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        let pid = fs::read_to_string(pidfile).unwrap();
        token.cancel();
        assert!(worker.join().unwrap().is_err());
        let scratch = fs::read_to_string(scratchfile).unwrap();
        assert!(
            !Path::new(&scratch).exists(),
            "cancelled factory left private scratch files"
        );
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let finished = fs::read_to_string(format!("/proc/{pid}/stat"))
                .map(|s| s.split_once(") ").unwrap().1.starts_with(['Z', 'X']))
                .unwrap_or(true);
            if finished {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "factory descendant still running"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
