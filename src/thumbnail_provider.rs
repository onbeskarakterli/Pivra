//! FD-02 worker-side contract only: no provider discovery, routing or implementation.
//!
//! Implementations must preserve source/cache identity checks and secure pixel decode.
//! A candidate is untrusted cache data, never a decoded/renderable image.
use crate::image_cache::FileKey;
use gtk::{gio, prelude::*};
use std::time::Instant;

// Keep the existing GNOME/cache data model and its validation semantics.
pub use crate::thumbnail::{Candidate, Capability, Outcome as CacheOutcome};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderId {
    Gnome,
    Freedesktop,
}

/// Passive priority declaration for FD-06, not an active selection algorithm.
/// Bind one provider per application/session, never rediscover on each file.
/// A healthy, enabled GNOME provider (including CacheOnly) takes precedence.
/// File errors, cache misses and ineligibility must not cause a provider switch.
/// A provider-wide loss may prompt a later controlled session re-evaluation;
/// never run both generators for one request.
pub const PROVIDER_PRIORITY: [ProviderId; 2] = [ProviderId::Gnome, ProviderId::Freedesktop];

/// Application/session observation, independent of per-file eligibility/results.
/// CacheOnly is usable, not a failure; TemporarilyFailed can retain readable cache.
/// Updating this observation requires provider-wide evidence, not a file error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProviderStatus {
    pub id: ProviderId,
    pub enabled: bool,
    pub capability: Capability,
}

/// A file may be unsupported even when its provider is fully operational.
/// A failed eligibility check is an Error, never silently Ineligible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eligibility {
    Unknown,
    Eligible,
    Ineligible,
}

/// Borrowed source identity: no second URI/mtime/size model or implicit discovery.
/// Implementations derive the URI with GIO and revalidate FileKey before publishing.
/// Cancellation/deadline apply to all work and child cleanup, not just admission.
/// Success requires another check before publication; no new work after expiry.
pub struct Request<'a> {
    pub source: &'a FileKey,
    pub cancel: &'a gio::Cancellable,
    pub deadline: Instant,
}
impl Request<'_> {
    /// Caller-supplied monotonic time permits deterministic deadline tests.
    /// Explicit cancellation wins when both conditions are already true.
    pub fn check_control(&self, now: Instant) -> Result<(), Error> {
        if self.cancel.is_cancelled() {
            Err(Error::Cancelled)
        } else if now >= self.deadline {
            Err(Error::TimedOut)
        } else {
            Ok(())
        }
    }
}

/// Operation failure, not a command to change the session's chosen provider.
/// Cache lookup retains existing CacheOutcome distinctions (including legacy
/// Cancelled/SourceUnavailable/Invalid); the errors here also serve eligibility
/// and generation, which cannot return a cache hit/miss as their result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Cancelled,
    TimedOut,
    SourceUnavailable(String),
    /// FD-04: identity changed after the request snapshot.
    SourceChanged,
    /// FD-04: bounded source/frame/output admission failed.
    LimitExceeded(String),
    /// FD-04: fail-fast admission, no queued or duplicate decode.
    Busy,
    InvalidSource(String),
    Ineligible,
    Io(String),
    GenerationFailed(String),
    /// Provider-wide failure must be supported by independent runtime evidence.
    ProviderUnavailable,
    ProviderTemporarilyUnavailable,
}

/// Object-safe blocking worker API; never invoke these operations on the GTK thread.
/// No implementation for the existing GNOME provider is installed by FD-02.
///
/// `status` reads an existing session observation; it must not discover providers.
/// `lookup` may return a valid cache candidate regardless of generation capability.
/// New implementations return Err(Cancelled/TimedOut) for control failures; a future
/// legacy adapter must normalize CacheOutcome::Cancelled at this boundary.
/// Cache Missing/Stale/FailedCached are file/cache outcomes, not provider failures.
/// `generate` returns Error::Ineligible for an unsupported file, distinct from
/// GenerationFailed. It is a single-provider operation, never a fallback/retry loop.
/// It must check eligibility/control/identity and preserve the existing resource
/// bounds, secure decoder, atomic-cache and child-cleanup requirements.
/// Returned candidates still require secure Glycin decoding by their consumer.
pub trait ThumbnailProvider: Send + Sync {
    fn status(&self) -> ProviderStatus;
    fn eligibility(&self, request: &Request<'_>) -> Result<Eligibility, Error>;
    fn lookup(&self, request: &Request<'_>) -> Result<CacheOutcome, Error>;
    fn generate(&self, request: &Request<'_>) -> Result<Candidate, Error>;
}
