//! Explicit, test-only GIO module selection. Never linked into the product.
use gtk::{gio, glib, prelude::*};

pub fn load() -> Result<Option<glib::Type>, String> {
    let Some(directory) = std::env::var_os("PIVRA_VIDEO_TEST_MODULE_DIR") else {
        return Ok(None);
    };
    let selected = std::env::var("GTK_MEDIA").map_err(|_| "GTK_MEDIA missing")?;
    let expected = match selected.as_str() {
        "pivra-color-stock" => "PivraStockMediaFile",
        "pivra-color-candidate" => "PivraCandidateMediaFile",
        _ => return Err(format!("unsupported test extension {selected}")),
    };
    let directory = std::path::Path::new(&directory)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let file = directory.join(format!("lib{selected}.so"));
    if !file.is_file() {
        return Err(format!("missing module {}", file.display()));
    }
    let ep = gio::IOExtensionPoint::lookup("gtk-media-file").unwrap_or_else(|| {
        gio::IOExtensionPoint::builder("gtk-media-file")
            .required_type(gtk::MediaFile::static_type())
            .build()
    });
    // Public GIO API, deliberately independent of GTK_PATH discovery order.
    gio::io_modules_scan_all_in_directory(&directory);
    let extension = ep
        .extension_by_name(&selected)
        .ok_or_else(|| format!("module did not register {selected}: {}", file.display()))?;
    let type_ = extension.type_();
    if type_.name() != expected || !type_.is_a(gtk::MediaFile::static_type()) {
        return Err(format!("extension {selected}: unexpected type {type_}"));
    }
    println!(
        "BACKEND registered={selected} type={type_} module={}",
        file.display()
    );
    Ok(Some(type_))
}

pub fn verify(media: &gtk::MediaFile, expected: Option<glib::Type>) -> Result<(), String> {
    println!("BACKEND selected={}", media.type_().name());
    if let Some(expected) = expected {
        if media.type_() != expected {
            return Err(format!(
                "backend selection failed: expected {expected}, actual {}",
                media.type_()
            ));
        }
    }
    Ok(())
}
