# Pivra v0.2.6 — development release candidate

> Historical unreleased development candidate record for v0.2.6. The statuses and checks below describe that preparation period, not the current release. **Current stable: v0.2.7**, published on GitHub and AUR; see the [v0.2.7 release notes](PIVRA_V0.2.7_RELEASE_NOTES.md).

**Status: Development candidate; isolated GTK native acceptance PASS. Stock GTK mixed-playback limitation and release risk decisions remain open. Not published.**

At the time of this candidate, the stable release was v0.2.5. This candidate was limited to
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

## Native acceptance and known GTK limitation — 2026-11-23

The owner reports physical Arch/GNOME / RX 7900 XT results on the GTK 4.24.1 /
Mesa 26.3.1-arch1.2 environment: original stock ownership Wayland 8/9,
isolated ownership-patched Wayland 9/9, X11 9/9. Normal production Wayland GPU
video, audio, colors and seeking PASS. These are **user-reported native results**,
not independently run by Codex. The newest run's raw logs/source/library hashes
were not supplied; the existing A/B control includes the common EGL config patch.

Stock MediaFile retirement can call eglTerminate on the EGLDisplay used by
another live GPU window. Returning from stock-routed media to eligible BT.709
video can then fail context creation and trigger stock fallback. Other windows
may also be affected; fallback is not a guarantee against every invalid-context
failure. This is a GTK stock-sink ownership boundary. The equivalent fix already exists
upstream at `95b7ec19b01c460c46d58ec52cd0c4eb2d730395` in inspected main/gtk-4-24
refs. A fixed Arch package/released-version requirement has not been verified.
The successful isolated local fix is preserved separately and is **not installed
or bundled**.
The earlier EGL config-selection correction remains a separate patch.
[Existing upstream fix and reproducible report](../upstream/GTK_4.24.1_WAYLAND_EGL_OWNERSHIP.md).

A per-process workaround, where XWayland/X11 is installed, is:

```sh
GDK_BACKEND=x11 GSK_RENDERER=gl pivra /path/to/video.mp4
```

The native X11 9/9 result supports this workaround on the tested AMD system;
it is not a global backend setting or cross-GPU guarantee. No CPU transfer
fallback, shader adjustment, leaked player reference or altered stock/HDR route
is introduced. A previous software candidate SIGSEGV during mixed retirement
remains **OPEN / UNCLASSIFIED** despite later debugger/native passes; see the
[separate investigation](../performance/V026_CANDIDATE_SIGSEGV.md).

**Unrestricted release recommendation: NO-GO pending owner risk decisions and
remaining package/source gates.** The stock GTK limitation could be an explicitly
accepted compatibility limitation with the tested X11 workaround, but the owner
has not accepted it or the separate SIGSEGV risk for v0.2.6. Patched-GTK 9/9 is
not stock-GTK 9/9. [Decision/checklist](PIVRA_V0.2.6_RC_VALIDATION.md).

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
