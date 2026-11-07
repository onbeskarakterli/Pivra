# Pivra v0.2.5 — Stable-release source

**Status: Stable-release source prepared for approval.**
Linux/GNOME, GPL-3.0-only. No release date, tag, immutable source archive/hash or
AUR publication is approved by this source preparation.

## Changes

- GPU-accelerated BT.1886 SDR display transfer on **stock GTK**, replacing the
  washed-out appearance of eligible BT.709 video without arbitrary contrast or
  saturation adjustments. The application-owned GStreamer GL path uses
  GLMemory → GLMemory shader processing and synchronized GDK texture presentation.
  End users do not need to build a custom GTK or enable a target environment flag.
- Automatic metadata-aware selection for a single 8-bit BT.709 SDR H.264 video
  stream, known full/limited range, at most 8,388,608 pixels and no subtitles.
  YUV matrix/range conversion stays in the GL pipeline before transfer correction;
  the resulting texture is presented as sRGB. HDR/PQ/HLG, sRGB/unknown metadata,
  other codecs and subtitle-containing video retain stock playback. Still-image
  decoding and its secure Glycin/Bubblewrap path are unchanged.
- Bounded asynchronous selection, deduplicated discovery and identity-aware
  metadata caching limit repeated probing. Generation checks prevent stale
  navigation results from reopening an old video. Direct playbin3 ownership,
  coalesced seeking and explicit teardown improve playback/window lifecycle safety.
- Existing audio, volume/mute, play/pause, seeking/frame stepping, loop, media
  navigation, fullscreen and viewing geometry remain available. An explicit
  `PIVRA_VIDEO_BACKEND=stock` runtime override preserves reversible stock playback.
- Distribution builds now require the GPU adapter's native APIs; missing headers
  cannot silently create a stock-only package. Actual ELF linkage, plugins and Arch
  ownership are checked. The Arch recipe retains `options=('!debug')`, producing
  only `pivra`, not a separate `pivra-debug` package.
- Both packaging frontends collect LGPL notices, modified adapter sources,
  corresponding application/locked crate sources and rebuilding/relinking guidance.
- The obsolete GNOME thumbnail test now matches the intentional valid-cache reuse
  policy after identical replacement/Trash restoration; changed metadata/content
  and in-flight identity checks have stronger regressions. Production freshness
  logic is unchanged. See [the cache decision](../THUMBNAIL_CACHE_FRESHNESS.md).

## Acceptance evidence

User-reported physical AMD Radeon RX 7900 XT acceptance: three BT.709 H.264 videos
at 720×1142, 1080×1712 and 1080×1920 selected the GPU path automatically; GLMemory
and synchronization were confirmed. Colors were visually MPV-like; smooth
playback, sound, controls and navigation passed without a custom GTK installation.
Native Arch GPU-enabled build/ELF/dependency/source-notice/DESTDIR staging and
preparation preflight also passed **before this version bump**. These are native
user reports, not tests independently executed by Codex or actual makepkg evidence.

Controlled checks and exact current candidate results are recorded in
[PIVRA_V0.2.5_RC_VALIDATION.md](PIVRA_V0.2.5_RC_VALIDATION.md). Software-GL tests do
not certify physical hardware acceleration or replace native visual acceptance.

## Known limitations

- Intel/NVIDIA are untested; the GPU adapter's Vulkan interop is not implemented.
- Native HDR visual preservation, hardware decoding and end-to-end zero-copy are
  unproven. The GPU correction is limited to the eligibility contract above.
- Missing GPU context/plugins/synchronization produces diagnostics; no CPU-only
  BT.1886 conversion is silently substituted. Stock playback remains reversible.
- Shared thumbnail validity is metadata-based, not a content-hash guarantee.
- The historical `Preload exceeds cache entry budget` observation remains
  documented in earlier audits; no fix is claimed by this version preparation.
- Experimental opt-in DNG/RAW is not promoted to default/release format support.
  Editing, slideshow, broader photo/HDR workflows and future roadmap items remain
  outside this release's implemented scope.
- Immutable source approval, actual Arch makepkg/.SRCINFO/package acceptance and
  publication authorization remain separate release gates.

## Source and publication evidence

README and these notes carry the prepared v0.2.5 stable-release wording. They do
not assert that a tag, GitHub Release or AUR update exists. The exact immutable
commit/archive digest is presented for explicit owner approval using the
[source review procedure](PIVRA_V0.2.5_SOURCE_APPROVAL.md). Archive/source approval,
actual makepkg acceptance and publication are separate decisions. No hash or
commit is predicted inside its own source tree.
