//! Same-directory, no-clobber Linux rename. No copy/delete fallback.
use std::{
    ffi::CString,
    fs::{self, File, OpenOptions},
    io,
    os::{
        fd::AsRawFd,
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt},
        },
    },
    path::{Path, PathBuf},
};
unsafe extern "C" {
    fn renameat2(
        oldfd: i32,
        old: *const std::ffi::c_char,
        newfd: i32,
        new: *const std::ffi::c_char,
        flags: u32,
    ) -> i32;
}
// Linux descriptor flags: inspect identity without opening a payload/device.
const O_DIRECTORY: i32 = 0x10000;
const O_NOFOLLOW: i32 = 0x20000;
const O_PATH: i32 = 0x200000;
pub fn validate_name(name: &str) -> Result<(), &'static str> {
    if name.trim().is_empty()
        || matches!(name, "." | "..")
        || name.len() > 255
        || name.chars().any(|c| c == '/' || c.is_control())
    {
        return Err("Enter a filename (1–255 bytes), without slashes or control characters.");
    }
    Ok(())
}
pub struct Ticket {
    directory: File,
    source: File,
    parent: PathBuf,
    name: std::ffi::OsString,
}
impl Ticket {
    pub fn capture(path: &Path) -> io::Result<Self> {
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("Missing parent folder"))?
            .to_owned();
        let directory = OpenOptions::new()
            .read(true)
            .custom_flags(O_PATH | O_DIRECTORY)
            .open(&parent)?;
        let name = path
            .file_name()
            .ok_or_else(|| io::Error::other("Missing filename"))?
            .to_owned();
        // O_PATH cannot block on a FIFO/device substituted for the media file.
        // O_NOFOLLOW keeps the identity check on the directory entry itself.
        let source = OpenOptions::new()
            .read(true)
            .custom_flags(O_PATH | O_NOFOLLOW)
            .open(PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd())).join(&name))?;
        if !source.metadata()?.is_file() {
            return Err(io::Error::other("Only regular media files can be renamed"));
        }
        Ok(Self {
            directory,
            source,
            parent,
            name,
        })
    }
    pub fn rename(&self, name: &str) -> io::Result<PathBuf> {
        validate_name(name).map_err(io::Error::other)?;
        let current_dir = fs::metadata(&self.parent)?;
        let held_dir = self.directory.metadata()?;
        if (current_dir.dev(), current_dir.ino()) != (held_dir.dev(), held_dir.ino()) {
            return Err(io::Error::other(
                "The source folder changed; reopen the file",
            ));
        }
        let source = self.source.metadata()?;
        let current = fs::symlink_metadata(
            PathBuf::from(format!("/proc/self/fd/{}", self.directory.as_raw_fd())).join(&self.name),
        )?;
        if (source.dev(), source.ino()) != (current.dev(), current.ino()) {
            return Err(io::Error::other("The source file changed; reopen the file"));
        }
        let target = self.parent.join(name);
        if self.name == name {
            return Ok(target);
        }
        let old = CString::new(self.name.as_bytes()).map_err(io::Error::other)?;
        let new = CString::new(name).map_err(io::Error::other)?;
        // RENAME_NOREPLACE is atomic even when another process creates target.
        let rc = unsafe {
            renameat2(
                self.directory.as_raw_fd(),
                old.as_ptr(),
                self.directory.as_raw_fd(),
                new.as_ptr(),
                1,
            )
        };
        if rc != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(target)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_collisions_and_identity_are_safe() {
        for bad in ["", " ", ".", "..", "../x", "/x", "a\0b", "a\nb"] {
            assert!(validate_name(bad).is_err());
        }
        assert!(validate_name("çekim 2.JPG").is_ok());
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.jpg");
        let b = dir.path().join("b.jpg");
        fs::write(&a, b"source").unwrap();
        fs::write(&b, b"target").unwrap();
        let ticket = Ticket::capture(&a).unwrap();
        assert_eq!(
            ticket.rename("b.jpg").unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read(&a).unwrap(), b"source");
        assert_eq!(fs::read(&b).unwrap(), b"target");
        let renamed = ticket.rename("çekim 2.JPG").unwrap();
        assert!(!a.exists());
        assert_eq!(fs::read(renamed).unwrap(), b"source");
    }
    #[test]
    fn replaced_sources_and_symlink_destinations_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.jpg");
        fs::write(&a, b"original").unwrap();
        let ticket = Ticket::capture(&a).unwrap();
        fs::rename(&a, dir.path().join("original.jpg")).unwrap();
        fs::write(&a, b"replacement").unwrap();
        assert!(ticket.rename("new.jpg").is_err());
        assert_eq!(fs::read(&a).unwrap(), b"replacement");
        std::os::unix::fs::symlink("missing", dir.path().join("link.jpg")).unwrap();
        assert_eq!(
            Ticket::capture(&a)
                .unwrap()
                .rename("link.jpg")
                .unwrap_err()
                .kind(),
            io::ErrorKind::AlreadyExists
        );
        assert!(Ticket::capture(&dir.path().join("link.jpg")).is_err());
    }
}
