//! Opt-in, process-bounded thumbnail lifecycle diagnostics; no retained history.
use std::{
    fmt,
    path::Path,
    sync::{
        OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};
const MAX_EVENTS: usize = 4096;
static START: OnceLock<Instant> = OnceLock::new();
static EVENTS: AtomicUsize = AtomicUsize::new(0);

pub fn enabled() -> bool {
    std::env::var_os("PIVRA_TRACE_THUMBNAILS").is_some_and(|v| v == "1")
        || std::env::var_os("PIVRA_TRACE_GNOME_THUMB").is_some()
}
fn reserve(counter: &AtomicUsize, limit: usize) -> Option<usize> {
    let mut n = counter.load(Ordering::Relaxed);
    loop {
        if n > limit {
            return None;
        }
        let next = n.checked_add(1)?;
        match counter.compare_exchange_weak(n, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return Some(n),
            Err(current) => n = current,
        }
    }
}
fn bounded(value: impl fmt::Display) -> String {
    format!("{value}")
        .chars()
        .take(512)
        .collect::<String>()
        .escape_debug()
        .to_string()
}
pub fn trace(stage: &str, path: Option<&Path>, details: impl fmt::Display) {
    if !enabled() {
        return;
    }
    let Some(sequence) = reserve(&EVENTS, MAX_EVENTS) else {
        return;
    };
    if sequence == MAX_EVENTS {
        eprintln!(
            "PIVRA_THUMB diagnostic limit reached ({MAX_EVENTS}); restart to capture a new session"
        );
        return;
    }
    eprintln!(
        "PIVRA_THUMB seq={sequence} elapsed_ms={} thread={:?} stage={stage} path={} detail={}",
        START.get_or_init(Instant::now).elapsed().as_millis(),
        std::thread::current().id(),
        path.map(|p| bounded(p.display())).unwrap_or_default(),
        bounded(details)
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn event_budget_saturates_instead_of_wrapping_or_growing_history() {
        let n = AtomicUsize::new(0);
        assert_eq!(reserve(&n, 2), Some(0));
        assert_eq!(reserve(&n, 2), Some(1));
        assert_eq!(reserve(&n, 2), Some(2));
        for _ in 0..100 {
            assert_eq!(reserve(&n, 2), None);
        }
        assert_eq!(n.load(Ordering::Relaxed), 3);
    }
    #[test]
    fn fields_are_bounded_and_cannot_inject_log_lines() {
        assert_eq!(bounded("a\nb"), "a\\nb");
        assert_eq!(bounded("x".repeat(10000)).len(), 512);
    }
}
