# Post-v0.2.7 viewer improvements

Development on `dev`; version remains 0.2.7. No release preparation.

## Empty-window Open

Open now uses the same GTK FileDialog and `open-file` URI action in empty and
active windows. Each window initializes one viewer/context; empty controls stay
hidden until admission. Selecting a file stops the old watcher/slideshow, reuses
the viewer and installs the new folder monitor. Cancel leaves selection intact;
close/destroy cancels an outstanding chooser.

Validation: formatting and locked compilation PASS. Controlled GTK empty/open/
reopen/same-window/new-folder-monitor/navigation gate PASS on X11. Actual native
GNOME file-picker selection/cancel and secure decoding: **NOT TESTED** here.

## Optional information panel

The header information toggle / `I` opens a right sidebar, closed by default.
It narrows the viewport instead of covering playback controls. File name/path,
size/date and current texture/MediaFile dimensions/duration are displayed;
unavailable fields are omitted. One coalesced worker and selection epochs prevent
stale results and unbounded metadata jobs.

EXIF comes from the existing secure Glycin load, using its already-locked
`gufo-exif 0.3.1` parser (now a direct dependency; no new crate version/runtime
package). Only bounded text is retained in a 16-entry identity-checked metadata
cache, never frames. Camera make/model, ISO, aperture, focal length, exposure and
orientation are shown when available. Codec/FPS/bitrate are harvested from the
existing video discoverer, not a second probe; forced-stock/cache misses may
omit these optional fields. Opening the panel never starts image decode.

Formatting, locked compilation, metadata identity/bounds test and controlled GTK
file/image/video/rapid-selection/no-extra-decode/discovery gate: PASS. Physical
GNOME, actual secure EXIF-to-panel presentation: **NOT TESTED** here. The GPU
renderer, shaders and XWayland workaround are unchanged.

## Photo minimap

A small top-right overlay appears only when part of the photo is outside the
viewport. It shares the existing texture and Transform; the outline represents
the visible source rectangle. Click/drag pans through the existing paintable.
Zoom, pan, resize and selection update it via the existing frame clock, without
starting a continuous animation clock, another decoder or a pixel copy.
Whole-image Fit, videos and slideshow hide it and release the extra texture
reference. Window unmap/close disconnects the frame observer.

## Final focused validation

- `cargo fmt --all -- --check`: PASS.
- `cargo check --locked`: PASS, no warnings.
- `cargo test --locked --lib media_information::tests`: 1 PASS.
- `cargo test --locked --lib viewing::minimap_tests`: 2 PASS.
- `python3 -B tests/toolbar_ux_smoke.py --session x11 --development all`:
  Open, information and minimap gates all PASS in one Xvfb/xfwm4 session.
  The Open action, watcher, widgets, GIO, rendering and stock test video are real;
  photo pixels use the explicitly controlled loader. This is not secure Glycin
  decode or physical GNOME/AMD acceptance. No full regression matrix was run.

Native GNOME manual check: run `cargo run --locked --`, choose a photo using Open
in the empty window, verify it stays in that window, then reopen a different
folder. Toggle Information (`I`), navigate photos/videos and check available
EXIF/duration plus playback controls. Zoom/pan/resize a photo and drag its minimap;
verify Fit, slideshow and video hide it. Native file-picker interaction, secure
EXIF presentation and visual layout remain **NOT TESTED** in this environment.
