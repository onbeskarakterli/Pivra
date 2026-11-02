# Issue #9 — SDR video color investigation (2026-11-02)

Status: **BLOCKED for a verified product fix; native visual acceptance NOT RUN.**
Reviewed source: `45b4d65418bad0be685d95d414f3ecf09cb3bde6` (`dev`).
Cargo version remains **0.2.4**; the requested v0.2.5 investigation does not bump it.
No production code, dependencies, playback policy, sandbox, or release metadata
was changed. These probes are diagnostics, not a color correction.

## Reported behavior versus reproduced evidence

The user reports H.264, `yuv420p`, TV/limited range, BT.709 primaries/transfer/matrix
looking washed out in both Pivra and direct GStreamer playback compared with MPV
(default settings, software decode). The original video, its decoded frame and
negotiated native sink caps were not supplied. The GitHub issue API request was
forbidden in this environment. These observations are user evidence, not local
visual reproduction.

### Production path

`ImageView::load_video()` constructs `gtk::MediaFile::for_filename()`, attaches the
existing media controls and passes the same media object to
`ActualPaintable::video()`. Its snapshot applies clip, translation and scaling,
then delegates to `media.snapshot()`. It neither downloads/relabels pixels nor
adjusts contrast, saturation, YUV matrix or range. GTK owns the underlying media
backend, decoder negotiation and sink color state. The current public MediaFile
integration does not expose that GStreamer pipeline or negotiated stream color
metadata to Pivra.

This code inspection establishes the boundary; it does not prove that every
native backend/renderer combination is correct. The new optional native probe
compares actual raw MediaFile snapshots with the same frame through the production
ActualPaintable at identical dimensions, with playback paused between snapshots.
A missing backend returns NOT RUN/77; decoder errors, deadlines, missing inputs
and pixel mismatches remain failures. Snapshot allocations are capped at 16 MP.
The probe mutes audio and pauses for measurement; it is not an audio, seeking,
keyboard or navigation acceptance test.

### Limited versus full range: controlled CPU evidence

`python3 tests/video_color_range.py` sends three neutral I420 patches with
Y=16/128/235, U=V=128 through real installed GStreamer `rawvideoparse` and
`videoconvert`, then reads RGB output. It does not decode H.264 or use GTK.

| Input colorimetry | Output RGB (each channel) | Result |
| --- | --- | --- |
| `bt709` (limited) | 0, 128, 253 | PASS; black expands to 0 |
| `1:3:5:1` (full, BT.709 matrix/transfer/primaries) | 16, 128, 235 | PASS; full-range values retained |

The assertions allow at most two code values for integer conversion rounding.
The full-range control distinguishes mistaken limited/full interpretation. This
is evidence against a blanket assertion that GStreamer always fails to expand
limited range. It does **not** establish the actual Arch decoder/sink caps.
Runtime version here: GStreamer 1.26.2, Debian 13, CPU conversion.

### Transfer interpretation: separate, measured mechanism

Inspected upstream GTK **4.18.6**, matching this development SDK:

- [gtkgstsink.c](https://github.com/GNOME/gtk/blob/4.18.6/modules/media/gtkgstsink.c):
  `gtk_gst_color_state_from_colorimetry()` copies primaries and transfer to CICP;
  the RGB texture path sets matrix=0 and full range after upstream conversion.
  This RGB labeling is not itself evidence of incorrectly treating limited YUV
  as full-range YUV. A DMA-BUF/YUV path needs its own negotiated-format evidence.
- [gdkcolorstate.c](https://github.com/GNOME/gtk/blob/4.18.6/gdk/gdkcolorstate.c):
  CICP transfer 1 selects `bt709_eotf`/`bt709_oetf`.
- [libplacebo v7.349.0 libav conversion](https://github.com/haasn/libplacebo/blob/v7.349.0/src/include/libplacebo/utils/libav_internal.h):
  `pl_transfer_from_av(AVCOL_TRC_BT709)` instead selects
  `PL_COLOR_TRC_BT_1886`, explicitly noting “EOTF != OETF”. This is a concrete
  different video-display interpretation, not proof of the user's exact MPV
  build, output profile, renderer or configuration.

The new `video_color_probe` builds real GDK BT.709 RGB textures and downloads them
as sRGB. Expected values use the inverse BT.709 encoding curve followed by sRGB
encoding, not an arbitrary contrast correction:

| BT.709 RGB code | Measured sRGB code |
| --- | --- |
| 0 | 0 |
| 16 | 31 |
| 32 | 48 |
| 64 | 79 |
| 128 | 140 |
| 235 | 237 |
| 255 | 255 |

All three channels and alpha are asserted. Endpoints stay 0 and 255; lifting dark
nonzero tones is distinct from failure to expand video black Y=16. This measured
transfer difference is a strong candidate explanation for lighter shadows,
**not a confirmed complete root cause of issue #9**. Transfer 1 remains correct
metadata for this synthetic RGB test; its successful assertions do not declare
the video presentation policy perceptually correct.

GTK 4.24.1 source could not be retrieved at the attempted upstream tag URL (404).
Do not extrapolate the inspected 4.18.6 implementation to the user's installed
backend without collecting its version/source and negotiated caps. ICC/display
profiles, hardware decoding, GL/DMABUF import and compositor color handling have
not been measured here.

## Executed checks

- `cargo fmt --check`: PASS.
- `cargo check --locked`: PASS.
- `cargo clippy --locked --all-targets -- -D warnings`: PASS, zero warnings.
- `CARGO_INCREMENTAL=0 cargo test --locked`: PASS, **205 tests**, zero failures.
- `cargo build --locked --release`: PASS.
- `python3 tests/video_color_range.py`: PASS, both range cases.
- `video_color_probe` synthetic CICP assertions: PASS on X11/Xvfb and headless
  Sway/Wayland (Cairo renderer); these are not physical GNOME/video tests.
- Native `video_color_probe` against a generated, explicitly TV-range/BT.709
  H.264 clip: **NOT RUN / 77**, `GTK could not find a media module. Check your
  installation.` Installed GStreamer plugins do not supply GTK's MediaFile module.
- Existing `view_modes_probe --synthetic --zoom-pan`: PASS on X11/Xvfb and
  headless Sway/Wayland; secure decode and physical input are not tested by this mode.
- Actual Arch/GNOME video, seeking, audio, keyboard, video navigation, native
  pixel equality and MPV visual comparison: **NOT RUN** here.

## Smallest safe next step

Run on the user's existing native GTK video setup, using the same problematic
file and display session as MPV:

```bash
GDK_BACKEND=wayland cargo run --locked --example video_color_probe -- \
  "/absolute/path/to/problematic.mp4"
```

A PASS only establishes raw-versus-wrapped pixel equality for one paused frame.
It does not establish correct range, metadata, playback controls or appearance.
Samples are printed in explicit sRGB; arbitrary-video sample values have no
reference assertion. For a visual check, run normal Pivra and MPV against the
same file on the same display/profile. No global GTK/GStreamer configuration
changes or color overrides are required by this probe.

If raw and wrapped pixels agree, capture the native GTK/GStreamer versions,
decoder output caps, sink input caps (format, memory type, colorimetry/range), and
renderer's color-state handling before choosing a correction. In particular,
distinguish CPU RGB, GL and DMA-BUF paths. Check whether the native backend uses
inverse BT.709 encoding versus the intended BT.1886 display EOTF.

A targeted upstream GTK sink/transfer fix may be the smallest solution once this
is demonstrated. If an application-controlled GStreamer backend is necessary,
that is a separate, reviewed integration preserving MediaStream controls and
lifecycle, with explicit format/transfer metadata and color reference tests.
Do not replace the current backend speculatively or reinterpret every video as
sRGB/BT.1886: HDR, full-range and other transfer functions would be damaged.
Do not remap 16–235 on an already converted RGB snapshot; that double-expands
range and clips valid data. No color matrix, gamma filter, decoder replacement,
version bump or security relaxation was introduced in this investigation.
