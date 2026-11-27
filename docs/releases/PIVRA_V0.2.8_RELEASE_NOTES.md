# Pivra v0.2.8

**Status: Stable release source.** This release improves how Pivra opens,
organizes and presents media in a single viewer window.

## Opening and organizing media

- The empty window has a centered Open File button and a simpler header.
  Ctrl+O, the button and the context menu share the native file chooser and
  open the selection in the current window.
- Right-click a photo or video to open the native context menu. The display
  mode submenu cascades on hover. Consistent rows and window-aware popup
  dismissal make the menu easier to use. The menu includes Open, Open With,
  Show in Files and Rename alongside playback and viewing actions.
- Open With lists installed applications. Show in Files requests the desktop
  file manager to select the current file asynchronously.
- F2 renames the current media file, preserving its extension by default and
  rejecting collisions. The thumbnail strip can filter photos and videos
  while keeping the current media in the viewer.
- The optional information panel shows available photo EXIF and video
  metadata. Press I to open or close it.

## Viewing and playback

- Zoomed photos can show a draggable minimap. The thumbnail strip and
  navigation arrows hide while the minimap is visible. The Zoom button
  also indicates temporary Ctrl+wheel zoom.
- Slideshow adds directional slide and zoom transitions to Fade and No
  transition. Left-clicking a photo advances the slideshow. Dragging leaves
  it selected, and videos advance on completion.
- A single video click plays or pauses. A double-click switches fullscreen
  in either direction without toggling playback. During ordinary video
  playback, Pivra requests display-sleep inhibition where supported.

## Stability and documentation

The release fixes a rapid slideshow navigation sequence that could restart
an image load and suppress a transition. The source separates window input,
folder monitoring, image presentation and thumbnail sizing into focused
modules. The documentation index now links to architecture, testing,
packaging, release procedures and historical records.

## Compatibility and known limitations

Native Arch/GNOME acceptance covers the installed local 0.2.8 package and
physical video interaction. The validation record separates that result
from the earlier regression suite and the checks for the final source.
[Validation record](PIVRA_V0.2.8_RC_VALIDATION.md)

The intermittent Open File and next-photo delays remain unresolved. The
investigation was paused after the delays could not be reproduced reliably.
[Open latency report](../diagnostics/OPEN_FILE_LATENCY.md)

Pivra uses the secure Glycin image loader and the existing OpenGL adapter
for eligible BT.709 SDR video. Codec and animation availability depend on
system providers. Stock GTK Wayland can still hit the EGL lifecycle issue.
Pivra defaults to XWayland where available. GPU acceptance applies to the
tested AMD configuration. [GTK EGL investigation](../upstream/GTK_4.24.1_WAYLAND_EGL_OWNERSHIP.md)

The Arch recipe targets `pivra 0.2.8-1`. Pivra is licensed under GPL-3.0-only.
[Package instructions](../packaging/README.md) · [License](../../LICENSE)
