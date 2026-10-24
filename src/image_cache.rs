//! Bounded, deterministic RAM ownership. No decoder, UI or disk-cache writes.
use std::{
    collections::VecDeque,
    fs, io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};
pub const DEFAULT_BUDGET: usize = 128 * 1024 * 1024;
pub const DEFAULT_ENTRY_LIMIT: usize = 64 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileKey {
    pub path: PathBuf,
    dev: u64,
    ino: u64,
    len: u64,
    mtime: (i64, i64),
    ctime: (i64, i64),
}
impl FileKey {
    pub fn read(path: &Path) -> io::Result<Self> {
        let m = fs::metadata(path)?;
        if !m.is_file() {
            return Err(io::Error::other("not a regular file"));
        }
        Ok(Self {
            path: path.to_owned(),
            dev: m.dev(),
            ino: m.ino(),
            len: m.len(),
            mtime: (m.mtime(), m.mtime_nsec()),
            ctime: (m.ctime(), m.ctime_nsec()),
        })
    }
}
/// Charge two worst-case RGBA32F representations; not an RSS/VRAM measurement.
pub fn texture_cost(width: u32, height: u32) -> Option<usize> {
    u64::from(width)
        .checked_mul(u64::from(height))?
        .checked_mul(32)?
        .try_into()
        .ok()
}
#[derive(Clone, Copy, Debug, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub rejected: u64,
    pub entries: usize,
    pub bytes: usize,
    pub budget: usize,
}
struct Entry<T> {
    key: FileKey,
    value: T,
    cost: usize,
}
pub struct Cache<T> {
    entries: VecDeque<Entry<T>>,
    pin: Option<FileKey>,
    budget: usize,
    entry_limit: usize,
    stats: CacheStats,
}
impl<T: Clone> Default for Cache<T> {
    fn default() -> Self {
        Self::new(DEFAULT_BUDGET, DEFAULT_ENTRY_LIMIT)
    }
}
impl<T: Clone> Cache<T> {
    pub fn new(budget: usize, entry_limit: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            pin: None,
            budget,
            entry_limit,
            stats: CacheStats::default(),
        }
    }
    pub fn entry_limit(&self) -> usize {
        self.entry_limit
    }
    pub fn peek(&self, path: &Path) -> Option<T> {
        self.entries
            .iter()
            .find(|e| e.key.path == path)
            .map(|e| e.value.clone())
    }
    pub fn invalidate_path(&mut self, path: &Path) {
        let count = self.entries.len();
        self.entries.retain(|e| e.key.path != path);
        self.stats.evictions = self
            .stats
            .evictions
            .saturating_add((count - self.entries.len()) as u64);
    }
    pub fn contains_path(&self, path: &Path) -> bool {
        self.entries.iter().any(|entry| entry.key.path == path)
    }
    pub fn lookup(&mut self, key: &FileKey) -> Option<T> {
        if let Some(i) = self.entries.iter().position(|e| &e.key == key) {
            let e = self.entries.remove(i).unwrap();
            let value = e.value.clone();
            self.entries.push_back(e);
            self.stats.hits = self.stats.hits.saturating_add(1);
            Some(value)
        } else {
            self.invalidate_path(&key.path);
            self.stats.misses = self.stats.misses.saturating_add(1);
            None
        }
    }
    pub fn pin(&mut self, key: Option<FileKey>) {
        self.pin = key;
    }
    pub fn clear(&mut self) {
        self.stats.evictions = self
            .stats
            .evictions
            .saturating_add(self.entries.len() as u64);
        self.entries.clear();
        self.pin = None;
    }
    fn protected(&self, e: &Entry<T>, paths: &[PathBuf]) -> bool {
        self.pin.as_ref() == Some(&e.key) || paths.contains(&e.key.path)
    }
    pub fn available(&self, protected: &[PathBuf]) -> usize {
        let kept: usize = self
            .entries
            .iter()
            .filter(|e| self.protected(e, protected))
            .map(|e| e.cost)
            .sum();
        self.budget.saturating_sub(kept).min(self.entry_limit)
    }
    pub fn insert(&mut self, key: FileKey, value: T, cost: usize, protected: &[PathBuf]) -> bool {
        if let Some(index) = self
            .entries
            .iter()
            .position(|e| e.key == key && e.cost == cost)
        {
            let mut entry = self.entries.remove(index).unwrap();
            entry.value = value;
            self.entries.push_back(entry);
            return true;
        }
        if cost > self.entry_limit || cost > self.available(protected) {
            self.stats.rejected = self.stats.rejected.saturating_add(1);
            return false;
        }
        if let Some(i) = self.entries.iter().position(|e| e.key.path == key.path) {
            self.entries.remove(i);
        }
        while self.entries.len() >= 3 || self.stats().bytes.saturating_add(cost) > self.budget {
            let Some(i) = self
                .entries
                .iter()
                .position(|e| !self.protected(e, protected))
            else {
                self.stats.rejected = self.stats.rejected.saturating_add(1);
                return false;
            };
            self.entries.remove(i);
            self.stats.evictions = self.stats.evictions.saturating_add(1);
        }
        self.entries.push_back(Entry { key, value, cost });
        true
    }
    pub fn pressure(&mut self) {
        self.budget /= 2;
        let before = self.entries.len();
        self.entries.retain(|e| self.pin.as_ref() == Some(&e.key));
        self.stats.evictions = self
            .stats
            .evictions
            .saturating_add((before - self.entries.len()) as u64);
        if self.stats().bytes > self.budget {
            self.clear();
        }
    }
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.entries.len(),
            bytes: self.entries.iter().map(|e| e.cost).sum(),
            budget: self.budget,
            ..self.stats
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn key(path: &Path) -> FileKey {
        FileKey::read(path).unwrap()
    }
    #[test]
    fn bytes_lru_pin_pressure_and_overflow() {
        let d = tempfile::tempdir().unwrap();
        let paths: Vec<_> = (0..5)
            .map(|i| {
                let p = d.path().join(format!("{i}.png"));
                fs::write(&p, [i]).unwrap();
                p
            })
            .collect();
        let mut c = Cache::new(100, 60);
        let a = key(&paths[0]);
        c.pin(Some(a.clone()));
        assert!(c.insert(a.clone(), 0, 40, &[]));
        assert!(c.insert(key(&paths[1]), 1, 40, &[]));
        assert!(!c.insert(key(&paths[2]), 2, 61, &[]));
        assert!(c.insert(key(&paths[2]), 2, 40, &[]));
        assert_eq!(c.lookup(&a), Some(0));
        assert_eq!(c.lookup(&key(&paths[1])), None);
        assert!(c.stats().bytes <= 100);
        c.pressure();
        assert_eq!(c.stats().bytes, 40);
        assert_eq!(c.stats().entries, 1);
        assert!(texture_cost(u32::MAX, u32::MAX).is_none());
        assert_eq!(texture_cost(16, 16), Some(8192));
    }
    #[test]
    fn identity_rejects_rewrite_atomic_replace_delete_and_rename() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a.png");
        fs::write(&p, b"a").unwrap();
        let old = key(&p);
        fs::write(&p, b"longer").unwrap();
        assert_ne!(old, key(&p));
        let before = key(&p);
        let tmp = d.path().join("save");
        fs::write(&tmp, b"longer").unwrap();
        fs::rename(tmp, &p).unwrap();
        assert_ne!(before, key(&p));
        let renamed = d.path().join("b.png");
        fs::rename(&p, &renamed).unwrap();
        assert!(FileKey::read(&p).is_err());
        assert_ne!(old, key(&renamed));
        fs::remove_file(renamed).unwrap();
    }
    #[test]
    fn higher_priority_neighbor_is_not_evicted() {
        let d = tempfile::tempdir().unwrap();
        let mut c = Cache::new(80, 60);
        for n in ["active", "next", "prev"] {
            fs::write(d.path().join(n), b"x").unwrap();
        }
        let a = key(&d.path().join("active"));
        c.pin(Some(a.clone()));
        assert!(c.insert(a, 0, 40, &[]));
        assert!(c.insert(key(&d.path().join("next")), 1, 40, &[]));
        assert!(!c.insert(key(&d.path().join("prev")), 2, 40, &[d.path().join("next")]));
    }
}
