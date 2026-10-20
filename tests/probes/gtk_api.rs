//! Synthetic API capability probe only: no media file, decoder or player.

use adw::prelude::*;
use gtk::{gdk, glib};

fn main() {
    adw::init().expect("probe needs a graphical display");
    let bytes = glib::Bytes::from_static(&[255, 255, 255, 255]);
    let texture = gdk::MemoryTexture::new(1, 1, gdk::MemoryFormat::R8g8b8a8, &bytes, 4);
    let paintable: gdk::Paintable = texture.clone().upcast();
    let picture = gtk::Picture::for_paintable(&paintable);
    assert_eq!(picture.paintable().as_ref(), Some(&paintable));
    assert_eq!((texture.width(), texture.height()), (1, 1));
    let mut downloaded = [0; 4];
    texture.download(&mut downloaded, 4);
    assert_eq!(downloaded, [255; 4]);
    println!("PASS synthetic MemoryTexture -> Paintable -> Picture; pixel download");

    let snapshot = gtk::Snapshot::new();
    paintable.snapshot(&snapshot, 1.0, 1.0);
    let node = snapshot
        .to_node()
        .expect("paintable produced a render node");
    assert_eq!(node.node_type(), gtk::gsk::RenderNodeType::TextureNode);
    assert_eq!((node.bounds().width(), node.bounds().height()), (1.0, 1.0));
    println!("PASS snapshot produces GSK TextureNode (not GPU rendering validation)");

    let gesture = gtk::GestureZoom::new();
    assert!(gesture.is::<gtk::Gesture>());
    let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::BOTH_AXES);
    assert_eq!(scroll.flags(), gtk::EventControllerScrollFlags::BOTH_AXES);
    println!("PASS gesture/scroll objects; actual pinch/touchpad input NOT RUN");

    let window = gtk::Window::builder()
        .title("PIV-05 synthetic API probe")
        .build();
    gtk::prelude::WidgetExt::realize(&window);
    let surface = window.surface().expect("realized window has a surface");
    assert!(surface.scale_factor() >= 1);
    let scale = surface.property::<f64>("scale");
    assert!(scale >= 1.0);
    println!(
        "PASS surface scale access: integer={}, scale={}; fractional hardware NOT RUN",
        surface.scale_factor(),
        scale
    );
    window.destroy();
}
