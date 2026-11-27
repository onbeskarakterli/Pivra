# Pivra v0.2.8 draft release notes

**Release candidate. Native Arch package acceptance and publication are pending.**
The current stable release is [v0.2.7](PIVRA_V0.2.7_RELEASE_NOTES.md).
The candidate source reports version 0.2.8 and the Arch recipe targets 0.2.8-1.

## Implemented on dev

- Empty windows now offer a centered Open File button. Ctrl+O and the context
  menu's Open use the same native chooser and replace media in the same window.
  Open With lists installed applications and launches them through GIO.
  Show in Files asks the file manager to select the current file.
- Right-click opens the media context menu, with a cascading display-mode
  submenu and window-aware dismissal. The menu exposes existing file,
  playback, display and slideshow actions.
- An optional information panel displays available photo EXIF and video
  metadata. F2 renames an individual media file with no-clobber protection.
  The thumbnail strip can show all entries, photos or videos while preserving
  the current viewer selection.
- Zoomed photos can show a draggable minimap. When visible, it temporarily
  hides the thumbnail and arrow overlays. The zoom button reflects temporary
  Ctrl+wheel zoom without persisting that gesture as wheel-zoom mode.
- Slideshow photo clicks advance using the existing clock and ignore drags.
  Slide Left, Right, Up and Down and Zoom In/Out join Fade and No transition.
  Videos continue to advance only on verified completion. While a video
  plays outside slideshow, Pivra can request display-sleep inhibition.
- Release testing fixed rapid slideshow round trips that could reissue a load
  and suppress the planned photo transition.
- Video click controls distinguish a single click for play/pause from a
  double click for fullscreen, including when the video is already fullscreen.

## Evidence and remaining gates

The [validation report](PIVRA_V0.2.8_RC_VALIDATION.md) records 241 Rust tests,
249 executed Python tests and 54 controlled GUI/video cases passing on the
earlier regression source. The owner confirmed 5/5 for the video mouse fix
on physical GNOME. Native Arch build and installation of the 0.2.8 package
remain pending. The validation report lists the scope of the other native checks.

The Open dialog and next-photo latency reports remain open. Measurements and
the paused investigation are recorded in the [Open diagnostics](../diagnostics/OPEN_FILE_LATENCY.md).
Native GTK Wayland/EGL limitations and the XWayland default still apply.
Hardware and format availability depend on the system. The draft contains
no approved source archive, checksum or publication result.
