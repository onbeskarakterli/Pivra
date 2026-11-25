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

A small bottom-right overlay appears only when part of the photo is outside the
viewport, bounded to 560×440 logical pixels with the source aspect ratio.
It shares the existing texture and Transform; the outline represents
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

## Layout regression correction

The closed information Revealer inherited horizontal expansion from its heading.
GTK therefore gave it spare width even at zero reveal progress. At `51c55e6`,
the 950 px content row gave 475 px to the empty viewer and 475 px to the closed
sidebar. The placeholder was displaced by 238 px; portrait Fit was also left
of the full content center. Explicitly disabling expansion on the sidebar root
restores the full viewer allocation, without changing image transforms or adding
position offsets. The existing centered status label and Picture then center
correctly in the content below the header.

The minimap's main Picture propagated the full texture's natural dimensions;
`size_request(160,120)` was a minimum, not a maximum. A controlled 640×1280
portrait texture produced a 609×571 overlay. The texture Picture is now an
unmeasured overlay child, and only the root sets the aspect-fitted allocation
within 180×140 logical pixels (or the smaller viewport minus 12 px margins).
The non-expanding root is bottom/end-aligned with subtle opacity and a thin
indicator. The same portrait now produces a 70×140 map at `(868,431)` inside a
950×583 viewport, with 12 px right/bottom margins. Texture sharing and panning
remain unchanged.

Targeted Xvfb/xfwm4 validation:

- `cargo fmt --all -- --check` and `cargo check --locked`: PASS.
- `python3 -B tests/toolbar_ux_smoke.py --session x11 --development layout`:
  PASS. Reproduced the original layout failure before correction; now verifies
  empty/portrait centering, bounded portrait minimap, sidebar open/close, resize,
  Fill/100% and fullscreen geometry. Content allocations, not assumed window
  decoration dimensions, are the resize barrier.
- `python3 -B tests/toolbar_ux_smoke.py --session x11 --development minimap`:
  PASS. Existing texture/paintable sharing, drag-to-pan, resize, no additional
  decode, Fit/new-photo/video/slideshow suppression remain covered.

These are controlled photo textures and real GTK layout/rendering, not native
Glycin or physical GNOME visual acceptance. Physical GNOME screenshots/interaction
remain **NOT TESTED**, pending user confirmation. No full regression matrix or
packaging checks were run. Version 0.2.7, GPU code and XWayland workaround unchanged.

## Larger minimap and usable viewport

The user subsequently confirmed empty-window centering, Fit centering and the
bounded minimap layout on physical GNOME. This is user-reported acceptance;
the earlier Codex limitations above remain historical test records.

The first enlarged cap was 280×220 logical pixels (portrait 1:2: 110×220), reduced
proportionally when space is limited. Placement uses real GTK allocations and
16 px clearance, not scrollbar thickness assumptions. The current photo viewer
uses Transform panning, not a ScrolledWindow. Its thumbnail strip is a separate
bottom overlay with External scrollbar policy. Visible strip bounds and visible,
mapped scrollbar bounds in the image overlay are excluded from usable space;
hidden obstacles release that space. Only overlay siblings/scroller bars are
inspected, without walking thumbnail cells or starting a new frame clock.
The image transform, viewport indicator coordinates, shared texture, opacity,
thin outline and click/drag behavior are unchanged.

Validation: `cargo fmt --all -- --check`, `cargo check --locked` and both focused
Xvfb/xfwm4 modes (`--development layout`, `--development minimap`) PASS. The layout
gate uses real GTK horizontal/vertical scrollbars of changing thickness, a smaller
window and the visible/resized/hidden thumbnail strip, requiring at least 16 px
clearance from actual bounds. For example, after resize: horizontal bar y=409,
vertical bar x=648, map `(522,173,110,220)` gives 16 px to both. Existing pan,
texture sharing, Fit/video/slideshow suppression, resize and fullscreen checks
remain passing. Photo pixels are controlled, not native decoder evidence.
Physical GNOME acceptance of this larger minimap/clearance change remains pending.
No full regression matrix, packaging or release work was performed.

## Navigation visibility synchronized with the minimap

The image overlay now temporarily masks the entire thumbnail scroller, its
integrated controls/resize edge and both complete navigation arrow buttons while
the minimap is mapped. Its actual map/unmap signals are the single source of
truth. A container-level `child_visible` mask preserves each control's normal
`visible` state, hover/auto-hide logic, enabled and pin preferences. Hover callbacks
cannot override the mask. Unmapping the minimap removes it synchronously, without
a settings write, new timer, CSS hiding or duplicate zoom/mode calculations.

Navigation is excluded from minimap placement even on its first frame, avoiding
a placement jump when it is masked. Other allocated scrollbars retain their
clearance rule. The hidden thumbnail resize gesture cannot claim minimap dragging;
unmapping the strip releases resize state/cursor. Video/slideshow controls and
image transforms are unchanged.

Validation: formatting and `cargo check --locked` PASS. Focused Xvfb/xfwm4
`tests/toolbar_ux_smoke.py --session x11 --development` modes `navigation`,
`layout` and `minimap` all PASS. The new gate exercises real pinned navigation,
hover attempts while masked, repeated zoom/Fit, resize, independent widget
visibility toggling, selection/video/slideshow transitions and disabled/unpinned
restoration. It compares enabled/pin settings bytes and observes map signals plus
rendered frames: zero simultaneous navigation/minimap mappings or frames.
The layout gate now requires navigation suppression rather than the superseded
visible-strip clearance behavior; independent scrollbar checks remain intact.
Controlled photo pixels are not secure decoder or physical GNOME acceptance.
Native GNOME interaction/visual confirmation remains pending. No full regression,
release build or packaging was performed; version remains 0.2.7.

## Doubled minimap cap

The maximum is now 560×440 logical pixels (portrait 1:2: 220×440). Only the
existing size caps changed: source aspect ratio, viewport/scrollbar constraints,
16 px margins, panning and navigation masking are preserved. Formatting, locked
compilation and focused Xvfb `layout`, `minimap` and `navigation` gates PASS,
including smaller-viewport fitting. Physical GNOME confirmation of the new size
is pending. No full regression suite, release build or packaging was run.

## Ctrl+O

The application accelerator `<Control>o` activates the existing window-local
`win.choose-media` action, sharing the Open button's chooser/cancel/completion
path in empty and active windows. No new loading state or viewer is created.
Formatting, locked compilation and focused Xvfb `--development shortcuts` PASS:
real keys open GTK's local chooser, Escape preserves empty/active state, and
selecting a file loads into the original window. The test uses SDK FileChooser
schemas compiled in a temporary directory with a memory GSettings backend.
GNOME/portal-specific physical interaction remains pending; controlled photo
pixels do not claim secure decoder acceptance.

## Temporary manual zoom indicator

The Zoom toggle now reflects the existing `image-mode=manual` state as well as
the user's explicit wheel-zoom preference. The existing blue suggested-action
styling is shared. State-to-button feedback blocks only the input/persistence
handler, so Ctrl+wheel cannot enable persistent wheel zoom or carry it to the
next photo. Preset/reset, replacement-photo and video transitions clear the
temporary manual indication; Fill/100% alone do not activate it. An explicitly
enabled wheel-mode preference still persists as before. `Z` toggles that existing
input preference, even while temporary manual zoom is indicated. No second zoom
flag, rendering/transform calculation or minimap behavior was introduced.

Formatting and locked compilation PASS. Focused Xvfb `--development zoom-state`
PASS with real Ctrl+wheel input: immediate active/blue state, no preference leak,
ordinary wheel navigation after Ctrl release, photo/video/preset resets and
explicit button/Z behavior. The final `shortcuts` and `minimap` gates also PASS
(real chooser interaction/cancel, shared texture/pan and normal suppression).
Physical GNOME/portal and hardware visual confirmation remain pending; photo
pixels are controlled. No full regression, release build or packaging was run.
