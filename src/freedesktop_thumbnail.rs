//! FD-03 read-only cache inspection. Not connected to provider selection or the UI.
//! PNG structure/metadata validation is NOT pixel decoding; Candidate remains untrusted.
use crate::{
    image_cache::FileKey,
    thumbnail_provider::{CacheOutcome, Candidate, Error, Request},
};
use gtk::{gio, glib, prelude::*};
use std::{
    ffi::OsStr,
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    os::{
        fd::AsRawFd,
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Component, Path, PathBuf},
    time::Instant,
};

const MAX_FILE: u64 = 2 * 1024 * 1024;
const MAX_TEXT: usize = 64 * 1024;
const MAX_CHUNKS: usize = 2048;
const MAX_FAILURE_APPS: usize = 64;
// Linux x86/x86_64/aarch64 open(2) flags; no new libc dependency.
// Other architectures fail closed until their ABI is explicitly verified.
const O_NOFOLLOW: i32 = 0x20000;
const O_NONBLOCK: i32 = 0x800;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeClass {
    Normal,
    Large,
    XLarge,
    XxLarge,
}
impl SizeClass {
    /// Smallest adequate standard class. No implicit larger-class fallback/read.
    pub fn for_edge(edge: u32) -> Option<Self> {
        match edge {
            1..=128 => Some(Self::Normal),
            129..=256 => Some(Self::Large),
            257..=512 => Some(Self::XLarge),
            513..=1024 => Some(Self::XxLarge),
            _ => None,
        }
    }
    pub fn directory(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Large => "large",
            Self::XLarge => "x-large",
            Self::XxLarge => "xx-large",
        }
    }
    pub fn max_edge(self) -> u32 {
        match self {
            Self::Normal => 128,
            Self::Large => 256,
            Self::XLarge => 512,
            Self::XxLarge => 1024,
        }
    }
}

fn absolute(path: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;
    path.is_absolute() && !path.as_os_str().as_bytes().contains(&0)
}
/// XDG requires an absolute path; invalid/unset values use HOME/.cache, not cwd.
/// Explicit inputs keep tests independent of global GLib directory caching/env races.
pub fn cache_home(xdg: Option<&OsStr>, home: &Path) -> Result<PathBuf, Error> {
    if let Some(path) = xdg.map(Path::new).filter(|p| absolute(p)) {
        return Ok(path.to_owned());
    }
    if !absolute(home) {
        return Err(Error::Io("absolute home directory unavailable".into()));
    }
    Ok(home.join(".cache"))
}
/// Hash GIO's file URI bytes, never source pixels or a canonicalized symlink target.
pub fn cache_name(source: &Path) -> String {
    let uri = gio::File::for_path(source).uri();
    let mut digest = glib::Checksum::new(glib::ChecksumType::Md5).expect("GLib MD5 support");
    digest.update(uri.as_bytes());
    format!("{}.png", digest.string().expect("GLib MD5 digest"))
}

pub struct CacheReader {
    home: PathBuf,
}
impl CacheReader {
    pub fn new(home: PathBuf) -> Result<Self, Error> {
        if !absolute(&home) {
            return Err(Error::Io("cache home must be absolute and NUL-free".into()));
        }
        Ok(Self { home })
    }
    pub fn from_environment() -> Result<Self, Error> {
        Self::new(cache_home(
            std::env::var_os("XDG_CACHE_HOME").as_deref(),
            &glib::home_dir(),
        )?)
    }
    /// Blocking worker operation. No directory creation, writes, generation or discovery.
    pub fn lookup(&self, request: &Request<'_>, class: SizeClass) -> Result<CacheOutcome, Error> {
        request.check_control(Instant::now())?;
        let source = match fs::metadata(&request.source.path) {
            Ok(m) if m.is_file() && m.mode() & 0o444 != 0 => m,
            _ => {
                return Ok(CacheOutcome::SourceUnavailable(
                    "source unavailable/unreadable".into(),
                ));
            }
        };
        if FileKey::read(&request.source.path).ok().as_ref() != Some(request.source) {
            return Ok(CacheOutcome::Stale);
        }
        let uri = gio::File::for_path(&request.source.path).uri().to_string();
        let name = cache_name(&request.source.path);
        let path = self
            .home
            .join("thumbnails")
            .join(class.directory())
            .join(&name);
        let outcome = inspect(&path, request, class, &uri, &source)?;
        if !matches!(outcome, CacheOutcome::Missing) {
            return Ok(outcome);
        }
        // Failure markers are namespaced by producer, never valid image candidates.
        let failed = self.home.join("thumbnails/fail");
        let directory = match open_directory(&failed) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(CacheOutcome::Missing),
            Err(e) => return cache_io(e),
        };
        let entries = fs::read_dir(fd_path(&directory)).map_err(|e| Error::Io(e.to_string()))?;
        for (count, entry) in entries.enumerate() {
            request.check_control(Instant::now())?;
            if count >= MAX_FAILURE_APPS {
                return Ok(CacheOutcome::Invalid(
                    "failure application count bound".into(),
                ));
            }
            let entry = entry.map_err(|e| Error::Io(e.to_string()))?;
            if !entry
                .file_type()
                .map_err(|e| Error::Io(e.to_string()))?
                .is_dir()
            {
                continue;
            }
            let path = failed.join(entry.file_name()).join(&name);
            match inspect(&path, request, SizeClass::XxLarge, &uri, &source)? {
                CacheOutcome::Hit(_) => return Ok(CacheOutcome::FailedCached),
                CacheOutcome::Missing | CacheOutcome::Stale => {}
                other => return Ok(other),
            }
        }
        request.check_control(Instant::now())?;
        if FileKey::read(&request.source.path).ok().as_ref() != Some(request.source) {
            return Ok(CacheOutcome::Stale);
        }
        Ok(CacheOutcome::Missing)
    }
}

fn fd_path(file: &File) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{}", file.as_raw_fd()))
}
fn open_component(path: &Path) -> io::Result<File> {
    if !cfg!(all(
        target_os = "linux",
        any(
            target_arch = "x86",
            target_arch = "x86_64",
            target_arch = "aarch64"
        )
    )) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "safe cache open ABI not verified for this target",
        ));
    }
    OpenOptions::new()
        .read(true)
        .custom_flags(O_NOFOLLOW | O_NONBLOCK)
        .open(path)
}
/// Walk from / using pinned directory descriptors; never traverse a cache symlink.
/// /proc/self/fd is kernel-owned. If unavailable, fail closed (no unsafe fallback).
fn open_directory(path: &Path) -> io::Result<File> {
    if !absolute(path) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "absolute cache path required",
        ));
    }
    let mut directory = File::open("/")?;
    for component in path.components() {
        if matches!(component, Component::RootDir | Component::CurDir) {
            continue;
        }
        let next = open_component(&fd_path(&directory).join(component.as_os_str()))?;
        let m = next.metadata()?;
        if !m.is_dir() || m.mode() & 0o111 == 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "cache directory inaccessible",
            ));
        }
        directory = next;
    }
    Ok(directory)
}
fn open_cache(path: &Path) -> io::Result<File> {
    let directory = open_directory(path.parent().expect("cache parent"))?;
    let file =
        open_component(&fd_path(&directory).join(path.file_name().expect("cache filename")))?;
    let m = file.metadata()?;
    if !m.is_file() || m.len() > MAX_FILE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "regular cache file <=2MiB required",
        ));
    }
    if m.mode() & 0o444 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "cache unreadable",
        ));
    }
    Ok(file)
}
fn cache_io(e: io::Error) -> Result<CacheOutcome, Error> {
    if e.kind() == io::ErrorKind::PermissionDenied {
        Err(Error::Io(format!("cache access: {e}")))
    } else {
        Ok(CacheOutcome::Invalid(format!("cache read: {e}")))
    }
}
fn same(a: &Metadata, b: &Metadata) -> bool {
    (
        a.dev(),
        a.ino(),
        a.len(),
        a.mtime(),
        a.mtime_nsec(),
        a.ctime(),
        a.ctime_nsec(),
    ) == (
        b.dev(),
        b.ino(),
        b.len(),
        b.mtime(),
        b.mtime_nsec(),
        b.ctime(),
        b.ctime_nsec(),
    )
}
fn current(
    path: &Path,
    file: &File,
    before: &Metadata,
    key: &FileKey,
    request: &Request<'_>,
) -> bool {
    FileKey::read(&request.source.path).ok().as_ref() == Some(request.source)
        && FileKey::read(path).ok().as_ref() == Some(key)
        && file.metadata().is_ok_and(|m| same(&m, before))
        && fs::symlink_metadata(path).is_ok_and(|m| m.is_file() && same(&m, before))
        && open_cache(path)
            .and_then(|f| f.metadata())
            .is_ok_and(|m| same(&m, before))
}
fn inspect(
    path: &Path,
    request: &Request<'_>,
    class: SizeClass,
    uri: &str,
    source: &Metadata,
) -> Result<CacheOutcome, Error> {
    request.check_control(Instant::now())?;
    let mut file = match open_cache(path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(CacheOutcome::Missing),
        Err(e) => return cache_io(e),
    };
    let before = file.metadata().map_err(|e| Error::Io(e.to_string()))?;
    let key = match FileKey::read(path) {
        Ok(k) => k,
        Err(_) => return Ok(CacheOutcome::Stale),
    };
    if !current(path, &file, &before, &key, request) {
        return Ok(CacheOutcome::Stale);
    }
    let metadata = match png_metadata(&mut file, before.len(), class, request) {
        Ok(m) => m,
        Err(ParseError::Control(e)) => return Err(e),
        Err(ParseError::Invalid(e)) => return Ok(CacheOutcome::Invalid(e)),
    };
    if metadata.uri.as_deref() != Some(uri) {
        return Ok(CacheOutcome::Invalid(
            "missing/mismatched Thumb::URI".into(),
        ));
    }
    let Some(mtime) = metadata.mtime else {
        return Ok(CacheOutcome::Invalid("missing Thumb::MTime".into()));
    };
    if source.mtime() < 0
        || mtime != source.mtime() as u64
        || metadata.size.is_some_and(|size| size != source.len())
        || (source.ctime(), source.ctime_nsec()) > (before.mtime(), before.mtime_nsec())
    {
        return Ok(CacheOutcome::Stale);
    }
    request.check_control(Instant::now())?;
    // Candidate requires bytes. Do not allocate the full PNG until validation passed.
    file.seek(SeekFrom::Start(0))
        .map_err(|e| Error::Io(e.to_string()))?;
    let mut png = Vec::with_capacity(before.len() as usize);
    let mut buffer = [0; 4096];
    loop {
        request.check_control(Instant::now())?;
        let result = file.read(&mut buffer);
        request.check_control(Instant::now())?;
        let n = result.map_err(|e| Error::Io(e.to_string()))?;
        if n == 0 {
            break;
        }
        if png.len() + n > before.len() as usize {
            return Ok(CacheOutcome::Stale);
        }
        png.extend_from_slice(&buffer[..n]);
    }
    request.check_control(Instant::now())?;
    if png.len() as u64 != before.len() || !current(path, &file, &before, &key, request) {
        return Ok(CacheOutcome::Stale);
    }
    Ok(CacheOutcome::Hit(Candidate {
        uri: uri.to_owned(),
        source: request.source.clone(),
        thumbnail: key,
        png,
    }))
}

#[derive(Default)]
struct PngMetadata {
    uri: Option<String>,
    mtime: Option<u64>,
    size: Option<u64>,
}
#[derive(Debug)]
enum ParseError {
    Invalid(String),
    Control(Error),
}
impl From<&str> for ParseError {
    fn from(message: &str) -> Self {
        Self::Invalid(message.into())
    }
}
fn control(request: &Request<'_>) -> Result<(), ParseError> {
    request
        .check_control(Instant::now())
        .map_err(ParseError::Control)
}
fn read_exact(
    reader: &mut impl Read,
    mut bytes: &mut [u8],
    request: &Request<'_>,
) -> Result<(), ParseError> {
    while !bytes.is_empty() {
        control(request)?;
        let result = reader.read(bytes);
        control(request)?;
        match result {
            Ok(0) => return Err("truncated PNG".into()),
            Ok(n) => bytes = &mut bytes[n..],
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(ParseError::Invalid(format!("PNG read: {e}"))),
        }
    }
    Ok(())
}
fn crc(mut value: u32, bytes: &[u8]) -> u32 {
    for byte in bytes {
        value ^= u32::from(*byte);
        for _ in 0..8 {
            value = (value >> 1) ^ (0xedb88320u32.wrapping_mul(value & 1));
        }
    }
    value
}
fn decimal(value: &[u8]) -> Result<u64, ParseError> {
    if value.is_empty() || value.len() > 20 || !value.iter().all(u8::is_ascii_digit) {
        return Err("invalid numeric thumbnail metadata".into());
    }
    std::str::from_utf8(value)
        .unwrap()
        .parse()
        .map_err(|_| "metadata integer overflow".into())
}
fn text(fields: &mut PngMetadata, data: &[u8]) -> Result<(), ParseError> {
    let nul = data
        .iter()
        .position(|b| *b == 0)
        .ok_or("PNG text keyword separator")?;
    if nul == 0 || nul > 79 {
        return Err("PNG text keyword bound".into());
    }
    let value = &data[nul + 1..];
    match &data[..nul] {
        b"Thumb::URI" => {
            if fields.uri.is_some() || value.is_empty() || value.len() > 16384 || value.contains(&0)
            {
                return Err("duplicate/invalid Thumb::URI".into());
            }
            fields.uri = Some(
                std::str::from_utf8(value)
                    .map_err(|_| "Thumb::URI encoding")?
                    .to_owned(),
            );
        }
        b"Thumb::MTime" | b"Thumb::Size" => {
            let slot = if &data[..nul] == b"Thumb::MTime" {
                &mut fields.mtime
            } else {
                &mut fields.size
            };
            if slot.is_some() {
                return Err("duplicate numeric thumbnail metadata".into());
            }
            *slot = Some(decimal(value)?);
        }
        _ => {}
    }
    Ok(())
}
/// Streaming PNG framing/CRC and tEXt fields. IDAT is never inflated/decoded.
fn png_metadata(
    reader: &mut impl Read,
    length: u64,
    class: SizeClass,
    request: &Request<'_>,
) -> Result<PngMetadata, ParseError> {
    if length > MAX_FILE {
        return Err("PNG file bound".into());
    }
    let mut signature = [0; 8];
    read_exact(reader, &mut signature, request)?;
    if &signature != b"\x89PNG\r\n\x1a\n" {
        return Err("PNG signature".into());
    }
    let mut at = 8u64;
    let mut fields = PngMetadata::default();
    let mut text_bytes = 0;
    let mut idat = false;
    let mut idat_ended = false;
    let mut palette = false;
    let mut color = 0;
    let mut depth = 0;
    for index in 0..MAX_CHUNKS {
        let mut header = [0; 8];
        read_exact(reader, &mut header, request)?;
        let n = u32::from_be_bytes(header[..4].try_into().unwrap()) as usize;
        let kind: &[u8; 4] = header[4..].try_into().unwrap();
        at = at.checked_add(n as u64 + 12).ok_or("PNG chunk overflow")?;
        if at > length || !kind.iter().all(u8::is_ascii_alphabetic) || !kind[2].is_ascii_uppercase()
        {
            return Err("PNG chunk bounds/type".into());
        }
        if (index == 0 && (kind != b"IHDR" || n != 13)) || (index > 0 && kind == b"IHDR") {
            return Err("PNG IHDR ordering".into());
        }
        if !matches!(kind, b"IHDR" | b"PLTE" | b"IDAT" | b"IEND") && kind[0].is_ascii_uppercase() {
            return Err("unknown critical PNG chunk".into());
        }
        let is_text = matches!(kind, b"tEXt" | b"zTXt" | b"iTXt");
        if is_text {
            text_bytes += n;
            if text_bytes > MAX_TEXT {
                return Err("PNG metadata bound".into());
            }
        }
        let collect = kind == b"IHDR" || is_text;
        let mut data = Vec::new();
        let mut checksum = crc(!0, kind);
        let mut remaining = n;
        let mut buffer = [0; 4096];
        while remaining > 0 {
            let count = remaining.min(buffer.len());
            read_exact(reader, &mut buffer[..count], request)?;
            checksum = crc(checksum, &buffer[..count]);
            if collect {
                data.extend_from_slice(&buffer[..count]);
            }
            remaining -= count;
        }
        let mut expected = [0; 4];
        read_exact(reader, &mut expected, request)?;
        if (!checksum).to_be_bytes() != expected {
            return Err("PNG CRC".into());
        }
        match kind {
            b"IHDR" => {
                let width = u32::from_be_bytes(data[..4].try_into().unwrap());
                let height = u32::from_be_bytes(data[4..8].try_into().unwrap());
                depth = data[8];
                color = data[9];
                let valid_depth = match color {
                    0 => matches!(depth, 1 | 2 | 4 | 8 | 16),
                    2 | 4 | 6 => matches!(depth, 8 | 16),
                    3 => matches!(depth, 1 | 2 | 4 | 8),
                    _ => false,
                };
                if width == 0
                    || height == 0
                    || width > class.max_edge()
                    || height > class.max_edge()
                    || !valid_depth
                    || data[10] != 0
                    || data[11] != 0
                    || data[12] > 1
                {
                    return Err("PNG dimensions/IHDR bound".into());
                }
            }
            b"PLTE" => {
                if palette
                    || idat
                    || n == 0
                    || n > 768
                    || n % 3 != 0
                    || matches!(color, 0 | 4)
                    || (color == 3 && n / 3 > (1 << depth))
                {
                    return Err("PNG palette".into());
                }
                palette = true;
            }
            b"IDAT" => {
                if idat_ended || (color == 3 && !palette) {
                    return Err("PNG IDAT ordering".into());
                }
                idat = true;
            }
            b"IEND" => {
                if n != 0 || !idat || at != length {
                    return Err("PNG ending".into());
                }
                return Ok(fields);
            }
            b"tEXt" => text(&mut fields, &data)?,
            // Standard Thumb fields are tEXt; don't hide conflicting fields in
            // another encoding. No compressed metadata/image data is inflated.
            b"zTXt" | b"iTXt" if data.starts_with(b"Thumb::") => {
                return Err("thumbnail metadata must use tEXt".into());
            }
            _ => {}
        }
        if idat && kind != b"IDAT" {
            idat_ended = true;
        }
    }
    Err("PNG chunk count bound".into())
}

#[cfg(test)]
#[path = "../tests/freedesktop/cache_reader.rs"]
mod tests;
