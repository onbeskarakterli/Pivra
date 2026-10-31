//! Session-bound routing. File failures never change the selected provider.
use crate::{
    freedesktop_generator::{self, GeneratedThumbnail},
    freedesktop_thumbnail::{CacheReader, SizeClass},
    freedesktop_writer::{CacheWriter, Publication},
    image_cache::FileKey,
    thumbnail::{self, Provider},
    thumbnail_provider::{CacheOutcome, Candidate, Error, ProviderId, Request},
};
use gtk::gio;
use gtk::gio::prelude::CancellableExt;
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
static WORKERS: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
pub struct Permit(PathBuf);
impl Permit {
    pub(crate) fn covers(&self, path: &Path) -> bool {
        self.0 == path
    }
    pub fn active_count() -> usize {
        WORKERS.lock().unwrap().len()
    }
    pub fn take(path: &std::path::Path) -> Option<Self> {
        let mut jobs = WORKERS.lock().unwrap();
        if jobs.len() >= 2 || jobs.iter().any(|p| p == path) {
            return None;
        }
        jobs.push(path.to_owned());
        Some(Self(path.to_owned()))
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        WORKERS.lock().unwrap().retain(|p| p != &self.0);
    }
}

#[cfg(test)]
static TEST_LOCK: Mutex<()> = Mutex::new(());

static SESSION: OnceLock<Manager> = OnceLock::new();
pub struct Response {
    resolved: Resolved,
    _permit: Permit,
    deadline: Instant,
}
impl Response {
    pub fn candidate(&self) -> Option<&Candidate> {
        match &self.resolved {
            Resolved::Cache(c) | Resolved::Published(c, _) => Some(c),
            _ => None,
        }
    }
    pub fn memory(&self) -> Option<&GeneratedThumbnail> {
        match &self.resolved {
            Resolved::Memory(t) | Resolved::Published(_, t) => Some(t),
            _ => None,
        }
    }
    pub fn deadline(&self) -> Instant {
        self.deadline
    }
    pub fn resolved(&self) -> &Resolved {
        &self.resolved
    }
}

pub struct CacheControl<'a>(crate::freedesktop_generator::DecodeControl<'a>);
impl<'a> CacheControl<'a> {
    pub fn complete(&mut self) {
        self.0.complete();
    }
    pub fn new(request: &Request<'a>, context: &gtk::glib::MainContext) -> Self {
        Self(crate::freedesktop_generator::DecodeControl::new(
            request, context,
        ))
    }
    pub fn cancellable(&self) -> &gio::Cancellable {
        &self.0.child
    }
}

pub enum Resolved {
    Cache(Candidate),
    Published(Candidate, GeneratedThumbnail),
    Memory(GeneratedThumbnail),
    Unavailable(CacheOutcome),
}
pub struct Manager {
    id: ProviderId,
    home: PathBuf,
    deferred: Mutex<Vec<FileKey>>,
}
impl Manager {
    pub fn session() -> &'static Self {
        SESSION.get_or_init(|| {
            let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
            let home = crate::freedesktop_thumbnail::cache_home(
                std::env::var_os("XDG_CACHE_HOME").as_deref(),
                &gtk::glib::home_dir(),
            )
            .unwrap_or_else(|_| gtk::glib::home_dir().join(".cache"));
            Self::new(
                desktop.split(':').any(|v| v.eq_ignore_ascii_case("GNOME")),
                home,
            )
        })
    }
    /// Explicit session snapshot; no per-file discovery or fallback debug environment key.
    pub fn new(gnome_enabled: bool, home: PathBuf) -> Self {
        Self::from_gnome_snapshot(
            gnome_enabled,
            crate::thumbnail_provider::Capability::CacheOnly,
            home,
        )
    }
    /// Temporary generation failure retains GNOME cache access until a new session.
    /// Only authoritative Unavailable/disabled session evidence selects fallback.
    pub fn from_gnome_snapshot(
        enabled: bool,
        capability: crate::thumbnail_provider::Capability,
        home: PathBuf,
    ) -> Self {
        Self {
            id: if enabled && capability != crate::thumbnail_provider::Capability::Unavailable {
                ProviderId::Gnome
            } else {
                ProviderId::Freedesktop
            },
            home,
            deferred: Mutex::new(Vec::new()),
        }
    }

    pub fn id(&self) -> ProviderId {
        self.id
    }
    pub fn resolve(&self, path: &Path, cancel: &gio::Cancellable) -> Result<Response, Error> {
        if cancel.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let deadline = Instant::now() + Duration::from_secs(90);
        crate::thumbnail_diagnostics::trace(
            "resolve-start",
            Some(path),
            format_args!("provider={:?} active={}", self.id, Permit::active_count()),
        );
        let permit = Permit::take(path).ok_or_else(|| {
            crate::thumbnail_diagnostics::trace(
                "admission-busy",
                Some(path),
                format_args!("active={}", Permit::active_count()),
            );
            Error::Busy
        })?;
        let resolved = self.resolve_admitted(path, cancel, &permit, deadline);
        crate::thumbnail_diagnostics::trace(
            "resolve-result",
            Some(path),
            match &resolved {
                Ok(Resolved::Cache(_)) => "cache".to_string(),
                Ok(Resolved::Memory(_)) => "memory".into(),
                Ok(Resolved::Published(..)) => "published".into(),
                Ok(Resolved::Unavailable(o)) => format!("unavailable: {o:?}"),
                Err(e) => format!("error: {e:?}"),
            },
        );
        let resolved = resolved?;
        Ok(Response {
            resolved,
            _permit: permit,
            deadline,
        })
    }
    fn resolve_admitted(
        &self,
        path: &Path,
        cancel: &gio::Cancellable,
        permit: &Permit,
        deadline: Instant,
    ) -> Result<Resolved, Error> {
        let key = FileKey::read(path).map_err(|e| Error::SourceUnavailable(e.to_string()))?;
        let request = Request {
            source: &key,
            cancel,
            deadline,
        };
        request.check_control(Instant::now())?;
        if self.id == ProviderId::Gnome {
            // Video previews are produced by the desktop thumbnailer, not by
            // Pivra's still-image decoder. GNOME may save them in any standard
            // thumbnail size class, so accept an already validated cache hit.
            if crate::discovery::candidate_kind(path) == Some(crate::media::MediaKind::Video) {
                if let Ok(reader) = CacheReader::new(self.home.clone()) {
                    for class in [
                        SizeClass::Normal,
                        SizeClass::Large,
                        SizeClass::XLarge,
                        SizeClass::XxLarge,
                    ] {
                        match reader.lookup(&request, class) {
                            Ok(CacheOutcome::Hit(candidate)) => {
                                return Ok(Resolved::Cache(candidate));
                            }
                            Err(Error::Cancelled) => return Err(Error::Cancelled),
                            Err(Error::TimedOut) => return Err(Error::TimedOut),
                            _ => {}
                        }
                    }
                }
            }
            // Preserve the existing GNOME GIO lookup for other cases.
            return match thumbnail::query_or_generate(&Provider::new(true, None), path, cancel) {
                CacheOutcome::Hit(c) => Ok(Resolved::Cache(c)),
                CacheOutcome::Cancelled => Err(Error::Cancelled),
                CacheOutcome::Busy => Err(Error::Busy),
                CacheOutcome::TimedOut => Err(Error::TimedOut),
                outcome => Ok(Resolved::Unavailable(outcome)),
            };
        }
        let reader = CacheReader::new(self.home.clone())?;
        let outcome = match reader.lookup(&request, SizeClass::Normal) {
            Ok(outcome) => outcome,
            Err(Error::Io(_)) => {
                CacheOutcome::Invalid("cache access unavailable; memory only".into())
            }
            Err(error) => return Err(error),
        };
        match outcome {
            CacheOutcome::Hit(c) => Ok(Resolved::Cache(c)),
            CacheOutcome::Missing | CacheOutcome::Stale | CacheOutcome::Invalid(_) => {
                let missing = matches!(outcome, CacheOutcome::Missing);
                let generated = self.produce(&request, permit)?;
                self.finish(generated, &request, &reader, missing)
            }
            outcome => Ok(Resolved::Unavailable(outcome)),
        }
    }
    pub fn recover_cache(
        &self,
        response: &Response,
        path: &Path,
        cancel: &gio::Cancellable,
    ) -> Result<GeneratedThumbnail, Error> {
        if self.id != ProviderId::Freedesktop {
            return Err(Error::GenerationFailed("GNOME cache failure".into()));
        }
        let candidate = response
            .candidate()
            .ok_or_else(|| Error::GenerationFailed("no cache recovery source".into()))?;
        let request = Request {
            source: &candidate.source,
            cancel,
            deadline: response.deadline,
        };
        current(&request)?;
        if candidate.source.path != path {
            return Err(Error::SourceChanged);
        }
        self.produce(&request, &response._permit)
    }
    fn is_deferred(&self, key: &FileKey, large: bool) -> bool {
        let mut deferred = self.deferred.lock().unwrap_or_else(|e| e.into_inner());
        if !large {
            deferred.clear();
            return false;
        }
        deferred.contains(key)
    }
    fn produce(&self, request: &Request<'_>, permit: &Permit) -> Result<GeneratedThumbnail, Error> {
        current(request)?;
        if self.is_deferred(request.source, freedesktop_generator::reservation_state().1) {
            return Err(Error::Busy);
        }
        let result = freedesktop_generator::generate_admitted(request, SizeClass::Normal, permit);
        if matches!(result, Err(Error::Busy)) && freedesktop_generator::reservation_state().1 {
            let mut deferred = self.deferred.lock().unwrap_or_else(|e| e.into_inner());
            if !deferred.contains(request.source) {
                if deferred.len() >= 128 {
                    deferred.remove(0);
                }
                deferred.push(request.source.clone());
            }
        }
        result
    }
    fn finish(
        &self,
        generated: GeneratedThumbnail,
        request: &Request<'_>,
        reader: &CacheReader,
        missing: bool,
    ) -> Result<Resolved, Error> {
        current(request)?;
        if generated.source != *request.source {
            return Err(Error::SourceChanged);
        }
        if missing {
            let result = CacheWriter::new(self.home.clone())
                .and_then(|writer| writer.write(&generated, request));
            current(request)?;
            if matches!(result, Ok(Publication::Existing(_))) {
                let _ = reader.lookup(request, generated.class);
            }
            // Existing is not trusted. Verify any published/existing record afresh.
            if matches!(result, Ok(Publication::Created(_))) {
                if let Ok(CacheOutcome::Hit(candidate)) = reader.lookup(request, generated.class) {
                    return Ok(Resolved::Published(candidate, generated));
                }
            }
        }
        current(request)?;
        Ok(Resolved::Memory(generated))
    }
}
fn current(r: &Request<'_>) -> Result<(), Error> {
    r.check_control(Instant::now())?;
    let key = FileKey::read(&r.source.path).map_err(|e| Error::SourceUnavailable(e.to_string()))?;
    if key != *r.source {
        Err(Error::SourceChanged)
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn routing_is_session_scoped_and_file_failures_do_not_switch() {
        let _lock = TEST_LOCK.lock().unwrap();
        let root = tempfile::tempdir().unwrap();
        for (enabled, id) in [(true, ProviderId::Gnome), (false, ProviderId::Freedesktop)] {
            let manager = Manager::new(enabled, root.path().join("cache"));
            assert_eq!(manager.id(), id);
            assert!(matches!(
                manager.resolve(&root.path().join("missing"), &gio::Cancellable::new()),
                Err(Error::SourceUnavailable(_))
            ));
            assert_eq!(manager.id(), id);
        }
    }
}

#[cfg(test)]
#[path = "../tests/freedesktop/manager.rs"]
mod integration_tests;
