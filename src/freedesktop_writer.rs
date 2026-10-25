//! FD-05 standalone worker writer. No decoding, provider routing or strip integration.
use crate::{
    freedesktop_generator::GeneratedThumbnail,
    freedesktop_thumbnail::{
        CacheReader, MAX_FILE, cache_home, cache_name, fd_path, open_component, open_directory,
    },
    image_cache::FileKey,
    thumbnail_provider::{Error, Request},
};
use gtk::{gdk::gdk_pixbuf, gio, glib, prelude::*};
use std::{
    ffi::CString,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::{
        fd::AsRawFd,
        raw::{c_char, c_int, c_uint},
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
        },
    },
    path::{Component, Path, PathBuf},
    time::Instant,
};
// Linux libc ABI, not copied application code. No fallback to clobbering rename.
unsafe extern "C" {
    fn mkdirat(dirfd: c_int, path: *const c_char, mode: c_uint) -> c_int;
    fn renameat2(
        oldfd: c_int,
        old: *const c_char,
        newfd: c_int,
        new: *const c_char,
        flags: c_uint,
    ) -> c_int;
    fn geteuid() -> c_uint;
}
const RENAME_NOREPLACE: c_uint = 1;

#[derive(Debug, PartialEq, Eq)]
pub enum Publication {
    Created(PathBuf),
    /// No overwrite, even if stale. Existing does NOT certify a reader Hit.
    Existing(PathBuf),
}
pub struct CacheWriter {
    home: PathBuf,
}
impl CacheWriter {
    pub fn new(home: PathBuf) -> Result<Self, Error> {
        CacheReader::new(home.clone())?;
        if home.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(Error::Io("parent traversal in cache home".into()));
        }
        Ok(Self { home })
    }
    pub fn from_environment() -> Result<Self, Error> {
        Self::new(cache_home(
            std::env::var_os("XDG_CACHE_HOME").as_deref(),
            &glib::home_dir(),
        )?)
    }
    /// Blocking admitted-worker call. It creates no thread, pool or queue.
    pub fn write(
        &self,
        thumbnail: &GeneratedThumbnail,
        request: &Request<'_>,
    ) -> Result<Publication, Error> {
        self.write_with(thumbnail, request, |_| {})
    }
    fn write_with(
        &self,
        t: &GeneratedThumbnail,
        r: &Request<'_>,
        mut checkpoint: impl FnMut(Stage),
    ) -> Result<Publication, Error> {
        source_current(r)?;
        if t.source != *r.source {
            return Err(Error::SourceChanged);
        }
        let row = (t.width as usize)
            .checked_mul(4)
            .ok_or_else(|| Error::LimitExceeded("RGBA row overflow".into()))?;
        let len = row
            .checked_mul(t.height as usize)
            .ok_or_else(|| Error::LimitExceeded("RGBA length overflow".into()))?;
        if t.width == 0
            || t.height == 0
            || t.width > t.class.max_edge()
            || t.height > t.class.max_edge()
            || t.stride != row
            || t.pixels.len() != len
        {
            return Err(Error::InvalidSource(
                "thumbnail dimensions/stride/bytes do not match class".into(),
            ));
        }
        let m =
            fs::metadata(&r.source.path).map_err(|e| Error::SourceUnavailable(e.to_string()))?;
        if m.mtime() < 0 {
            return Err(Error::InvalidSource(
                "negative Thumb::MTime unsupported by reader".into(),
            ));
        }
        let uri = gio::File::for_path(&r.source.path).uri().to_string();
        if uri.len() > 16384 {
            return Err(Error::LimitExceeded("URI metadata bound".into()));
        }
        let mtime = m.mtime().to_string();
        let size = m.len().to_string();
        source_current(r)?;
        let path = self.home.join("thumbnails").join(t.class.directory());
        let directory = create_directory(&path, &self.home, r)?;
        let name = cache_name(&r.source.path);
        let target = path.join(&name);
        if existing(&directory, &name)? {
            source_current(r)?;
            verify_directory(&path, &directory)?;
            return Ok(Publication::Existing(target));
        }
        let mut temp = Temporary::new(directory)?;
        checkpoint(Stage::Temporary);
        source_current(r)?;
        let sink = Sink {
            file: temp.file.try_clone().map_err(io_error)?,
            source: r.source.clone(),
            cancel: r.cancel.clone(),
            deadline: r.deadline,
            written: 0,
            failure: None,
        };
        let stream = gio::WriteOutputStream::new(sink);
        let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(
            &t.pixels,
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            t.width as i32,
            t.height as i32,
            t.stride as i32,
        );
        // Existing PNG encoder only; tEXt values are ASCII URI/numbers/software.
        let encoded = pixbuf.save_to_streamv(
            &stream,
            "png",
            &[
                ("tEXt::Thumb::URI", &uri),
                ("tEXt::Thumb::MTime", &mtime),
                ("tEXt::Thumb::Size", &size),
                ("tEXt::Software", "Pivra"),
                ("compression", "6"),
            ],
            Some(r.cancel),
        );
        // save_to_streamv does not close the caller's stream. Recover the bounded sink.
        let sink = stream
            .close_and_take()
            .downcast::<Sink>()
            .map_err(|_| Error::Io("PNG sink type mismatch".into()))?;
        source_current(r)?;
        if let Some(error) = sink.failure {
            return Err(error);
        }
        encoded.map_err(|e| Error::Io(format!("PNG encoding: {e}")))?;
        let sealed = temp.file.metadata().map_err(io_error)?;
        if sealed.mode() & 0o777 != 0o600 || sealed.nlink() != 1 || sealed.len() > MAX_FILE {
            return Err(Error::Io(
                "temporary file permissions/links/size changed".into(),
            ));
        }
        checkpoint(Stage::Encoded);
        source_current(r)?;
        temp.file.sync_all().map_err(io_error)?;
        checkpoint(Stage::BeforePublish);
        source_current(r)?;
        verify_directory(&path, &temp.directory)?;
        if !crate::freedesktop_thumbnail::same(&sealed, &temp.file.metadata().map_err(io_error)?)
            || !temp.owns_name()
        {
            return Err(Error::Io("temporary file identity changed".into()));
        }
        let old = cstring(&temp.name)?;
        let new = cstring(&name)?;
        // SAFETY: live pinned dirfd, NUL-terminated leaf names and Linux flag1.
        let status = unsafe {
            renameat2(
                temp.directory.as_raw_fd(),
                old.as_ptr(),
                temp.directory.as_raw_fd(),
                new.as_ptr(),
                RENAME_NOREPLACE,
            )
        };
        if status != 0 {
            let e = io::Error::last_os_error();
            if e.kind() == io::ErrorKind::AlreadyExists {
                if existing(&temp.directory, &name)? {
                    source_current(r)?;
                    verify_directory(&path, &temp.directory)?;
                    return Ok(Publication::Existing(target));
                }
                return Err(Error::Io("concurrent target disappeared".into()));
            }
            return Err(io_error(e));
        }
        temp.published = true; // Point of commit: never delete a committed record on late cancel.
        checkpoint(Stage::Published);
        temp.directory
            .sync_all()
            .map_err(|e| Error::Io(format!("published, directory fsync failed: {e}")))?;
        verify_directory(&path, &temp.directory)?;
        if FileKey::read(&r.source.path).ok().as_ref() != Some(r.source) {
            return Err(Error::SourceChanged);
        }
        Ok(Publication::Created(target))
    }
}
#[derive(Clone, Copy)]
enum Stage {
    Temporary,
    Encoded,
    BeforePublish,
    Published,
}
fn io_error(e: io::Error) -> Error {
    Error::Io(e.to_string())
}
fn cstring(name: &str) -> Result<CString, Error> {
    CString::new(name).map_err(|_| Error::Io("NUL filename".into()))
}
fn source_current(r: &Request<'_>) -> Result<(), Error> {
    r.check_control(Instant::now())?;
    let key = FileKey::read(&r.source.path).map_err(|e| Error::SourceUnavailable(e.to_string()))?;
    if key != *r.source {
        return Err(Error::SourceChanged);
    }
    Ok(())
}
fn create_directory(path: &Path, home: &Path, r: &Request<'_>) -> Result<File, Error> {
    let mut directory = File::open("/").map_err(io_error)?;
    let mut walked = PathBuf::from("/");
    // SAFETY: geteuid has no pointer arguments or side effects.
    let uid = unsafe { geteuid() };
    for component in path.components() {
        if matches!(component, Component::RootDir | Component::CurDir) {
            continue;
        }
        source_current(r)?;
        let leaf = CString::new(component.as_os_str().as_bytes())
            .map_err(|_| Error::Io("NUL directory".into()))?;
        walked.push(component.as_os_str());
        let entry = fd_path(&directory).join(component.as_os_str());
        let next = match open_component(&entry) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // SAFETY: pinned parent fd and an individual, NUL-terminated path component.
                let status = unsafe { mkdirat(directory.as_raw_fd(), leaf.as_ptr(), 0o700) };
                if status != 0 && io::Error::last_os_error().kind() != io::ErrorKind::AlreadyExists
                {
                    return Err(io_error(io::Error::last_os_error()));
                }
                directory.sync_all().map_err(io_error)?;
                open_component(&entry).map_err(io_error)?
            }
            Err(e) => return Err(io_error(e)),
        };
        let m = next.metadata().map_err(io_error)?;
        if !m.is_dir() || m.mode() & 0o111 == 0 {
            return Err(Error::Io("cache directory inaccessible".into()));
        }
        if m.mode() & 0o022 != 0 && m.mode() & 0o1000 == 0 {
            return Err(Error::Io("unsafe writable ancestor".into()));
        }
        if walked.starts_with(home)
            && (m.uid() != uid || m.mode() & 0o300 != 0o300 || m.mode() & 0o022 != 0)
        {
            return Err(Error::Io("cache directory owner/permissions".into()));
        }
        directory = next;
    }
    Ok(directory)
}
fn verify_directory(path: &Path, original: &File) -> Result<(), Error> {
    let a = original.metadata().map_err(io_error)?;
    let b = open_directory(path)
        .map_err(io_error)?
        .metadata()
        .map_err(io_error)?;
    // SAFETY: geteuid has no arguments and does not change process state.
    let uid = unsafe { geteuid() };
    if b.uid() != uid || b.mode() & 0o300 != 0o300 || b.mode() & 0o022 != 0 {
        return Err(Error::Io(
            "cache permissions changed before publication".into(),
        ));
    }
    if a.dev() != b.dev() || a.ino() != b.ino() {
        return Err(Error::Io("cache namespace changed".into()));
    }
    Ok(())
}
fn existing(dir: &File, name: &str) -> Result<bool, Error> {
    match fs::symlink_metadata(fd_path(dir).join(name)) {
        Ok(m) if m.is_file() => Ok(true),
        Ok(_) => Err(Error::Io("unsafe existing target (not regular)".into())),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(io_error(e)),
    }
}
struct Temporary {
    directory: File,
    file: File,
    name: String,
    published: bool,
}
impl Temporary {
    fn new(directory: File) -> Result<Self, Error> {
        let mut entropy = [0u8; 16];
        File::open("/dev/urandom")
            .map_err(io_error)?
            .read_exact(&mut entropy)
            .map_err(io_error)?;
        let random: String = entropy.iter().map(|byte| format!("{byte:02x}")).collect();
        let name = format!(".pivra-{random}.tmp");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(fd_path(&directory).join(&name))
            .map_err(io_error)?;
        let temp = Self {
            directory,
            file,
            name,
            published: false,
        };
        temp.file
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(io_error)?;
        Ok(temp)
    }
    fn owns_name(&self) -> bool {
        self.file
            .metadata()
            .ok()
            .zip(fs::symlink_metadata(fd_path(&self.directory).join(&self.name)).ok())
            .is_some_and(|(a, b)| b.is_file() && a.dev() == b.dev() && a.ino() == b.ino())
    }
}
impl Drop for Temporary {
    fn drop(&mut self) {
        if !self.published && self.owns_name() {
            let _ = fs::remove_file(fd_path(&self.directory).join(&self.name));
        }
    }
}
struct Sink {
    file: File,
    source: FileKey,
    cancel: gio::Cancellable,
    deadline: Instant,
    written: u64,
    failure: Option<Error>,
}
impl Write for Sink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let request = Request {
            source: &self.source,
            cancel: &self.cancel,
            deadline: self.deadline,
        };
        let result = (|| {
            source_current(&request)?;
            if bytes.len() as u64 > MAX_FILE - self.written {
                return Err(Error::LimitExceeded("PNG exceeds reader 2MiB bound".into()));
            }
            let n = self.file.write(bytes).map_err(io_error)?;
            self.written += n as u64;
            Ok(n)
        })();
        match result {
            Ok(n) => Ok(n),
            Err(error) => {
                self.failure = Some(error);
                Err(io::Error::other("bounded PNG stream stopped"))
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
#[cfg(test)]
#[path = "../tests/freedesktop/writer.rs"]
mod tests;
