# Pivra 0.1.0 — release scope and preparation notes

Updated: 2026-10-29 (Europe/Istanbul). Accepted RC application/package source:
`7e26e780acffdb1f2db0f37e8cf6485eba0baa63`; original B5 package source was
`e8fc9ec011ad6759bc6eef018b9a6f71465f80f1`.
**B6-F2: technical GO for this exact RC; no publication authorization.**
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
  RC `.SRCINFO` generation/consistency and clean build/install/integrity are
  now user-reported PASS in F2. Permission for main/tag/publication remains
  separate; a different source requires a fresh checksum/metadata review. The user reports Nautilus JPEG double-click and PNG Open With
  rendering PASS. Pivra is the JPEG default; Loupe remains the PNG default.
- One unreproduced `Preload exceeds cache entry budget` during main-image
  loading remains OPEN OBSERVATION, with no confirmed cause or fix. The owner
  explicitly accepts it as a documented v0.1.0 release risk; not a blocker for
  this version by that decision, and not authorization to publish.
- B6-W1 closes the strict all-target Clippy warning gate without changing
  product behavior. One Sway zoom/pan pixel check failed once and passed an
  unchanged isolated rerun; both results remain recorded, with no cause/fix claim.
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

Original B6 Codex checks: 197 Rust tests; Python thumbnail 52, observer/report
33 and memory-driver 9, with the then-failing strict lint gate. B6-W1 reruns:
197 Rust, thumbnail 53, observer/report 33, memory-driver 9 PASS; fmt/check and
strict all-target Clippy PASS with zero warnings. Four controlled GUI gates pass
on X11; Sway gates pass with one initial zoom/pan FAIL and successful unchanged
isolated rerun. Complete commands, warning fixes and limits are in the B6 audit.
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


## B6-F1 native desktop acceptance

Received 2026-10-29 as user-reported Arch/GNOME results: Pivra is registered for
JPEG/PNG, selected as JPEG default; Loupe remains PNG default. Nautilus JPEG
double-click and PNG Open With → Pivra render real images: PASS. Existing B5
package install/remove and W1 strict all-target Clippy (zero warnings) are
reported PASS. The new physical-test message does not specify a binary SHA;
B5's earlier source remains pinned to `e8fc9ec`, not silently reassigned.
Codex did not perform these physical tests or change MIME associations.
The RC is preparatory only; final source/risk approval and publication are
separate. The preload observation remains open and unfixed; the owner explicitly accepts
it as a documented v0.1.0 release risk (2026-10-29), without publication approval.


F1 fresh checks on unchanged `eabc0e5` application code: fmt/check/strict Clippy
PASS with zero warnings; 197 Rust and Python 53 + 33 + 9 PASS. Four controlled
GUI gates PASS on both X11/Xvfb and Sway, all first F1 attempts. Physical tests
remain user-reported; makepkg/.SRCINFO for the new candidate are NOT RUN in
Codex. No release/tag/main/AUR/CI action is authorized by these results.


## Frozen F1 candidate reference

Reviewed dev RC: `7e26e780acffdb1f2db0f37e8cf6485eba0baa63` (same application
code as W1 `eabc0e5`, including subsequent acceptance/risk documentation).
Actual source archive SHA-256:
`828a97503173f4fef568345e2d8b2334fcf35b6b587928692ac2261ef3b68277`.
All 211 tracked files/modes and the manifest were verified against Git.
[RC receipt and native metadata commands](PIVRA_V0.1_RELEASE_CANDIDATE.md).
Private RC source config exists outside the repo; `.SRCINFO` generation and
consistency remain NOT RUN (no makepkg here). Later receipt documentation
commits are not silently substituted as this candidate source. Final main/tag/
publication approval remains separate; B6 PARTIAL. Preload is explicitly
accepted as a v0.1.0 risk, open/unfixed. Nautilus acceptance is user-reported PASS.


## B6-F2 — native accepted RC and technical GO

User-reported actual Arch/GNOME at `7e26e780acffdb1f2db0f37e8cf6485eba0baa63`:
source verification (`--verifysource --force`) PASS for both files;
`--printsrcinfo` exit0 and exact 0.1.0-1/full-SHA URL/two hashes PASS;
`--cleanbuild --force` exit0. Package `pivra-0.1.0-1-x86_64.pkg.tar.zst`
approximately 22 MB, installed with pacman PASS, `Pivra 0.1.0`,
`pacman -Qkk`: 1309 files, 0 altered. Problematic JPEG, complete thumbnail
loading, navigation, window dimensions/maximize and prior Nautilus acceptance
PASS. This is not independently executed Codex evidence or a new removal test.

Archive SHA-256 remains
`828a97503173f4fef568345e2d8b2334fcf35b6b587928692ac2261ef3b68277`.
Expected collector hash remains
`e0bad72c99f4629b0668f59450e53a75022ea746efe57191ab8da34fb5cf1201`.
Strict Clippy zero warnings, Rust197/Python53+33+9 and eight controlled GUI
passes are retained verified F1 results for unchanged code, not rerun in F2.
Source/license/recipe consistency and the retained archive hash were rechecked.

**Recommendation: technical GO.** `Preload exceeds cache entry budget` remains
**OPEN — explicitly accepted by the owner as a documented v0.1.0 release risk**,
not fixed. Prior W1 transient Sway observation and untested platforms remain
recorded. This is no authorization to merge main, tag or publish GitHub/AUR.
License/notices/corresponding-source duties still apply. Later documentation
commits do not replace the accepted RC; different source needs new metadata.
