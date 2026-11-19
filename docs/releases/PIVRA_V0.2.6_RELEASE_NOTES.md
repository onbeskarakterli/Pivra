# Pivra v0.2.6 — development release candidate

**Status: Development candidate; native AMD lifecycle acceptance BLOCKED. Not published.**

The current stable release remains v0.2.5. This candidate is limited to
performance, RAM ownership, GPU resource lifetime and regression coverage.

- Path-specific folder notifications keep unrelated decoded image cache entries,
  LRU/pin state and unaffected background work. Changed/deleted paths still
  invalidate identity, and a changed current file invalidates outstanding results.
- Already packed, correctly sized RGBA8 generated thumbnails avoid a redundant
  normalization/downloader/texture-object round trip. Alpha, pixel values,
  resizing quality and the existing texture budgets remain unchanged.
- Video reset destroys its queued presentation sources immediately, releasing
  retained texture/buffer references without waiting for another main-loop turn.
  Two queued frames remain the hard bound; epoch checks still reject stale work.
- The catalog regression targets the named thumbnail motion controller rather
  than whichever of several motion controllers GTK enumerates first. Input
  behavior and the Capture assertion are preserved.

No color shader, CPU video conversion, preload budget, source-media content,
thumbnail-provider ordering or secure Glycin/Bwrap policy change is included.
JPEG/PNG/GIF/video navigation, controls, zoom/pan and the 64 MP foreground policy
remain in scope; this is not a new feature release.

Native acceptance runner follow-up (2026-11-20) separates real secure-thumbnail
repair from controlled missing/corrupt-cache failure injection. A successful real
repair is no longer tested using the synthetic-mode failed-cell expectation.
This is a test correction, not a production provider or GPU fix; see
[latest acceptance evidence](../performance/V026_NATIVE_ACCEPTANCE_FIXES.md).

## Blocking limitations

Native user logs on GTK 4.24.1 / AMD demonstrate stock MediaFile retirement
calling eglTerminate on an EGLDisplay shared with the other live GPU window.
Subsequent context creation fails and the mixed probe exits 101. This is a
remaining GTK/GStreamer ownership blocker, not a BT.1886 shader failure.
The stock media pipeline is private; no global hooks, leaked player references,
system GTK replacement or altered stock/HDR routing are used to work around it.
An upstream/distribution ownership correction needs native validation. All nine
AMD lifecycle gates must pass before release approval.

Controlled llvmpipe checks are not AMD acceptance, VRAM measurements, or proof
of hardware decoding/end-to-end zero-copy. Intel/NVIDIA, Vulkan interop, native
HDR visual preservation, sustained hardware resource stability and new native
Arch package validation remain open. The older unreproduced `Preload exceeds
cache entry budget` observation is not declared fixed by these changes; its
v0.1 risk acceptance is not automatically approval for this release.

See [sprint evidence](../performance/V026_FINAL_SPRINT.md) and
[candidate gates](PIVRA_V0.2.6_RC_VALIDATION.md). GPL-3.0-only and existing LGPL
adapter source/relink obligations remain unchanged. No source checksum, release
tag, main merge or publication is approved by this document.
