# Post-v0.2.7 viewer improvements

**Unreleased development on `dev`, after the published stable v0.2.7.** The source version remains 0.2.7; none of the additional features below is included in that published tag. No release preparation.

Stable installation and release scope: [README](../../README.md) and [v0.2.7 release notes](../releases/PIVRA_V0.2.7_RELEASE_NOTES.md). In particular, the empty-window Open fix below was implemented **after** publication; the limitation remains documented in the historical release notes.

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

## Media information, F2, type filters and video inhibition

- Information reuses the existing asynchronous panel/metadata cache, now grouped
  with native libadwaita preferences rows. Available camera/ISO/aperture/shutter/
  original capture-date fields come from already loaded EXIF; codec/FPS from the
  existing discoverer; dimensions/duration from the live frame/MediaFile. No new
  dependency, directory scan or decode is requested for the panel.
- F2 opens a native rename dialog; the stem is selected, leaving the extension
  intact by default. Validation rejects empty/path/control-character/oversized
  names. An anchored same-directory Linux `RENAME_NOREPLACE` rejects collisions
  atomically without a copy/delete/overwrite fallback. Source identity is checked;
  UTF-8 regular files are supported (symlinks should be renamed in the file
  manager). Catalog sorting/selection, title, metadata and thumbnails are updated.
  Ready photo/video rendering is retained; unfinished image requests reuse the
  existing retarget path. Filesystem errors use the existing toast mechanism.
- The thumbnail controls contain All/Photos/Videos. This is a window-local
  projection: the main media and full keyboard catalog stay unchanged. The
  active thumbnail follows the same type filter as every other thumbnail; an
  excluded active item leaves no visible selection. Natural order and original
  catalog indices remain;
  cells/textures are retained under the existing 64-entry/16 MiB policy. Hidden
  types do not start new thumbnail jobs. Filter state survives Open, Rename and
  folder reconciliation without a filter-triggered rescan.
- Video playback owns a separate GTK Application IDLE-inhibit lease. Only a
  prepared, playing, non-ended, non-failed video qualifies. Pause, switch, EOS,
  error and close release the cookie. Late/reentrant acquisition is released if
  obsolete; an unsupported service is retried only on a new playing session.
  Slideshow's existing inhibition, including pause, is unchanged.

Validation: formatting/locked compilation PASS after each feature. Small library
checks: rename safety 2 PASS (names, collisions, replacement identity, symlinks),
rename catalog state 1 PASS, media filter order/current-selection 1 PASS, inhibit
cookie lifecycle 2 PASS. No full Rust/Python/GUI matrix, stress/performance loop,
release build or packaging was run. Version stays 0.2.7.

Physical GNOME acceptance remains **NOT TESTED**: inspect available EXIF/video
fields and path wrapping; F2 cancel/collision/rename during navigation and video;
filter All/Photos/Videos across Open/Rename/folder changes; verify real session
inhibitors on play/pause/EOS/switch/close and that paused slideshow still inhibits.
GPU rendering, zoom/minimap, Delete/Undo and existing keyboard behavior require
combined native acceptance; no hardware success is claimed by these unit checks.

Final rename review: O_PATH directory/source handles inspect identity without
opening an unexpected FIFO/device stream. Deferred GPU→stock recovery uses the
current video path after an F2 rename, without changing rendering or routing.
The two filesystem safety tests were repeated for the descriptor change; PASS.

Thumbnail filter correction: removed the former `selected || type_matches`
exception. The viewport/catalog selection stays intact, but an excluded current
thumbnail is hidden like all other non-matching cells. Hidden selected entries
also cannot anchor strip alignment or receive selected-item hidden warming.
Formatting, locked compilation and the single focused
`media_filter::tests::filtering_active_photo_or_video_never_changes_viewer_selection`
regression PASS (both photo→Videos and video→Photos, All restore/natural order).
Physical GNOME visual confirmation remains pending.


## Native viewport context menu (unreleased)

Right-clicking the photo/video viewport opens a native Gio/GTK popover. The
window/context_menu module routes Rename, Zoom, display mode, fullscreen,
Start/Stop Slideshow, Media Info and Trash to their existing owners. Thumbnail
Strip reflects the saved shared preference, independent of minimap suppression;
its pin state is retained. Videos additionally offer Play / Pause through the
existing playback controls. Empty windows have no media menu. Menu state follows
existing actions/toggles; navigation dismisses an old popup. Only button 3 opens
it, leaving left-button panning and the separate minimap unchanged.

Open With uses GTK FileLauncher with always-ask application selection; Show in
Files uses its containing-folder integration. Neither changes a default file
association. Operations are asynchronous, cancellable on window close and report
non-cancellation errors through existing toasts. Controller closures hold weak
viewer/action references to avoid retaining the Picture through its own handler.

Validation: formatting and locked compilation PASS. The focused Xvfb context-menu
gate covers empty suppression, photo/video menu entries/order, live zoom and
saved thumbnail state, Escape dismissal without closing the viewer, and close.
Photo pixels are controlled; no native decoder/GPU acceptance is claimed.
Physical GNOME remains NOT TESTED: application chooser/cancel and launching an
installed application, file-manager reveal/selection, menu pointer placement,
checkmarks, slideshow, minimap/pan, rename and Trash/Undo integration.

## Centered Open File entry point (unreleased)

The far-left header Open button is removed. An AdwStatusPage in the content area
below the header provides a folder icon, “Open a photo or video to get started”
and a suggested Open File button. The small window/empty_state module observes
a read-only presentation action updated by the existing selected-catalog/title
path; it switches between the empty page and the original viewer row. It does
not create a second viewer or replace folder monitoring, admission or rendering.
The page disappears on selection and returns when the catalog has no selection.

Open File activates the existing win.choose-media action. Ctrl+O still uses that
same chooser in both empty and active windows; Open With remains a separate
current-file/application-launch operation in the context menu.

Validation: formatting and locked compilation PASS. Focused Xvfb Open gate PASS
(button opens chooser, cancel preserves empty state, accepted file/URI uses the
same viewer/window, replacement folder monitoring/navigation). Existing shortcuts
gate PASS with real Ctrl+O, cancel in empty/active states and local chooser file
selection. The updated layout gate PASS: the native empty-state content is
centered at (0,0) error in a 950×583 content viewport; portrait Fit, minimap,
scrollbar clearance, sidebar, resize and fullscreen checks remain passing.
These are controlled photo pixels and SDK GTK/file chooser tests; physical
GNOME/portal presentation, secure decode and GPU acceptance were not rerun.
No full regression suite, release build, packaging or version change was done.

## Context-menu dismissal and cascading submenu correction

At `d6f54fe`, the menu unconditionally popped down on `notify::title`.
FolderWatch reconciles every five seconds and calls update_title even for an
unchanged selection. GTK's set_title emits notify even for identical text.
A focused reproduction on that unmodified implementation failed with
“unchanged selection/title refresh dismissed context menu”. No pointer leave
was necessary. The menu now tracks selected-file URI identity through the
existing title/state update path; only a real selection change dismisses an
obsolete menu. Repeated popup requests while open do not rebuild the model;
slideshow state changes replace only their own label row.

GTK supports native NESTED menus, but its model-button hover delay is internal
(80 ms in the reviewed GTK 4.24.1 source), without a public setting for the
requested 200 ms. Display Mode therefore uses the supported custom-menu slot:
a GtkMenuButton owns a separate GtkPopoverMenu/GdkPopup, positioned right with
GTK/GDK screen-edge fallback. It retains the existing Fit/Fill/100% Gio actions
and radio state. A single 200 ms hover source is cancelled on leave/unmap; leave
never hides an already open child. There is no polling, standalone window,
private GTK widget manipulation, positioning offset or CSS submenu workaround.

Child Escape/Left returns focus to the parent; Right and click open the child.
The root does not cascade-close merely because the child returns focus. Mode
action activation closes the whole chain, including GTK's native order where
the child is already hidden before action dispatch. Closing/unmapping the root
hides the child, cancels hover and disconnects its temporary action handler when
the row is destroyed. File actions, ordering, right-button ownership, panning,
minimap, empty-state Open File and underlying media actions are unchanged.

Validation: fmt and locked check PASS. The single focused X11/Xvfb context-menu
gate PASS: unchanged-title/repeated-popup stability, all photo/video entries,
200 ms one-shot hover, distinct parent/child native surfaces, emitted GTK
enter/leave without closure, real Escape/Right/Left and Down/Return activation,
whole-chain cleanup, zoom and saved thumbnail preference state. The pre-fix
failure is retained above; no assertions were removed. Photo pixels remain
controlled; this is not secure decode or hardware acceptance.

Physical GNOME pointer travel, diagonal crossings, outside-click behavior,
screen-edge left placement, theme/accessibility presentation and external
application/file-manager integration remain NOT TESTED here. Placement is
subject to the compositor's popup constraints; no absolute-side guarantee is
made for a screen without sufficient room. No full regression matrix, packaging,
release build, version or publication changes were performed.

## Shared context rows and popup ownership follow-up

The previous mixed renderer combined GtkModelButton's shared leading indicator
size group with a separate GtkMenuButton. Checked actions reserved leading
space for unrelated labels; the Display Mode label also inherited a different
button font. The actual old GTK tree showed Sans 13.333px for ordinary model
labels and Sans Bold 13.333px for Display Mode. These were separate control and
layout policies, not an image viewport or media transform issue.

The renderer now uses one small rows helper: native GtkButtons, a shared label
layout and a 16-logical-pixel trailing indicator. Unchecked indicators use zero
opacity without removing their allocation. Menu classes and a common normal
text weight retain the theme's font family/size; no item-specific CSS or offset
compensation is applied. Root and Fit/Fill/100% rows use the same construction,
explicit accessible menu/check/radio roles and existing Gio action bindings.

A single Menus lifecycle controller owns a root GtkPopover and at most one
child transient GtkPopover through weak references. Every unrelated root row's
pointer/focus entry closes the current child and cancels pending hover. Leaving
Display Mode only cancels a pending open, preserving an already open child while
crossing the gap or returning to its parent. The one-shot 200 ms open uses a
generation check; unmap/close invalidates stale callbacks. Window inactivity,
compositor toplevel focus loss/minimization, suspension, hide/unmap and destroy
close the entire chain. Popups remain children of the correct media widget/row;
no application windows or explicit global input grabs are used for menus.

The old ownership was already a GdkPopup/widget hierarchy, but it lacked owning
window activation/minimization observers. Closing only on selected-media change
and close-request was insufficient when switching windows. On X11, an owned
popup's native grab can temporarily make GtkWindow::is_active false; actual
external deactivation is distinguished using the owning GdkToplevel FOCUSED
state and its event notifications. No polling, idle retry or focus-close timer
is used. Other GTK windows do not inherit or keep the menu above them.

The selected-file URI/five-second-title-refresh fix and all existing file,
zoom, fullscreen, slideshow, thumbnail and playback action-handler bodies were
compared against ee4af27 and remain unchanged. The empty-state Open File code,
main window builder, GPU engine, media operations and preferences are untouched.
Only menu presentation/lifetime and its focused test changed.

Validation: fmt and locked compilation PASS without warnings. The same focused
X11/Xvfb/xfwm4 context-menu gate PASS with real GTK objects: action order and
photo/video entries, shaped Pango normal fonts, equal row heights/label starts,
constant checked/unchecked geometry, available check icon, 200 ms hover and
cancellation past its deadline, sibling entry versus parent/child gap, native
popup surface ownership, real keyboard selection/Escape/Left/Right, unchanged
title/reopen stability, zoom/preference state, real window-manager focus transfer
to another GTK window and owner-hide cleanup. Measured rows: label x=0 relative
to the button, 24 logical pixels high, Noto Sans 9.999 in this SDK/theme.

Photo pixels are controlled and this is not secure decoder/GPU acceptance.
Physical GNOME pointer paths, native Wayland window switching/minimization,
screen-edge placement, outside clicks and assistive-technology presentation
remain for user acceptance. GTK/GDK/compositor popup constraints still determine
edge placement. No full regression suite, release build or packaging was run.
