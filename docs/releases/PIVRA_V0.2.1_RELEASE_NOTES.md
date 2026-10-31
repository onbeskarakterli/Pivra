# Pivra v0.2.1

Pivra 0.2.1 is a maintenance update for the native GNOME photo and video viewer.

## Changes since v0.2.0

- Remove the obsolete “Video playback is not available yet” subtitle displayed for videos, even though playback was working.
- Synchronize Rust formatting and the Cargo lockfile with the package version.
- Make CLI version tests read the package version automatically instead of relying on hard-coded version text.

## Existing features

- Browse photos and videos together using the media catalog and thumbnail strip.
- Control video playback, seeking, volume, and mute.
- Play animated GIF, WebP, and APNG images.
- Move media to Trash and restore it with Undo.
- View supported images up to 64 MP, within memory, file-size, and decoder limits.

## Compatibility and limitations

Linux with GTK4, libadwaita, Glycin loaders, and suitable media backends/codecs. Arch Linux remains the primary tested environment. Editing, batch conversion, subtitles, slideshows, and advanced HDR workflows are not included in this release.

See the [README](https://github.com/onbeskarakterli/Pivra) for installation and usage instructions.
