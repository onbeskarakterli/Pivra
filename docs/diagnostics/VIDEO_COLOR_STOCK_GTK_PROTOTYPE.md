# Issue #9 — application-owned stock-GTK GPU prototype

2026-11-03; source base `167a27479fbbb3e61268c5b375f70dc0e6f36a13`.
**Implemented, opt-in GL prototype; not a production default or issue closure.**
No GTK replacement, version, dependency lockfile, packaging or security-policy
change. This supersedes the architecture-only checkpoint in the GPU candidate
record: there is now executable application integration.

## Implemented path

`src/gpu_video.rs` constructs a separate public `GtkMediaFile` subclass for one
explicit canonical target path. That also satisfies `GtkMediaStream`, so existing
Pivra controls, ActualPaintable zoom/geometry, resume timestamps and per-window
media ownership remain in use. All other files and ordinary launches retain
stock GTK. Initialization pins the stock factory before registering the opt-in
subclass at low priority; no global GTK_MEDIA override is used. Explicit failure
of the selected prototype is shown as an error, not silently substituted with a
CPU color path. Removing the two prototype environment variables reverses it.

`tests/video_gpu/backend/build.py` builds a small application-owned adapter from
hash-pinned **unmodified GTK 4.18.6 standalone backend sources** and local changes.
This is reuse of the public MediaFile subclass/sink implementation, **not a GTK
build/patch**. GTypes and ELF symbols are renamed. It links public GTK/GStreamer
APIs, removes the private GDK constants header, and retains upstream LGPL notices
in the fetched sources. No CPU-reference patch is applied. The existing source
hash/renaming helpers are reused; system libraries are not installed/overwritten.

Pipeline inside that adapter:

```text
GstPlayer (existing clock/audio/seek/stream events)
  → source metadata gate
  → glupload → glcolorconvert
  → explicit BT709 full-range RGB GLMemory caps
  → application GstGLFilter: BT1886 EOTF then sRGB OETF
  → GL-only GDK texture sink → existing Pivra paintable/controls
```

The source gate parses an explicit colorimetry tag, including on DMA_DRM caps;
unknown, absent, HDR/other primaries and invalid tuples are rejected. The runner
also checks source ffprobe tags and restricts the experiment to a single 8-bit
BT709 SDR video stream, with full/limited BT709 YUV. Negotiated tags may still be
inferred upstream; their origin is not certified by GStreamer. This is a narrow
prototype, not general HDR/media support. Non-target navigation remains stock.

`glcolorconvert` performs range/matrix conversion on GPU exactly once. Explicit
post-conversion RGB caps avoid the metadata loss found in initial negotiation.
The shader does ideal-black BT1886 and sRGB encoding; the final texture is tagged
sRGB, avoiding another BT709 inverse transfer in GSK. No contrast/saturation
adjustment or reinterpretation of photos/global CICP states is used.

GL context sharing, GstGLSyncMeta fence publication and frame-retention ownership
are carried over from the reference sink. Missing sync metadata or failed GL maps
are errors. The sink's **CPU mapping/MemoryTexture fallback is removed**, not
merely deprioritized. Filter work runs on GStreamer's GL context; shader resources
are released on GL stop. At most two queued UI texture invocations are retained;
the downstream pool proposes 2–8 buffers. RGBA output is limited to 32 MiB/frame.
These are local bounds, not proof of a total driver/decoder memory bound.

The prototype explicitly requires stock GSK GL (`ngl` before GTK 4.20, `gl`
thereafter). Vulkan presentation is rejected: a GL texture is not assumed to
cross into GSK Vulkan without readback. Both legacy/compatibility GLES and desktop
OpenGL-core shader variants are implemented. No hardware-driver compatibility
claim follows just from compiling these variants.

## One native Arch command

From the repository, with stock GTK and native development/runtime dependencies:

```bash
python3 tests/video_gpu/backend/run.py /absolute/path/to/original-bt709.mp4
```

Requires Cargo, Python, ffprobe, C compiler and pkg-config modules `gtk4`,
`gstreamer-player-1.0`, `gstreamer-gl-1.0`, `gstreamer-allocators-1.0`; runtime
`glupload`, `glcolorconvert` and media codecs must exist. Exact Arch package
ownership is not newly verified here. No GTK compilation or system install is
performed. The runner builds only the adapter and ordinary Pivra, retains a private
workspace, refuses a GTK library containing our earlier patch marker, and records
GTK linkage/hash/version, ffprobe metadata and playback traces. It chooses the
correct stock GL renderer name; it does not force software rendering.

Use `--check` for a bounded real-frame/controls probe. Test-only snapshot readback
is explicit; normal playback has no such operation. `--bars` and `--ramp` are
assertions for the known generated fixtures, not arbitrary user clips.
`--fallback-file /path/to/non-target.mp4` with `--check` verifies same-process stock
routing. Close normal Pivra to finish and retain the log; check sound, seek,
play/pause, repeat, navigation, zoom and MPV visual appearance on native hardware.

## Acceptance and remaining limits

The patched-GTK AMD GL/Vulkan/visual acceptance belongs to the earlier experiment,
not this new stock-GTK backend. A new native comparison is required. Current
Codex runs use **llvmpipe**, not physical hardware. Real decoded H264 frames,
texture presentation, independent pixel references and control/cleanup events
are testable here; hardware decoding, zero-copy, audible audio, physical
frame-step/keyboard/mouse interaction and Intel/NVIDIA behavior are not certified.

This is a file-scoped prototype: automatic eligible-video routing across the whole
catalog and streaming metadata changes/fallback handover are not implemented.
The runner rejects unknown/HDR/non-target media before opting in; opening other
files normally continues through stock GTK. All-format passthrough, high bit
depth, alpha video and native HDR output remain outside the prototype.
GLMemory upload from software-decoded YUV is possible; that is an upload, not
GPU-to-CPU color conversion or a zero-copy claim. Native DMA-BUF import depends
on glupload/driver/modifier support and must be observed separately.

Production blockers: new-backend native MPV visual acceptance; physical AMD/Intel/
NVIDIA controls/resource testing; robust all-catalog metadata routing; GPU-only
Vulkan interop/export/synchronization; packaged adapter/dependency/license review
(not authorized in this task); stress/capture/lifetime acceptance under GPU load.
The baseline-reproducible GTK/Mesa download SIGBUS remains open, with both native
logs/provenance/build options preserved by the owner. No whole-suite rerun needed.

## Executed development checks

- Strict C adapter and actual metadata-policy unit build: `-Wall -Wextra -Werror`
  (only unused parameters in upstream callbacks exempted). Positive BT709, unknown,
  sRGB/PQ/HLG/BT2020/limited-RGB and missing-tag rejection checks PASS.
- Stock GTK **4.18.6**, GStreamer **1.26.2**, Mesa **25.0.7** llvmpipe: real H264
  limited-range 192×96 bars through X11 and headless Sway/Wayland PASS. GLMemory
  → shader → synchronized sRGB GdkGLTexture. Pixels `[0,123,255]` versus independent
  `[0,124,255]` reference, fixed tolerance 2 LSB. Wrapper difference 0.
- Desktop GL **4.5 core** and GLES **3.2** both executed. Full-range 256×64 ramp,
  all **256 levels**, independent ideal-black BT1886/sRGB reference: PASS within
  2 LSB. Test-only snapshot/download is explicit, not in normal playback.
- Prepared controls, play/pause, loop flag, mute/volume state, actual seek
  completion at 1 second and GL-filter stop/clear: PASS. This is not audible
  audio, natural EOS repeat, exact frame-step or physical keyboard acceptance.
- Same-process non-target routing: PASS. The isolated SDK's default GTK backend
  is **GtkNoMediaFile**, so non-target native **decode is NOT RUN**, not PASS.
  This is an existing SDK limitation: the custom subclass works independently
  of the missing stock media plugin. Normal native Arch GTK must validate it.
- Metadata preflight Python tests and missing-field/other-space negatives are
  controlled tests, not HDR display acceptance. Synthetic/real-image existing
  test categories retain their original limitations; no secure image decoder
  assertion is replaced.

Initial rejected attempts exposed absent RGB colorimetry, an invalid caps range
string and an unsupported lossless High-4:4:4 H264 fixture. These were corrected,
not counted as PASS or hidden behind retries. The committed fixture generator
uses supported 8-bit High-profile encoding. To reproduce numeric gates:

```bash
python3 tests/video_gpu/backend/fixtures.py /new/private/fixtures
python3 tests/video_gpu/backend/run.py /new/private/fixtures/bars.mp4 --check --bars
python3 tests/video_gpu/backend/run.py /new/private/fixtures/ramp.mp4 --check --ramp
```

Each command retains its source/adapter/metadata/provenance/playback workspace.
No tracked binary or source media is modified.

The actual application-window close gate initially exposed missing explicit
prototype retirement: the window/process exited, but no GL-filter stop was
observed. The fix is scoped to `PivraGpuMediaFile`: detach/clear it on window
close and clear after saving position on navigation. Native stock MediaFile keeps
its existing asynchronous drop policy. The reproducible X11/Xvfb test is
`tests/video_gpu/backend/window_smoke.py` (normal WM close, not SIGTERM). It requires
an actual draw, exit 0 and filter-stop trace; no assertion was removed to pass it.

Final verification: **206 Rust tests**, **32 GPU-tooling Python tests** and
**13 existing CPU-reference harness Python tests** PASS. Rust fmt/check, strict
all-target Clippy with zero warnings, and locked release build PASS. Strict
native C adapter/policy build PASS. The actual application-window WM-close gate
now PASS with filter-stop observed; the original failure remains described above.
The close/navigation paths release RefCell guards before emitting stream
notifications/clearing the prototype, allowing reentrant UI callbacks safely.

No physical GPU run is claimed by Codex: X11/core GL and Wayland/GLES evidence
here is software-driver execution. Native AMD acceptance from the patched GTK
experiment does not automatically accept this new application-owned backend.
