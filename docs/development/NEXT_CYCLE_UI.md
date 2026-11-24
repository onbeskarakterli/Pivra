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
