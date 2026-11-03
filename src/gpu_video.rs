//! Opt-in, file-scoped GPU backend prototype. Stock GTK remains the default.
use gtk::{gio, glib, prelude::*};
use std::path::Path;

fn selected(path: &Path) -> bool {
    let Some(target) = std::env::var_os("PIVRA_GPU_VIDEO_TARGET") else {
        return false;
    };
    matches_target(path, Path::new(&target))
}

fn matches_target(path: &Path, target: &Path) -> bool {
    matches!((path.canonicalize(), target.canonicalize()), (Ok(a), Ok(b)) if a == b)
}

pub fn is_owned(media: &gtk::MediaFile) -> bool {
    media.type_().name() == "PivraGpuMediaFile"
}

pub fn open(path: &Path) -> Result<gtk::MediaFile, String> {
    if !selected(path) {
        let media = gtk::MediaFile::for_filename(path);
        if std::env::var_os("PIVRA_GPU_VIDEO_MODULE_DIR").is_some() {
            eprintln!(
                "PIVRA_GPU_STOCK type={} file={}",
                media.type_(),
                path.display()
            );
        }
        return Ok(media);
    }
    let renderer = if gtk::minor_version() < 20 {
        "ngl"
    } else {
        "gl"
    };
    if std::env::var("GSK_RENDERER").as_deref() != Ok(renderer) {
        return Err(format!(
            "GPU prototype requires GSK_RENDERER={renderer}; Vulkan interop is not implemented"
        ));
    }
    let directory = std::env::var_os("PIVRA_GPU_VIDEO_MODULE_DIR")
        .ok_or("GPU prototype selected without a module directory")?;
    let directory = Path::new(&directory)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !directory.join("libpivra-color-gpu.so").is_file() {
        return Err("GPU prototype module missing".into());
    }
    // Resolve and cache the stock implementation before registering our low-rank type.
    drop(gtk::MediaFile::new());
    let ep = gio::IOExtensionPoint::lookup("gtk-media-file").unwrap_or_else(|| {
        gio::IOExtensionPoint::builder("gtk-media-file")
            .required_type(gtk::MediaFile::static_type())
            .build()
    });
    gio::io_modules_scan_all_in_directory(&directory);
    let type_ = ep
        .extension_by_name("pivra-color-gpu")
        .ok_or("GPU module did not register")?
        .type_();
    if type_.name() != "PivraGpuMediaFile" || !type_.is_a(gtk::MediaFile::static_type()) {
        return Err(format!("unexpected GPU module type {type_}"));
    }
    // Construct explicitly: do not override GTK's process-wide media factory.
    let media = glib::Object::with_type(type_)
        .downcast::<gtk::MediaFile>()
        .map_err(|_| "GPU MediaFile construction failed")?;
    if !is_owned(&media) {
        return Err("GPU constructor returned an unexpected subtype".into());
    }
    media.set_filename(Some(path));
    eprintln!("PIVRA_GPU_BACKEND type={type_} target={}", path.display());
    Ok(media)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_scope_does_not_capture_other_navigation() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.mp4");
        let other = dir.path().join("other.mp4");
        std::fs::write(&target, []).unwrap();
        std::fs::write(&other, []).unwrap();
        assert!(matches_target(&target, &target));
        assert!(!matches_target(&other, &target));
        std::fs::remove_file(&target).unwrap();
        assert!(!matches_target(&target, &target));
    }
}
