//! Probe-only Glycin pool ownership events; no production logger/dependency.
use std::{
    fmt,
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};
use tracing::{
    Event, Metadata, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Id, Record},
};
struct PoolTrace {
    start: Instant,
    ids: AtomicU64,
}
struct Message(String);
impl Visit for Message {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        use fmt::Write;
        let _ = write!(self.0, " {}={value:?}", field.name());
    }
}
impl Subscriber for PoolTrace {
    fn enabled(&self, meta: &Metadata<'_>) -> bool {
        matches!(
            meta.target(),
            "glycin::pool" | "glycin::api_loader" | "glycin::dbus"
        )
    }
    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(self.ids.fetch_add(1, Ordering::Relaxed))
    }
    fn record(&self, _: &Id, _: &Record<'_>) {}
    fn record_follows_from(&self, _: &Id, _: &Id) {}
    fn event(&self, event: &Event<'_>) {
        let mut message = Message(String::new());
        event.record(&mut message);
        eprintln!(
            "POOL_EVENT elapsed_ms={} target={}{}",
            self.start.elapsed().as_millis(),
            event.metadata().target(),
            message.0
        );
    }
    fn enter(&self, _: &Id) {}
    fn exit(&self, _: &Id) {}
}
pub(super) fn install() {
    let shared = std::sync::Arc::ptr_eq(&glycin::Pool::global(), &glycin::Pool::global());
    eprintln!(
        "POOL_SCOPE same_global_arc={shared}; default Loader::new uses this process-wide pool"
    );
    tracing::subscriber::set_global_default(PoolTrace {
        start: Instant::now(),
        ids: AtomicU64::new(1),
    })
    .expect("probe pool logger");
}

#[cfg(test)]
mod tests {
    #[test]
    fn global_pool_is_shared_within_process() {
        assert!(std::sync::Arc::ptr_eq(
            &glycin::Pool::global(),
            &glycin::Pool::global()
        ));
    }
}
