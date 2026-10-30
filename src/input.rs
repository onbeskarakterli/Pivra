//! Local input boundary. No decoding, directory discovery or GTK initialization.

use std::{ffi::OsStr, fs, path::Path};

use gtk::{gio, glib, prelude::*};
use pivra::media::MediaItem;
#[cfg(test)]
use pivra::media::MediaKind;

/// Window-owned association, not a decoded image or an open descriptor.
#[derive(Clone, Debug)]
pub(super) struct OpenedFile {
    pub file: gio::File,
    pub item: MediaItem,
}

impl OpenedFile {
    pub fn from_argument(argument: &OsStr) -> Result<Self, String> {
        let file = match argument.to_str() {
            Some(text) if glib::Uri::parse_scheme(text).is_some() => {
                if text.bytes().any(|c| c.is_ascii_whitespace() || c == 0) {
                    return Err("invalid file URI: encode whitespace and special characters".into());
                }
                let uri =
                    glib::Uri::parse(text, glib::UriFlags::NONE).map_err(|_| "invalid file URI")?;
                if !uri.scheme().eq_ignore_ascii_case("file") {
                    return Err("only local paths and file:// URIs are accepted".into());
                }
                if uri.query().is_some()
                    || uri.fragment().is_some()
                    || uri.userinfo().is_some()
                    || uri.port() != -1
                {
                    return Err(
                        "file URI query, fragment, credentials and port are unsupported".into(),
                    );
                }
                let (path, host) =
                    glib::filename_from_uri(text).map_err(|_| "invalid local file URI")?;
                if host
                    .as_deref()
                    .is_some_and(|h| !h.is_empty() && !h.eq_ignore_ascii_case("localhost"))
                {
                    return Err("remote file URI hosts are unsupported".into());
                }
                gio::File::for_path(path)
            }
            _ => gio::File::for_path(Path::new(argument)),
        };
        Self::from_file(&file)
    }

    pub fn from_file(file: &gio::File) -> Result<Self, String> {
        let path = file.path().ok_or("only local files are accepted")?;
        // Follow symlinks, but preserve the requested GFile/path association.
        let metadata = fs::metadata(&path).map_err(|e| format!("cannot access local file: {e}"))?;
        if !metadata.is_file() {
            return Err("input must be a regular file, not a directory or special file".into());
        }
        let kind = pivra::discovery::candidate_kind(&path)
            .ok_or("unsupported media extension")?;
        // Open read-only and inspect the descriptor; never read file contents.
        // This is a point-in-time check, not a promise against later replacement.
        let handle = fs::File::open(&path).map_err(|e| format!("cannot read local file: {e}"))?;
        if !handle
            .metadata()
            .map_err(|e| format!("cannot inspect local file: {e}"))?
            .is_file()
        {
            return Err("input changed to a non-regular file".into());
        }
        Ok(Self {
            file: file.clone(),
            item: MediaItem::new(path, kind),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::os::unix::{
        ffi::OsStringExt,
        fs::{PermissionsExt, symlink},
    };

    fn fixture(root: &Path, name: &OsStr) -> std::path::PathBuf {
        let path = root.join(name);
        fs::write(&path, include_bytes!("../tests/fixtures/pixels-2x2.png")).unwrap();
        path
    }

    #[test]
    fn local_path_uri_and_localhost_preserve_unicode_spaces_and_reserved_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = fixture(dir.path(), OsStr::new("fotoğraf #?% örneği 🦀.PNG"));
        let uri = glib::filename_to_uri(&path, None).unwrap();
        assert!(uri.contains("%23") && uri.contains("%25") && uri.contains("%20"));
        let localhost = glib::filename_to_uri(&path, Some("localhost")).unwrap();
        for argument in [
            path.as_os_str(),
            OsStr::new(uri.as_str()),
            OsStr::new(localhost.as_str()),
        ] {
            let input = OpenedFile::from_argument(argument).unwrap();
            assert_eq!(input.item.path(), path);
            assert_eq!(input.item.kind(), MediaKind::Image);
            assert_eq!(input.file.path().as_deref(), Some(path.as_path()));
        }
        assert_eq!(
            fs::read(path).unwrap(),
            include_bytes!("../tests/fixtures/pixels-2x2.png")
        );
    }

    #[test]
    fn relative_paths_and_non_utf8_filenames_do_not_require_string_conversion() {
        let dir = tempfile::Builder::new()
            .prefix("pivra-relative-")
            .tempdir_in(".")
            .unwrap();
        let path = fixture(dir.path(), OsStr::new("göreli.png"));
        let input = OpenedFile::from_argument(path.as_os_str()).unwrap();
        assert!(input.item.path().is_absolute());
        assert_eq!(
            input.item.path().canonicalize().unwrap(),
            path.canonicalize().unwrap()
        );
        let raw = OsString::from_vec(b"photo-\xff.png".to_vec());
        let path = fixture(dir.path(), &raw);
        assert_eq!(
            OpenedFile::from_argument(path.as_os_str())
                .unwrap()
                .item
                .path()
                .file_name(),
            Some(raw.as_os_str())
        );
    }

    #[test]
    fn rejects_remote_malformed_and_ambiguous_uris_without_network_access() {
        for text in [
            "https://example.org/photo.png",
            "smb://host/share/a.png",
            "file://host/tmp/a.png",
            "file:///tmp/bad%ZZ.png",
            "file:///tmp/a%00.png",
            "file:relative.png",
            "file:///tmp/a.png?query",
            "file:///tmp/a.png#fragment",
            "file:///tmp/a b.png",
            "file://user@localhost/tmp/a.png",
            "file://localhost:80/tmp/a.png",
        ] {
            assert!(
                OpenedFile::from_argument(OsStr::new(text)).is_err(),
                "{text}"
            );
        }
        assert!(
            OpenedFile::from_file(&gio::File::for_uri("https://example.org/photo.png")).is_err()
        );
    }

    #[test]
    fn rejects_missing_directories_special_files_and_unsupported_extensions() {
        let dir = tempfile::tempdir().unwrap();
        for path in [
            dir.path().to_owned(),
            dir.path().join("missing.png"),
            std::path::PathBuf::from("/dev/null"),
        ] {
            assert!(OpenedFile::from_argument(path.as_os_str()).is_err());
        }
        for name in ["text.txt", "no-extension"] {
            let path = fixture(dir.path(), OsStr::new(name));
            assert!(OpenedFile::from_argument(path.as_os_str()).is_err());
        }
    }

    #[test]
    fn accepts_supported_video_extensions_as_video_media() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["clip.mp4", "clip.mkv", "clip.webm", "clip.mov", "clip.avi"] {
            let path = fixture(dir.path(), OsStr::new(name));
            let opened = OpenedFile::from_argument(path.as_os_str()).unwrap();
            assert_eq!(opened.item.kind(), MediaKind::Video, "{name}");
            assert_eq!(opened.item.path(), path);
        }
    }

    #[test]
    fn follows_readable_symlinks_but_preserves_alias_and_rejects_dangling_links() {
        let dir = tempfile::tempdir().unwrap();
        let target = fixture(dir.path(), OsStr::new("target.png"));
        let alias = dir.path().join("bağlantı.jpg");
        symlink(&target, &alias).unwrap();
        assert_eq!(
            OpenedFile::from_argument(alias.as_os_str())
                .unwrap()
                .item
                .path(),
            alias
        );
        fs::remove_file(target).unwrap();
        assert!(OpenedFile::from_argument(alias.as_os_str()).is_err());
    }

    #[test]
    fn checks_read_permission_without_modifying_contents() {
        let dir = tempfile::tempdir().unwrap();
        let path = fixture(dir.path(), OsStr::new("private.png"));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        // The development/test environment is unprivileged; privileged runners
        // may read mode-000 files, so compare against the actual open result.
        let readable = fs::File::open(&path).is_ok();
        assert_eq!(
            OpenedFile::from_argument(path.as_os_str()).is_ok(),
            readable
        );
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            fs::read(path).unwrap(),
            include_bytes!("../tests/fixtures/pixels-2x2.png")
        );
    }

    #[test]
    fn extension_is_not_decode_and_large_file_is_not_loaded() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("not-decoded.jpg");
        let file = fs::File::create_new(&path).unwrap();
        file.set_len(8 * 1024 * 1024 * 1024).unwrap(); // sparse, zero contents
        assert!(OpenedFile::from_argument(path.as_os_str()).is_ok());
        assert_eq!(file.metadata().unwrap().len(), 8 * 1024 * 1024 * 1024);
    }
}
