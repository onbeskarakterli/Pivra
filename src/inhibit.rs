//! Inhibit-cookie ownership, independent of IPC/GTK. Reentrant acquisitions can
//! complete after pause/close; their cookies must be returned for immediate release.
#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    None,
    Acquire(u64),
    Release(u32),
}
#[derive(Default)]
pub struct Lease {
    active: bool,
    epoch: u64,
    cookie: Option<u32>,
}
impl Lease {
    pub fn update(&mut self, active: bool) -> Change {
        if self.active == active {
            return Change::None;
        }
        self.active = active;
        self.epoch = self.epoch.wrapping_add(1);
        if active {
            Change::Acquire(self.epoch)
        } else {
            self.cookie.take().map_or(Change::None, Change::Release)
        }
    }
    pub fn complete(&mut self, epoch: u64, cookie: u32) -> Option<u32> {
        if cookie == 0 {
            return None;
        }
        if self.active && self.epoch == epoch && self.cookie.is_none() {
            self.cookie = Some(cookie);
            None
        } else {
            Some(cookie)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pause_end_switch_and_close_release_exactly_once() {
        let mut lease = Lease::default();
        for cookie in [11, 12, 13, 14] {
            let Change::Acquire(epoch) = lease.update(true) else {
                panic!("acquire");
            };
            assert_eq!(lease.complete(epoch, cookie), None);
            assert_eq!(lease.update(true), Change::None);
            assert_eq!(lease.update(false), Change::Release(cookie));
            assert_eq!(lease.update(false), Change::None);
        }
    }
    #[test]
    fn late_acquisition_and_unsupported_service_do_not_leak_or_spin() {
        let mut lease = Lease::default();
        let Change::Acquire(old) = lease.update(true) else {
            panic!("acquire");
        };
        assert_eq!(lease.update(false), Change::None);
        let Change::Acquire(new) = lease.update(true) else {
            panic!("acquire");
        };
        assert_eq!(lease.complete(old, 21), Some(21));
        assert_eq!(lease.complete(new, 0), None);
        assert_eq!(lease.update(true), Change::None);
        assert_eq!(lease.update(false), Change::None);
    }
}
