# Issue #9 — GPU video-display-transfer candidate

2026-11-02, `dev`, based on `7c2abf220d53a9fb5807d94a3351fa6725e551a4`.
Target: v0.2.5 development; Cargo version remains **0.2.4**.

**Status: PARTIAL development prototype / BLOCKED for product acceptance.**
The user has accepted the Arch/GNOME **CPU reference** visually as similar to MPV.
That is user-reported native evidence. GPU acceleration is mandatory; the CPU
reference is not enabled or proposed as Pivra's default. Issue #9 remains OPEN.
No production Rust, Cargo dependency/lockfile, default backend, packaging, main,
tag, release, AUR or CI change is made by this work.

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
