# Pivra v0.2.0

Pivra 0.2.0 expands the native GNOME image viewer into a unified photo, animation, and video browser.

## What's new

- **Mixed media browsing:** Navigate photos and videos together in natural filename order, with a unified thumbnail strip.
- **Video playback:** Native GTK playback with play/pause, seeking, volume and mute controls, looping, keyboard shortcuts, auto-hiding controls, and in-window resume positions.
- **Animated images:** Playback and controls for GIF, animated WebP, and APNG.
- **Trash and Undo:** Move a selected photo or video to Trash and restore it using Undo.
- **Thumbnail improvements:** More reliable image/video thumbnails and refresh behavior, including after restoring a file from Trash.
- **Larger images:** View still images up to 64 megapixels, subject to file size, memory, and maximum-edge limits.
- **Refined interface:** Improvements to fullscreen viewing, navigation overlays, and the resizable thumbnail strip.

## Requirements and compatibility

Pivra targets Linux desktops and integrates with GNOME using GTK4 and libadwaita. Image decoding uses Glycin. Video playback depends on the installed GTK media backend and system codecs; supported file formats can vary by system.

Arch Linux is the primary tested environment. Build instructions for other Linux distributions are provided in the repository.

## Current limitations

Editing, slideshows, batch conversion, advanced comparison, subtitles, timeline hover previews, and HDR workflows are not included in this release. Oversized, malformed, or unsupported media may be rejected by the decoder.

## Installation

- **Arch Linux (AUR):** `yay -S pivra`
- **Other Linux distributions:** Follow the [source-build instructions](https://github.com/onbeskarakterli/Pivra#installation).

Pivra is free software licensed under GPL-3.0-only.
