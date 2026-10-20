//! One-directory candidate discovery. No GTK/GIO, sorting, decoding or watching.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    catalog::MediaCollection,
    media::{MediaItem, MediaKind},
};

#[derive(Debug)]
pub struct ScanIssue {
    pub path: PathBuf,
    pub error: io::Error,
}

/// An unsorted snapshot. Nonempty issues mean the scan was incomplete.
#[derive(Debug)]
pub struct ScanReport {
    pub collection: MediaCollection,
    pub issues: Vec<ScanIssue>,
}

/// Extension admission only; never a format/content or decoder capability claim.
pub fn candidate_kind(path: &Path) -> Option<MediaKind> {
    let extension = path.extension()?.to_str()?;
    if ["jpg", "jpeg", "png", "webp", "gif", "bmp", "tif", "tiff"]
        .iter()
        .any(|s| extension.eq_ignore_ascii_case(s))
    {
        Some(MediaKind::Image)
    } else if ["mp4", "mkv", "webm", "mov", "avi"]
        .iter()
        .any(|s| extension.eq_ignore_ascii_case(s))
    {
        Some(MediaKind::Video)
    } else {
        None
    }
}

/// Keep the lexical parent/alias, not the canonical symlink target's folder.
/// The opened entry must still be a readable candidate when the scan sees it.
pub fn scan_neighbors(opened: &Path, cancelled: &AtomicBool) -> io::Result<ScanReport> {
    let opened = std::path::absolute(opened)?;
    let parent = opened
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "input has no parent folder"))?;
    let mut report = scan_directory(parent, cancelled)?;
    let selected = report
        .collection
        .items()
        .iter()
        .position(|item| item.path() == opened)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "opened file is missing, inaccessible or no longer a media candidate",
            )
        })?;
    report.collection.select(selected);
    Ok(report)
}

/// Blocking filesystem work: callers must keep this off the GTK main thread.
pub fn scan_directory(directory: &Path, cancelled: &AtomicBool) -> io::Result<ScanReport> {
    check_cancelled(cancelled)?;
    let directory = std::path::absolute(directory)?;
    let entries = fs::read_dir(&directory)?;
    collect_entries(
        &directory,
        entries.map(|entry| entry.map(|entry| entry.path())),
        cancelled,
    )
}

fn check_cancelled(cancelled: &AtomicBool) -> io::Result<()> {
    if cancelled.load(Ordering::Relaxed) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "folder scan cancelled",
        ))
    } else {
        Ok(())
    }
}

fn collect_entries(
    directory: &Path,
    entries: impl Iterator<Item = io::Result<PathBuf>>,
    cancelled: &AtomicBool,
) -> io::Result<ScanReport> {
    let mut items = Vec::new();
    let mut issues = Vec::new();
    for entry in entries {
        check_cancelled(cancelled)?;
        let path = match entry {
            Ok(path) => path,
            Err(error) => {
                issues.push(ScanIssue {
                    path: directory.to_owned(),
                    error,
                });
                continue;
            }
        };
        let Some(kind) = candidate_kind(&path) else {
            continue;
        };
        let inspect = || -> io::Result<bool> {
            // Follow file symlinks, but never recurse into a directory/link.
            if !fs::metadata(&path)?.is_file() {
                return Ok(false);
            }
            let file = fs::File::open(&path)?;
            // No bytes are read; check the descriptor as well as the path.
            Ok(file.metadata()?.is_file())
        };
        match inspect() {
            Ok(true) => items.push(MediaItem::new(path, kind)),
            Ok(false) => {}
            Err(error) => issues.push(ScanIssue { path, error }),
        }
    }
    check_cancelled(cancelled)?;
    Ok(ScanReport {
        collection: MediaCollection::new(items),
        issues,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_during_iteration_discards_partial_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("present.png");
        fs::write(&path, b"not decoded").unwrap();
        let cancelled = AtomicBool::new(false);
        let entries = [Ok(path.clone()), Ok(path)]
            .into_iter()
            .enumerate()
            .map(|(index, entry)| {
                if index == 1 {
                    cancelled.store(true, Ordering::Relaxed);
                }
                entry
            });
        assert_eq!(
            collect_entries(dir.path(), entries, &cancelled)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Interrupted
        );
    }

    #[test]
    fn disappearing_entry_and_iteration_error_are_reported_without_losing_other_candidates() {
        let dir = tempfile::tempdir().unwrap();
        let present = dir.path().join("present.png");
        let removed = dir.path().join("removed.jpg");
        fs::write(&present, b"not decoded").unwrap();
        fs::write(&removed, b"not decoded").unwrap();
        let entries = [
            Ok(present.clone()),
            Ok(removed.clone()),
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "directory iteration failed",
            )),
        ]
        .into_iter()
        .inspect(|entry| {
            if entry.as_ref().is_ok_and(|path| path == &removed) {
                fs::remove_file(&removed).unwrap();
            }
        });
        let report = collect_entries(dir.path(), entries, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            report.collection.items(),
            &[MediaItem::new(present, MediaKind::Image)]
        );
        assert_eq!(report.issues.len(), 2);
        assert_eq!(report.issues[0].path, removed);
        assert_eq!(report.issues[0].error.kind(), io::ErrorKind::NotFound);
        assert_eq!(
            report.issues[1].error.kind(),
            io::ErrorKind::PermissionDenied
        );
    }
}
