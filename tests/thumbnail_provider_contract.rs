//! Contract-only doubles: no GNOME generation, decoder or provider manager.
use gtk::{gio, prelude::*};
use pivra::{
    image_cache::FileKey,
    thumbnail_provider::{
        CacheOutcome, Candidate, Capability, Eligibility, Error, PROVIDER_PRIORITY, ProviderId,
        ProviderStatus, Request, ThumbnailProvider,
    },
};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Stub {
    status: ProviderStatus,
    eligibility: Result<Eligibility, Error>,
    candidate: Option<Candidate>,
    calls: AtomicUsize,
}
impl Stub {
    fn new(id: ProviderId, capability: Capability) -> Self {
        Self {
            status: ProviderStatus {
                id,
                enabled: true,
                capability,
            },
            eligibility: Ok(Eligibility::Eligible),
            candidate: None,
            calls: AtomicUsize::new(0),
        }
    }
}
impl ThumbnailProvider for Stub {
    fn status(&self) -> ProviderStatus {
        self.status
    }
    fn eligibility(&self, request: &Request<'_>) -> Result<Eligibility, Error> {
        request.check_control(Instant::now())?;
        self.eligibility.clone()
    }
    fn lookup(&self, request: &Request<'_>) -> Result<CacheOutcome, Error> {
        request.check_control(Instant::now())?;
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(match &self.candidate {
            Some(candidate) => CacheOutcome::Hit(Candidate {
                uri: candidate.uri.clone(),
                source: candidate.source.clone(),
                thumbnail: candidate.thumbnail.clone(),
                png: candidate.png.clone(),
            }),
            None => CacheOutcome::Missing,
        })
    }
    fn generate(&self, request: &Request<'_>) -> Result<Candidate, Error> {
        request.check_control(Instant::now())?;
        if self.eligibility(request)? == Eligibility::Ineligible {
            return Err(Error::Ineligible);
        }
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(Error::GenerationFailed(
            "controlled failure; no decoder".into(),
        ))
    }
}

struct Fixture {
    _root: tempfile::TempDir,
    source: FileKey,
    cancel: gio::Cancellable,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("source.png");
        std::fs::write(&path, include_bytes!("fixtures/images/wide.png")).unwrap();
        Self {
            source: FileKey::read(&path).unwrap(),
            _root: root,
            cancel: gio::Cancellable::new(),
        }
    }
    fn request(&self) -> Request<'_> {
        Request {
            source: &self.source,
            cancel: &self.cancel,
            deadline: Instant::now() + Duration::from_secs(60),
        }
    }
}

#[test]
fn two_distinct_provider_ids_have_explicit_gnome_first_priority() {
    assert_ne!(ProviderId::Gnome, ProviderId::Freedesktop);
    assert_eq!(
        PROVIDER_PRIORITY,
        [ProviderId::Gnome, ProviderId::Freedesktop]
    );
}

#[test]
fn availability_and_file_eligibility_are_independent() {
    let f = Fixture::new();
    let mut provider = Stub::new(ProviderId::Gnome, Capability::CacheAndGenerate);
    let before = provider.status();
    for eligibility in [
        Eligibility::Unknown,
        Eligibility::Eligible,
        Eligibility::Ineligible,
    ] {
        provider.eligibility = Ok(eligibility);
        assert_eq!(provider.eligibility(&f.request()), Ok(eligibility));
        assert_eq!(provider.status(), before);
    }
    provider.eligibility = Err(Error::InvalidSource("broken JPEG".into()));
    assert!(matches!(
        provider.eligibility(&f.request()),
        Err(Error::InvalidSource(_))
    ));
    assert_eq!(provider.status(), before);
}

#[test]
fn ineligible_and_failed_checks_are_distinct_without_changing_session_status() {
    let f = Fixture::new();
    let mut provider = Stub::new(ProviderId::Gnome, Capability::CacheAndGenerate);
    provider.eligibility = Ok(Eligibility::Ineligible);
    assert_eq!(
        provider.eligibility(&f.request()),
        Ok(Eligibility::Ineligible)
    );
    assert!(matches!(
        provider.generate(&f.request()),
        Err(Error::Ineligible)
    ));
    assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
    provider.eligibility = Err(Error::Io("metadata inaccessible".into()));
    assert!(matches!(
        provider.eligibility(&f.request()),
        Err(Error::Io(_))
    ));
    assert_eq!(provider.status().capability, Capability::CacheAndGenerate);
}

#[test]
fn cache_miss_and_generation_failure_leave_provider_observation_unchanged() {
    let f = Fixture::new();
    let provider = Stub::new(ProviderId::Gnome, Capability::CacheOnly);
    let before = provider.status();
    assert!(matches!(
        provider.lookup(&f.request()),
        Ok(CacheOutcome::Missing)
    ));
    assert!(matches!(
        provider.generate(&f.request()),
        Err(Error::GenerationFailed(_))
    ));
    assert_eq!(provider.status(), before);
}

#[test]
fn corrupt_source_missing_source_and_provider_loss_are_distinct() {
    let errors = [
        Error::InvalidSource("decode rejected".into()),
        Error::SourceUnavailable("source deleted".into()),
        Error::ProviderUnavailable,
        Error::ProviderTemporarilyUnavailable,
    ];
    for (index, error) in errors.iter().enumerate() {
        for other in &errors[index + 1..] {
            assert_ne!(error, other);
        }
    }
    // Keep existing cache-layer errors as-is rather than collapsing them.
    assert!(matches!(
        CacheOutcome::Invalid("bad cache".into()),
        CacheOutcome::Invalid(_)
    ));
    assert!(matches!(
        CacheOutcome::SourceUnavailable("deleted".into()),
        CacheOutcome::SourceUnavailable(_)
    ));
}

#[test]
fn cancelled_and_expired_requests_do_not_start_lookup_or_generation() {
    let f = Fixture::new();
    let provider = Stub::new(ProviderId::Gnome, Capability::CacheAndGenerate);
    let mut request = f.request();
    request.deadline = Instant::now();
    assert!(matches!(provider.lookup(&request), Err(Error::TimedOut)));
    assert!(matches!(provider.generate(&request), Err(Error::TimedOut)));
    assert_eq!(provider.eligibility(&request), Err(Error::TimedOut));
    f.cancel.cancel();
    assert!(matches!(provider.lookup(&request), Err(Error::Cancelled)));
    assert!(matches!(provider.generate(&request), Err(Error::Cancelled)));
    assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn monotonic_deadline_boundary_and_late_cancellation_are_deterministic() {
    let f = Fixture::new();
    let request = f.request();
    let before = request.deadline - Duration::from_nanos(1);
    assert_eq!(request.check_control(before), Ok(()));
    assert_eq!(
        request.check_control(request.deadline),
        Err(Error::TimedOut)
    );
    assert_eq!(
        request.check_control(request.deadline + Duration::from_nanos(1)),
        Err(Error::TimedOut)
    );
    f.cancel.cancel();
    assert_eq!(request.check_control(before), Err(Error::Cancelled));
    assert_eq!(
        request.check_control(request.deadline),
        Err(Error::Cancelled)
    );
}

#[test]
fn cache_candidate_reuses_identity_and_remains_available_without_generation() {
    let f = Fixture::new();
    let request = f.request();
    let cache_path = f.source.path.with_file_name("cache.png");
    let png = include_bytes!("fixtures/images/wide.png").to_vec();
    for capability in [Capability::CacheOnly, Capability::TemporarilyFailed] {
        std::fs::write(&cache_path, &png).unwrap();
        let cache_key = FileKey::read(&cache_path).unwrap();
        let mut provider = Stub::new(ProviderId::Gnome, capability);
        provider.candidate = Some(Candidate {
            uri: gio::File::for_path(&f.source.path).uri().to_string(),
            source: f.source.clone(),
            thumbnail: cache_key.clone(),
            png: png.clone(),
        });
        let CacheOutcome::Hit(candidate) = provider.lookup(&request).unwrap() else {
            panic!("cached candidate is independent of generator status");
        };
        assert_eq!(candidate.source, f.source);
        assert_eq!(candidate.thumbnail, cache_key);
        assert_eq!(candidate.png, png);
        assert!(candidate.still_current(&f.cancel));
        // Original Candidate validation, not a second identity implementation.
        std::fs::remove_file(&cache_path).unwrap();
        assert!(!candidate.still_current(&f.cancel));
        assert_eq!(provider.status().capability, capability);
    }
}

#[test]
fn two_provider_identities_fit_one_object_safe_worker_contract() {
    let f = Fixture::new();
    let providers: Vec<Box<dyn ThumbnailProvider>> = vec![
        Box::new(Stub::new(ProviderId::Gnome, Capability::CacheOnly)),
        Box::new(Stub::new(
            ProviderId::Freedesktop,
            Capability::CacheAndGenerate,
        )),
    ];
    for (provider, id) in providers.iter().zip(PROVIDER_PRIORITY) {
        assert_eq!(provider.status().id, id);
        assert!(matches!(
            provider.lookup(&f.request()),
            Ok(CacheOutcome::Missing)
        ));
    }
}
