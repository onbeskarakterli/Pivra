# Pivra 0.1.0 — release scope and preparation notes

Updated: 2026-10-29 (Europe/Istanbul). Audited application/package source:
`e8fc9ec011ad6759bc6eef018b9a6f71465f80f1`.
These are preparation notes on `dev`, not a published release/tag. B4 source
licensing remains PASS. Current evidence and remaining gates are in the
[B6 audit](PIVRA_V0.1_B6_READINESS_AUDIT.md); earlier release-04 evidence used
`be2b9e1a6cbd4880e54b5546bb5ec7727bd25e86`.

## Platform and supported media

Native Linux / GNOME image viewer using Rust, GTK4 and libadwaita.
The current Cargo features require native GTK >= 4.16 and libadwaita >= 1.2;
clean Arch package acceptance at the audited source is user-reported PASS,
not an independently repeated Codex installation.
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
  directional navigation skips to supported JPEG/PNG photos without removing
  catalog entries. At the edge selection stays unchanged; there is no wrap.
  Direct video CLI arguments remain rejected; the window video state, if
  supplied directly, retains its unsupported-playback message.
- Overlay thumbnail strip: scrolling, selection, pinning/resizing, saved strip
  preferences; GNOME-first session selection and Freedesktop when GNOME is
  unavailable/disabled. File errors do not trigger session-provider failover.
- Independent windows, cancellable bounded asynchronous jobs and stale-result
  rejection. Controlled media errors do not stop subsequent navigation.
- Folder monitoring for additions/removals/renames; selected-file changes show
  a Refresh notice. Explicit Refresh, not an automatic reload, reads changes.
- Per-window adjacent-photo preload and bounded image cache (128 MiB / three
  entries, 64 MiB per entry). Thumbnail textures have a separate 64 / 16 MiB
  policy. Accounted bytes are not whole-process RSS/VRAM limits.
- Fullscreen using the existing GTK window; no separate rendering engine.
- Saved normal window dimensions and separate maximize state. Fullscreen
  allocations are not persisted; invalid/oversized saved sizes safely fall back.

## Known limitations and open release gates

- Full-resolution-only pipeline: no genuine early reduced-resolution preview.
- No video playback, image editing or GIF/WebP/APNG animation playback.
  Other format candidates or OS loader capabilities are not product support.
- Main-image admission limits are separate from the opt-in large-thumbnail
  path; a large thumbnail does not prove that its full photo can be opened.
- Clean Arch makepkg, source SHA-256, installed package integrity and
  removal/reinstallation are user-reported PASS at the audited source.
  Final release-source approval, generated .SRCINFO and publication remain
  separate; actual Nautilus integration is not explicitly accepted by this report.
- One unreproduced `Preload exceeds cache entry budget` during main-image
  loading remains OPEN OBSERVATION, with no confirmed cause or fix.
- Normal Clippy passes with warnings; strict all-target `-D warnings` fails on
  existing dead-code/identical-branch warnings (see B6).
- Binary distributions must retain third-party notices and satisfy GPL/LGPL
  source/relinking duties. Source license closure does not complete packaging.

## Secure decoding

The existing Glycin 3.0.8 client uses **Bwrap sandbox selection**, cancellables,
file/content validation, bounded jobs and generation/source identity checks.
There is no NotSandboxed or host-decoder fallback. OS-supplied compatible 2+
loaders and Bubblewrap/user-namespace support are runtime requirements.
The GNOME sidecar has a bounded 2 GiB virtual AS limit; CPU=15s,
FSIZE=256MiB and parent deadline=20s remain unchanged. Failed resource-limit
setup exits before entering GNOME. The B6 task changes documentation only.

## Test evidence and its limits

User-reported **Arch/GNOME Wayland** acceptance at the source above: clean
makepkg, archive checksum, installed integrity, remove/reinstall, thumbnail
stress, scrollbar removal, video skipping, saved window state and the formerly
failing JPEG under the bounded 2 GiB helper. No raw package/hash/transcript was
supplied with this B6 report; attribution is retained rather than inventing it.

Fresh Codex Debian/SDK checks: 197 Rust tests; Python thumbnail 52, observer/report
33 and memory-benchmark 9; fmt/check and normal Clippy PASS (existing warnings).
Four controlled GUI gates pass on both X11/Xvfb and headless Sway. Strict
all-target Clippy fails; complete commands and limits are in the B6 audit.
These GUI paths use synthetic pixels/controlled workers. Physical GNOME,
real secure decode and Arch installation were **NOT RUN by Codex in B6**.
Historical release-03/04 test results remain in their retained records.

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
