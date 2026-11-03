# Issue #9 — GPU video-display-transfer candidate

2026-11-02, `dev`, based on `7c2abf220d53a9fb5807d94a3351fa6725e551a4`.
Target: v0.2.5 development; Cargo version remains **0.2.4**.

**Status: PARTIAL development prototype / BLOCKED for product acceptance.**
The user has accepted the Arch/GNOME **CPU reference** visually as similar to MPV.
That is user-reported native evidence. GPU acceleration is mandatory; the CPU
reference is not enabled or proposed as Pivra's default. Issue #9 remains OPEN.
No production Rust, Cargo dependency/lockfile, default backend, packaging, main,
tag, release, AUR or CI change is made by this work.

**Latest checkpoint:** the owner has now reproduced the exact sRGB DMA-BUF
SIGBUS on both clean and patched GTK 4.24.1. It is baseline-reproducible, with
patch-specific effects not excluded. See the final checkpoint below; earlier
NOT RUN/unknown-leaf statements are historical records. Portable production
integration remains unimplemented; accepted AMD native playback uses private GTK.

## Verified GTK 4.24.1 architecture

The earlier investigation used 4.18 module sources because the old
`modules/media/` URL is absent in 4.24. The actual 4.24 sources are under
**`gtk/media/`**, built into GTK itself via `media_sources`/`media_deps` in
`gtk/media/meson.build`, using GstPlay and requiring GStreamer >=1.28. The public
MediaFile API does not expose a display-EOTF setter or GStreamer sink pipeline.
A wrapper has no supported way to attach a different display EOTF to the source
texture. A post-render solution would need an additional GPU color stage,
metadata extraction and precision validation; no such workaround is enabled.

The native user report established DMA-BUF YU12, CICP `1/1/1/Narrow`, on GTK
4.24.1 / GStreamer 1.28.8. The 4.24 sink does preserve matrix and range. Its
memory, GL and DMA-BUF texture builders share the color state constructed by
`gtk_gst_color_state_from_colorimetry()`.

The 4.24 renderer already has a general GPU CICP conversion operation:

- `gsk/gpu/shaders/gskgpuconvertcicp.glsl` applies range, YUV matrix, transfer,
  gamut conversion and output transfer. It uses inverse BT.709 encoding for
  transfer 1, rather than the accepted BT.1886 video display interpretation.
- `gsk/gpu/gskglframe.c` imports DMA-BUF using EGL or per-plane textures;
  `gsk/gpu/gskvulkanframe.c` has Vulkan imports/YCbCr conversion.
- `gsk_gpu_color_state_apply_conversion()` reconstructs a color state when an
  import/sampler already converted YUV/range to RGB. A video-intent flag must
  survive this reconstruction to avoid silently losing the correction.
- Generic GPU import/upload fallback still exists. A DMA-BUF source alone does
  not prove zero-readback playback or hardware decoding.

The public `GskComponentTransferNode` (4.20+) was also reviewed: its public
constructor operates in sRGB; the explicit-color-state variant is private.
The GPU component-transfer implementation packs tables into 32 shared float
slots across channels and obtains the child as an image. An arbitrary gamma or
large post-render LUT is therefore not substituted for the exact, metadata-aware
video transfer: that alternative needs its own precision/intermediate-pass work.
The current Pivra build also targets GTK 4.16 APIs.

These statements come from the exact GTK 4.24.1 source, not extrapolation from
4.18. Source archive:

`https://codeload.github.com/GNOME/gtk/tar.gz/refs/tags/4.24.1`

SHA-256: `6b03cc3eac17e6d7b252c7ac96559e58aa7c990874ca47da696d81d0add7521d`.
The runner also pins every changed upstream file, the source version definition,
the shader generator and the shared color shader. This hash is for test sources;
no Pivra release-source checksum/configuration was changed.

## Concrete upstream integration patch

`tests/video_gpu/gtk-4.24.1-video-bt1886.patch` is an **experimental full-GTK source
patch**, not a loadable 4.18 media module, installed library or finished product
fix. It needs the native full-build/review gates below.

| Component | Change |
| --- | --- |
| GDK private color state | Separate immutable `video_bt1886` intent, distinct from image CICP transfer 1; identity/cache equality includes it. |
| GTK native video sink | Select intent only for negotiated BT.709 primaries and BT.709 transfer; retain source CICP range/matrix. |
| GPU import conversion | Preserve intent when EGL/Vulkan has already expanded YUV/range into full-range RGB. |
| GSK GPU CICP operation | Carry one extra integer instance parameter in both forward and reverse calls. |
| Shared GL/Vulkan shader | Apply ideal-black BT.1886 EOTF/inverse in the existing conversion pass; do not add another filter/pass. |
| CPU color-state methods | Keep explicit exports/Cairo behavior consistent with the state. These are not forced into the GPU path. |
| Node serialization | Serialize/parse video intent separately; reject invalid explicit intent instead of inventing a CICP transfer number. |
| Instrumentation/tests | Bounded GPU/CPU-path traces, GL driver identification, GDK state tests and GSK serialization tests. |

The reference EOTF is `L = max(V, 0)^2.4`, normalized white 1 / ideal black 0,
as used by the accepted CPU reference. It is not an adjustable gamma/contrast or
saturation filter. The patch does not change global CICP interpretation, encoded
pixel data, primaries, range, caps, decoder selection, GstPlay controls, buffer
ownership or cancellation. There is no per-frame CPU download, color conversion,
new frame buffer, decoder or mandatory CPU transport introduced into playback.

Eligibility is deliberately bounded: BT.709 RGB **full range**, or BT.709-matrix
YUV full/narrow range, with primaries/transfer 1. Unknown, sRGB, PQ/HLG, BT.2020,
other primaries and unsupported tuples keep existing behavior. Limited-range RGB
is not newly corrected: its handling requires a separate normalization audit.
The shader also checks metadata defensively before accepting the intent.
Negotiated tags may be inferred by GStreamer; the API does not prove bitstream
metadata provenance. Missing-field fallback policy is unchanged.

BT.709 and sRGB primaries coincide. The new video state uses an exact identity
for that gamut conversion, in CPU and shader paths. The reverse transfer exposed
otherwise tiny XYZ roundtrip noise near zero; fixing that identity kept the
strict numeric test intact rather than loosening assertions. Other states retain
their original gamut conversions.

Raw CICP describes encoding, not this video viewing intent. GSK serialization is
covered by the new pending native test. Raw texture-to-image export and native
Pivra frame capture still require acceptance; do not assume serializing the raw
CICP tuple alone preserves viewing intent. Existing Pivra frame capture renders
the media snapshot first, but that native fork scenario has not been run here.

## Executed checks and their limits

`tests/video_gpu/probe.py` prepares a full private GTK source copy, applies the
pinned patch with zero fuzz, and runs GTK's generator for C/header/GLSL interfaces.
It extracts the **actual** original/patched CICP math and vertex helpers into
OpenGL test stages; uniforms replace GSK's instance attributes. Vertex metadata
computation stays in the vertex stage. This tests the shader math and stage link,
not the full GSK node processor, attribute upload, compositor or DMA-BUF importer.

- **PASS:** strict C build of the shader probe (`-Wall -Wextra -Werror`).
- **PASS:** shader execution, 73 batches × 264 vectors, RGB and full/narrow YUV,
  forward/reverse, premultiplied/zero alpha, boundaries and color patches.
- **PASS:** maximum video-vs-CPU float error **2.38418579e-7**, with fixed absolute
  tolerance **3e-5**. The CPU transfer helper is checked against independent
  double-precision `pow`; the image/video transfer difference is asserted active.
- **PASS:** original-versus-patched non-target/intent-disabled shader comparisons:
  sRGB, PQ, HLG, linear, other transfers/primaries and the shader's unknown/error
  behavior. This is preservation evidence, not decoder support for invalid tags.
- **PASS:** generated instance ABI layout, **64 → 80 bytes**, one extra 16-byte
  attribute slot; generator's attribute-limit checks also pass.
- **PASS:** 10 source-integrity/hardware-classifier tests and the existing 13
  backend-harness tests (mocked classifier outputs are not hardware evidence).
- **PASS:** Rust fmt/check, strict all-target Clippy (zero warnings), **205 Rust
  tests**, and release build.
- **PASS syntax only:** added public GSK serialization-test C against the available
  SDK. Its runtime roundtrip and the GDK private-state tests are **NOT RUN**.
- **NOT RUN / 77:** required hardware context. Actual renderer is
  `llvmpipe (LLVM 19.1.7, 256 bits)`, vendor Mesa, no DRM render node.
- **NOT RUN:** full patched GTK build/link, native GL/Vulkan video, GPU import,
  hardware decode, audible audio/controls/navigation/zoom/frame capture under
  the new GTK build, physical GNOME/MPV comparison.

A software GL driver never satisfies `--require-hardware`. The gate requires a
non-software known driver plus the EGL-reported Linux DRM render character device
(major 226), not merely a renderer name or `/dev/dri` existing somewhere.

### Performance/memory observations — not native performance acceptance

One bounded measurement, llvmpipe, 1920×1080, one warmup and eight fullscreen
passes per variant; no texture upload or readback inside the measured draw loop:

| Measurement (8 passes total) | Original transfer | Candidate transfer |
| --- | ---: | ---: |
| GL elapsed query | 17.197 ms | 57.208 ms |
| Monotonic wall time | 186.204 ms | 182.160 ms |

These software-driver counters disagree with wall-time ordering and do not
predict product FPS, power, GPU performance or a speedup/regression on Arch.
The harness uses a static RGBA8 input and RGBA16F target, 8,294,400 + 16,588,800
bytes, shared by both variants. Peak process RSS was 119,152 KiB versus 70,608 KiB
before cases; it includes driver/JIT/test allocations, not decoder or measured
hardware VRAM. No loader child is created. Physical GPU memory/FPS remain NOT RUN.
The implementation adds 16 bytes per draw-instance plus private per-state intent;
it does not allocate a CPU pixel buffer or additional GPU pass per frame.

## Why the full GTK build is blocked here

The active Codex SDK has GLib 2.84.4, Pango 1.56.3, Wayland 1.23.1 and protocols
1.44. GTK 4.24.1 source requires GLib >=2.89.3, Pango >=1.58, Wayland >=1.24 and
protocols >=1.48, plus GStreamer >=1.28 (the previous isolated development runtime
is 1.26.2). GStreamer development metadata is absent from the active base SDK;
the separate older video SDK does not satisfy 1.28. Meson/Ninja are not installed.
The exact checks are recorded in `REPORT.json` by the probe.

No dependency checks were bypassed, opaque ABI structs patched, system libraries
replaced or existing SDK reset. A successful kernel test is not a successful
compile/link of the full GTK patch. The latter remains a release blocker, along
with GPU/import/native visual and performance acceptance.

## Reproduction and native integration gates

Kernel/hardware probe (no installation, retains report and patched source):

```bash
python3 tests/video_gpu/probe.py --require-hardware
```

Requires Python, patch, a C compiler and `pkg-config glib-2.0 epoxy egl`, with a
usable EGL/OpenGL 3.3 context. The pinned source is downloaded or can be supplied
with `--source /path/to/unmodified/gtk-4.24.1`. An empty `--output` fixes the retained
workspace location. `--prepare-only` prepares source/interfaces, never passes a
hardware gate and cannot be combined with `--require-hardware`.

The hardware probe alone is not native video acceptance. On a native development
system meeting the exact GTK dependencies, use the prepared source as follows.
These **commands have not been executed in Codex**; missing dependencies must
stop the build, not trigger ABI/version hacks or system upgrades automatically.

```bash
PIVRA_GTK_SOURCE=/path/from/probe/gtk-4.24.1
PIVRA_GTK_PREFIX="$HOME/.local/opt/pivra-video-gpu/gtk-4.24.1"
meson setup "$PIVRA_GTK_SOURCE/_build" "$PIVRA_GTK_SOURCE" \
  --prefix="$PIVRA_GTK_PREFIX" --libdir=lib --buildtype=debugoptimized \
  --wrap-mode=nofallback -Dmedia-gstreamer=enabled \
  -Dbuild-testsuite=true -Dbuild-tests=false -Dbuild-demos=false \
  -Dbuild-examples=false -Dintrospection=disabled -Ddocumentation=false
meson compile -C "$PIVRA_GTK_SOURCE/_build"
meson test -C "$PIVRA_GTK_SOURCE/_build" --print-errorlogs colorstate-internal
"$PIVRA_GTK_SOURCE/_build/testsuite/gsk/serialize" \
  "$PIVRA_GTK_SOURCE/testsuite/gsk/compare/color-states.node"
meson install -C "$PIVRA_GTK_SOURCE/_build"
```

The install prefix must remain a new user-owned test prefix, never `/usr`. Check
Meson's actual GL/Vulkan feature summary; disabled backends are NOT RUN. Before
launching, verify the app resolves `libgtk-4.so.1` from this prefix using process-
local `LD_LIBRARY_PATH`/`ldd` or loader logs. GTK's version number alone does not
prove which build was loaded. Do not leave any earlier CPU-test environment or
external module active:

```bash
env -u GTK_MEDIA -u GTK_PATH -u GIO_EXTRA_MODULES -u PIVRA_VIDEO_TEST_MODULE_DIR \
  -u PIVRA_GTK_BT1886_CPU -u PIVRA_GTK_COLOR_CPU_TRANSPORT \
  LD_LIBRARY_PATH="$PIVRA_GTK_PREFIX/lib:${LD_LIBRARY_PATH:-}" \
  GDK_BACKEND=wayland GSK_RENDERER=gl \
  GDK_DEBUG=misc,opengl,dmabuf GSK_DEBUG=renderer,fallback \
  GST_DEBUG=gtkgstsink:5 \
  cargo run --locked -- "/absolute/path/to/the/problematic.mp4"
```

Required evidence before any product integration:

1. Correct loaded GTK path, hardware GL driver, real video source storage/caps,
   and **`video-bt1886 GPU CICP op`** trace. Run Vulkan separately if enabled.
2. No **`video-bt1886 CPU transfer`** or **`GPU readback/upload fallback`** during
   steady playback. These optional traces are bounded to 16 samples; exports can
   intentionally download and must be measured separately. Source DMA-BUF alone
   or absence of a debug log from a non-debug build is not sufficient proof.
3. Verify GL/EGL and Vulkan YUV sampler conversions retain video intent and do
   not expand range twice. Source/decoder bit depth, range and the exact paused
   frame must match when comparing to the CPU reference. Defined kernel float
   tolerance is not a blanket allowance for differing decoder output/frames.
4. Measure native parent/decoder RSS, GPU memory, frame timing and allocation/
   readback activity on representative clips. Check audio, seek, pause/resume,
   loop, frame step, fast navigation, two windows, zoom and capture.
5. Check non-target sRGB/PQ/HLG/other primaries and invalid/unknown metadata, then
   obtain physical Arch/GNOME visual acceptance against MPV on the original file.

No gate above is silently considered complete. Upstream review must also decide
how general GtkMediaFile users opt into video display intent (including image
streams through GStreamer) and how raw texture exports communicate it. Pivra's
normal images use Glycin and are not changed by this prototype. Do not ship a
private GTK fork or enable the CPU reference by default to bypass these gates.

## Native integration checkpoint — 2026-11-03

**User-reported native Arch evidence**, not independently executed in Codex:
patched GTK 4.24.1 built successfully; the hardware shader probe passed on
AMD/Mesa. GTK tests: **8108 PASS, 109 FAIL, 457 SKIP**. Dedicated
`/colorstate/video-bt1886` and render-node serialization tests passed.
This supersedes the earlier *native build not yet tested* checkpoint, but does
not turn the entire suite or physical video comparison into PASS.

`gtk:colorstate` reproducibly receives **SIGBUS** during DMA-BUF download:
`gdk_gl_context_download()` (`gdkglcontext.c:2621`) →
`gsk_gpu_download_into_op_gl_command()` →
`gsk_gpu_renderer_dmabuf_downloader_download()` →
`gdk_dmabuf_texture_download()`, with Mesa
`libgallium-26.2.6-arch1.1.so` and libc in the crashing stack.
**Attribution OPEN**: this is not evidence that the patch is innocent, nor that
it caused the crash. No core, complete test path, or clean native baseline
result has been supplied. Do not disable DMA-BUF to turn this gate green.

### Single isolated Arch comparison command

After building/installing the patched GTK into a **private user prefix only**
using the previous section, run from the Pivra checkout:

```bash
python3 tests/video_gpu/native.py \
  --prefix "$HOME/.local/opt/pivra-video-gpu/gtk-4.24.1" \
  --video "/absolute/path/to/original-bt709-video.mp4" \
  --output "$HOME/pivra-gpu-native-$(date +%Y%m%d-%H%M%S)"
```

Requires native Wayland, cargo, ffprobe and mpv. Does not install GTK or change
system settings. Builds ordinary Pivra with `cargo build --locked`; no CPU
reference switch or application source change. It clears inherited private
library/module paths and CPU-reference overrides, uses process-local library
selection and verifies `/proc/PID/maps` for **both** Pivra instances. The private
library must contain patch tracing. A patched stock/system library fails the
stock check. No version-number-only acceptance is used. Logs retain ffprobe
stream metadata, actual loaded paths, private library checksum, GTK/GStreamer
traces and process exits. mpv uses `--no-config`; compositor/output profile must
remain the same for all windows. Close all windows when finished; interrupt
cleans up launched process groups. No unattended visual PASS is emitted.

Add `--control-video /path/to/known-srgb-or-PQ-or-HLG.mp4` to open a non-target
control. Verify negotiated metadata in logs (source ffprobe tags alone are not
negotiated sink caps). Eligible BT.709 playback must produce the GPU CICP trace;
the non-target must not produce it. No CPU-transfer or readback fallback traces
are allowed during steady playback. Bounded trace absence alone is not proof:
record actual hardware renderer, DMA-BUF import versus fallback, and native GPU
profiler/readback observations. Hardware shader success alone does not establish
hardware video decoding, end-to-end DMA-BUF use or zero-copy playback.

Compare black levels and colors on the **original clip**, stock versus patched
versus mpv, with matched pause/frame/display conditions. Exercise sound, seek,
pause/resume, loop, navigation, zoom and two windows. Record frame timing and
parent/decoder/GPU memory separately. Return `REPORT.json` and logs plus visual
acceptance explicitly; the runner deliberately leaves acceptance NOT RUN/NOT
PROVEN rather than inferring it from windows opening.

### Narrow SIGBUS control (no whole-suite rerun)

Build a second **unmodified** GTK 4.24.1 from the pinned archive above into a
separate clean workspace/prefix. Use the same compiler, Meson options, environment
and Mesa packages as the patched build. Retain archive checksum, Meson logs and
`meson-info/intro-buildoptions.json` for both; never reuse the patched build tree.
Obtain the exact failing `/colorstate/...` path from the native test log or list
`testsuite/gdk/colorstate -l`; source registers DMA-BUF cases under both
`/colorstate/convert-dmabuf/srgb/...` and `/colorstate/convert/dmabuf/srgb/cicp/...`.
Do not guess which case crashed based solely on the stack.

Extend the **same launcher command** with:

```bash
  --baseline-build /absolute/path/to/clean-gtk/_build \
  --patched-build /absolute/path/to/patched-gtk/_build \
  --colorstate-path '/exact/failing/test/path'
```

Only this exact GLib `-p` case is run once per build, under the same GL/Wayland
settings, with its build-tree GTK library directory; stdout/stderr and signal
exit codes are retained. Inspect loader resolution for each control (for example
`LD_DEBUG=libs`) before attributing results: clean source/options and runtime
library provenance are obligations, not automatically certified by directory
names. If both crash, preserve this as baseline-reproducible but still audit
patch-specific changes; if patched-only crashes, block integration and inspect
color-state identity, conversion/import and download strides/mappings. If neither
crashes, reproduction remains incomplete. A SKIP is not a crash PASS. Obtain
GDB `bt full`, the exact test seed/path and mapping/stride/format details for a
reproducing case; do not raise timeouts or remove assertions.

### Checks for this launcher change

Codex: **15 Python tests PASS** (existing GPU tests plus five isolation/control
regressions); Python syntax PASS. No Rust/product or GTK patch changes were
made. Native launcher, clean/patched SIGBUS comparison, physical video playback,
DMA-BUF/profiler evidence and MPV visual acceptance: **NOT RUN in Codex**.
Full native suite remains partly failing as reported above. Issue #9 stays OPEN;
production integration is blocked on the targeted crash classification and
end-to-end GPU/native visual acceptance. No system GTK replacement is proposed.

## GL/Vulkan acceptance checkpoint and portable integration — 2026-11-03

### Native evidence supplied by the owner

These are **user-reported Arch/GNOME results**, not Codex hardware measurements:

| Gate | Result |
| --- | --- |
| Patched GTK runtime mapping | PASS for GL and Vulkan |
| GL playback | 16 BT.1886 GPU CICP traces; 0 CPU transfer; 0 readback/upload fallback traces |
| Physical GL visual comparison | User reports much better, MPV-like colors |
| Vulkan playback | `Vulkan uploaded 720x1142 YU12 ... YUV dmabuf`; 16 GPU CICP traces; 0 CPU transfer/fallback traces |
| Stock and patched process exits | Successful |
| Hardware decoding / continuous zero-copy | **NOT PROVEN** |
| Other GPU vendors / HDR display acceptance | **NOT RUN** |
| `gtk:colorstate` DMA-BUF SIGBUS attribution | **OPEN**, still unclassified |

The trace counters saturate at 16; they establish execution of the corrected
operation, not frame count or a full capture of every possible readback. Vulkan
`uploaded` is GTK's log wording, not proof of a CPU upload nor continuous
zero-copy. Do not infer physical Vulkan visual acceptance from GL acceptance.

### GL/Vulkan range and metadata audit

Reviewed the pinned GTK 4.24.1 sources and the existing patch:

- `gtk/media/gtkgstsink.c:gtk_gst_color_state_from_colorimetry()` retains CICP
  primaries/transfer/matrix/range; only the BT.709 video constructor adds intent.
  Ordinary image construction is unchanged. No global transfer-1 reinterpretation.
- `gsk/gpu/gskglframe.c` single-image EGL YUV import supplies REC709 plus
  **full or narrow range hints**, and records the applied conversion on the image.
  Its multi-plane import records `GSK_GPU_CONVERSION_NONE`; conversion therefore
  remains in the shader. The patch does not force single-plane import.
- `gsk/gpu/gskvulkanframe.c` passes the source conversion into
  `gsk_vulkan_image_new_for_dmabuf()`. `gskvulkanimage.c` constructs the YCbCr
  sampler conversion when needed. Device format/modifier/YCbCr support may reject
  an import; hardware support is not created by the color patch.
- `gsk/gpu/gskgpuutils.c:gsk_gpu_cicp_apply_conversion()` validates the conversion
  against the incoming tuple. Applied BT709 narrow conversion becomes matrix 0,
  full range; BT709 full becomes matrix 0 without a second range expansion.
  The patched `gsk_gpu_color_state_apply_conversion()` retains the video intent
  when rebuilding that normalized state. NONE retains the original state.
- Both backends use `gskgpuconvertcicp.glsl` and the generated instance field.
  Raw YUV gets one matrix/range conversion before EOTF; sampler-converted RGB
  must not get it again. No GL-only texture color-state mutation is introduced.
  Driver precision, chroma siting, modifiers and synchronization still need
  native coverage; arithmetic equivalence does not exercise these mechanisms.

No proven production defect was found in this review, so the upstream patch and
production defaults are unchanged. Supported intent remains primaries 1,
transfer 1, matrix 1 full/narrow or matrix 0 full. Limited RGB, unknown metadata,
other primaries, sRGB, PQ and HLG retain original behavior. That is preservation
of existing implementation, not new HDR correctness certification. General
GtkMediaFile image streams remain an upstream policy question; Pivra's photos
continue to use the untouched Glycin path.

### Additional executable regressions

`gpu_probe.c` now compares original versus patched shader output for non-target
primaries/transfers across **both matrix and range values and forward/reverse**
conversion. It includes intent-disabled BT.709 image states and rejected
limited-RGB intent. The original 3e-5 tolerance and CPU-reference assertions are
unchanged. Additional cases compare raw full/narrow YUV shader conversion with
pre-normalized RGB plus retained intent, checking that import conversion must
happen exactly once. These are shared GL/Vulkan shader **arithmetic** checks;
no Vulkan device execution or real DMA-BUF import is simulated as PASS.

Codex run: **532 batches × 264 vectors**, maximum video/reference error
**2.38418579e-7**, PASS on llvmpipe. Hardware gate remains NOT RUN. No Vulkan
shader compiler/device tooling is available here. Existing native AMD GL/Vulkan
playback evidence is kept separately. Intel and NVIDIA, proprietary NVIDIA EGL
import/modifiers, Vulkan YCbCr support, Intel VA-API interop, HDR/wide-gamut
outputs and source chroma/bit-depth permutations remain untested.

`native.py --renderer gl|vulkan` now selects the renderer process-locally for
stock, patched and optional non-target playback, and records the request. A
requested renderer alone is not verification of actual backend choice: retain
GTK renderer/import logs. Default remains GL for compatibility with the earlier
command. No environment or system library is modified persistently.

### Exact SIGBUS baseline control

`tests/video_gpu/baseline.py` is a standalone control with **no video windows or
full-suite run**. It requires an exact registered test leaf (checked with `-l`),
checks the baseline's pinned upstream touched-source hashes, rejects swapped
patched/clean libraries, validates `ldd` GTK resolution, and records loader logs,
library hashes, build options, exits and skip status. Timeout/skip/other signals
are inconclusive. Matching SIGBUS results establish baseline reproduction;
**they do not exonerate every patch-specific effect**. A clean PASS versus patched
SIGBUS blocks integration. Both PASS means not reproduced, not fixed.

```bash
python3 tests/video_gpu/baseline.py \
  --baseline-build /absolute/path/to/unmodified-gtk-4.24.1/_build \
  --patched-build /absolute/path/to/patched-gtk-4.24.1/_build \
  --case '/exact/failing/colorstate/leaf' \
  --output "$HOME/pivra-sigbus-$(date +%Y%m%d-%H%M%S)"
```

Use the same Mesa, session, renderer, build configuration and original failing
case/seed for both. Source checks cover the pinned files, not arbitrary edits
elsewhere: clean archive provenance and matching build options still require
review. No exact failing leaf, clean native build or baseline result is available
in Codex. SDK prerequisites remain insufficient to build GTK 4.24.1 here; do not
substitute a different GTK/Mesa version or run thousands of tests to guess.
**Actual classification: BLOCKED pending this native evidence.** The two stacks
alone do not prove a Mesa-only bug. Preserve a reproducing GDB backtrace/core.

### Portable deployment design (not enabled)

The current patch uses private GTK/GDK APIs and a generated GPU instance ABI.
It cannot safely be injected into arbitrary system GTK with LD_PRELOAD or a
MediaFile extension. Stock GTK's public API does not expose the required video
viewing intent. A standalone shader over a MediaFile paintable lacks reliable
original color metadata/import semantics; adding an arbitrary gamma node would
not implement this patch correctly. Replacing the entire player with a custom
GStreamer/GL sink would duplicate control/synchronization/audio/capture ownership
and is not a small accepted change.

Two supported deployment routes should be reviewed before production:

1. **Preferred upstream route:** submit the video-intent design, import-state
   preservation, serialization and tests to GTK; resolve opt-in semantics for
   other GtkMediaFile users. Depend on a released distribution GTK containing
   the accepted API/behavior, with an explicit minimum version/capability gate.
   This requires no end-user GTK build or system-library replacement by Pivra.
   Until upstream acceptance exists, do not invent a released capability/version.
2. **Interim, separately approved binary distribution:** build the reviewed GTK
   patch once in a reproducible release environment and ship it as an
   application-private runtime, e.g. `/opt/pivra/lib/`, with the Pivra launcher
   and executable also inside that isolated tree. Use a private RUNPATH/launcher,
   never `/usr/lib` replacement or a global `ldconfig`/LD_LIBRARY_PATH change.
   Ship matching private GTK support libraries, media resources and any minimum
   GLib/Pango/GStreamer runtime needed; inventory the complete ELF dependency
   closure and plugins rather than assuming the host ABI is new enough. Resolve
   all GTK-using plugins to this same GTK. Keep GPU vendor drivers, kernel DRM
   and Mesa/Vulkan ICD integration host-provided; do not bundle a vendor driver
   or silently load two GTK builds in one process. The user installs a prebuilt
   package, not a compiler toolchain or custom system GTK. This is a deployment
   design only; no new package, AUR, release metadata or defaults are enabled.

For the interim route, pin source/patch/build options and checksums, provide GTK
LGPL corresponding source/build instructions and notices, retain GPL Pivra
source, audit redistributed dependency licenses, and own security updates for
**every** bundled library. Make decoder plugin discovery and helper dependencies
explicit without changing Glycin/Bubblewrap security. CI/release artifacts are
not created by this experiment. Accept clean native installation, ELF resolution,
GL/Vulkan and other-vendor tests, non-target/HDR preservation and SIGBUS control
before approving this route. No mandatory CPU fallback is acceptable: if required
GPU capabilities cannot be established, block the experimental corrected path
with a clear diagnostic rather than silently switching rendering to the CPU.

### Short native playback acceptance command

```bash
python3 tests/video_gpu/native.py --prefix "$HOME/.local/opt/pivra-video-gpu/gtk-4.24.1" --video "/path/original.mp4" --renderer vulkan --output "$HOME/pivra-vk-$(date +%s)"
```

Use `--renderer gl` for GL and `--control-video /path/non-target.mp4` for actual
metadata controls. Do not repeat already accepted original-video tests solely
to generate another log; the outstanding SIGBUS needs the separate exact-case
command above. No universal one-command baseline can identify an unknown failing
leaf or manufacture a missing native build.

New tooling checks: **21 Python tests PASS**, including isolation, renderer
selection and crash-classification negatives; no fabricated native controls.
Issue #9 remains OPEN / experimental integration PARTIAL. Native GL visual
acceptance is recorded; portable packaging approval, SIGBUS classification,
non-target native display coverage and other-GPU testing remain open.

Follow-up verification on source base
`03fab1d4eabf6a9bff028533d0c4550da98b4b39`: fmt, locked check,
strict all-target Clippy (zero warnings), **205 Rust tests** and the existing
**13 CPU-harness Python tests** PASS. The 21 GPU-tooling Python tests above are
additional (34 Python tests total). No application Rust, dependencies, loader,
controls, version or production default changed. Baseline control uses the same
GLib random seed for both runs; supply `--seed R02S...` from the failing log to
reproduce its seed exactly. The new control is preferable to the older optional
unvalidated in-session baseline loop when investigating SIGBUS.
Locked release build also PASS for this follow-up; generated build artifacts
remain outside the tracked change set. No new physical GPU acceptance is claimed.

### Exact native SIGBUS reproduction supplied subsequently

The owner independently reproduced **SIGBUS** on patched GTK 4.24.1 with the
exact leaf **`/colorstate/convert-dmabuf/srgb/srgb`**, under Wayland, patched
build-tree `LD_LIBRARY_PATH`, build/source `testsuite/gdk` directories,
`GIO_USE_VFS=local` and `GIO_USE_VOLUME_MONITOR=unix`. Clean GTK has **not** been
built or tested. This supersedes the earlier statement that the leaf was unknown;
causality remains OPEN. The control now defaults to this leaf and sets those
same G_TEST/GIO variables separately for each build. No other case is executed.

Smallest next native control:

1. Extract the pinned, SHA-256-verified **unmodified** GTK 4.24.1 archive into a
   new private source directory. Configure a separate `_build` using the **same
   Meson options/toolchain** as the patched build. Never reuse the patched tree,
   install it over system GTK, or change Mesa between the two runs. Compare
   `meson-info/intro-buildoptions.json` files; retain any differences.
2. Build only the baseline test and its dependencies:
   `ninja -C /path/to/clean-gtk-4.24.1/_build testsuite/gdk/colorstate`.
   No `meson test` suite run or installation is required.
3. Run this one command (two executions of the same leaf, one per build):

```bash
python3 tests/video_gpu/baseline.py \
  --baseline-build /path/to/clean-gtk-4.24.1/_build \
  --patched-build /path/to/patched-gtk-4.24.1/_build \
  --output "$HOME/pivra-sigbus-control-$(date +%s)"
```

The command explicitly uses the GL renderer; the reported backtrace is the GL
DMA-BUF downloader. Preserve the original GLib seed with `--seed R02S...` if
available. The sRGB case does not request video BT.1886 intent, but this fact
alone does **not** rule out shared ABI/download/driver interactions from the
patch. Only the clean control provides baseline evidence. New environment/default
changes are test tooling only; no production or GTK patch code changed.

## Baseline comparison completed; production architecture decision record

2026-11-03. Documentation source base:
`b5b4455e65fd0a120c561a217c8b3e58d843f18e`.

### New native evidence and precise classification

The owner reports running `tests/video_gpu/baseline.py` against independently
built **unmodified GTK 4.24.1** and **patched GTK 4.24.1**, for the exact leaf
`/colorstate/convert-dmabuf/srgb/srgb`. Result:

```text
BASELINE REPRODUCIBLE SIGBUS; patch-specific effects not excluded
```

**Classification: baseline-reproducible native DMA-BUF download SIGBUS.** The
BT.1886 patch is not necessary to reproduce this crash. This does not prove a
Mesa-only root cause, clear all patch effects, or make the 109 previously reported
GTK failures PASS. No production assertion, DMA-BUF path or GPU gate is disabled.
The baseline comparison requirement is satisfied by **user-reported evidence**;
Codex has not independently reviewed the raw native artifacts. Earlier statements
that no clean baseline was built are superseded, not erased.

Preserve the native output directory unchanged, including `REPORT.json`,
`baseline.log`, `patched.log`, `baseline-ldd.log`, `patched-ldd.log`,
`baseline-loader.*`, `patched-loader.*`, and both `*-buildoptions.json` files.
Keep the source archive/patch checksums, GDB backtrace/core, original GLib seed,
Mesa version and session details with it. The runner's report records resolved
GTK paths, library SHA-256 hashes, source directories and direct subprocess
signal exits (SIGBUS is normally `-7`). The supplied summary does not contain
these raw fields, so **no hashes, filenames outside the defined runner outputs,
build-option equivalence or independently observed signal numbers are invented**.
Do not delete these artifacts because the crash is baseline-reproducible.
Native artifacts have not been uploaded to this Codex workspace or committed.

Existing AMD GL/Vulkan execution and GL physical MPV-like visual acceptance
remain recorded above. Hardware decoding, continuous zero-copy and Intel/NVIDIA
acceptance remain unproven. The baseline crash no longer supports attributing
this exact failure solely to the video patch; it remains an upstream/platform
risk, particularly for texture exports/capture/download operations.

### Proposed stock-GTK production architecture

**Requirement:** regular users install Pivra and normal runtime dependencies,
not a privately compiled/replacement GTK. A private GTK package is therefore not
the proposed stock-GTK solution. Upstream GTK integration is the least duplicate
implementation in the long term, but cannot supply an immediate stock-GTK fix
until a supported release exists. No such release/capability is assumed here.

For an independently deployable implementation, use an **application-owned
GStreamer playback adapter and GPU video sink**, outside GTK's built-in MediaFile
backend. This is a concrete proposal, **not implemented or accepted production
code**. It requires a bounded playback-backend project, not a paintable filter:

1. **Control adapter:** retain a `GtkMediaStream`-compatible object (subclass) as
   the control/paintable boundary, backed by GstPlay or playbin with an explicitly
   selected custom sink. Reuse existing `VideoControls::attach(MediaStream)`;
   implement prepared/error/duration/timestamp/seekable/ended state, audio,
   seek completion, mute/volume and cancellation on the GTK main context. Keep
   one pipeline/clock per video, not a second parallel decoder.
2. **Negotiation and ownership:** receive the actual negotiated GstVideoInfo,
   colorimetry, bit depth, crop/orientation, memory features and buffer identity.
   Preserve DMA-BUF fds, modifier/plane strides/offsets and GstBuffer lifetime
   until GPU completion. Accept GLMemory with compatible context sharing, or
   import supported DMA-BUF using EGL. Reject unsupported modifiers/formats
   explicitly; do not secretly call a frame-download/CPU-conversion API. GPU
   rendering does not imply hardware decoding. A decoder delivering CPU memory
   may require upload; that must be measured/disclosed, not called zero-copy.
3. **One metadata-aware GPU color stage:** on the original video samples, apply
   source range normalization and YUV matrix **exactly once**, followed by the
   validated BT.1886 EOTF only for the eligible BT.709 SDR tuple. If EGL sampling
   already normalized/matrix-converted, select the normalized-RGB variant, as
   validated by the import arithmetic cases. Do not apply it to the already
   color-managed snapshot of MediaFile. Carry a per-frame/generation metadata
   key so source/caps changes cannot reuse stale shader state.
4. **Tagged output:** render into a GPU RGB texture and hand it to stock GDK
   through public GL texture/color-state interfaces. For the corrected BT.709
   path, store sRGB-encoded values and tag sRGB (or a validated supported linear
   RGB equivalent); GDK then performs ordinary output-display color management.
   Never tag BT.1886-processed samples as encoded BT.709 or apply another gamma
   node. No public CICP tag is invented for the viewing intent.
5. **Non-target route:** keep the existing MediaFile backend for sRGB/other
   primaries/PQ/HLG/unknown or unsupported metadata until the new sink's passthrough
   is independently verified. Determine eligibility from a bounded metadata
   inspection/probe, not a filename or container MIME type; treat incomplete
   metadata conservatively. Such a probe has I/O/cancellation/stream-selection
   implications and is itself a blocker until implemented. Do not switch between
   two active decoders mid-frame or relabel HDR as sRGB. If metadata changes after
   startup, pause/retire the corrected pipeline under a generation barrier and
   transition explicitly, preserving position/controls. Do not silently apply SDR
   correction to an HDR segment. Existing Glycin image rendering stays untouched.
6. **Rendering backends:** a shared EGL/GL context and `GdkGLTexture` is the first
   portable path; stock GSK GL can consume it. GSK Vulkan cannot be assumed to
   consume a GL texture without a readback. For Vulkan presentation, export the
   corrected RGB GPU image as a synchronized, supported DMA-BUF and present a
   tagged `GdkDmabufTexture`, or implement a native Vulkan compute/render sink.
   Verify importability, fences, queue ownership and lifetime on each backend.
   If cross-API sharing is unsupported, report that capability as unavailable;
   do not make CPU copies the hidden interoperability fallback. A pure GL-only
   first prototype must be labelled GL-only, not portable Vulkan acceptance.
7. **Lifetime and resource policy:** bounded in-flight GPU buffers, explicit
   GPU-completion ownership, weak UI callbacks plus per-load generation, and
   teardown on navigation/window close. No decoder stop or GL allocation from a
   GTK callback that can deadlock a streaming thread. Instrument imports, uploads,
   texture downloads, CPU conversions, outstanding buffers and driver identity.

Source integration boundaries verified in this checkout:
`src/image_view.rs::load_video()` creates `gtk::MediaFile` and retains that
concrete type; stop/resume/capture paths also depend on it.
`src/actual_paintable.rs::ActualPaintable::video()` takes `&gtk::MediaFile` while
internally storing a generic paintable. `src/video_controls.rs` already attaches
the base `gtk::MediaStream`. These concrete MediaFile boundaries need narrow
adapter changes; simply replacing the texture wrapper would not supply negotiated
metadata or control a GPU sink. No such production changes are made now.

New GStreamer/GL bindings and runtime plugin requirements must be audited for
version, license and packaging cost before adding dependencies. No decoder,
Glycin sandbox, provider priority, existing safety limit or release metadata is
changed by this architecture record. A libmpv replacement was not selected: it
would introduce another playback backend/dependency and control/lifetime work
rather than finishing the current GTK/GStreamer integration.

### Implementation sequence and exact remaining blockers

1. **Prototype control/negotiation adapter against stock GTK**, disabled by
   default. Preserve playback controls/resume/loop/window teardown in controlled
   tests before routing user media to it. No custom GTK installation permitted.
2. **GL sink prototype:** import real negotiated video frames and run the shared
   metadata-aware color kernel. Compare the same decoded samples/PTS against the
   accepted CPU reference at the existing numeric tolerance; verify native video
   pixel tolerances separately from shader floating-point tolerance. No CPU
   readback in steady playback (test-only readback allowed for assertions).
3. **Interop prototype:** prove zero-readback presentation into both stock GSK GL
   and Vulkan, including synchronization and memory bounds. Current native
   patched-GTK Vulkan evidence does not validate this new adapter.
4. **Non-target/HDR policy and regressions:** accurate eligibility routing,
   renegotiation and original-path preservation; real HDR output and images,
   source ranges and bit depths. Existing shader comparisons alone are not
   physical HDR acceptance.
5. **Native release gate:** original MPV comparison on stock GTK, audio/seek/
   navigation/loop/zoom/two-window/close/capture, measured GPU/memory behavior,
   installable dependency closure, and explicit Intel/NVIDIA/AMD support matrix.
   Until tested, Intel/NVIDIA are NOT RUN; no cross-GPU claim.
6. **Platform risk disposition:** retain the baseline-reproducible download
   SIGBUS as open; obtain an upstream GTK/Mesa issue/reproducer and decide its
   impact on capture/export separately. Baseline reproducibility is not an
   authorization to ignore it or an automatic production GO.

**Current status:** GPU transfer correction has user-reported AMD native evidence
with patched GTK; portable stock-GTK production integration is **BLOCKED /
UNIMPLEMENTED**, not ready for ordinary users. No version/default is changed.
The above steps are outstanding engineering/acceptance, not completed work.

### Shortest current acceptance test and issue update text

The current test still requires the experimental private GTK prefix; there is
**no truthful stock-GTK acceptance command for an unimplemented sink**:

```bash
python3 tests/video_gpu/native.py --prefix "$HOME/.local/opt/pivra-video-gpu/gtk-4.24.1" --video /path/original.mp4 --renderer vulkan --output "$HOME/pivra-accept-$(date +%s)"
```

Use GL for the already physically accepted path; add `--control-video` for a
known non-target source. No baseline rerun or whole-suite run is needed merely
to record this completed comparison. A future stock-GTK acceptance runner must
fail if it resolves a private patched GTK, otherwise it would not establish
portable integration.

Issue #9 update prepared for posting:

> Native owner report: the exact
> `/colorstate/convert-dmabuf/srgb/srgb` control SIGBUS reproduces on independently
> built unmodified and patched GTK 4.24.1. Runner classification is
> `BASELINE REPRODUCIBLE SIGBUS; patch-specific effects not excluded`.
> The patch is not necessary to reproduce this crash, but it is not fully cleared.
> Preserve both logs, loader/library provenance, build options, seed and signal
> exits; raw artifacts have not been independently reviewed by Codex.
> AMD GL/Vulkan GPU execution and GL MPV-like visual acceptance remain
> user-reported PASS. Hardware decoding and continuous zero-copy remain unproven.
> Issue stays open: portable stock-GTK integration requires an application-owned
> metadata-aware GStreamer GPU sink/control adapter or an accepted upstream GTK
> release. Neither is production-ready here. No CPU fallback or system GTK
> replacement is enabled. See `docs/diagnostics/VIDEO_COLOR_GPU_CANDIDATE.md`
> for the architecture, staged blockers and existing experimental test command.

Posting attempt in this session: GitHub CLI/API returned **Forbidden** even for
reading Issue #9. No GitHub issue edit/comment is claimed. The text above is
retained in the repository so the update is reviewable without claiming success.

## Application-owned implementation checkpoint

See [the stock-GTK GPU prototype](VIDEO_COLOR_STOCK_GTK_PROTOTYPE.md). An opt-in
public MediaFile subclass now executes an application-owned GL filter and presents
synchronized GdkGLTexture frames in Pivra, without patched GTK or CPU color fallback.
Earlier architecture-only statements are historical. The prototype is file-scoped
and GL-only; default behavior, native GPU/visual acceptance and Vulkan interop
are not silently promoted to production readiness.
