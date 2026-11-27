# Pivra v0.2.8 pre-release validation

**Status: ready for documentation review and physical acceptance. Publication is
blocked pending native verification and source approval.** The published stable
release is v0.2.7. The version in Cargo and the Arch recipe remains 0.2.7.

This run began at `9793b723fa9442b909dc77fe17813f71f7e05480`.
Validated development source after fixes:
`91da58dd60a361a458fb56fe402269f262bfee31`.
Earlier passing probes ran on the same production code where unaffected, and
the affected gates were rerun. This report and later document edits are separate
from the tested source. Neither this SHA nor this report is an approved archive.

## Automated validation

The Linux SDK ran on Debian 13 with Rust/Cargo 1.99.1, GTK 4.18.6,
libadwaita 1.7.6 and GStreamer 1.26.2. GUI cases used private Xvfb/xfwm4,
session D-Bus, XDG paths, GTK schemas and temporary fixture copies.
Each consolidated X11 probe had a 55-second deadline. Native GPU control
used software GL. These sessions cannot establish GNOME compositor or hardware
acceptance.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --locked` | PASS |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS after lint fixes |
| `cargo test --locked` | PASS, 241 tests, zero failures |
| `cargo build --locked` and `cargo build --locked --examples --bins` | PASS |
| Python discovery under `tests`, `tests/thumbnail`, `tests/fd05a`, `tests/video_color`, and `tests/video_gpu` | PASS, 249 executed, zero failures, one GDB test class skipped because GDB was unavailable |
| `python3 -B tests/fixtures/generate.py --check` and `python3 -B tests/fixtures/images/generate.py --check` | PASS |

Python command per suite: `python3 -B -m unittest discover -s SUITE -p 'test_*.py' -v`.
The first top-level run encountered an X11 display connection error in the
pointer test. The unchanged suite passed inside a private D-Bus session.

## Functional and stability gates

**54 distinct controlled GUI/video probe cases passed after rechecks.** Counts
exclude repeated runs and the secure PNG/JPEG probes which returned 77. The
consolidated driver reused `tests/video_gpu/lifecycle.py` and the window manager
helper in `tests/slideshow_smoke.py`. Commands below abbreviate
`target/debug/examples/` for Rust probe binaries.

| Area | Executed commands or cases | Result |
| --- | --- | --- |
| Window, chooser and menu | `slideshow_probe --controlled --development-{open,shortcuts,context-menu,opening,reveal} CLIP` and `tests/toolbar_ux_smoke.py --session x11 --development open-latency` | PASS. Same-window Open, Cancel, real Ctrl+O, context Open, menu ordering/focus/Escape/cascades, external app launch and private FileManager1 current-file URI. Both same-folder direct/Open cases passed. |
| Production startup and file routing | `python3 -B tests/gui_smoke.py`, then with `target/debug/pivra --files --expect-image-error`, `python3 -B tests/desktop_smoke.py target/debug/pivra --expect-image-error`, `python3 -B tests/discovery_smoke.py target/debug/pivra --expect-image-error` | PASS. Real X11 mapping, CLI path and encoded URI/GIO desktop launch, normal close, independent windows, source bytes and catalog. File cases explicitly expect the secure loader error. |
| Display, minimap and viewport | `slideshow_probe --controlled --development-{layout,minimap,navigation,zoom-state} CLIP`, `view_modes_probe --synthetic` with no extra arguments, `--zoom-pan`, and `--resampling`, plus `view_modes_probe --controlled-navigation` | PASS. Fit/Fill/100%, centering, zoom/pan, minimap interaction, temporary overlay suppression, warm cache frame identity and error recovery. |
| Media information, filters and rename | `slideshow_probe --controlled --development-{info,file-actions} CLIP`, `window_failure_race_probe --rename-pending` | PASS. Metadata panel, real F2, Photos/Videos/All with filtered-out current media, byte-preserving rename, natural order, stale path cancellation. |
| Slideshow and stock video | `slideshow_probe --controlled CLIP`, separate `--eos-regression` and `--video-interval-regression` runs, `--development-{slideshow-click,slideshow-effects}`, `python3 -B tests/toolbar_ux_smoke.py --session x11` (two processes), `video_preparation_probe` | PASS. Video EOS and long-video interval isolation, pause/resume, seek behavior, manual click/drag distinction, timer reset, eight effect choices, layer disposal, repeated start/stop, preferences and toasts. Stock video used generated H.264. |
| Video ownership and shader | `dbus-run-session -- bash tests/run_video_context_lifecycle.sh --session x11 --software` | 9 PASS, exit 0. Mixed GPU/stock, context ownership, error cleanup, limited/full-range shader and playback control. The C current-API policy also passed. |
| Folder and race handling | `folder_watch_probe --expect-image-error` with and without `--media-failures`, `navigation_race_probe`, `window_failure_race_probe`, `async_cancel_probe --controlled` | PASS. Live external saves, renames, deletion, alias paths, close with pending load, cancellation and stale-result protection. Corrupt-image handling was exercised with an unavailable secure loader. |
| Thumbnail lifecycle and fullscreen | `thumbnail_overlay_probe` with separate `--catalog-regression`, `--thumbnail-scheduler-regression`, `--thumbnail-scrollbar-regression`, `--retention-regression`, `--fullscreen-regression` and `--fd06-close-harness-regression` runs. `window_state_probe` | PASS. Controlled GNOME scheduler used its private helper fixture, catalog pointer started at `(1279,1023)`, and fullscreen used an actual X11 window manager. No native generation claim. |
| Cache/resource bounds | `preload_probe --controlled`, `preload_probe --cache-churn-benchmark`, `thumbnail_memory_probe` | PASS. 300 selections in the preload stress case, 1,000 latest-only pending requests in async cancellation, at most three image workers, drained jobs and weak references after close. |

The controlled catalog decoded 120 entries once each and retained 64 textures
charged at 1 MiB. The 81/120 retention gate and scheduler admission/retry tests
passed. The 12 unrelated-folder-churn iterations caused zero extra decodes.
The packed-thumbnail probe sampled an RSS delta of 225,280 bytes for its
bounded synthetic fixture. These bounded observations do not establish a
long-session or hardware VRAM budget.

## Failures investigated

- `e41b56eb0c4c0db126bbf38e3b67c7816ba5898c` clears strict Clippy.
  The rename action groups its notification dependencies and the context menu
  passes its predicate directly.
- `a9531067b595983c8f13f6f4cff586f426480896` fixes a reproduced slideshow bug.
  A rapid A → B → A sequence could finish on the last polled path with a new
  revision. The observer logged `catalog-revision-reload`, started a redundant
  load and cleared the fade. The action now records its own selection revision
  and path immediately. The fade and dependent restart initially failed at
  `slideshow_probe.rs:115` and `toolbar_ux.rs:53`. Both passed after the fix,
  as did effects, click, regular slideshow, EOS and video interval rechecks.
- `81ab5747dce405368cd860ce7657c5c00827bf2f` adds the narrowly scoped GTK F2
  and filter case using a private folder and the production window.
- `91da58dd60a361a458fb56fe402269f262bfee31` repairs stale test assumptions.
  Probes now locate the actual error label and refresh banner, exercise the
  current wheel-zoom control, invalid WebP and stock video failure, and expect
  the current empty title and mapped empty page. The fullscreen strip segment
  pins the strip and starts in Fit, where the minimap does not hide navigation.
  Separate auto-hide coverage remains. Python launch probes now recognize
  the filename-bearing image error. All affected gates passed on recheck.
  The scheduler initially ran without its required GNOME selector and private
  helper, then passed with those documented settings.

## Arch package and physical acceptance

| Gate | Result |
| --- | --- |
| `bash -n packaging/arch/PKGBUILD`, `desktop-file-validate data/io.github.onbeskarakterli.Pivra.desktop`, icon XML/desktop ID and recipe `!debug` inspection | PASS for source files |
| `bash packaging/arch/PKGBUILD` source guard | PASS as a negative check. Exit 1 with approved `release-source.conf` missing. |
| `bash packaging/arch/verify-video-dependencies.sh` | SKIP, exit 77. This host has no Arch pacman. |
| Native `makepkg`, package install, ownership and inventory, clean metadata, installed icon/menu launch and absence of debug/test artifacts | SKIP. No build-eligible Arch environment and no owner-approved immutable v0.2.8 source/configuration. A bounded pull of the Arch build image did not complete. |
| Secure PNG/JPEG/EXIF and real photo format coverage | SKIP. `image_probe` and `image_probe --jpeg-first` stopped at the sandbox preflight with exit 77. `bwrap` reported `setting up uid map: Read-only file system`. |
| GIF, animated WebP/APNG, RAW, native thumbnail creation and active secure-decoder teardown | SKIP. No successful secure frame in this environment. |
| Delete → system Trash → Undo | SKIP. `gio info trash:///` reported `Operation not supported` in the isolated desktop. Filesystem deletion, identity, missing-file and restoration-cache regressions are covered separately. |
| Physical GNOME, GPU hardware, display sleep inhibition and installed package integration | SKIP. Requires the target desktop. |

For native acceptance use disposable copies of a mixed folder. Verify empty
Open, repeated Ctrl+O, context Open/Cancel, Open With/Nautilus selection, focus
and cascading menus, then exercise the zoom/minimap/strip, rename, Trash/Undo,
secure formats, each slideshow transition, video EOS and physical audio/GPU
controls. Run the existing secure preview/active-close and video lifecycle
commands on that system with stock GTK and capture both successful and failed
gates. The stock GTK Wayland limitation in earlier records remains open. The
[Open latency report](../diagnostics/OPEN_FILE_LATENCY.md) records the unresolved
chooser and subsequent navigation reports.

After source approval and version preparation, use the established
`python3 packaging/arch/validate-release.py --approved-source SHA --sha256 HASH`
workflow in a native Arch environment. Review the actual package before a local
install, run `pacman -Qip`, `pacman -Qlp`, `pacman -Qkk pivra`, and check the
installed desktop entry, icon, helper and GUI. Prior v0.2.7 package evidence
remains in its historical record.

The controlled gates support documentation cleanup and user acceptance.
Release approval remains pending on secure native media and Trash/Undo,
physical GNOME and hardware checks, and the exact native package.
