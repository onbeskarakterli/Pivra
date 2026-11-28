//! Same-directory, no-clobber Linux rename. No copy/delete fallback.
use std::{
    ffi::{CString, OsStr},
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
    sync::Arc,
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
        self.rename_entry(&self.name, OsStr::new(name))?;
        Ok(self.parent.join(name))
    }

    pub fn rename_with_undo(self: &Arc<Self>, name: &str) -> io::Result<Option<Reversal>> {
        let renamed = self.rename(name)?;
        let original = self.parent.join(&self.name);
        Ok((renamed != original).then(|| Reversal {
            ticket: self.clone(),
            original,
            renamed,
        }))
    }

    fn rename_entry(&self, from: &OsStr, to: &OsStr) -> io::Result<()> {
        let current_dir = fs::metadata(&self.parent)?;
        let held_dir = self.directory.metadata()?;
        if (current_dir.dev(), current_dir.ino()) != (held_dir.dev(), held_dir.ino()) {
            return Err(io::Error::other(
                "The source folder changed; reopen the file",
            ));
        }
        let source = self.source.metadata()?;
        let current = fs::symlink_metadata(
            PathBuf::from(format!("/proc/self/fd/{}", self.directory.as_raw_fd())).join(from),
        )?;
        if (source.dev(), source.ino()) != (current.dev(), current.ino()) {
            return Err(io::Error::other("The source file changed; reopen the file"));
        }
        if from == to {
            return Ok(());
        }
        move_no_replace(&self.directory, from, &self.directory, to)
    }
}

/// Atomically move a directory entry on the same filesystem without overwriting.
pub fn move_no_replace(
    from_directory: &File,
    from: &OsStr,
    to_directory: &File,
    to: &OsStr,
) -> io::Result<()> {
    if [from, to]
        .into_iter()
        .any(|name| Path::new(name).file_name() != Some(name))
    {
        return Err(io::Error::other("Expected a single filename"));
    }
    let old = CString::new(from.as_bytes()).map_err(io::Error::other)?;
    let new = CString::new(to.as_bytes()).map_err(io::Error::other)?;
    // RENAME_NOREPLACE is atomic even when another process creates the target.
    let rc = unsafe {
        renameat2(
            from_directory.as_raw_fd(),
            old.as_ptr(),
            to_directory.as_raw_fd(),
            new.as_ptr(),
            1,
        )
    };
    if rc == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

pub struct Reversal {
    ticket: Arc<Ticket>,
    original: PathBuf,
    renamed: PathBuf,
}

impl Reversal {
    pub fn paths(&self) -> (&Path, &Path) {
        (&self.original, &self.renamed)
    }

    pub fn undo(&self) -> io::Result<()> {
        self.ticket.rename_entry(
            self.renamed.file_name().unwrap(),
            self.original.file_name().unwrap(),
        )
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
    #[test]
    fn reversal_refuses_replaced_sources_and_dangling_destinations() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("a.jpg");
        let renamed = dir.path().join("b.jpg");
        let moved = dir.path().join("moved.jpg");
        fs::write(&original, b"untouched").unwrap();
        let ticket = Arc::new(Ticket::capture(&original).unwrap());
        assert!(ticket.rename_with_undo("a.jpg").unwrap().is_none());
        let reversal = ticket.rename_with_undo("b.jpg").unwrap().unwrap();

        std::os::unix::fs::symlink("missing", &original).unwrap();
        assert_eq!(
            reversal.undo().unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert!(original.is_symlink());
        fs::remove_file(&original).unwrap();

        fs::rename(&renamed, &moved).unwrap();
        fs::write(&renamed, b"replacement").unwrap();
        assert!(reversal.undo().is_err());
        assert!(!original.exists());
        assert_eq!(fs::read(&renamed).unwrap(), b"replacement");
        fs::remove_file(&renamed).unwrap();
        fs::rename(&moved, &renamed).unwrap();

        reversal.undo().unwrap();
        assert_eq!(fs::read(&original).unwrap(), b"untouched");
        assert!(!renamed.exists());
        assert!(reversal.undo().is_err());
    }
    #[test]
    fn atomic_publish_between_directories_keeps_both_files_on_collision() {
        let dir = tempfile::tempdir().unwrap();
        let stage = dir.path().join("stage");
        fs::create_dir(&stage).unwrap();
        fs::write(stage.join("media"), b"restore").unwrap();
        let target = dir.path().join("a.jpg");
        fs::write(&target, b"collision").unwrap();
        let staged = File::open(&stage).unwrap();
        let original = File::open(dir.path()).unwrap();
        let publish =
            || move_no_replace(&staged, OsStr::new("media"), &original, OsStr::new("a.jpg"));

        assert_eq!(publish().unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&target).unwrap(), b"collision");
        assert_eq!(fs::read(stage.join("media")).unwrap(), b"restore");
        fs::remove_file(&target).unwrap();
        publish().unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"restore");
        assert!(!stage.join("media").exists());
    }
}
