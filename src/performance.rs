//! Explicit opt-in, bounded JSONL diagnostics. No retained samples or per-frame hook.
use std::{
    sync::{
        OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};
static ENABLED: OnceLock<bool> = OnceLock::new();
static EVENTS: AtomicUsize = AtomicUsize::new(0);
const LIMIT: usize = 4096;
pub fn enabled() -> bool {
    *ENABLED.get_or_init(|| std::env::var("PIVRA_PERF").as_deref() == Ok("1"))
}
fn reserve(counter: &AtomicUsize) -> bool {
    let mut n = counter.load(Ordering::Relaxed);
    loop {
        if n >= LIMIT {
            return false;
        }
        match counter.compare_exchange_weak(n, n + 1, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return true,
            Err(actual) => n = actual,
        }
    }
}
fn cpu_ticks(stat: &str) -> Option<u64> {
    let fields: Vec<_> = stat.rsplit_once(')')?.1.split_whitespace().collect();
    fields
        .get(11)?
        .parse::<u64>()
        .ok()?
        .checked_add(fields.get(12)?.parse().ok()?)
}
pub fn emit(event: &str, status: &'static str, elapsed_us: u128, value: u64) {
    if !enabled() || !reserve(&EVENTS) {
        return;
    }
    let event = if event
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        event
    } else {
        "invalid_event"
    };
    let rss = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find_map(|l| {
                    l.strip_prefix("VmRSS:")
                        .and_then(|v| v.split_whitespace().next()?.parse::<u64>().ok())
                })
                .map(|kb| kb * 1024)
        });
    let cpu = std::fs::read_to_string("/proc/self/stat")
        .ok()
        .and_then(|s| cpu_ticks(&s));
    eprintln!(
        "{{\"pivra_perf\":1,\"pid\":{},\"event\":\"{event}\",\"status\":\"{status}\",\"elapsed_us\":{elapsed_us},\"value\":{value},\"rss_bytes\":{},\"cpu_ticks\":{}}}",
        std::process::id(),
        rss.map_or("null".into(), |v| v.to_string()),
        cpu.map_or("null".into(), |v| v.to_string())
    );
}
pub struct Span {
    start: Option<Instant>,
    event: &'static str,
    status: &'static str,
}
impl Span {
    pub fn new(event: &'static str) -> Self {
        Self {
            start: enabled().then(Instant::now),
            event,
            status: "returned",
        }
    }
    pub fn status(&mut self, status: &'static str) {
        self.status = status;
    }
}
impl Drop for Span {
    fn drop(&mut self) {
        if let Some(start) = self.start {
            emit(self.event, self.status, start.elapsed().as_micros(), 0);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cpu_parser_handles_spaces_and_parentheses() {
        let s = "1 (a ) b) S 0 0 0 0 0 0 0 0 0 0 12 34 0";
        assert_eq!(cpu_ticks(s), Some(46));
        assert_eq!(cpu_ticks("broken"), None);
    }
    #[test]
    fn diagnostics_are_process_bounded() {
        let n = AtomicUsize::new(0);
        for _ in 0..LIMIT {
            assert!(reserve(&n));
        }
        for _ in 0..5 {
            assert!(!reserve(&n));
        }
        assert_eq!(n.load(Ordering::Relaxed), LIMIT);
    }
}
