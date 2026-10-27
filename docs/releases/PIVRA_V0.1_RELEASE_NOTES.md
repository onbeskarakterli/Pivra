# Pivra 0.1.0 — release scope and preparation notes

Date: 2026-10-27 (Europe/Istanbul). Source baseline:
`be2b9e1a6cbd4880e54b5546bb5ec7727bd25e86`.
This is preparation on `dev`, not a published release or tag. **B4 licensing audit passed for the current source scope**; see the
[dependency license audit](PIVRA_V0.1_DEPENDENCY_LICENSE_AUDIT.md).

## Platform and supported media

Native Linux / GNOME image viewer using Rust, GTK4 and libadwaita.
The current Cargo features require native GTK >= 4.16 and libadwaita >= 1.2;
actual packaging and clean-system requirements need independent validation.
The still-image opening/display scope is **JPEG and PNG**, including local
paths and local file URIs. A candidate's extension is not decoder success;
content type and resource limits are checked by the secure loading path.

## Available features

- Fit, Fill and 100% view modes (F6/F7/F8). 100% is in GTK logical units,
  not necessarily physical display pixels on HiDPI.
- Cursor-anchored zoom and bounded drag pan; Zoom/Z enables wheel zoom.
  Without zoom mode, wheel input navigates media.
- Smooth/Nearest filtering without re-decoding. Filtering preference survives
  photo changes within a window, not application restarts.
- Natural filename ordering, nonrecursive same-folder catalogs and bounded
  previous/next navigation. Catalogs may include video/unsupported candidates;
  those entries show an unavailable state rather than playing them.
- Overlay thumbnail strip: scrolling, selection, pinning/resizing, saved strip
  preferences; GNOME provider priority and Freedesktop fallback.
- Independent windows, cancellable bounded asynchronous jobs and stale-result
  rejection. Controlled media errors do not stop subsequent navigation.
- Folder monitoring for additions/removals/renames; selected-file changes show
  a Refresh notice. Explicit Refresh, not an automatic reload, reads changes.
- Per-window adjacent-photo preload and bounded image cache (128 MiB / three
  entries, 64 MiB per entry). Thumbnail textures have a separate 64 / 16 MiB
  policy. Accounted bytes are not whole-process RSS/VRAM limits.
- Fullscreen using the existing GTK window; no separate rendering engine.

## Known limitations and open release gates

- Full-resolution-only pipeline: no genuine early reduced-resolution preview.
- No video playback, image editing or GIF/WebP/APNG animation playback.
  Other format candidates or OS loader capabilities are not product support.
- Main-image admission limits are separate from the opt-in large-thumbnail
  path; a large thumbnail does not prove that its full photo can be opened.
- Clean native installation, desktop icon/helper packaging, actual Nautilus
  integration and a publishable package/tag remain separate work.
- Binary distributions must retain third-party notices and satisfy GPL/LGPL
  source/relinking duties. Source license closure does not complete packaging.

## Secure decoding

The existing Glycin 3.0.8 client uses **Bwrap sandbox selection**, cancellables,
file/content validation, bounded jobs and generation/source identity checks.
There is no NotSandboxed or host-decoder fallback. OS-supplied compatible 2+
loaders and Bubblewrap/user-namespace support are runtime requirements.
This licensing task changes no decoder, thumbnail provider, cache, preload,
cancellation, viewport or UI behavior.

## Test evidence and its limits

The prior release-03 baseline was tested in Codex Debian 13 with Rust/Cargo
1.99.1, GTK 4.18.6 and libadwaita 1.7.6: 186 Rust tests and controlled GUI
regressions passed under X11/Xvfb and headless Sway 1.10.1. X11's pointer was
verified outside the windows at (1279,1023) from the beginning. These are
historical results, not fresh native-decode acceptance for this document.
The mandatory Rust/metadata/integrity checks are rerun for release-04 and
recorded in the final delivery report.

**NOT RUN in this task:** physical GNOME/Wayland, hardware input/rendering,
real secure Glycin pixel decode and active loader-process cleanup. The known
Codex namespace/proc restriction is not evidence of normal Arch incompatibility.
Older user-reported Arch/GNOME results remain attributed to the user in the
[commit tracker](../PIVRA_COMMIT_TRACKER.md) and retained validation documents;
they are not independent Codex PASS results.

## License

Pivra's selected project license is **GPL-3.0-only**, not GPL-3.0-or-later.
The complete unmodified GNU GPL version 3 is in [LICENSE](../../LICENSE).
Dependencies retain their own grants, notices and source/relinking obligations.
The [audit](PIVRA_V0.1_DEPENDENCY_LICENSE_AUDIT.md) inventories every locked
Cargo dependency and separately discusses native libraries and loader code.
No project copyright owner is inferred, and no release is published here.

## Historical records

[README](../../README.md), [roadmap](../PIVRA_DEVELOPMENT_ROADMAP.md),
[technical capabilities](../PIVRA_TECHNICAL_CAPABILITIES.md),
[release-01 audit](PIVRA_V0.1_FINAL_RELEASE_AUDIT.md) and milestone validation
records remain available; prior open/closed statements describe their dates.
