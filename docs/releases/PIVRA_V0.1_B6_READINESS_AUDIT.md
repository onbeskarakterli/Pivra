# B6 — Pivra 0.1.0 release readiness audit

Date: 2026-10-29 (Europe/Istanbul). Audited application and latest user-verified
package source: **`e8fc9ec011ad6759bc6eef018b9a6f71465f80f1`** on `dev`.
Original B6 changed documentation only. The B6-W1 follow-up below is a minimal
warning cleanup; no main change, tag, release, AUR upload or CI.

## Decision: PARTIAL; publication remains BLOCKED

The application/package acceptance listed by the user is **PASS, user-reported**.
It is no longer correct to describe those B5 installation checks as unperformed.
The original controlled Rust/GUI/packaging consistency checks passed except
for the strict warning gate, now closed by B6-W1 below. This audit is complete as an assessment,
not a declaration that every release gate passed or permission to publish.

B6-W1 strict all-target Clippy now PASSes with zero warnings; no lint policy
was relaxed. Release preparation still needs approved final immutable
source/archive SHA-256 and generated release metadata. The one-off preload observation remains visible and unfixed;
the owner explicitly accepted it as a documented v0.1.0 release risk in F1.
Explicit Nautilus acceptance has now been supplied by the user in B6-F1 below. No newly
confirmed product crash was found by these checks. PIV-18's wider historical
scope, TH-03/FD native gates and independent phase acceptance are not silently
marked DONE; v0.1.0 is the stated full-only JPEG/PNG scope below.

## B6-F1 — native desktop acceptance and candidate snapshot

Received 2026-10-29. Starting/reviewed code commit:
`eabc0e51eb9434c72a0b08ab1c7fbfbba5eaf2a7`. The following are **user-reported
native Arch/GNOME acceptance**, not Codex physical tests:

| Acceptance | User result |
|---|---|
| JPEG association | Pivra registered and selected as default |
| PNG association | Pivra registered; **Loupe remains default** |
| Nautilus JPEG double-click | PASS |
| Nautilus PNG Open With → Pivra | PASS |
| Actual JPEG and PNG rendering | PASS |
| Existing B5 package installation/removal | PASS (earlier package source `e8fc9ec`) |
| B6-W1 strict all-target Clippy | PASS, zero warnings |

The F1 physical-test message does not separately identify its binary commit;
no invented SHA is attached to those desktop results. B5's earlier exact source
and W1's Codex command evidence retain their original references. Pivra is not
claimed as the PNG default or a video handler; no association is changed here.
The explicit Nautilus gate is now satisfied by the user report, without turning
it into an independently repeated Codex native test.

This documentation snapshot records acceptance before freezing a full-SHA RC.
A follow-up records the real source archive hash, separate RC-only packaging
workspace and tooling/test results. It does not approve a final main source,
create a tag, publish anything, or claim `.SRCINFO` has been generated.
`Preload exceeds cache entry budget` remains OPEN and unfixed. After the first
acceptance snapshot, the project owner explicitly answered **“Accept as a
documented v0.1.0 release risk”** on 2026-10-29. It is therefore not a v0.1.0
release blocker by owner decision; no cause/fix or acceptance for later versions
is claimed. This is risk acceptance only, not publication/main/tag authorization.
Overall B6 remains PARTIAL pending native candidate metadata and final source/
publication approval. The final candidate snapshot includes this decision;
the preliminary `f91c298` snapshot predates it and is not the final RC.

### B6-F1 fresh candidate checks

Application/build/test/packaging blobs are unchanged from reviewed code commit
`eabc0e51eb9434c72a0b08ab1c7fbfbba5eaf2a7`; F1 edits are documentation only.
Fresh execution in the existing Codex SDK, no test assertions changed:

- `cargo fmt --check`, `cargo check --locked`,
  `cargo clippy --locked --all-targets -- -D warnings`: PASS, zero warnings.
- `CARGO_INCREMENTAL=0 cargo test --locked`: PASS, 197 tests.
- Python discovery: thumbnail 53, observer/report 33, memory-driver 9 PASS.
- Existing catalog, synthetic zoom/pan, controlled async cancellation and
  window-failure race probes: all four PASS on X11/Xvfb and headless Sway,
  first F1 attempts. X11 pointer verified at `(1279,1023)` before testing.
  These are controlled/synthetic GTK tests, not real secure-decoder or
  physical GNOME acceptance. The earlier W1 single-run Sway failure remains
  historical evidence, not a newly claimed fix.
- Both fixture integrity scripts, desktop-file-validate, SVG XML/path checks,
  package Bash syntax, version/license/Cargo.lock checks: PASS.
- Offline metadata + corresponding-source/notice collection: PASS, all 184
  checksum-matched locked external crates (185 packages including Pivra).
- Distribution-mode debug build and byte guard: PASS, no Cargo OUT_DIR helper
  fallback string. This does not independently inspect a final native package.
- Native GNOME4 helper compilation is mandatory in the actual PKGBUILD; its
  sibling install/discovery, dependencies, source delivery and security limits
  are unchanged. No makepkg/pacman available in Codex; `.SRCINFO` and new RC
  package creation are NOT RUN, and no generated metadata is invented.
- Current tracked-tree credential/artifact scan: no matching keys/tokens,
  generated binaries, source config, `.SRCINFO` or CI files. Limited scan is not
  a secret-free-history certification. No Cargo manifest/lock or license changed.

The exact full-SHA RC and real archive digest are recorded after freezing this
accepted-risk snapshot; the configuration stays in a separate RC-only workspace.
It is not an approved final main source or permission to publish.

## B6-W1 — strict warning cleanup — 2026-10-29

Starting commit: **`2f13309ba07950c94ed1efc6fbb6aa7247e2ee6f`**.
**Strict gate: PASS / exit 0 / zero warnings.** Overall B6 remains PARTIAL;
release-source approval, preload observation and native evidence boundaries
above are not closed by warning cleanup. Native user acceptance remains pinned
to `e8fc9ec`; W1 is not independently accepted on physical Arch/GNOME here.

### Every warning addressed

| Warning/source | Minimal resolution and retained behavior |
|---|---|
| Unused `ImageView::load_stats` (`src/image_view.rs`) | Retain useful queue diagnostics, consumed by async/preload/overlay probes. Add only a method-level `allow(dead_code)` with explicit reason for unused production/some-probe copies. No crate/module/impl blanket allowance, no diagnostics removed |
| Identical pin icon branches (`src/thumbnail_overlay.rs`) | Set the same `bookmark-new-symbolic` icon directly. Active state, pinned CSS class, tooltip, persistence and visibility branches unchanged |
| Deprecated GTK `allocation()` (`examples/window_state_probe.rs`) | Use supported `compute_bounds(window)` border-box bounds in a small size predicate, keeping exact width/height checks and deadlines. Do not substitute content-only width or raw shadow-including surface size; no assertion/tolerance weakened |
| Unused included `OpenedFile.file`, `from_argument` / `from_file` in no-file probe | Actually validate the existing PNG fixture with the included production input boundary and assert its Gio path, without starting decode. Then run the same production no-file window. No unused-code allowance or stub input module |
| Expected helper build-mode Cargo warnings (`build.rs`) | Successful built/distribution/cache-only modes become `cargo::metadata=gnome-helper-status=...`, retained in verbose build output (`cargo -vv`). Real compile/link failures still emit `cargo:warning` and retain the safe empty-fallback policy |

The initial strict run reproduced the dead-code/identical-branch failures.
After minimal changes, the full all-target command revealed no further Rust
warnings. The normal Cargo status warning was then reclassified as metadata
rather than discarded or globally suppressed. Build-policy regression verifies
all three expected metadata modes, retained resolver/fallback behavior and a
new real-compile-failure fixture that still reports a warning. That test uses a
mock compiler/pkg-config with real build.rs/resolver execution, not native
GNOME API proof. Cargo's metadata syntax is supported before declared Rust/Cargo
1.85; no dependency upgrade or Cargo.lock change was made.

### Fresh required verification

| Command/suite | B6-W1 result |
|---|---|
| `cargo fmt --check` | PASS / exit 0 |
| `cargo check --locked` | PASS / exit 0 / no warnings |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS / exit 0 / **zero warnings** |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 197 tests, 0 failures, no warnings |
| Build thumbnail-provider/four GUI/window-state examples | PASS, no warnings |
| Thumbnail Python discovery (`tests/thumbnail`, `test_*.py`) | PASS, **53 tests**, including 5 build-policy cases |
| Observer/report Python discovery (`tests`, `test_*.py`) | PASS, **33 tests** |
| Memory-driver Python discovery (`tests/fd05a`, `test_*.py`) | PASS, **9 tests** |
| `git diff --check`; Cargo.lock hash comparison | PASS; lock hash unchanged from B6 |

The one new Python test ensures helper compile failures remain visible rather
than being silenced. Existing assertions remain; pin selection/style branches,
geometry pixel assertions and security/resource policies were not relaxed.

| Existing GUI gate | X11/Xvfb | Headless Sway/Wayland |
|---|---|---|
| `thumbnail_overlay_probe --catalog-regression` | PASS / exit 0 | PASS / exit 0 |
| `view_modes_probe --synthetic --zoom-pan` | PASS / exit 0 | Initial FAIL; isolated repeat PASS / exit 0; details below |
| `async_cancel_probe --controlled` | PASS / exit 0 | PASS / exit 0 |
| `window_failure_race_probe` | PASS / exit 0 | PASS / exit 0 |

Same private-XDG drivers as B6, `G_DEBUG=fatal-criticals`, Cairo renderer. X11
pointer verified at `(1279,1023)` before all tests, Xlib connection held. Sway
has XWayland disabled. Catalog regression exercises both pin/unpin paths in
addition to existing click/scroll/navigation/visibility and window isolation.

Additional directly affected window-state probe:

- X11/Xvfb + Xfwm4, `bash tests/run_window_state_probe.sh --external-resize`:
  **PASS**. Separate processes, external xdotool resize, maximize/restore,
  fullscreen roundtrip and close/restart preserve exact border-box size checks.
- Sway, `window_state_probe --normal-only` through the existing Wayland driver:
  **PASS** for resize/restart/fullscreen. Maximize/restore remains **NOT RUN**
  under this compositor, as before. Full maximize coverage is the X11 run.
- Physical GNOME, real secure pixels, actual decoder cancellation and updated
  Arch package installation: **NOT RUN** in W1. GTK/controller runs use synthetic
  pixels/controlled workers; window-state/input checks start no decoder.

### Retained transient Sway observation (not fixed)

During the first, concurrently scheduled Sway zoom/pan run the unchanged pixel
assertion failed at `examples/support/zoom_pan_cases.rs:57`; probe exit 101,
Wayland driver exit 1:

```text
zoom/pan snapshot pixel mismatch Geometry { x: -84.47999999999996, y: -517.1199999999999, width: 691.1999999999999, height: 1382.3999999999999 } at 121,120
```

One isolated rerun **after the other gates completed** passed unchanged,
exit 0. This is an observed non-reproduced GUI-test failure, not a confirmed
production regression, nor a confirmed harness-timing diagnosis. No render,
zoom/pan, frame synchronization, source geometry, tolerance or assertion was
changed to obtain the rerun PASS. Warning-only changes do not establish its
cause. Preserve both runs; do not describe this as four first-run Sway passes
or claim a fix. This is distinct from the existing open preload observation.

W1 changed code/test files: `build.rs`, `src/image_view.rs`,
`src/thumbnail_overlay.rs`, `examples/window_state_probe.rs`,
`tests/thumbnail/test_build_helper_policy.py`. Directly related status updates:
this report, README, release notes, roadmap and commit tracker. No Cargo files,
security limits, sandbox, provider priority, cache/preload policy, main, tag,
release, AUR or CI changed. The delivery response pins the W1 commit SHA.

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
or bit-identical artifacts were reproduced twice. Actual Nautilus/Open With
is now user-reported PASS in F1; explicit user-data-preservation assertions on removal, 32-bit platforms, Rust
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
selection, memory pressure and logs if it recurs. Keep it in release notes; do not label it resolved. In B6-F1 the owner explicitly
accepted this as a documented v0.1.0 release risk. The observation stays open;
that decision does not establish a fix or authorize publication.

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

### Original B6 warning debt (closed by B6-W1 below)

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
No code cleanup was authorized in the original B6 audit. The separately
authorized B6-W1 cleanup and its fresh gate evidence are recorded below;
these original FAIL records are retained as history.

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

## Original B6 changed files (documentation only)

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

The strict warning gate is closed by B6-W1. Next decide how to handle the open
preload observation (and the W1 transient Sway observation below); obtain any
missing Nautilus/data-preservation
acceptance explicitly; approve the exact immutable final release source and
checksum, then regenerate/review metadata and package against that source.
Do not infer publication permission from this report. B6 documentation commit
is provided in the delivery response; this file does not invent a self-referential
commit hash. No next feature or release action is started.
