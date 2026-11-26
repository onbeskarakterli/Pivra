//! Opt-in event timestamps; never change chooser routing, scheduling or messages.
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Instant,
};
fn enabled() -> bool {
    std::env::var("PIVRA_TRACE_OPEN").as_deref() == Ok("1")
}
fn stamp() -> u128 {
    pivra::performance::monotonic_us()
}
static NEXT: AtomicU64 = AtomicU64::new(1);

pub(super) fn created() {
    if enabled() {
        eprintln!(
            "PIVRA_OPEN mono_us={} stage=file-dialog-descriptor-created native_visibility=not-requested",
            stamp()
        );
    }
}
pub(super) struct Trace {
    id: u64,
    start: Instant,
    parent: glib::WeakRef<adw::ApplicationWindow>,
    model: gio::ListModel,
    changed: RefCell<Option<glib::SignalHandlerId>>,
    windows: RefCell<
        Vec<(
            glib::WeakRef<gtk::Window>,
            glib::SignalHandlerId,
            glib::SignalHandlerId,
        )>,
    >,
    frame: RefCell<Option<(gdk::FrameClock, glib::SignalHandlerId)>>,
    mapped: Cell<bool>,
}
impl Trace {
    pub(super) fn begin(parent: &adw::ApplicationWindow, modal: bool) -> Option<Rc<Self>> {
        if !enabled() {
            return None;
        }
        let trace = Rc::new(Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            start: Instant::now(),
            parent: parent.downgrade(),
            model: gtk::Window::toplevels(),
            changed: RefCell::new(None),
            windows: RefCell::new(vec![]),
            frame: RefCell::new(None),
            mapped: Cell::new(false),
        });
        trace.mark("action-activated");
        eprintln!(
            "PIVRA_OPEN id={} parent_mapped={} parent_realized={} modal={} backend={} gtk={}.{}.{}",
            trace.id,
            parent.is_mapped(),
            parent.is_realized(),
            modal,
            gtk::prelude::WidgetExt::display(parent).type_().name(),
            gtk::major_version(),
            gtk::minor_version(),
            gtk::micro_version()
        );
        let weak = Rc::downgrade(&trace);
        let id = trace
            .model
            .connect_items_changed(move |model, start, _, added| {
                if let Some(trace) = weak.upgrade() {
                    for n in start..start + added {
                        trace.watch(model.item(n));
                    }
                }
            });
        trace.changed.replace(Some(id));
        for n in 0..trace.model.n_items() {
            trace.watch(trace.model.item(n));
        }
        Some(trace)
    }
    pub(super) fn mark(&self, stage: &'static str) {
        eprintln!(
            "PIVRA_OPEN id={} mono_us={} elapsed_us={} stage={stage}",
            self.id,
            stamp(),
            self.start.elapsed().as_micros()
        );
    }
    fn watch(self: &Rc<Self>, object: Option<glib::Object>) {
        let Some(window) = object.and_downcast::<gtk::Window>() else {
            return;
        };
        // Public GTK type identity only. Portal windows live in another process
        // and must never be represented as a locally observed map.
        if window.type_().name() != "GtkFileChooserDialog" || self.windows.borrow().len() >= 16 {
            return;
        }
        let weak = Rc::downgrade(self);
        let mapped = window.connect_map(move |window| {
            let Some(trace) = weak.upgrade() else {
                return;
            };
            if window.transient_for() != trace.parent.upgrade().map(|p| p.upcast()) {
                return;
            }
            trace.mapped.set(true);
            trace.mark("local-chooser-mapped");
            if let Some(clock) = window.frame_clock() {
                if let Some((old, id)) = trace.frame.borrow_mut().take() {
                    old.disconnect(id);
                }
                let weak = Rc::downgrade(&trace);
                let id = clock.connect_after_paint(move |clock| {
                    if let Some(trace) = weak.upgrade() {
                        trace.mark("local-chooser-after-paint");
                        if let Some((_, id)) = trace.frame.borrow_mut().take() {
                            clock.disconnect(id);
                        }
                    }
                });
                trace.frame.replace(Some((clock, id)));
            }
        });
        let weak = Rc::downgrade(self);
        let unmapped = window.connect_unmap(move |window| {
            if let Some(trace) = weak.upgrade() {
                if window.transient_for() == trace.parent.upgrade().map(|p| p.upcast()) {
                    trace.mark("local-chooser-unmapped");
                }
            }
        });
        self.windows
            .borrow_mut()
            .push((window.downgrade(), mapped, unmapped));
    }
    pub(super) fn finish(&self) {
        if !self.mapped.get() {
            self.mark("visibility-unobserved-external-or-cancelled-before-map");
        }
    }
}
impl Drop for Trace {
    fn drop(&mut self) {
        if let Some(id) = self.changed.get_mut().take() {
            self.model.disconnect(id);
        }
        for (window, map, unmap) in self.windows.get_mut().drain(..) {
            if let Some(window) = window.upgrade() {
                window.disconnect(map);
                window.disconnect(unmap);
            }
        }
        if let Some((clock, id)) = self.frame.get_mut().take() {
            clock.disconnect(id);
        }
    }
}
pub(super) fn mark(trace: &Option<Rc<Trace>>, stage: &'static str) {
    if let Some(trace) = trace {
        trace.mark(stage);
    }
}

// Observe this process's existing session connection only. No eavesdropping,
// activation, extra portal request, message rewriting or GTK call on the I/O thread.
pub(super) fn observe_bus(app: &adw::Application) {
    static INSTALLED: AtomicBool = AtomicBool::new(false);
    if !enabled() || INSTALLED.swap(true, Ordering::Relaxed) {
        return;
    }
    let cancel = gio::Cancellable::new();
    let closing = cancel.clone();
    app.connect_shutdown(move |_| closing.cancel());
    let weak = app.downgrade();
    let ticket = cancel.clone();
    gio::bus_get(gio::BusType::Session, Some(&cancel), move |result| {
        if ticket.is_cancelled() {
            return;
        }
        let (Ok(bus), Some(app)) = (result, weak.upgrade()) else {
            return;
        };
        let calls = Arc::new(Mutex::new(std::collections::BTreeMap::<
            u32,
            (Instant, &'static str),
        >::new()));
        let filter = bus.add_filter(move |_, msg, incoming| {
            let mut calls = calls.lock().unwrap();
            if !incoming && msg.destination().as_deref() == Some("org.freedesktop.portal.Desktop") {
                let stage = if msg.interface().as_deref() == Some("org.freedesktop.portal.FileChooser") && msg.member().as_deref() == Some("OpenFile") {
                    Some("open-file")
                } else if msg.interface().as_deref() == Some("org.freedesktop.DBus.Properties") && msg.member().as_deref() == Some("Get") && msg.body().and_then(|v| v.get::<(String,String)>()).is_some_and(|(i, _)| i == "org.freedesktop.portal.FileChooser") {
                    Some("file-chooser-version")
                } else { None };
                if let Some(stage) = stage.filter(|_| calls.len() < 16) {
                    calls.insert(msg.serial(), (Instant::now(), stage));
                    eprintln!("PIVRA_OPEN_BUS mono_us={} stage={stage}-sent serial={}", stamp(), msg.serial());
                }
            } else if incoming && matches!(msg.message_type(), gio::DBusMessageType::MethodReturn | gio::DBusMessageType::Error) {
                if let Some((start, stage)) = calls.remove(&msg.reply_serial()) {
                    eprintln!("PIVRA_OPEN_BUS mono_us={} stage={stage}-reply serial={} elapsed_us={} error={}", stamp(), msg.reply_serial(), start.elapsed().as_micros(), msg.error_name().as_deref().unwrap_or("none"));
                }
            }
            Some(msg.clone())
        });
        eprintln!(
            "PIVRA_OPEN_BUS mono_us={} stage=observer-ready visibility=external-unobservable",
            stamp()
        );
        let filter = RefCell::new(Some(filter));
        app.connect_shutdown(move |_| {
            if let Some(filter) = filter.borrow_mut().take() {
                bus.remove_filter(filter);
            }
        });
    });
}
