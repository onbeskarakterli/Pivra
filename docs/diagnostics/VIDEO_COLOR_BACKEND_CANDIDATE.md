# Issue #9 — real GTK/GStreamer backend candidate

2026-11-02; continuation of `573bc2727960b96a2a66d08ca5eb563ae4158011`.
**Status: CPU reference implementation tested on real H.264; production
integration and final Arch/MPV visual acceptance remain OPEN.**
Application version remains 0.2.4. No production Rust, dependency, package,
version, sandbox or default-backend change is included.

## Native DMA-BUF failure and explicit-loader correction

[Issue #9's latest native comment](https://github.com/onbeskarakterli/Pivra/issues/9#issuecomment-6581600333)
reports **FAIL** on Arch GTK 4.24.1 / GStreamer 1.28.8. Even candidate-enabled
playback remained `GdkDmabufTexture`, accepted caps
`video/x-raw(memory:DMABuf), format=DMA_DRM, drm-format=YU12, colorimetry=bt709`,
CICP `1/1/1/Narrow`; patches were `0 / 142 / 255` against CPU expectations
`0 / 121 / 253`. This is user-reported native evidence, not a Codex execution.
It establishes that the requested CPU experiment was not active; it neither
refutes BT.1886 nor establishes that the original color issue is fixed. DMA-BUF
identifies storage/import, not whether the decoder itself was hardware-assisted.

The confirmed harness defect was selection/provenance: `GTK_MEDIA=gstreamer`
and a duplicate `libmedia-gstreamer.so` name did not distinguish our module from
the installed backend, and CPU activation was not checked before pixel assertions.
The precise Arch loader mechanism (search order, built-in registration, symbol
binding or newer integration) cannot be reconstructed from that comment alone.
Do not claim a proven ABI incompatibility or change installed GTK libraries.

The new isolation is independent of `GTK_PATH`:

- Original-policy and candidate copies have unique module filenames, extension
  IDs (`pivra-color-stock/candidate`), GTypes (`PivraStock/CandidateMediaFile`,
  paintable and sink) and exported symbols. This prevents both GType collisions
  and ELF symbol binding to the installed backend's implementation.
- A shared **test-only** Rust helper explicitly scans the private directory with
  public GIO APIs after GTK initialization, verifies the named extension/type,
  and checks the factory-created MediaFile's actual type. The loaded file path,
  module SHA-256, instantiated implementation marker and source texture are logged.
- Missing/incorrect selection, absent provenance or DMA-BUF/GL/unknown source
  storage returns **NOT RUN / 77 before CPU pixel expectations**. Real decode,
  seeking and CPU pixel failures remain FAIL; assertions were not removed.
- `video_color_window` reuses production `app`, window, loader, controls and
  navigation code, with this helper called before application launch. It verifies
  an empty MediaFile factory object (no extra decode) before showing the window.
  The installed `pivra` binary and its default backend are unchanged.
- Color controls use matched CPU RGB transport in **all three private variants**
  (`PIVRA_GTK_COLOR_CPU_TRANSPORT=1`), while the correction flag remains separate.
  Only the sink-capability/GL-wrapper negotiation code gets this transport flag;
  the color-conversion predicate is unchanged. Thus opt-out still tests the
  original color policy, without asserting exact equality across GPU and CPU
  YUV rounding or relying on old YUV DMA-BUF import semantics. The installed
  native backend is captured separately, before fixture controls, without either
  test-module selection or the CPU transport override.

Fresh local results with the original GStreamer module also available via
`GTK_PATH`: **12 real H.264/control runs PASS on X11 and 12 on Sway**, selected
`PivraCandidateMediaFile`, CPU memory and linear CICP transfer 8 on eligible
converted frames. **13 harness unit tests PASS** (mocked output, not decoder
proof), including stale native-class selection, missing markers, DMA-BUF/GL/mixed
storage, real-failure preservation and visual-launch orchestration. A real empty
GIO module-directory test returns NOT RUN/77 without falling back. The normal
production application/window code mapped, negotiated candidate CPU RGB and
closed through the X11 WM with exit 0; that smoke test is not visual acceptance.
Rust fmt/check, zero-warning strict all-target Clippy, 205 Rust tests and release
build also PASS. **Native Arch activation and MPV visual comparison are pending.**

For the final native check, use the single `--visual` command below. It captures
the installed backend first, runs all gates, and opens the production-code window
only after verified candidate activation and successful regressions. Closing a
window does not automatically record visual-color PASS.

## Evidence and root-cause boundary

The user reports native Arch/GNOME `video_color_probe` PASS at 720×1142, maximum
raw MediaFile/ActualPaintable channel difference **0**, plus the synthetic CICP
PASS. These are user-reported results. They exclude the wrapper for that frame;
they do not identify the native texture storage, sink caps or display profile.

The earlier Codex media-module limitation was removed by extracting the official
Debian `libgtk-4-media-gstreamer` 4.18.6+ds-2 into an isolated user directory.
SHA-256: `74b69f1d116e108944311dccadb1289df5897a9e4b8abf817f665a52c66773fe`.
Additional official development headers were checksum-verified and extracted
into a separate SDK overlay. No system package/library was replaced or installed.
Local runs use GTK 4.18.6, GStreamer 1.26.2, software H.264 decode, Debian 13.

Actual accepted sink caps for the tagged limited-range fixture:

```text
video/x-raw, width=192, height=96, format=RGB, colorimetry=1:1:5:1
```

That GStreamer tuple means full-range RGB matrix, BT.709 transfer and primaries.
The actual source texture is `GdkMemoryTexture` (CPU memory), with GDK CICP
`primaries=1 transfer=1 matrix=0 range=Full`. These are accepted caps and inspected
source textures, not advertised alternatives or downloaded/rendered textures.
GL and DMA-BUF capability advertisement is not proof of use. The enhanced probe
prints source texture types and CICP before the renderer can convert them;
unhandled node/storage types are reported as unknown.

Limited-range black reaches RGB **0** in the properly tagged real H.264 fixture.
The CPU range test also distinguishes limited and full YUV correctly. Thus a
blanket extra 16–235 expansion on Pivra snapshots would be incorrect.

There is a reproducible **display transfer interpretation difference**:

- GTK's sink copies `GstVideoColorimetry.transfer` into GDK CICP. GDK 4.18.6 treats
  transfer 1 using inverse BT.709 encoding (`bt709_eotf`).
- libplacebo v7.349.0 `pl_transfer_from_av(AVCOL_TRC_BT709)` selects
  `PL_COLOR_TRC_BT_1886`, explicitly distinguishing EOTF from OETF.
- BT.1886 with ideal display black 0 and normalized white 1 reduces to
  `L = V^2.4`. This is a defined reference-display interpretation, not a freely
  chosen gamma/contrast/saturation correction. Real display black, ICC profile,
  output luminance and MPV configuration can still affect final appearance.

The sources are linked in [the initial investigation](VIDEO_COLOR_ISSUE_9.md).
This mechanism is confirmed in the controlled GTK backend. Its exact contribution
to the user's Arch rendering is not yet confirmed without native caps/storage
and the final visual comparison. No claim is made that the Arch GTK version has
identical internals or that all washed-out-video reports share this cause.

## Executable backend patch

`tests/video_color/gtk-4.18.6-bt1886-cpu.patch` modifies the actual GTK media module,
not Pivra's paintable, zoom, snapshot geometry or UI. It is deliberately **opt-in**
with the experimental process environment `PIVRA_GTK_BT1886_CPU=1`.

The CPU reference path:

1. Negotiates CPU RGB8, so GStreamer performs YUV matrix/range conversion first.
2. Accepts only explicit BT.709 primaries **and** transfer, RGB matrix, full RGB
   range and RGB8 format. Unknown, sRGB, PQ, HLG, other primaries, YUV and limited
   RGB are not reinterpreted by the conversion function.
   “Explicit” here means present in negotiated `GstVideoInfo`: a decoder can
   infer defaults for an untagged source, and this API does not expose that
   provenance. The unknown-metadata fixture does not prove every untagged file
   stays unknown on every decoder.
3. Applies the ideal-black BT.1886 EOTF via an immutable, thread-safe 256-entry LUT.
4. Supplies float32 RGBA to GDK, labelled **linear sRGB**, preserving precision
   instead of relabelling the original nonlinear samples or applying a GSK filter.
5. Releases the mapped input after constructing the output. Checked allocation
   multiplication, a 256 MiB per-output-frame ceiling and allocation-failure
   reporting prevent an unbounded individual output buffer. This is not a total
   process/decoder RSS limit. 720×1142 needs 13,155,840 additional bytes per frame;
   queued/current frames and decoder allocations are separate costs.

With the environment option absent, the patched module retains the original
negotiation and texture path. The runner verifies color-policy opt-out pixel equivalence on matched CPU
transport; it does not claim CPU/GPU pixel equivalence.

**Not enabled in Pivra's build/install or packaging.** The opt-in reference forces
CPU/RGB negotiation for the process, including non-BT.709 streams; the conversion
is metadata-selective but the negotiation experiment is not a universal backend.
It must not become the default: GPU zero-copy, alpha/HDR precision, 4K/60 FPS cost,
RSS, latency and power have not been accepted. There is no GL download shim,
private ABI pointer mutation, new decoder or system-wide override.

## Reproducible real-decode tests

The runner pins all seven GTK 4.18.6 source/header SHA-256 values, retains upstream
source/license notices, applies the patch with zero fuzz, then adds unique
identities and matched test-only CPU transport. It builds original-color-policy
and candidate modules in a private retained directory. It does not install modules
or edit user media/configuration. Sources are downloaded unless a verified local
cache is supplied. Only existing native GTK/GStreamer libraries are linked.

Four generated 192×96 H.264 fixtures have independently checked ffprobe metadata:
BT.709 limited, BT.709 full, sRGB and unknown transfer. Three executions per
fixture compare original, candidate-disabled and candidate-enabled backends.
Each execution checks real frame snapshots, exact raw/wrapped equality,
play/pause/loop/volume/mute state, and an actual completed midpoint seek with
seeking transitions and timestamp verification. The fixtures have no audio;
these tests do not establish audible output, physical keyboard or UI interaction.

Measured sRGB neutral-patch channels on both local display backends:

| Real H.264 metadata | Original-policy CPU | Candidate enabled |
| --- | --- | --- |
| BT.709 limited | 0 / 140 / 253 | 0 / 121 / 253 |
| BT.709 full | 31 / 140 / 237 | 4 / 121 / 234 |
| sRGB | unchanged | unchanged |
| Unknown transfer | unchanged | unchanged |

Full-range black-code 16 remains a nonzero tone; it is not double-expanded as
limited range. Integer conversion tolerances are bounded in the assertions.
The unknown case is preserved, not declared color-correct. Source/decoder loss
of color metadata is a separate possible failure and is not hidden by fallback.

- **PASS X11/Xvfb:** 12 real-decode/control executions, all four fixture groups.
- **PASS headless Sway/Wayland:** the same 12 executions; Cairo renderer,
  no XWayland fallback. These are not physical GNOME tests.
- **PASS C policy regression:** all 256 channel values, monotonicity/endpoints,
  metadata/range/format exclusions including PQ/HLG/sRGB, checked multiplication,
  zero-size/overflow rejection and the exact frame-allocation ceiling.
- **PASS Rust:** fmt, `cargo check --locked`, strict all-target Clippy (zero
  warnings), `CARGO_INCREMENTAL=0 cargo test --locked` (**205 tests**), release build.
- **PASS existing controlled GStreamer CPU limited/full range regression.**
- **NOT RUN:** physical Arch GPU/GL/DMA-BUF, audible audio, native keyboard/UI
  navigation, actual MPV visual match and high-resolution performance acceptance.

The new C policy test uses `-Wall -Wextra -Werror`. Building the unchanged upstream
module exposes two existing signedness warnings; they remain visible rather
than being described as fixed. Its callback unused-parameter warnings are
excluded. Rust strict Clippy is independently warning-free.

## One native command

Run from the updated repository in the existing GNOME session:

```bash
GDK_BACKEND=wayland python3 tests/video_color/backend_probe.py \
  --video "/absolute/path/to/the/problematic.mp4" --visual
```

Requires the existing Rust/C toolchain, Python, patch, network/source cache,
FFmpeg with libx264, ffprobe and the actual development modules reported by:

```bash
pkg-config --modversion gtk4 gstreamer-player-1.0 gstreamer-gl-1.0 gstreamer-allocators-1.0
```

Missing tools/modules are NOT RUN/77; compile, checksum, decode or assertion errors
are failures. The runner prints the retained directory and accepted caps/source
texture for the **installed native backend** on the user's file, then the
explicitly loaded candidate. With `--visual` it builds/opens the test-only
`video_color_window` example using production application/window code and the
verified private module. Compare with MPV on the same file/display/profile,
and recheck seeking, audio, keys and navigation. This supersedes the previous
`GTK_PATH`-only launch instructions, which failed to activate the Arch experiment. The patch
can be compiled against a newer installed public GTK API, but that configuration
has not been tested here. It is an experimental 4.18.6 media-module source, not a
patched copy of the user's unknown GTK implementation.

No PASS from this tool means native visual acceptance. Do not copy the module
into `/usr`, export its environment globally or package it as the release fix.

## Concrete production/upstream strategy

The proper integration boundary is GTK/GDK, since MediaFile's public API does
not expose the negotiated GStreamer sink, source colorimetry or a display-EOTF
setter. Changing a global GDK CICP transfer-1 implementation would also change
image color interpretation and is not the proposed fix.

1. Add/use a **distinct video display-EOTF color state** in GDK, representing
   BT.1886 separately from inverse BT.709 encoding. CPU conversion, state equality,
   serialization/ICC/CICP reporting and GPU shader conversion must agree. Do not
   pretend a custom state is ordinary CICP transfer 1 or poke opaque structs.
2. In `modules/media/gtkgstsink.c::gtk_gst_color_state_from_colorimetry()`, select
   that state only for the appropriate explicit SDR video metadata. Preserve
   primaries and already-negotiated matrix/range; do not assume YUV DMA-BUF is RGB.
3. Use that state consistently at the existing memory, GL and DMA-BUF texture
   builders, eliminating this reference patch's forced-CPU negotiation and
   float32 frame copy. No Pivra control/MediaStream replacement is required.
4. Validate real RGB/YUV GL and DMA-BUF imports against the same near-black,
   grayscale, color-bar, full/limited and non-BT.709 controls, then check real
   display profiles, playback timing, memory and native MPV/Pivra appearance.

The included working CPU patch provides an executable reference for this backend
change. A production GPU fix or wholesale backend replacement cannot be claimed
safe from the current evidence. The next acceptance needs the user's installed
sink/storage evidence and the normal-Pivra visual result, not another wrapper test.
