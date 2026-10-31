# Pivra v0.2.0 — Release Notes

Release date: 2026-10-31 (pending publication)

## Highlights

- Navigate mixed photo and video folders through one ordered media catalog and thumbnail strip.
- Play videos through GTK's native media backend with playback/pause, seek, volume/mute, looping, keyboard controls, and remembered playback position within a window.
- Play animated GIF, WebP, and APNG files with animation controls.
- Move selected media to Trash and restore with Undo.
- Improve thumbnail cache freshness after Trash/Undo, thumbnail strip scheduling, and media transitions.
- Support still images up to 64 MP, with a maximum 32,768-pixel edge and existing memory/input bounds.
- Refine fullscreen navigation and native translucent thumbnail/navigation overlays.

## Compatibility

Linux desktop with GTK 4.16+, libadwaita 1.2+, Rust 1.85+ for source builds, Glycin image loaders, and system video codecs/backend. Available media formats depend on installed loaders/codecs. Arch Linux is the primary verified build environment; broader distribution-specific testing is still needed.

## Known limitations

- No separate editing mode, batch conversion, slideshow or advanced comparison tools yet.
- Subtitle support, timeline hover previews, ICC/HDR workflows, and other advanced video/image tools remain planned.
- Video playback depends on the system's GTK/GStreamer backend and codecs. Rapid video-to-video navigation should be included in release smoke testing; historical crashes have been investigated but not fully certified across systems.
- Secure decode and memory limits may reject oversized or malformed files.

## Validation

Before publication, the development branch passed local Rust tests, formatting, Clippy with warnings denied, and a release build. Manual multimedia smoke testing and packaging validation remain release gates.

## Distribution

After tagging `v0.2.0`, update the AUR `pivra` PKGBUILD to the immutable tagged source commit, recompute SHA-256, regenerate `.SRCINFO`, and publish package version `0.2.0-1`. Preserve `options=('!debug')` so optional debug packages are not generated automatically.
