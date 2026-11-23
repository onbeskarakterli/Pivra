//! Controlled GTK state-order regression, not a decoder/GPU success claim.
#[path = "support/video_preparation.rs"]
mod video_preparation;
use gtk::{gdk, glib, prelude::*, subclass::prelude::*};
use std::{
    cell::OnceCell,
    time::{Duration, Instant},
};

mod imp {
    use super::*;
    #[derive(Default)]
    pub struct ControlledMedia {
        pub texture: OnceCell<gdk::Texture>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for ControlledMedia {
        const NAME: &'static str = "PivraControlledPreparationProbe";
        type Type = super::ControlledMedia;
        type ParentType = gtk::MediaFile;
        type Interfaces = (gdk::Paintable,);
    }
    impl ObjectImpl for ControlledMedia {}
    impl MediaFileImpl for ControlledMedia {
        fn open(&self) {}
        fn close(&self) {}
    }
    impl MediaStreamImpl for ControlledMedia {
        fn play(&self) -> bool {
            true
        }
        fn pause(&self) {}
    }
    impl gdk::subclass::prelude::PaintableImpl for ControlledMedia {
        fn intrinsic_width(&self) -> i32 {
            1
        }
        fn intrinsic_height(&self) -> i32 {
            1
        }
        fn snapshot(&self, snapshot: &gdk::Snapshot, width: f64, height: f64) {
            snapshot
                .downcast_ref::<gtk::Snapshot>()
                .unwrap()
                .append_texture(
                    self.texture.get().unwrap(),
                    &gtk::graphene::Rect::new(0., 0., width as f32, height as f32),
                );
        }
    }
}
glib::wrapper! {
    pub struct ControlledMedia(ObjectSubclass<imp::ControlledMedia>)
        @extends gtk::MediaFile, gtk::MediaStream, @implements gdk::Paintable;
}
fn media() -> ControlledMedia {
    let media: ControlledMedia = glib::Object::new();
    let texture = gdk::MemoryTexture::new(
        1,
        1,
        gdk::MemoryFormat::R8g8b8a8,
        &glib::Bytes::from_static(&[0, 0, 0, 255]),
        4,
    );
    media.imp().texture.set(texture.upcast()).unwrap();
    media
}
fn fail(media: &ControlledMedia) {
    use glib::translate::{IntoGlibPtr, ToGlibPtr};
    let error = glib::Error::new(gtk::gio::IOErrorEnum::Failed, "controlled prepare failure");
    unsafe {
        gtk::ffi::gtk_media_stream_gerror(
            media.upcast_ref::<gtk::MediaStream>().to_glib_none().0,
            error.into_glib_ptr(),
        );
    }
}
fn main() {
    gtk::init().unwrap();
    let first = media();
    let snapshot = gtk::Snapshot::new();
    first.snapshot(&snapshot, 1., 1.);
    assert!(
        snapshot.to_node().is_some(),
        "controlled first frame really exists"
    );
    assert!(!first.is_prepared());
    assert!(!first.is_seekable());
    assert_eq!(first.duration(), 0);
    println!(
        "CONTROLLED rendered frame precedes prepared/seekable/duration; old immediate control check fails"
    );
    let later = first.clone();
    glib::idle_add_local_once(move || later.stream_prepared(false, true, true, 2_000_000));
    video_preparation::wait(&first, Instant::now() + Duration::from_secs(1)).unwrap();
    assert!(first.is_prepared() && first.is_seekable() && first.duration() == 2_000_000);
    video_preparation::wait(&first, Instant::now()).unwrap();
    println!("PASS controlled delayed preparation and already-ready fast path");
    for (name, prepare, seekable, duration) in [
        ("unprepared", false, false, 0),
        ("nonseekable", true, false, 2_000_000),
        ("unknown-duration", true, true, 0),
    ] {
        let stream = media();
        if prepare {
            stream.stream_prepared(false, true, seekable, duration);
        }
        let error = video_preparation::wait(&stream, Instant::now()).unwrap_err();
        assert!(
            error.contains(&format!(
                "prepared={prepare} seekable={seekable} duration_us={duration}"
            )),
            "{error}"
        );
        println!("PASS controlled {name} remains FAIL with exact capability state");
    }
    let erroneous = media();
    erroneous.stream_prepared(false, true, true, 2_000_000);
    fail(&erroneous);
    assert!(
        video_preparation::wait(&erroneous, Instant::now())
            .unwrap_err()
            .contains("controlled prepare failure")
    );
    let pending = media();
    let later = pending.clone();
    glib::idle_add_local_once(move || fail(&later));
    assert!(
        video_preparation::wait(&pending, Instant::now() + Duration::from_secs(1))
            .unwrap_err()
            .contains("controlled prepare failure")
    );
    println!("PASS controlled existing/late errors cannot become control success");
    println!("PASS control preparation regression: 7 GTK state cases, no real decode or GPU claim");
}
