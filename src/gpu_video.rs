//! File-scoped metadata-gated GPU selection; GTK objects stay on the UI thread.
use gtk::{gio, glib, prelude::*};
use std::{
    collections::{HashSet, VecDeque},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Stock,
    // Preserve the selector API in intentionally stock-only native builds.
    #[cfg_attr(not(pivra_gpu_backend), allow(dead_code))]
    Gpu,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Identity {
    path: PathBuf,
    stamp: [u64; 7],
}
fn identity(path: &Path) -> Option<Identity> {
    let path = path.canonicalize().ok()?;
    let m = path.metadata().ok()?;
    m.is_file().then_some(Identity {
        path,
        stamp: [
            m.dev(),
            m.ino(),
            m.len(),
            m.mtime() as u64,
            m.mtime_nsec() as u64,
            m.ctime() as u64,
            m.ctime_nsec() as u64,
        ],
    })
}
#[derive(Default)]
struct Cache {
    entries: VecDeque<(Identity, Backend)>,
    pending: HashSet<Identity>,
}
fn cache() -> &'static Mutex<Cache> {
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}
static ACTIVE: AtomicUsize = AtomicUsize::new(0);
static PROBES: AtomicUsize = AtomicUsize::new(0);
static STOCK: AtomicUsize = AtomicUsize::new(0);
static GPU: AtomicUsize = AtomicUsize::new(0);
#[cfg(pivra_gpu_backend)]
unsafe extern "C" {
    fn pivra_gpu_backend_type() -> glib::ffi::GType;
    fn pivra_gpu_eligible(uri: *const std::ffi::c_char) -> std::ffi::c_int;
    fn pivra_gpu_live_filters() -> std::ffi::c_int;
}
fn discover(path: &Path) -> Backend {
    let _measurement = pivra::performance::Span::new("video_metadata_discovery");
    PROBES.fetch_add(1, Ordering::SeqCst);
    #[cfg(pivra_gpu_backend)]
    {
        let uri = std::ffi::CString::new(gio::File::for_path(path).uri().as_str()).unwrap();
        // The native discoverer owns its resources and has a two-second deadline.
        if unsafe { pivra_gpu_eligible(uri.as_ptr()) } == 1 {
            return Backend::Gpu;
        }
    }
    #[cfg(not(pivra_gpu_backend))]
    let _ = path;
    Backend::Stock
}
pub async fn choose(path: PathBuf, current: impl Fn() -> bool) -> Option<Backend> {
    if !cfg!(pivra_gpu_backend) || std::env::var("PIVRA_VIDEO_BACKEND").as_deref() == Ok("stock") {
        if !cfg!(pivra_gpu_backend) {
            eprintln!("Pivra GPU backend unavailable at build time; stock video retained");
        }
        return current().then_some(Backend::Stock);
    }
    let Some(key) = identity(&path) else {
        return current().then_some(Backend::Stock);
    };
    loop {
        if !current() {
            return None;
        }
        if identity(&path).as_ref() != Some(&key) {
            return Some(Backend::Stock);
        }
        {
            let mut c = cache().lock().unwrap();
            if let Some(i) = c.entries.iter().position(|(k, _)| k == &key) {
                let entry = c.entries.remove(i).unwrap();
                let result = entry.1;
                c.entries.push_back(entry);
                return Some(result);
            }
            if !c.pending.contains(&key) && ACTIVE.load(Ordering::SeqCst) < 2 {
                ACTIVE.fetch_add(1, Ordering::SeqCst);
                c.pending.insert(key.clone());
                break;
            }
        }
        glib::timeout_future(Duration::from_millis(10)).await;
    }
    let result = gio::spawn_blocking(move || {
        let backend = discover(&path);
        let unchanged = identity(&path).as_ref() == Some(&key);
        let mut c = cache().lock().unwrap();
        c.pending.remove(&key);
        if unchanged {
            c.entries.push_back((key, backend));
            while c.entries.len() > 64 {
                c.entries.pop_front();
            }
        }
        ACTIVE.fetch_sub(1, Ordering::SeqCst);
        if unchanged { backend } else { Backend::Stock }
    })
    .await
    .unwrap_or(Backend::Stock);
    if std::env::var_os("PIVRA_TRACE_VIDEO").is_some() {
        let s = stats();
        eprintln!(
            "PIVRA_VIDEO_SELECTION {result:?} workers={} pending={} cached={} filters={} probes={} stock={} gpu={}",
            s.workers, s.pending, s.cached, s.filters, s.probes, s.stock, s.gpu
        );
    }
    current().then_some(result)
}
pub struct Stats {
    pub workers: usize,
    pub pending: usize,
    pub cached: usize,
    pub filters: usize,
    pub probes: usize,
    pub stock: usize,
    pub gpu: usize,
}
pub fn stats() -> Stats {
    let c = cache().lock().unwrap();
    #[cfg(pivra_gpu_backend)]
    let filters = unsafe { pivra_gpu_live_filters() }.max(0) as usize;
    #[cfg(not(pivra_gpu_backend))]
    let filters = 0;
    pivra::performance::emit("video_live_filters", "observed_count", 0, filters as u64);
    Stats {
        workers: ACTIVE.load(Ordering::SeqCst),
        pending: c.pending.len(),
        cached: c.entries.len(),
        filters,
        probes: PROBES.load(Ordering::SeqCst),
        stock: STOCK.load(Ordering::SeqCst),
        gpu: GPU.load(Ordering::SeqCst),
    }
}
pub fn is_owned(media: &gtk::MediaFile) -> bool {
    media.type_().name() == "PivraGpuMediaFile"
}
fn stock(path: &Path) -> gtk::MediaFile {
    STOCK.fetch_add(1, Ordering::SeqCst);
    let media = gtk::MediaFile::for_filename(path);
    if std::env::var_os("PIVRA_TRACE_VIDEO").is_some() {
        eprintln!(
            "PIVRA_GPU_STOCK type={} file={}",
            media.type_(),
            path.display()
        );
    }
    media
}
pub fn open_selected(path: &Path, backend: Backend) -> Result<gtk::MediaFile, String> {
    if backend == Backend::Stock {
        return Ok(stock(path));
    }
    let renderer = if gtk::minor_version() < 20 {
        "ngl"
    } else {
        "gl"
    };
    if std::env::var("GSK_RENDERER").as_deref() != Ok(renderer) {
        eprintln!(
            "Pivra GPU correction unavailable with selected renderer; stock retained (requires {renderer})"
        );
        return Ok(stock(path));
    }
    #[cfg(pivra_gpu_backend)]
    {
        let type_: glib::Type = unsafe { glib::translate::from_glib(pivra_gpu_backend_type()) };
        if !type_.is_a(gtk::MediaFile::static_type()) {
            return Err("GPU media API unavailable".into());
        }
        let media = glib::Object::with_type(type_)
            .downcast::<gtk::MediaFile>()
            .map_err(|_| "GPU construction failed")?;
        debug_assert!(is_owned(&media));
        GPU.fetch_add(1, Ordering::SeqCst);
        media.set_filename(Some(path));
        if let Some(error) = media.error() {
            return Err(error.to_string());
        }
        eprintln!(
            "PIVRA_GPU_BACKEND automatic type={type_} file={}",
            path.display()
        );
        Ok(media)
    }
    #[cfg(not(pivra_gpu_backend))]
    Err("GPU adapter not built".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_tracks_modification_replacement_and_deletion() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("video");
        std::fs::write(&path, b"first").unwrap();
        let first = identity(&path).unwrap();
        std::fs::write(&path, b"changed size").unwrap();
        assert_ne!(identity(&path).unwrap(), first);
        std::fs::remove_file(&path).unwrap();
        assert!(identity(&path).is_none());
        std::fs::write(&path, b"first").unwrap();
        assert_ne!(identity(&path).unwrap(), first);
    }
    #[test]
    fn cancelled_and_deleted_selection_does_not_probe() {
        glib::MainContext::new().block_on(async {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("missing");
            assert_eq!(choose(path.clone(), || false).await, None);
            assert_eq!(choose(path, || true).await, Some(Backend::Stock));
        });
    }
    #[cfg(pivra_gpu_backend)]
    #[test]
    fn discovery_cache_is_bounded_and_reuses_identity() {
        glib::MainContext::new().block_on(async {
            let dir = tempfile::tempdir().unwrap();
            let first = dir.path().join("0.mp4");
            std::fs::write(&first, b"invalid").unwrap();
            assert_eq!(choose(first.clone(), || true).await, Some(Backend::Stock));
            let before = stats().probes;
            for i in 1..67 {
                assert_eq!(choose(first.clone(), || true).await, Some(Backend::Stock));
                let path = dir.path().join(format!("{i}.mp4"));
                std::fs::write(&path, b"invalid").unwrap();
                assert_eq!(choose(path, || true).await, Some(Backend::Stock));
            }
            assert_eq!(stats().probes, before + 66);
            assert!(stats().cached <= 64);
            assert_eq!(stats().pending, 0);
            assert_eq!(stats().workers, 0);
        });
    }
}
