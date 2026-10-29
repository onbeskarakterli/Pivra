//! Shared real GTK frame synchronization for presentation probes.
use super::pump;
use adw::prelude::*;
use gtk::glib;
use std::time::{Duration, Instant};
// WidgetPaintable reflects GTK's rendered widget cache, not decoder completion.
// Keep it alive across a real frame; a fixed sleep cannot certify layout/paint.
pub(super) fn rendered_node(w: &adw::ApplicationWindow, p: &gtk::Picture) -> gtk::gsk::RenderNode {
    use std::{cell::Cell, rc::Rc};
    assert!(
        p.paintable().is_some(),
        "snapshot requested without image content"
    );
    let paintable = gtk::WidgetPaintable::new(Some(p));
    w.present(); // In two-window cases the target may be compositor-occluded.
    let clock = p.frame_clock().expect("mapped picture frame clock");
    let painted = Rc::new(Cell::new(false));
    let observed = painted.clone();
    let handler = clock.connect_after_paint(move |_| observed.set(true));
    let tick = p.add_tick_callback(|_, _| glib::ControlFlow::Continue);
    p.queue_draw();
    let end = Instant::now() + Duration::from_secs(20);
    let node = loop {
        if painted.replace(false) && p.is_mapped() && p.width() > 0 && p.height() > 0 {
            let snapshot = gtk::Snapshot::new();
            paintable.snapshot(&snapshot, p.width() as f64, p.height() as f64);
            if let Some(node) = snapshot.to_node() {
                break Some(node);
            }
            p.queue_draw();
        }
        if Instant::now() >= end {
            break None;
        }
        pump();
    };
    tick.remove();
    clock.disconnect(handler);
    node.unwrap_or_else(|| panic!(
        "rendered image missing after GTK after-paint: mapped={} visible={} allocation={}x{} content={} mode={}",
        p.is_mapped(), p.is_visible(), p.width(), p.height(), p.paintable().is_some(), w.lookup_action("image-mode").and_then(|a| a.state()).map_or_else(|| "unavailable".into(), |v| v.to_string())
    ))
}
