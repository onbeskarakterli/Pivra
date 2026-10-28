# B6 — Pivra 0.1.0 release readiness audit

Date: 2026-10-29 (Europe/Istanbul). Audited application and latest user-verified
package source: **`e8fc9ec011ad6759bc6eef018b9a6f71465f80f1`** on `dev`.
B6 changes documentation only. No main change, tag, release, AUR upload or CI.

## Decision: PARTIAL; publication remains BLOCKED

The application/package acceptance listed by the user is **PASS, user-reported**.
It is no longer correct to describe those B5 installation checks as unperformed.
Fresh controlled Rust/GUI/packaging consistency checks below also pass, except
for the existing strict warning gate. This audit is complete as an assessment,
not a declaration that every release gate passed or permission to publish.

Release preparation still needs an explicit warning-policy resolution (strict
Clippy is FAIL), approved final immutable source/archive SHA-256 and generated
release metadata. The one-off preload observation must remain visible for a
release-risk decision; it is neither a reproduced confirmed defect nor fixed.
Explicit Nautilus acceptance has not been supplied by this report. No newly
confirmed product crash was found by these checks. PIV-18's wider historical
scope, TH-03/FD native gates and independent phase acceptance are not silently
marked DONE; v0.1.0 is the stated full-only JPEG/PNG scope below.

## Evidence classes and B5 correction

The following were supplied by the user for source `e8fc9ec`, not independently
run by Codex in B6. No raw .pkg archive, archive checksum value, full pacman
output or native transcript was attached to this request; none is fabricated.

| Native acceptance supplied by user | Status |
|---|---|
| Clean Arch makepkg build and source SHA-256 validation | PASS — user report |
| Installed package integrity | PASS — user report |
| Package removal and reinstallation | PASS — user report |
| GNOME/Wayland thumbnail stress | PASS — user report |
| Invisible thumbnail scrollbar/input-area removal | PASS — user report |
| Photo navigation skipping unsupported video | PASS — user report |
| Remembered normal dimensions and maximize state | PASS — user report |
| Formerly failing JPEG with 2 GiB GNOME factory AS | PASS — user report |

**B5 native package acceptance: PASS (user-reported at this exact source).**
This does not mean a main release commit/tag is approved, an AUR package exists,
or bit-identical artifacts were reproduced twice. Actual Nautilus/Open With,
explicit user-data-preservation assertions on removal, 32-bit platforms, Rust
1.85 compilation and independent Codex Arch installation remain **NOT RUN /
not separately evidenced**. Removal/reinstall PASS is not silently expanded
into all those additional claims. The historical B5 NO-GO records are retained
with prominent current-status notices, rather than deleting old evidence.

## Supported scope checked against source

| Area | v0.1.0 behavior / source evidence |
|---|---|
| Files/display | One local JPEG/PNG path or file URI; input validation and content MIME checks; secure Glycin 3.0.8 Bwrap with EXIF transformations (`input.rs`, `image_loader.rs`) |
| Viewing | Fit/Fill/100% in GTK logical units, cursor-anchored zoom, bounded pan, Smooth/Nearest reusing texture (`viewing.rs`, `image_view.rs`) |
| Catalog | Nonrecursive natural order; monitor updates; selected-file changes require explicit Refresh (`discovery.rs`, `natural.rs`, `window.rs`) |
| Navigation | Common window actions for keys/arrows scan to next/previous JPEG/PNG, skip videos/unsupported images, retain catalog, stop at bounds (`catalog.rs::photo_target/select_photo`, `window.rs`) |
| Direct video | CLI remains photo-only and rejects video arguments; explicit window video state remains unavailable-playback. No video engine or video default-app registration |
| Thumbnails | Overlay, selection, wheel/drag scroll, height/pin preferences, visibility handling; GNOME-first session selection. Freedesktop chosen on authoritative unavailable/disabled GNOME, not ordinary per-file errors (`thumbnail_manager.rs`) |
| Scheduling | Busy admission is not a generation failure/retry; true transient failures have at most one retry; permanent errors are terminal (`thumbnail_strip.rs`) |
| Cache/preload | Per-window image accounting 128MiB/3 entries, 64MiB entry cap; separate 64-texture/16MiB strip accounting; cancellable adjacent-photo preload. These are not process RSS/VRAM guarantees |
| Window | Existing-window fullscreen, saved normal size and separate maximize flag; fullscreen allocation not saved; Wayland positioning left to compositor (`fullscreen.rs`, `window_state.rs`) |
| Limitations | Full-resolution-only, no genuine early preview; no video playback, editing, GIF/WebP/APNG animation or broad TIFF/RAW support inferred from installed loaders |

The normal-mode wheel navigates; Zoom/Z enables wheel zoom. Quality preference
survives photo changes within a window, not restart. View-preset/pin/height/
window preferences have their existing individual persistence rules; not every
UI or viewport state is claimed persistent. No source media is written.

## Open observation — NOT CLOSED

**`Preload exceeds cache entry budget` appeared once during main-image loading
in the user's native session; later tests could not reproduce it.**

The exact guard exists at `src/image_loader.rs:137` when the optional preload
budget is exceeded. Its existence explains the string, not why it surfaced in
that particular main-image flow. No reproduction fixture/stack/timing evidence
was supplied for this occurrence. Unit/GUI PASS does not establish its cause
or a fix. No assertion, cache policy, source code, error classification or
retry behavior is changed. Record filename/dimensions, foreground/background
selection, memory pressure and logs if it recurs. Keep it in release notes and
obtain an explicit release-risk disposition; do not label it resolved.

## Fresh Codex execution at the audited source

Debian environment with existing private GTK SDK; Rust/Cargo 1.99.1, GTK4
4.18.6, libadwaita 1.7.6. Activated existing
`/workspace/pivra-environment/piv09/image-activate.sh`; no installation/reset.
The restored checkout was clean on an old `work` branch. It was switched to
`dev`, fast-forwarded from local `f363425` to origin's `e8fc9ec` before auditing.
No audit edits were performed on `work` or `main`.
The stale PIV-12 summary row was aligned to the already-recorded 24.10.2026
user acceptance at `b12970f6a3a7ea2dddc890a747977d5d68437a63`; its original test
history remains. This does not invent a new native PIV-12 run.

| Executed check | Result |
|---|---|
| `cargo fmt --check` | PASS |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 197 tests; 0 failures |
| `cargo check --locked` | PASS |
| `cargo clippy --locked` | PASS, exit 0, existing warnings |
| `cargo clippy --locked --all-targets -- -D warnings` | **FAIL**, exit 101; details below |
| `cargo build --locked --example thumbnail_provider_probe` | PASS; Python suite prerequisite |
| `python3 -B -m unittest discover -s tests/thumbnail -p 'test_*.py'` | PASS, 52 tests after building prerequisite |
| `python3 -B -m unittest discover -s tests -p 'test_*.py'` | PASS, 33 observer/report tests |
| `python3 -B -m unittest discover -s tests/fd05a -p 'test_*.py'` | PASS, 9 memory-driver tests |
| `python3 tests/fixtures/generate.py --check` and `tests/fixtures/images/generate.py --check` | PASS |
| Build four GUI probes plus window-state probe | PASS; warnings retained |
| `desktop-file-validate data/io.github.onbeskarakterli.Pivra.desktop` | PASS |
| XML parsing of both SVGs; names/hicolor paths and desktop Icon | PASS, structural; not a new physical visual acceptance |
| `bash -n packaging/arch/PKGBUILD` | PASS |
| Source recipe without approval config | PASS negative test: exits 1 with BLOCKED message |
| Locked offline Cargo metadata + license/source collector | PASS, 185 packages including project; 184 exact checksum-verified crate archives/notices |
| `PIVRA_DISTRIBUTION_BUILD=1 cargo build --locked --bin pivra` + byte guard | PASS, debug artifact has no `/out/pivra-thumbnail-factory` fallback string |
| `target/debug/pivra --version` | PASS, `Pivra 0.1.0` |
| Arch makepkg/pacman, ShellCheck | NOT RUN in Codex: tools absent |
| Physical GNOME, real secure decode, native source JPEG/stress | NOT RUN in Codex B6; user reports remain separate |

The first thumbnail Python invocation preceded rebuilding its required
`target/debug/examples/thumbnail_provider_probe` in this restored environment.
It failed with missing-binary errors and resulting driver failures (6 failures,
21 errors). After building the unchanged prerequisite, all 52 tests passed.
No tests were edited or assertions weakened; this setup failure is not omitted
or described as a passing first run.

### Fresh GUI gates

X11: existing Xvfb/dbus tools, screen 1280×1024, held Xlib connection to prevent
server resets, `xdotool mousemove 1279 1023` **before any test**, actual pointer
query returned X=1279, Y=1023. Test windows were inside those bounds; four
private-XDG test processes use `GDK_BACKEND=x11`, Cairo renderer and
`G_DEBUG=fatal-criticals`. Sway: existing `tests/wayland_smoke.py`, headless
Wayland with XWayland disabled and DISPLAY removed, private XDG directories.

| Probe | X11/Xvfb | Sway/Wayland |
|---|---|---|
| `thumbnail_overlay_probe --catalog-regression` | PASS / exit 0 | PASS / exit 0 |
| `view_modes_probe --synthetic --zoom-pan` | PASS / exit 0 | PASS / exit 0 |
| `async_cancel_probe --controlled` | PASS / exit 0 | PASS / exit 0 |
| `window_failure_race_probe` | PASS / exit 0 | PASS / exit 0 |

Sway command pattern: `XDG_CURRENT_DESKTOP=GNOME dbus-run-session -- python3
 tests/wayland_smoke.py target/debug/examples/<probe> --navigation-probe`, plus
`--thumbnail-catalog`, `--synthetic --zoom-pan`, `--controlled`, or no extra flag
respectively. The process-local GNOME setting selects controlled cache tests;
it does not disable provider priority or decoder security. These probes test
production GTK actions/controllers/lifecycle with synthetic pixels or controlled
workers, **not real decoder pixels/physical hardware/active Glycin cleanup**.

### Warning debt (not fixed in this audit)

Observed strict Clippy errors:

- `src/image_view.rs:208`: unused `load_stats` (`dead_code`), also seen via example
  source inclusion.
- `src/thumbnail_overlay.rs:196` and `:198`: identical pin-icon branches
  (`clippy::if_same_then_else`).

The window-state probe build additionally warns about GTK `allocation()` being
deprecated since 4.12 (e.g. `examples/window_state_probe.rs:125`), unused included
`OpenedFile.file` and input constructors. Optional GNOME development API absence
is an SDK build warning, not proof of Arch factory failure. Ordinary Clippy's
exit 0 must not be presented as a strict-gate PASS. `tests/check.sh` includes
that strict all-target gate and therefore cannot be called wholly green.
No code cleanup or lint suppression was authorized here; resolve warning policy
or authorize a separate minimal cleanup before claiming warning-clean readiness.

## Distribution, licenses, artifacts and unresolved risks

- Cargo manifest/binary and recipe version are 0.1.0 (`pkgrel=1`, x86_64),
  GPL-3.0-only. No Cargo.lock/manifest/dependency changes. Manifest description
  still mentions seamless photo/video navigation; that is aspirational metadata,
  not video playback support. Not edited in this docs-only task.
- LICENSE is byte-identical to installed GNU GPLv3 text. SHA-256:
  `3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986`.
  Cargo.lock SHA-256 remains
  `41c86b8c1df465f4cdacb81b9287ecd0930c170b6cb9403106e5192004544e34`.
- License audit covers every current locked name/version; no metadata license
  field missing. No dependency declares MSRV above 1.85, but actual Rust 1.85
  compilation is NOT RUN. This review is not a new vulnerability or legal audit.
- The collector reverified all 184 registry archive checksums, produced full
  crate source copies and notice inventory outside the repository. Its hash
  matches the recipe: `e0bad72c99f4629b0668f59450e53a75022ea746efe57191ab8da34fb5cf1201`.
- Recipe includes Cargo --locked/offline release build, mandatory GNOME4 sidecar,
  pkg-config compile/link ordering, shared LCMS/seccomp checks and independent
  helper install at `/usr/bin/pivra-thumbnail-factory`. Distribution mode omits
  development OUT_DIR fallback; sibling discovery remains. Actual Arch release
  artifact bytes were not available for independent inspection in this audit.
- Runtime: gtk4>=4.16, libadwaita>=1.2, glib2, gdk-pixbuf2, gnome-desktop-4,
  glycin, bubblewrap, libseccomp, lcms2 and listed toolkit/runtime libraries.
  Build: cargo/pkgconf/python>=3.11; check: desktop-file-utils. Native package
  success is user evidence, not fresh pacman ownership data. Rust Glycin 3.0.8
  and OS Glycin loaders (previously user-reported 2.2.4) are distinct.
- Desktop Exec=`pivra %u`, only JPEG/PNG MIME, DBusActivatable=false; full app ID
  matches Icon and scalable/symbolic hicolor paths. Both SVGs parse and hashes
  remain `88a317c8…92628ae` and `0d39394d…40cf2c3`. GNOME visual readability and
  physical file-manager interaction are not newly tested by XML validation.
- Recipe installs application/helper, desktop/icons, LICENSE/notices, exact
  corresponding-source crate archives, Pivra archive and build recipe/config.
  Dynamic LGPL libraries retain source/replacement obligations if redistributed;
  system runtime packages are not bundled by this recipe. Final distributor
  must preserve actual notices and corresponding source/relinking duties.
- **Final source gate remains BLOCKED**: no approved `release-source.conf` in
  checkout, no `.SRCINFO`. Missing config fails closed. User's successful test
  archive SHA validation is not an approved final main release archive digest;
  its actual value was not supplied. Do not use moving dev, SKIP or a fabricated
  checksum. Final source selection/approval, metadata generation and any stable
  main/tag/publishing action are separate explicit work.
- 210 tracked files inspected: no >2MiB files, generated package/build output,
  tracked source config/.SRCINFO, private SDK copy or CI workflows found.
  Private-key/GitHub-token/AWS-access-key pattern scan found no matches (file
  names only, no values printed). This is a bounded current-tree scan, not a
  guarantee over Git history or all possible secrets. `target/`/Python caches
  remain ignored. Temporary SDK/test paths in test records are not install paths.
- 2 GiB GNOME helper AS is per-process virtual space, not a process-tree RSS cap;
  CPU=15s, FSIZE=256MiB and deadline=20s remain. Sandbox and provider priorities
  are unchanged. Image-cache accounting similarly is not a global memory cap.

## Changed files (documentation only)

- `README.md`
- `docs/PIVRA_COMMIT_TRACKER.md`
- `docs/PIVRA_DEVELOPMENT_ROADMAP.md`
- `docs/releases/PIVRA_V0.1_B6_READINESS_AUDIT.md` (this new report)
- `docs/releases/PIVRA_V0.1_RELEASE_NOTES.md`
- `docs/releases/PIVRA_V0.1_ARCH_PACKAGING_PLAN.md`
- `docs/releases/PIVRA_V0.1_ARCH_DEPENDENCIES.md`
- `docs/releases/PIVRA_V0.1_ARCH_PACKAGE_IMPLEMENTATION.md`
- `docs/releases/PIVRA_V0.1_DEPENDENCY_LICENSE_AUDIT.md`
- `docs/releases/PIVRA_V0.1_FINAL_RELEASE_AUDIT.md`
- `packaging/arch/README.md`

No source, tests, Cargo files, recipe, icons, licenses or security code changed.

## Exact source references and next decision

| Change | Exact commit |
|---|---|
| B3 format closure | `be2b9e1a6cbd4880e54b5546bb5ec7727bd25e86` |
| Distribution helper-path removal | `66ca07c4ac0ddbacb9998100e76f730b54b734ae` |
| Unsupported-video skipping | `e39accad6cc39eb922aebefe071c0bb7a4a22d8d` |
| Window-state persistence | `fa6a3a2d2167d623b2e3110a6889b6d7a5ec2b45` |
| GNOME failure diagnostics | `304742fc64018f221fe139bc370e28808721d4f4` |
| Controlled RLIMIT_AS comparison | `d2dd5641c6bd7ffbfc92e1d330ba89815842388d` |
| Production 2 GiB fix / latest user-verified package source | `e8fc9ec011ad6759bc6eef018b9a6f71465f80f1` |

Recommend a bounded next step: decide how to handle the strict warning gate and
the open preload observation; obtain any missing Nautilus/data-preservation
acceptance explicitly; approve the exact immutable final release source and
checksum, then regenerate/review metadata and package against that source.
Do not infer publication permission from this report. B6 documentation commit
is provided in the delivery response; this file does not invent a self-referential
commit hash. No next feature or release action is started.
