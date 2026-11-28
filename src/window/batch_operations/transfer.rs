//! Local, identity-checked transfer and inverse operations.
use gtk::{gio, glib, prelude::*};
use pivra::rename::move_no_replace;
use std::{
    ffi::{OsStr, OsString},
    fs::{self, File, Metadata, OpenOptions},
    io::{self, BufRead, BufReader},
    os::{
        fd::AsRawFd,
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Copy)]
pub(super) enum Collision {
    Skip,
    KeepBoth,
}

pub(super) struct Directory {
    path: PathBuf,
    file: File,
}

impl Directory {
    pub(super) fn open(path: PathBuf) -> io::Result<Arc<Self>> {
        let file = File::open(&path)?;
        if !file.metadata()?.is_dir() {
            return Err(io::Error::other("The selected destination is not a folder"));
        }
        Ok(Arc::new(Self { path, file }))
    }

    fn current(&self) -> io::Result<()> {
        if inode(&self.file.metadata()?) != inode(&fs::metadata(&self.path)?) {
            return Err(io::Error::other("The folder changed"));
        }
        Ok(())
    }

    fn descriptor_path(&self) -> PathBuf {
        PathBuf::from(format!("/proc/self/fd/{}", self.file.as_raw_fd()))
    }
}

pub(super) struct Source {
    path: PathBuf,
    dir: Arc<Directory>,
    file: File,
}

impl Source {
    pub(super) fn capture(path: PathBuf) -> io::Result<Self> {
        let dir = Directory::open(
            path.parent()
                .ok_or_else(|| io::Error::other("Missing source folder"))?
                .to_owned(),
        )?;
        let name = path
            .file_name()
            .ok_or_else(|| io::Error::other("Missing source filename"))?;
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(0x200000 | 0x20000) // O_PATH | O_NOFOLLOW
            .open(dir.descriptor_path().join(name))?;
        if !file.metadata()?.is_file() {
            return Err(io::Error::other(
                "Only regular files can be copied or moved",
            ));
        }
        Ok(Self { path, dir, file })
    }

    fn check_current(&self) -> io::Result<Metadata> {
        self.dir.current()?;
        let held = self.file.metadata()?;
        if inode(&held)
            != inode(&fs::symlink_metadata(
                self.dir.descriptor_path().join(self.name()),
            )?)
        {
            return Err(io::Error::other("The source file changed"));
        }
        Ok(held)
    }

    fn name(&self) -> &OsStr {
        self.path.file_name().expect("captured filename")
    }

    pub(super) fn transfer(
        &self,
        to: &Arc<Directory>,
        moving: bool,
        collision: Collision,
        cancel: &gio::Cancellable,
    ) -> io::Result<ResultFile> {
        self.transfer_as(to, self.name(), moving, collision, cancel)
    }

    fn transfer_as(
        &self,
        to: &Arc<Directory>,
        name: &OsStr,
        moving: bool,
        collision: Collision,
        cancel: &gio::Cancellable,
    ) -> io::Result<ResultFile> {
        cancelled(cancel)?;
        let original = self.check_current()?;
        to.current()?;
        if moving && inode(&self.dir.file.metadata()?) == inode(&to.file.metadata()?) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "The file is already in that folder",
            ));
        }
        if moving && original.dev() == to.file.metadata()?.dev() {
            let held = self.file.try_clone()?;
            let result = publish(to, name, collision, |candidate| {
                cancelled(cancel)?;
                self.check_current()?;
                move_no_replace(&self.dir.file, self.name(), &to.file, candidate)?;
                if inode(&fs::symlink_metadata(to.descriptor_path().join(candidate))?)
                    != inode(&original)
                {
                    return Err(rollback(
                        &to.file,
                        candidate,
                        &self.dir,
                        self.name(),
                        "The source changed during the move",
                    ));
                }
                Ok(())
            });
            match result {
                Ok(name) => {
                    return Ok(ResultFile {
                        name: to.path.join(name),
                        file: held,
                    });
                }
                Err(error) if error.raw_os_error() == Some(18) => {} // EXDEV, including mount boundaries
                Err(error) => return Err(error),
            }
        }

        if matches!(collision, Collision::Skip) {
            match fs::symlink_metadata(to.descriptor_path().join(name)) {
                Ok(_) => return Err(io::Error::from(io::ErrorKind::AlreadyExists)),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        let stage = Staging::new(to)?;
        let from = PathBuf::from(format!("/proc/self/fd/{}", self.file.as_raw_fd()));
        gio::File::for_path(&from)
            .copy(
                &gio::File::for_path(stage.path()),
                gio::FileCopyFlags::ALL_METADATA,
                Some(cancel),
                None,
            )
            .map_err(io::Error::other)?;
        let copied = File::open(stage.path())?;
        let verified = copied.metadata()?;
        if content_stamp(&self.check_current()?) != content_stamp(&original)
            || verified.len() != original.len()
        {
            return Err(io::Error::other("The source changed while copying"));
        }
        if moving {
            verify_copy(&from, &stage.path(), cancel)?;
        }
        let candidate = publish(to, name, collision, |candidate| {
            cancelled(cancel)?;
            if content_stamp(&self.check_current()?) != content_stamp(&original) {
                return Err(io::Error::other("The source changed before publishing"));
            }
            move_no_replace(&stage.dir, OsStr::new("payload"), &to.file, candidate)
        })?;
        let output = to.path.join(candidate);
        if moving {
            // Stage the source on its own filesystem before deleting it, so a
            // replacement can never be unlinked by looking up the old pathname.
            if let Err(error) = self.retire(&original, &output, &verified) {
                return Err(io::Error::other(format!(
                    "Copied to {} · {error}",
                    output.display()
                )));
            }
        }
        Ok(ResultFile {
            name: output,
            file: copied,
        })
    }

    fn retire(&self, original: &Metadata, output: &Path, verified: &Metadata) -> io::Result<()> {
        if content_stamp(&self.check_current()?) != content_stamp(original) {
            return Err(io::Error::other("The original changed and was kept"));
        }
        let mut retired = Staging::new(&self.dir)?;
        retired.keep_on_drop = true;
        move_no_replace(
            &self.dir.file,
            self.name(),
            &retired.dir,
            OsStr::new("payload"),
        )?;
        // A rename may change ctime. Identity, size and data mtime must match.
        let retire = || -> io::Result<()> {
            let source = fs::symlink_metadata(retired.path())?;
            if data_stamp(&source) != data_stamp(original)
                || data_stamp(&fs::symlink_metadata(output)?) != data_stamp(verified)
            {
                return Err(io::Error::other("The source or copied file changed"));
            }
            fs::remove_file(retired.path())
        };
        retire().map_err(|error| {
            rollback(
                &retired.dir,
                OsStr::new("payload"),
                &self.dir,
                self.name(),
                &format!(
                    "{error}; preserved source at {}",
                    retired.original_path().display()
                ),
            )
        })
    }
}

pub(super) struct ResultFile {
    pub name: PathBuf,
    file: File,
}

pub(super) struct Moved {
    original: PathBuf,
    original_dir: Arc<Directory>,
    destination: Source,
}

impl Moved {
    pub(super) fn new(source: Source, to: Arc<Directory>, result: ResultFile) -> Self {
        Self {
            original: source.path,
            original_dir: source.dir,
            destination: Source {
                path: result.name,
                dir: to,
                file: result.file,
            },
        }
    }

    pub(super) fn original_path(&self) -> &Path {
        &self.original
    }

    pub(super) fn undo(&self) -> io::Result<(PathBuf, PathBuf)> {
        let name = self.original.file_name().expect("original filename");
        self.destination.transfer_as(
            &self.original_dir,
            name,
            true,
            Collision::Skip,
            &gio::Cancellable::new(),
        )?;
        Ok((self.destination.path.clone(), self.original.clone()))
    }
}

fn publish(
    to: &Directory,
    name: &OsStr,
    collision: Collision,
    mut commit: impl FnMut(&OsStr) -> io::Result<()>,
) -> io::Result<OsString> {
    let mut candidate = name.to_owned();
    for suffix in 2..=1025 {
        to.current()?;
        match commit(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error)
                if error.kind() == io::ErrorKind::AlreadyExists
                    && matches!(collision, Collision::KeepBoth) =>
            {
                candidate = unique_name(name, suffix);
            }
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "No available filename",
    ))
}

fn unique_name(filename: &OsStr, suffix: usize) -> OsString {
    let name = Path::new(filename);
    let mut stem = name.file_stem().unwrap_or(filename).to_owned();
    stem.push(format!(" ({suffix})"));
    if let Some(extension) = name.extension() {
        stem.push(".");
        stem.push(extension);
    }
    stem
}

fn rollback(
    from: &File,
    name: &OsStr,
    to: &Directory,
    original: &OsStr,
    message: &str,
) -> io::Error {
    match move_no_replace(from, name, &to.file, original) {
        Ok(()) => io::Error::other(format!("{message} · original retained")),
        Err(error) => io::Error::other(format!("{message} · rollback failed: {error}")),
    }
}

fn inode(meta: &Metadata) -> (u64, u64) {
    (meta.dev(), meta.ino())
}
fn data_stamp(meta: &Metadata) -> ((u64, u64), u64, i64, i64) {
    (inode(meta), meta.len(), meta.mtime(), meta.mtime_nsec())
}
fn content_stamp(meta: &Metadata) -> (((u64, u64), u64, i64, i64), i64, i64) {
    (data_stamp(meta), meta.ctime(), meta.ctime_nsec())
}

fn cancelled(token: &gio::Cancellable) -> io::Result<()> {
    token.set_error_if_cancelled().map_err(io::Error::other)
}

fn verify_copy(from: &Path, to: &Path, cancel: &gio::Cancellable) -> io::Result<()> {
    let (mut left, mut right) = (
        BufReader::new(File::open(from)?),
        BufReader::new(File::open(to)?),
    );
    loop {
        cancelled(cancel)?;
        let (a, b) = (left.fill_buf()?, right.fill_buf()?);
        let count = a.len().min(b.len());
        if a.is_empty() != b.is_empty() || a[..count] != b[..count] {
            return Err(io::Error::other("Copied bytes differ from the original"));
        }
        if count == 0 {
            return Ok(());
        }
        left.consume(count);
        right.consume(count);
    }
}

struct Staging {
    root: PathBuf,
    parent: PathBuf,
    dir: File,
    keep_on_drop: bool,
}

impl Staging {
    fn new(parent: &Directory) -> io::Result<Self> {
        let root = glib::mkdtemp(parent.descriptor_path().join(".pivra-transfer-XXXXXX"))
            .ok_or_else(io::Error::last_os_error)?;
        let dir = File::open(&root).inspect_err(|_| {
            let _ = fs::remove_dir(&root);
        })?;
        Ok(Self {
            root,
            parent: parent.path.clone(),
            dir,
            keep_on_drop: false,
        })
    }

    fn path(&self) -> PathBuf {
        self.root.join("payload")
    }
    fn original_path(&self) -> PathBuf {
        self.parent
            .join(self.root.file_name().expect("staging directory name"))
            .join("payload")
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        if !self.keep_on_drop {
            let _ = fs::remove_file(self.path());
        }
        let _ = fs::remove_dir(&self.root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_copies_unicode_and_never_clobbers() {
        let from = tempfile::tempdir().unwrap();
        let to = tempfile::tempdir().unwrap();
        let source = from.path().join("çekim.png");
        let destination = to.path().join("çekim.png");
        fs::write(&source, b"media data").unwrap();
        fs::write(&destination, b"unrelated").unwrap();
        let file = Source::capture(source.clone()).unwrap();
        let folder = Directory::open(to.path().to_owned()).unwrap();
        let token = gio::Cancellable::new();
        assert_eq!(
            file.transfer(&folder, false, Collision::Skip, &token)
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::AlreadyExists
        );
        let copied = file
            .transfer(&folder, false, Collision::KeepBoth, &token)
            .unwrap();
        assert_eq!(copied.name.file_name().unwrap(), "çekim (2).png");
        assert_eq!(fs::read(copied.name).unwrap(), b"media data");
        assert_eq!(fs::read(destination).unwrap(), b"unrelated");
        assert_eq!(fs::read(&source).unwrap(), b"media data");
        assert!(
            file.transfer(&file.dir, true, Collision::KeepBoth, &token)
                .is_err()
        );
        token.cancel();
        assert!(
            file.transfer(&folder, false, Collision::KeepBoth, &token)
                .is_err()
        );
        assert_eq!(fs::read_dir(to.path()).unwrap().count(), 2);
    }

    #[test]
    fn transfer_rejects_replaced_source_and_move_undo_collision() {
        let from = tempfile::tempdir().unwrap();
        let to = tempfile::tempdir().unwrap();
        let source = from.path().join("source.png");
        fs::write(&source, b"first").unwrap();
        let old = Source::capture(source.clone()).unwrap();
        fs::rename(&source, from.path().join("original")).unwrap();
        fs::write(&source, b"replacement").unwrap();
        let folder = Directory::open(to.path().to_owned()).unwrap();
        let token = gio::Cancellable::new();
        assert!(
            old.transfer(&folder, true, Collision::Skip, &token)
                .is_err()
        );
        assert!(fs::read_dir(to.path()).unwrap().next().is_none());
        let actual = Source::capture(source.clone()).unwrap();
        let result = actual
            .transfer(&folder, true, Collision::Skip, &token)
            .unwrap();
        let moved = result.name.clone();
        let record = Moved::new(actual, folder, result);
        assert!(!source.exists());
        fs::write(&source, b"collision").unwrap();
        assert!(record.undo().is_err());
        assert_eq!(fs::read(&source).unwrap(), b"collision");
        assert_eq!(fs::read(&moved).unwrap(), b"replacement");
        fs::remove_file(&source).unwrap();
        record.undo().unwrap();
        assert_eq!(fs::read(&source).unwrap(), b"replacement");
        assert!(!moved.exists());
        assert!(record.undo().is_err());
    }

    #[test]
    #[ignore = "requires PIVRA_CROSS_MOUNT_DIR on a writable filesystem distinct from TMPDIR"]
    fn transfer_cross_mount_validates_before_source_retirement_and_undo() {
        let from = tempfile::tempdir().unwrap();
        let to = tempfile::tempdir_in(std::env::var_os("PIVRA_CROSS_MOUNT_DIR").unwrap()).unwrap();
        assert_ne!(
            fs::metadata(from.path()).unwrap().dev(),
            fs::metadata(to.path()).unwrap().dev()
        );
        let source = from.path().join("cross.png");
        let bytes = vec![0x92; 100_123];
        fs::write(&source, &bytes).unwrap();
        let token = gio::Cancellable::new();
        let folder = Directory::open(to.path().to_owned()).unwrap();
        let actual = Source::capture(source.clone()).unwrap();
        let result = actual
            .transfer(&folder, true, Collision::Skip, &token)
            .unwrap();
        let moved = result.name.clone();
        assert!(!source.exists());
        assert_eq!(fs::read(&moved).unwrap(), bytes);
        Moved::new(actual, folder, result).undo().unwrap();
        assert_eq!(fs::read(source).unwrap(), bytes);
        assert!(fs::read_dir(to.path()).unwrap().next().is_none());
    }
}
