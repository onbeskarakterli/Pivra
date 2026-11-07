# Issue #9 — normal stock-GTK GPU integration candidate

Base: `785a4f3077309a0ba6460ededa33488dc976911a` (`dev`).
Status: implementation candidate, **native integration acceptance pending**.
No version, release, AUR, packaging or main changes.

## Evidence boundaries

The user accepted the opt-in prototype at the base commit on physical AMD
Radeon RX 7900 XT / GLES 3.2: 430 frames, GLMemory → GLMemory, retained GL sync,
MPV-like visual appearance, smooth playback, audio/controls and clean shutdown.
These are **user-reported native results**, not Codex tests of this new controller.

Historical patched GTK GL/Vulkan experiments remain separate. The exact GTK
4.24.1 `/colorstate/convert-dmabuf/srgb/srgb` SIGBUS reproduced in independently
built clean and patched GTK. This establishes a baseline failure; patch-specific
effects are not excluded. No new entire GTK test-suite run is required here.

## Implemented normal path

* Cargo compiles an application-owned C GtkMediaFile/GdkPaintable adapter using
  public stock GTK/GStreamer APIs. No custom GTK installation, dynamic test
  module, network source preparation or user target environment variable.
* Bounded GstDiscoverer queries run off the GTK thread: two workers, two-second
  native discoverer deadline, per-source pending deduplication and a 64-entry
  LRU keyed by canonical path/device/inode/size/mtime/ctime including nanoseconds.
  Discovery may preroll/decode; it is not a free header-only operation. New source
  identities require another query; cache hits do not repeatedly probe.
* Conservative eligibility: exactly one non-image H.264 video, no subtitle
  streams, explicit BT.709 primaries/transfer/matrix, known full or limited range,
  8-bit and at most 8,388,608 pixels. HDR/PQ/HLG, sRGB, unknown metadata, other
  codecs, subtitles and images retain the stock route. This deliberately does
  not claim correction for every SDR codec.
* GL upload/import → GL color conversion (source YUV matrix/range) → full-range
  RGBA → BT.1886 ideal-black EOTF and sRGB OETF shader → GDK GL texture.
  Raw source caps are validated again; no arbitrary contrast/saturation or
  transfer operation on unrelated content. Metadata changes outside the contract
  fail with a controlled error, rather than applying an incorrect transfer.
* Original GL synchronization and texture/buffer ownership are retained.
  Presentation requires GL memory and sync metadata. No pixel readback or CPU
  transfer-function implementation in the playback path. Test texture downloads
  exist only in pixel assertions. Decoder output may require GPU upload;
  hardware decoding and zero-copy end-to-end are **not proven**.
* The main executable chooses the supported GTK GL renderer before initialization
  by re-executing itself once with a child environment. Explicit renderer overrides
  are respected; unsupported explicit renderers retain stock video with a
  diagnostic. Images keep their existing rendering/loading code, but renderer
  selection is process-wide and still requires native image/HDR regression.
* Missing native development APIs produce a stock-only build diagnostic. Missing
  GL plugins/context/synchronization or software GL produce clear errors; a NULL
  GPU sink never silently selects playbin's automatic CPU/other-window sink.
  `PIVRA_VIDEO_BACKEND=stock` is a reversible diagnostic override, not a required
  end-user setup step. No automatic CPU BT.1886 path is shipped.
* Known software GL is rejected in normal operation. Explicit
  `PIVRA_GPU_ALLOW_SOFTWARE_TEST=1` permits headless test execution only; those
  results do not certify hardware acceleration.

## Controls and ownership

The adapter uses direct `playbin3`, with weak bus/timer callback userdata and
idempotent NULL-state teardown. It delegates audio/mute/volume and playback,
reports duration/seekability/timestamps via GtkMediaStream, and coalesces pending
seek requests to the last target. Existing Pivra controls, frame stepping,
manual loop handling, paintable geometry and navigation remain connected through
GtkMediaStream. No UI or thumbnail provider redesign.

Selection checks the existing ImageView generation before applying a result.
Navigation and close invalidate it; stale discovery results cannot start another
video. Detach the paintable before stopping an owned stream, and defer error
cleanup outside its emitting callback. No RefCell guard spans stream teardown.

A prior uncommitted controller experiment using the deprecated GstPlayer/GstPlay
helper intermittently SIGSEGV'd during a two-window lifecycle test. Its private
backtrace included `g_object_unref` and `libgstplay`; debugger repetitions passing
did not exonerate it. Full upstream root cause was not established. That helper
path has been removed from this adapter, rather than suppressing the assertion
or claiming an upstream fix. Current direct-controller checks are recorded below.
This is distinct from the historical Mesa DMA-BUF SIGBUS.

## Dependencies and distribution

Native build: C compiler/ar/pkg-config, GTK >=4.16, GStreamer >=1.24 GL,
video, allocators and pbutils development APIs. Runtime also needs GL plugins,
playbin3, demuxer/decoder and audio sink supplied by system GStreamer.
No new Cargo dependency or Cargo.lock change. No GstPlayer/GstPlay dependency
in the integrated path. Native libraries remain dynamically linked; no absolute
build directory is used for runtime adapter discovery.

GTK source provenance and LGPL texts: `native/video/README.md`. Corresponding
modified sources, build instructions and notices must be included when packaging
this adapter. Packaging changes are intentionally deferred under this task's
restriction; this candidate is not a distribution approval.

## Acceptance command

With ordinary native development/runtime dependencies installed, stock GTK:

```sh
cargo run --locked -- /absolute/path/to/bt709-video.mp4
```

Compare the original accepted video against MPV. Verify color, pause/resume,
audio/volume/mute, repeated seek/frame stepping, EOS/loop, fullscreen/zoom/pan,
video-to-image navigation, two windows, rapid switching, deletion and normal
close. New direct-controller native acceptance is required despite the prior
prototype acceptance. Optional `PIVRA_TRACE_VIDEO=1` records actual source/caps,
context and GLMemory draw/retirement; it is not needed to enable correction.

## Remaining limits

Intel/NVIDIA, Vulkan interop, native HDR/non-target visual preservation,
hardware decoding/zero-copy and new-controller physical audio/EOS acceptance
are not certified. Midstream metadata changes are rejected, not seamlessly
handed to stock. Stock decoding on the Codex GTK SDK may be unavailable even
when non-target selection is proven. No Issue #9 closure before native acceptance.

## Current verification

Current-environment command results will be recorded after the bounded checks.

Executed in the current Codex Linux environment with stock GTK 4.18.6,
GStreamer 1.26.2, Mesa 25.0.7 llvmpipe; private development headers/plugins,
no system GTK replacement:

| Check | Result |
|---|---|
| `cargo fmt --check`, `cargo check --locked` | PASS |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS, zero warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 208 tests |
| Python video_gpu / video_color suites | PASS, 32 + 13 tests |
| C policy, strict compiler warnings | PASS, raw/encoded gates and negative metadata/format/size cases |
| `cargo build --release --locked` | PASS |
| Stock-only build (`PIVRA_DISABLE_GPU_BUILD=1`) | PASS; native-enabled build restored afterward |
| X11/Xvfb and Sway/Wayland integrated production-window probe | PASS, exit 0 on each |
| 256 decoded full-range levels vs independent BT.1886 reference | PASS on each, tolerance 2 LSB |
| Limited-range decoded 16/128/235 bars | PASS on X11: 0/123/255, tolerance 2 LSB |
| Coalesced seeking to final target, play/pause/volume/mute/loop properties | PASS on each |
| Fit/Fill/Actual synthetic zoom/pan | PASS on each |
| Controlled async cancellation / production-window GIO error races | PASS on each after status lookup correction |
| Real release executable, automatic renderer selection, actual draw and WM close | PASS on X11; no GSK_RENDERER or target/module configuration supplied |
| Normal software-driver rejection, no draw or GStreamer critical | PASS, controlled exit 1 |
| Physical new-controller AMD/audio/MPV comparison, Intel/NVIDIA/Vulkan, HDR visuals | NOT RUN |

All headless shader tests explicitly enabled the software-test override. They
prove API execution, synchronization, actual decoded pixel results and lifecycle,
**not physical GPU acceleration**. The integrated probe exercises two real windows,
limited/full target selection, sRGB/unknown stock routing, one discovery for the
same source, bounded cache, rapid multi-file navigation, repeated reopen/close,
and deletion while a new selection is pending. Final Sway close checkpoint:
`workers=1 filters=0`; after close workers/pending/filters all reached zero.
Earlier checks also closed a live filter. Stock-route pixel decoding is NOT RUN
in the SDK (GtkNoMediaFile), distinct from successful routing assertions.

Initial controlled async/window probes failed at `async_cancel_probe.rs:142` and
`window_failure_race_probe.rs:90`: both assumed the next sibling of the photo was
a Label, but it was the existing video Picture. The same hierarchy and lookup
were present at the base commit. The test-only correction finds the actual Label
among siblings; error/pixel/cancellation assertions remain intact. Both corrected
probes were rebuilt and executed successfully on X11 and Wayland.

One measured 192×96 synthetic integrated lifecycle run (software GL, multiple
windows/reopens): exit 0, elapsed **1.496 s**, process maximum RSS **354,068 KiB**,
user **1.786 s**, system **0.236 s** (monotonic clock and wait4 resource usage).
This includes GTK/Mesa/test overhead; it is neither GPU VRAM nor a hardware
playback benchmark, and no stock-vs-integrated memory delta is established.
The sink proposes 2–8 buffers, bounds pending presentation callbacks to two,
and accepts at most 32 MiB RGBA frames; discovery is limited to two active jobs.
Native high-resolution performance/VRAM measurement remains open.

## Subsequent native acceptance and packaging preparation

The user subsequently accepted the integrated automatic backend on AMD RX 7900 XT
with three BT.709 H.264 videos (720×1142, 1080×1712, 1080×1920), GLMemory/sync,
MPV-like colors, sound, controls and navigation, without a custom GTK or manual
backend enablement. This supersedes the pending AMD integration acceptance above;
it does not extend acceptance to Intel/NVIDIA/Vulkan/HDR/zero-copy.
The distribution build gate, source/license collection, staging results and open
release gates are in [v0.2.5 preparation](../releases/PIVRA_V0.2.5_RC_VALIDATION.md).
