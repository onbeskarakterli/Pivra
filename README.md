# Pivra 0.1.0

Pivra is a native Linux image viewer built with GTK4 and libadwaita for GNOME.
The v0.1.0 scope is **JPEG and PNG still images**; it is full-resolution-only,
without an early low-resolution preview pipeline.

- Open one local JPEG/PNG path or file URI; decode through secure Glycin/Bubblewrap,
  with EXIF orientation handling and controlled media errors.
- Fit, Fill and 100% modes; 100% means one source pixel per GTK logical unit,
  rather than necessarily one physical display pixel on HiDPI.
- Cursor-anchored zoom and bounded drag pan. Enable wheel zoom with the Zoom
  control or `Z`; the default wheel behavior navigates media. Smooth/Nearest
  filtering reuses the decoded texture.
- Naturally sorted, nonrecursive folder catalogs; Left/Right navigation stops
  at the boundaries and skips video/unsupported candidates to the next JPEG/PNG.
  Catalog entries are retained; each window has independent selection/loading.
- An overlay thumbnail strip with selection/navigation, scrolling, pinning,
  resizing and saved visibility preferences; GNOME-first provider selection,
  with a Freedesktop alternative when GNOME is unavailable/disabled at session
  selection, and bounded in-memory thumbnail storage. File errors do not switch
  the session provider.
- Folder monitoring updates catalog membership. Changes to the selected file
  produce a Refresh notice; the file is not automatically reloaded.
- Cancellable asynchronous loading, per-window image cache and adjacent-image
  preload. Cache accounting is bounded, but is not a hard process-RSS/VRAM cap.
- Per-window fullscreen through the existing GTK window. Normal window size
  and maximized state persist; fullscreen dimensions do not overwrite normal size.

Video entries can appear in the folder catalog, but **video playback and image
editing are not available**. GIF/WebP/APNG animation is not supported. Other
catalog candidates do not imply display/decoder support.

This remains release preparation on `dev`, not a published tag or AUR package.
The user reports clean Arch build/checksum/package integrity, removal/reinstall,
GNOME/Wayland thumbnail stress, video skipping, window persistence and the 2 GiB
GNOME-helper correction PASS at `e8fc9ec`. These are **user-reported native
results**, separate from Codex's controlled checks. See the
[B6 readiness audit](docs/releases/PIVRA_V0.1_B6_READINESS_AUDIT.md).
The user additionally reports JPEG-default/Nautilus double-click and PNG
Open With rendering PASS; Loupe remains the PNG default. These physical checks
were not run by Codex. Final approved release source/metadata remain separate.
The one-off `Preload exceeds cache entry budget` observation remains unresolved;
subsequent successful tests do not establish a fix. B6-W1 now passes strict
all-target Clippy with `-D warnings` and zero warnings; other release gates
remain open. Its non-reproduced Sway zoom/pan test failure is retained in the audit.

See [v0.1.0 release notes](docs/releases/PIVRA_V0.1_RELEASE_NOTES.md),
[dependency license audit](docs/releases/PIVRA_V0.1_DEPENDENCY_LICENSE_AUDIT.md),
[roadmap](docs/PIVRA_DEVELOPMENT_ROADMAP.md),
[technical capabilities](docs/PIVRA_TECHNICAL_CAPABILITIES.md), and
[historical validation/commit tracker](docs/PIVRA_COMMIT_TRACKER.md).

## License

Pivra is licensed under **GPL-3.0-only**, not GPL-3.0-or-later.
See the complete, unmodified [GNU GPL version 3 text](LICENSE).
Dependencies retain their own licenses and distribution notices; see the audit.
No project copyright holder or contributor ownership has been inferred.

## Retained development history

The following records and links are preserved as historical evidence. Their
milestone-specific capability, status and license statements are not the
current release scope.

### Original development introduction
A fast, lightweight GNOME image viewer with seamless photo and video navigation, smart zoom, and basic editing.

## Historical development snapshot — before release licensing

**PIV-01–17: DONE (PIV-17 user-reported native/manual acceptance); PIV-18: TESTING, on `dev`.**
Pivra accepts one local JPEG/PNG path or file URI and discovers a naturally ordered
photo/video catalog. Left/Right now navigate that window's catalog without
wrapping; video entries show an explicit unavailable-playback message. Every new
photo uses the existing secure Glycin path and starts in centered Fit.
PIV-10's real GNOME/Wayland pixel and physical-key validation was reported by
the user and recorded in the tracker. PIV-11 now watches each window's folder:
additions, removals and renames update the naturally ordered catalog while
preserving selection. External changes show **Dosya değiştirildi — Yenile**;
only the explicit **Yenile** action reloads the image. PIV-11 native acceptance
was reported by the user and is recorded in [its validation log](docs/PIV11_VALIDATION.md).
PIV-12 native media-failure and two-window acceptance was reported by the user.
PIV-13 adds a compact **Fit / Fill / 100%** header menu, with **F6 / F7 / F8**
shortcuts. Fit is the default and returns on a new load. Modes reuse the decoded
texture; 100% means one source pixel per GTK logical unit, including on HiDPI.
PIV-13 native acceptance was reported by the user; see its validation record.
PIV-14 adds cursor anchored wheel zoom (10–3200%) and bounded left-button drag
pan. The header shows **Zoom** for manual transforms. Presets, a new photo and
Refresh reset the transform; each window is independent. PIV-14 native/mouse
acceptance was reported by the user. PIV-15 adds **Smooth / Nearest** in the
existing menu. Smooth is default; quality changes preserve zoom/pan and reuse
the decoded texture. The preference survives photo changes in that window;
it is not saved across restarts. PIV-15 native acceptance was reported by the user.
PIV-16 bounds each view to three admitted image jobs and one latest pending
request, with cooperative cancellation and a five-second admission deadline.
Stale results cannot replace the latest image or its settings. Native PIV-16
acceptance was reported by the user; [limits and evidence](docs/PIV16_VALIDATION.md).
PIV-17 preloads the next then previous supported photo with one background
worker and a per-window 128 MiB / three-entry RAM cache (64 MiB per entry).
Selected work keeps priority; file identity/monitor invalidation and explicit
Refresh prevent stale cache reuse. GMemoryMonitor is connected; actual OS
pressure delivery is unverified here. Accounted bytes are not RSS/VRAM.
[Policy, measured tests and remaining native acceptance](docs/PIV17_VALIDATION.md).

```bash
bash tests/run_preview_pipeline_probe.sh
```

PIV-09 DONE is backed by **user-reported Arch/GNOME Wayland tests**, including
active secure-decode cancellation; see the tracker closure at `8991b78`/`c11c7b7`.
Codex still cannot start the Bwrap decoder under its read-only `/proc` UID-map
restriction. That environment limitation is separate from the native results.
This retained development snapshot is historical; use the v0.1.0 introduction and release notes above for current scope.

See [technical capabilities and open prerequisites](docs/PIVRA_TECHNICAL_CAPABILITIES.md) and [verified results and commit evidence](docs/PIVRA_COMMIT_TRACKER.md). PIV-05 is DONE; Phase 0 remains INCOMPLETE because a fresh native environment installation has not been independently reproduced. A clean Cargo target build and the existing headless regressions passed.

Launching without arguments opens a blank `Pivra` window with a native libadwaita header and system theme. Each process owns an independent application/window (`io.github.onbeskarakterli.Pivra`, GIO NON_UNIQUE); one local photo path or file URI opens a file-associated placeholder through GIO `open`. The source desktop entry is ready for later native installation; actual Nautilus/default-application behavior is NOT RUN. Closing the last window ends that process normally. If GTK cannot initialize a graphical session, the program prints a controlled error to stderr and exits 1. Help/version flags and invalid-argument exit 2 work without initializing GTK.

### PIV-03 module boundaries

| Module | Responsibility and allowed dependencies |
|---|---|
| `src/main.rs` | Dispatch CLI commands or launch the application; process exit codes |
| `src/cli.rs` | Parse OS arguments and print help; no GTK initialization or file I/O |
| `src/input.rs` | Binary-side local path/URI, regular-file/readability and extension checks; GFile + MediaItem association; no decode |
| `src/app.rs` | GTK initialization, NON_UNIQUE + HANDLES_OPEN, activate/open lifecycle; input boundary, windows and settings |
| `src/window.rs` | Native header/viewport, window-owned file/catalog context, worker completion/cancellation and decoder-free status; settings dimensions |
| `src/lib.rs` | Expose GUI-independent models, std-only folder discovery and shared application name |
| `src/discovery.rs` | Nonrecursive candidate classification, read-only inspection, naturally ordered catalog/selection and explicit scan issues; no GTK/GIO |
| `src/natural.rs` | Pure byte-preserving natural filename/path comparison, no I/O or GTK |
| `src/media.rs` | `MediaItem`: supplied path and explicit `MediaKind`; no existence checks, extension guessing or decoding |
| `src/catalog.rs` | `MediaCollection`: caller-ordered items and a bounded selection; depends only on media |
| `src/settings.rs` | In-memory window defaults (960×640); no globals, persistence or GSettings |

The core library has no GTK/GIO imports and no dependency back into app/CLI/window. The binary depends on it, not the reverse. Core edges are `catalog -> media` and `discovery -> catalog/media/natural`; there are no module dependency cycles. PIV-03 left native Cargo dependencies and Cargo.lock unchanged; PIV-04 adds only the test-only `tempfile` dependency.

A collection starts with its first item selected, or no selection if empty. Selecting an out-of-range index returns false and leaves the current item unchanged; callers receive an immutable item slice. Input order is preserved exactly. Directory discovery, natural sorting, next/previous actions, looping, file watching and UI/catalog wiring are deliberately outside PIV-03. The media kind is caller-provided metadata, not proof that a file is supported or readable. Settings currently carry only the dimensions actually used by the window; system theme behavior remains inherited.

### PIV-03 validation — 2026-10-21

- Format, check, build and Clippy: PASS with the existing activated Rust/GTK environment, offline and locked builds.
- Cargo tests: **9 unit + 4 CLI integration tests passed; 0 failed, 0 ignored**. The original three CLI parsing tests, application ID test and CLI process regressions remain in place. Five core tests cover empty/single lists, valid/invalid selection, preserved order, and explicit media metadata.
- Core independence: `rustc +stable --edition=2024 --test src/lib.rs -o /tmp/pivra-piv03-core-tests` followed by `/tmp/pivra-piv03-core-tests` passed the same five core tests without Cargo or GTK linkage. These are not five additional unique tests.
- Existing Xvfb/D-Bus GUI smoke: PASS for two independent `Pivra` windows and normal closure with exit 0 and empty stdout/stderr. The geometry assertion now checks the unchanged 960×640 dimensions, verifying that the settings model is actually consumed. CLI help/version/error outputs and headless exit 1 are unchanged.
- No physical GNOME/Wayland or hardware-renderer test is claimed. No decoder, playback, directory scan, zoom, edit, GSettings integration, new dependency or CI was added. See the commit tracker for closure SHAs and push evidence.

### PIV-04 test infrastructure — 2026-10-21

**Desktop priority: GNOME/Wayland first, X11 compatibility second.** Test categories below are independent: a PASS in one never substitutes for another.

| Category | Current result and exact scope |
|---|---|
| Rust and CLI | PASS: 9 unit + 7 integration tests, 0 failed/ignored. Existing tests preserved; headless launch checks both forced X11 and forced Wayland with a fresh socket-free runtime directory |
| X11/Xvfb smoke | PASS: two independent mapped Pivra windows, title, 960×640, normal WM_DELETE_WINDOW close, exit 0 and empty stdout/stderr |
| Headless Wayland / Sway 1.10.1 (wlroots 0.18.2) | PASS: two real mapped `xdg_shell` surfaces observed through compositor IPC; title/app ID/PID/960×640 checked; normal Wayland close requests, exit 0, empty stdout/stderr, independent lifetime |
| Headless Wayland / Weston 14.0.2 | Surface checks PASS: server scene graph contains Pivra's `xdg_toplevel` on the headless output with a SHM buffer; server protocol confirms title, 960×640 window geometry, configure acknowledgement and buffer attach/commit. **Normal close NOT RUN**: this Weston setup lacks standard close-control IPC; cleanup SIGTERM is not a passing close test |
| Physical GNOME Shell/Wayland | **NOT RUN**: desktop theme/integration, HiDPI, touchpad, multiple monitors, accessibility and hardware acceleration remain separate future validation |

Production Rust/GTK code is unchanged. Fixtures and temporary-directory strategy are documented in [tests/fixtures/README.md](tests/fixtures/README.md). The 86-byte original PNG and 83-byte text placeholder are generated from source values, with SHA-256 and provenance in a manifest. The text placeholder is **not playable video**; no decoder or playback success is claimed. JPEG, WebP, animation and real video needs are documented for later work without downloading media or selecting a license.

Test audit: existing unit tests already cover empty/single/multiple catalogs, preserved order and invalid indices, so these are not copied into redundant tests. The new model integration test covers actual private fixture copies with Turkish/Unicode/space-containing names, explicit Image/Video kinds, selection and unchanged bytes. Two helper tests check cleanup after normal scope/panic unwinding and parallel filename isolation. Test-only `tempfile` provides private unique directories and RAII cleanup; no process-global environment is modified by Rust tests. CLI checks now use socket-free runtime directories, avoiding accidental connection to a developer's real Wayland session. Python GUI subprocess operations have bounded timeouts and shared owned-process cleanup; forced cleanup is never reported as normal closure.

After activating the existing cloud environment, fetch test dependencies once if not cached, then run the core checks:

```sh
source /workspace/pivra-environment/gtk-activate.sh
cargo +stable fetch --locked
bash tests/check.sh
# Or let the check runner activate the existing environment:
PIVRA_ENV_ACTIVATE=/workspace/pivra-environment/gtk-activate.sh bash tests/check.sh
```

The runner performs the read-only fixture integrity check, fmt/check/build/test/Clippy with locked offline dependencies. It does not silently skip or claim GUI tests. Run graphical categories separately after build:

```sh
# X11 compatibility; Xvfb and its D-Bus session end with the test.
GTK_A11Y=none GSK_RENDERER=cairo GDK_BACKEND=x11 \
  xvfb-run -a dbus-run-session -- python3 tests/gui_smoke.py

# Native Wayland lifecycle: creates a private headless Sway compositor.
dbus-run-session -- python3 tests/wayland_smoke.py

# Additional Weston 14 surface-only check; full close test is NOT RUN.
PIVRA_WESTON_PREFIX=/workspace/pivra-environment/weston/usr \
  dbus-run-session -- python3 tests/weston_smoke.py
```

On a standard Debian 13 test machine, `sway` supplies sway/swaymsg and `weston` supplies Weston 14/debug helpers. The cloud instance has no administrative installation access: official signed Debian packages were downloaded with APT and extracted locally. Sway helpers are under the existing `/workspace/pivra-environment/native` tool tree, and Weston is separately under `/workspace/pivra-environment/weston`; no system package was installed/upgraded and Rust/GTK installation or activation was not reset. Weston relocation uses per-test library/module/data/client paths, not system configuration edits. The prefix option currently targets Debian amd64 Weston 14; other layouts/versions require adaptation and are not claimed tested.

Both Wayland tests create private 0700 runtime/cache/config/data directories and use an isolated `dbus-run-session`. `GDK_BACKEND=wayland` is forced, DISPLAY removed and XWayland disabled. Sway uses wlroots headless/Pixman; Weston uses headless/Pixman and a debug subscription restricted to its private runtime. Each compositor is stopped and temporary state removed when the test ends. Accessibility initialization is disabled only for these minimal test sessions, and GTK uses Cairo software rendering; these are not accessibility/hardware-renderer tests.

Missing compositor tools cause the relevant script to report **BLOCKED** and exit 77; startup/assertion failures are nonzero **FAIL**, never a passing skip. Record the actual result rather than relabeling it as X11 success. If headless Wayland is unavailable, PIV-04's fixture/core infrastructure may still be complete, but Wayland support must not be claimed validated.

### Native prerequisites

Use Rust stable (minimum declared Rust 1.85 for Rust 2024 and the resolved dependency metadata) and pkg-config. The selected API baseline is **GTK >= 4.6, libadwaita >= 1.2, GLib >= 2.66**; the actual distribution libraries can impose higher requirements. The resolved lockfile and current stable toolchain are the tested combination; older-toolchain/minimum-library combinations have not been tested.

On Debian 13, the native development packages are `libgtk-4-dev` and `libadwaita-1-dev` (including their GLib and other dependencies). `xvfb`, `xauth`, `xdotool`, Python 3 and a D-Bus session are only needed for the automated X11 window smoke test. No GTK GStreamer media package is requested.

The existing cloud Rust installation was preserved. Because this machine has no administrative package installation access, official signed Debian packages were downloaded with APT and extracted under `/workspace/pivra-environment/native`; existing system runtime libraries are reused. Cloud GTK activation also configures pkg-config/linker paths and writable XDG state directories. On a normally provisioned desktop, use the distribution's installed libraries instead of these cloud paths.

## Development and validation

Use a Rust stable toolchain supporting the Rust 2024 edition, with rustfmt and Clippy. The prepared cloud environment can be activated in each shell with:

```sh
source /workspace/pivra-environment/gtk-activate.sh
cd /workspace/Pivra
```

That activation path is specific to the prepared cloud environment; other machines can use their own Rust stable installation. Work on `dev`; do not change `main`.

```sh
cargo +stable fmt --check
cargo +stable check --offline --locked
cargo +stable build --offline --locked
cargo +stable test --offline --locked
cargo +stable clippy --offline --locked --all-targets -- -D warnings
cargo +stable run --offline --locked --quiet -- --help
cargo +stable run --offline --locked --quiet -- --version
cargo +stable run --offline --locked --quiet --
cargo +stable run --offline --locked --quiet -- --unknown
```

The last command intentionally exits **2** with a controlled error. Argument-free launch needs a graphical session; it stays running until its window closes, then exits 0. Without a display it exits 1. The build checks and help/version commands exit 0.

### PIV-02 validation — 2026-10-21

- Native libraries: GTK 4.18.6, libadwaita 1.7.6, GLib 2.84.4; Rust bindings: gtk4 0.10.3, libadwaita 0.8.1 (locked).
- Format, check, build and Clippy: PASS. Unit tests: **4 passed**, including the three original PIV-01 tests. CLI integration tests: **4 passed**, covering seven help/version/error argument cases and controlled headless launch failure. No tests ignored.
- Real GTK windows: PASS on **Xvfb X11**, with an isolated D-Bus session and Cairo software rendering. The smoke test opens two independent processes/windows, checks mapped visibility and the `Pivra` title, sends WM_DELETE_WINDOW to each, verifies that one close does not affect the other, and checks exit 0 and empty stdout/stderr. A rendered window screenshot was also inspected for the header, native controls and empty viewport.
- No physical GNOME/Wayland session, hardware rendering, desktop integration or accessibility validation is claimed. The Xvfb test disables accessibility initialization only for its minimal test session; the application does not override accessibility or theme preferences.

After building, reproduce the graphical test with:

```sh
GTK_A11Y=none GSK_RENDERER=cairo GDK_BACKEND=x11 \
  xvfb-run -a dbus-run-session -- python3 tests/gui_smoke.py
```

The GUI script fails if no X11 display or window appears; it does not silently skip. On a regular graphical desktop, `cargo +stable run --locked --` is the normal startup command.

### Historical PIV-01 results — 2026-10-21

- Toolchain: rustc 1.99.1, Cargo 1.99.1, rustfmt 1.10.0-stable, Clippy 0.1.99.
- Format, check, build and Clippy: PASS, exit 0; Clippy treats warnings as errors.
- Unit tests: **3 passed, 0 failed, 0 ignored**.
- Eight CLI scenarios: PASS, exact stdout/stderr and exit codes checked. `--help`/`-h` print usage (exit 0); `--version`/`-V` print `Pivra 0.1.0` (exit 0); no arguments print `Pivra: graphical application is not yet implemented` (exit 0). `--unknown`, `one two`, and `--help --version` print `Pivra: invalid arguments; use --help` to stderr, leave stdout empty, and exit 2.
- Compilation and CLI validation used a separate initially empty target directory, `/tmp/pivra-piv01-validation`, with the checked-in dependency-free Cargo.lock and offline/locked commands.

The validated source is commit `86a54f7094639d6a75a0c8798d8b48e7a65e8f12`; this closure only adds Cargo.lock and updates documentation. Actual closure commit SHAs are recorded in the [commit tracker](docs/PIVRA_COMMIT_TRACKER.md), with the final documentation commit available from its Git history.

Earlier audits on 2026-10-20 were BLOCKED by the unavailable execution environment; those historical results are preserved in the tracker and are superseded by the successful run above. The feature specification and roadmap also contain historical pre-code wording; their accepted product decisions remain unchanged.

See the [development roadmap](docs/PIVRA_DEVELOPMENT_ROADMAP.md), [feature specification](docs/PIVRA_FEATURE_SPEC_TR.md), and [commit tracker](docs/PIVRA_COMMIT_TRACKER.md). At that historical milestone license selection remained open; the current project license is GPL-3.0-only (see the introduction above). The PIV-01 results above describe the earlier CLI-only bootstrap; PIV-02 intentionally replaces its no-argument placeholder with the GTK window.


### PIV-06 local file input — 2026-10-21

PIV-06 provides input infrastructure, **not image decoding**. One readable regular
local `.jpg`, `.jpeg` or `.png` file is admitted (case-insensitive extension).
The file name appears in the title; the viewport and subtitle explicitly say
`Image decoding is not implemented yet.` A malformed image with an admitted
extension can still reach this placeholder. Other formats and videos are rejected
at this boundary; this is not a decoder support matrix.

```sh
pivra
pivra /home/user/Pictures/photo.jpg
pivra './fotoğraf örneği.png'
pivra 'file:///home/user/Pictures/foto%C4%9Fraf%20%C3%B6rne%C4%9Fi.png'
pivra --help
pivra --version
```

CLI parsing uses `args_os`, so local non-UTF-8 filenames do not panic. Plain paths
are not percent-decoded; `file:` URIs use strict GLib parsing and filename
conversion. Unicode, spaces, literal `#`, `?`, `%` and encoded URI bytes are tested.
Only an empty authority or `localhost` is admitted; remote schemes/hosts,
query/fragment/credentials/ports and malformed escapes are rejected. Prefix an
ambiguous colon-containing or `-`-starting relative filename with `./`. Exactly
one file argument is accepted; multiple positional arguments and unknown flags
retain exit 2 and the previous invalid-argument diagnostic. Help/version flags
and aliases still work without GTK; help adds the file-input syntax.

`input` checks metadata, opens read-only, checks the descriptor's regular-file
type, and drops the handle **without reading file contents**. Symlinks to readable
regular files are accepted with the requested alias retained; dangling links,
directories, special files, unreadable files and unsupported extensions fail
with a controlled stderr diagnostic/exit 2 before GTK initialization. GTK/session
failure remains exit 1. At GIO `open`, the file is checked again; a now-invalid/missing
file fails with exit 1 and no success window. Each window owns its GFile/MediaItem
association for its lifetime. Core library modules remain GTK/GIO independent.

These are point-in-time checks. **TOCTOU remains:** a path/symlink may be replaced
between checks or before future decoding; no persistent descriptor/content
identity is promised. Local filesystem checks are synchronous; mounted network
filesystems and adversarial replacement with a blocking special file require
later cancellation/descriptor policy. Remote/network/VFS support is not claimed.
Decoder integration must revalidate the actual resource; no sandbox bypass or
Glycin integration was added. The 8 GiB sparse-file test verifies admission
without loading its contents; it is not a decoder performance benchmark.

Native desktop source: [`data/io.github.onbeskarakterli.Pivra.desktop`](data/io.github.onbeskarakterli.Pivra.desktop).
`Exec=pivra %u` admits one URI; `MimeType=image/jpeg;image/png;` lists input
candidates; `DBusActivatable=false` matches independent Exec launches, not remote
single-instance activation. A future native installation must place `pivra` on
PATH and install the entry under the distribution's applications directory.
This task **installs nothing and changes no MIME defaults**, including video defaults.
`desktop-file-validate`, GIO parsing, and real `gio launch` of the source entry
with a temporary PATH binary and isolated XDG directories passed on Xvfb.
This technical launch test is distinct from **real Nautilus double-click/default
photo application testing: NOT RUN (gate DESKTOP-06)**. JPEG/PNG decode is also
NOT RUN; that is PIV-09. No default photo-viewer change is recommended for this
placeholder-only development build.

Validation: **26 Rust tests passed (16 unit + 10 integration), 0 failed/ignored**;
fmt/check/build/Clippy and fixture integrity passed. Original empty-window
X11/Sway/Weston checks passed. Additional path + encoded-URI windows passed X11
and Sway with independent normal close/exit 0; Weston proved an associated
native Wayland surface and real buffer commit for URI input. Weston normal
close and physical GNOME/Wayland remain NOT RUN. The GIO desktop launch test
observes normal WM close/process termination, but cannot retrieve the detached
application exit status; direct GUI tests check exit 0 separately.

Run additional file-input checks after the existing activation/build:

```sh
GTK_A11Y=none GSK_RENDERER=cairo GDK_BACKEND=x11 \
  xvfb-run -a dbus-run-session -- python3 tests/gui_smoke.py target/debug/pivra --files
dbus-run-session -- python3 tests/wayland_smoke.py target/debug/pivra --files
PIVRA_WESTON_PREFIX=/workspace/pivra-environment/weston/usr \
  dbus-run-session -- python3 tests/weston_smoke.py target/debug/pivra --files
desktop-file-validate data/io.github.onbeskarakterli.Pivra.desktop
GTK_A11Y=none GSK_RENDERER=cairo GDK_BACKEND=x11 \
  xvfb-run -a dbus-run-session -- python3 tests/desktop_smoke.py
```

**PIV-06 DONE** covers the tested input/placeholder/desktop-entry infrastructure;
the final Nautilus double-click acceptance remains DESKTOP-06, NOT RUN. Phase 0's
independent clean-native-install gate remains INCOMPLETE as recorded in PIV-05;
this explicit PIV-06 request authorized this limited work without declaring that
gate passed. Glycin loader/ABI/SDK and Codex sandbox restrictions, GStreamer
SDK/gtk4paintablesink/Libav gaps remain in the technical capabilities document.
Before later decoder/player work those runtime gates must pass. The following PIV-07 section records folder discovery; sorting/navigation remain unimplemented.


### PIV-07 neighboring media discovery — 2026-10-21

Opening a PIV-06 photo now starts **one nonrecursive folder-discovery job per
window** on GIO's blocking I/O pool. `src/discovery.rs` uses only std and existing
core models; it returns a `MediaCollection` plus explicit per-entry issues.
The window retains its own GFile, opened item and catalog. GTK updates run on the
main context after completion; no directory walk runs on the GTK thread. The
PIV-06 initial input checks remain unchanged. No production dependency was added.

| Candidate kind | Case-insensitive extensions |
|---|---|
| Image | jpg, jpeg, png, webp, gif, bmp, tif, tiff |
| Video | mp4, mkv, webm, mov, avi |

These are **candidate extensions, not decoder/playback support**. Direct CLI/GIO
admission remains one local jpg/jpeg/png file; the desktop entry still lists
only image/jpeg and image/png. Video defaults are untouched. No image decoding,
video playback, natural sorting, navigation, thumbnails or watching was added.

The scanner uses the opened path's lexical parent (relative paths are made
absolute), without canonicalizing to a symlink target's directory. Readable
regular-file symlinks are followed for inspection while their **alias paths stay
in the catalog**. Different aliases/hard links are distinct entries; no inode
or canonical-target deduplication occurs. A target may be outside the folder,
but discovery never recurses there. Directory links, subdirectories and special
files are excluded; hidden regular candidate files are admitted. Broken links,
disappearing entries and unreadable candidate files produce explicit issues.
Unsupported extensions are ignored without decoding or content sniffing.
Linux non-UTF-8 filename bytes are retained in paths; only display text is lossy.

Selection uses the **opened alias/path**, not the first enumerated entry or
canonical target. At PIV-07 the enumeration order was unspecified; PIV-08 now sorts the completed candidate list naturally (policy below). If the opened entry has
disappeared or is no longer an accessible candidate, neighbor discovery fails.
A standalone empty/no-candidate directory yields an empty collection with no
selection; the app never substitutes an arbitrary neighbor for its opened file.

Successful file-window titles append the real catalog count, for example
`photo.png — Pivra (3 candidates)`; the selected name is read from the completed
catalog. The placeholder still explicitly states that image decoding is not
implemented. Per-entry errors display a warning count and incomplete status,
with escaped-path stderr diagnostics. A fatal folder error displays
`folder scan failed`, retains only the opened item, and reports stderr. Closing
such a valid placeholder normally still exits 0; this indicates window lifecycle,
**not a successful folder scan**. No-argument windows remain unchanged.

Jobs check a per-window cancellation flag between entries and before returning;
close/destroy sets it, and weak UI references prevent late updates to closed
windows. Cancellation returns an error rather than a successful partial catalog.
**Limits:** cancellation cannot interrupt a blocked kernel/network filesystem
call. Metadata/read-only descriptor checks are point-in-time, not a stable
snapshot; symlink/TOCTOU replacement remains possible. Scanning is O(entries)
in time and O(candidates + issues) in memory, with no hard directory-size/RAM
cap. Shared GIO workers may be delayed on slow mounts. The 1,200-candidate test
is a correctness test, not a latency benchmark; real network mounts, very large
directories and close-during-blocked-I/O behavior are NOT RUN. Descriptor checks
read no media bytes and never modify user contents.

Validation: **40 Rust tests passed (18 unit + 22 integration), 0 failed/ignored**;
fmt/check/build/Clippy, fixtures and desktop-file validation passed. New tests
cover mixed formats/case, selection, empty/single/many (1,200 candidates plus
100 excluded files), Unicode/raw Linux names, relative/alias parents, file/link
errors, inaccessible folders and cancellation. A deterministic iterator seam
removes an entry between enumeration and inspection and injects an iteration
error; healthy candidates survive with explicit issues. Permission tests actually
ran as an unprivileged user. The library also compiled directly with rustc,
without Cargo/GTK/GIO linkage.

X11 and Sway passed both original empty-window regressions and two independent
file/URI windows whose titles prove the **selected catalog entry and exact count**
(two PNGs + one text-only video-model fixture; an unsupported file and nested PNG
are excluded). Both windows closed normally with exit 0 and empty diagnostics.
Weston proved the same selected URI/catalog count in a real native Wayland
surface with SHM buffer/configure/commit evidence; normal close remains NOT RUN.
The video model is not playable video. X11 additionally verified visible partial
warning and fatal permission-error states with controlled diagnostics; real GIO
desktop launch still passed with the single-candidate title. Fixture copies
remained unchanged. Physical GNOME/Wayland and Nautilus DESKTOP-06 remain NOT RUN.

Existing `--files` smoke commands above now validate catalog completion as well.
Additional error-state check (after build/activation):

```sh
GTK_A11Y=none GSK_RENDERER=cairo GDK_BACKEND=x11 \
  xvfb-run -a dbus-run-session -- python3 tests/discovery_smoke.py
```

**PIV-07 DONE; PIV-08 completion is recorded below.** PIV-05 media/sandbox risks, PIV-06
DESKTOP-06, and the separate Phase 0 clean-native-install gate remain open.


### PIV-08 natural filename order — 2026-10-21

The completed candidate list is now naturally sorted in the existing worker,
before catalog construction and opened-path selection. Images/videos share one
order. Paths/kinds are unchanged; the opened alias remains selected even when
`foto10.jpg` sorts after `foto1.jpg`. Other MediaCollection callers still retain
caller order; no generic reordering/navigation API was added.

The pure std-only `natural::compare_paths` compares the **whole filename including
extension**, using Linux OS bytes without lossy UTF-8 conversion:

1. Primary comparison folds only ASCII A–Z. Maximal ASCII 0–9 runs compare by
   significant digit count then digit bytes, ignoring leading zeros. No integer
   parsing occurs; number length cannot overflow an integer conversion.
2. Only when the entire primary comparison ties, the first different numeric-run
   width decides: fewer leading zeros first (`foto2 < foto02 < foto002`). Later
   primary tokens take precedence over padding (`a02b3 < a2b10`).
3. Remaining ties use exact filename bytes (uppercase before lowercase when all
   primary tokens/widths tie), then exact full-path bytes for identical filenames
   in different parents. Identical byte paths compare equal; no canonicalization.

Examples: `foto1.jpg, foto2.mp4, foto10.jpg`; same roots use extension text:
`same.jpg, same.mp4, same.png`. Signs, decimals and punctuation are ordinary bytes;
non-ASCII digits are not number tokens. This is **not Turkish locale collation**,
Unicode case folding or normalization. ASCII I/i fold; dotted İ and dotless ı
remain distinct byte sequences. Tested example: `I2, i2, ç2, İ2, ı2, ş2, 🦀2`.
Invalid UTF-8 Linux filenames are compared and preserved as raw bytes.

In-place sorting has no comparator I/O or allocations/new crate. Cost is
O(N log N × L) byte-comparison work (L: compared filename/path length), not a
benchmark claim. Cancellation checks surround sorting and remain in enumeration;
the standard sort itself is not preemptible. Existing close/weak-reference guards
remain unchanged. Blocked syscall/network mounts, extreme directory scale and
TOCTOU/RAM risks remain; no latency guarantee is claimed.

Validation: **47 Rust tests passed (23 unit + 24 integration), 0 failed/ignored**;
fmt/check/build/Clippy and fixtures passed. Five new comparator tests cover natural
numbers, padding, extensions, multiple blocks, ASCII case, Unicode/raw bytes,
10,000-digit values and an 18-name corpus checked for antisymmetry, equality,
transitivity and rotated/reversed input independence. This is finite test evidence,
not an exhaustive proof. Two new real-filesystem catalog tests assert mixed-media
order under opposite creation orders and alias/raw-byte selection. The existing
1,200-candidate test now checks the entire `0.png … 1199.png` sequence and selected
index 917. Empty/single, discovery errors and cancellation regressions passed.

X11/Sway empty and file/URI GUI tests passed with independent normal close/exit 0.
Weston passed native mapped-surface/title/SHM-buffer/configure/commit checks in both
modes; normal close remains NOT RUN. GIO desktop launch, desktop-file validation
and X11 scan warning/failure UI checks passed. Actual order is asserted in real
catalog tests; no list/navigation UI was added. Physical GNOME/Wayland and Nautilus
DESKTOP-06 remain NOT RUN. See the tracker for exact commands and commit SHAs.

**PIV-08 DONE; PIV-09 not started.** Direct CLI/GIO admission, JPEG/PNG MIME entry,
video defaults, production dependencies and existing environment are unchanged.
No decoder, player, navigation, thumbnail, zoom, watcher, CI or license was added.
PIV-05 media/sandbox risks and the separate Phase 0 clean-native-install gate remain open.


### PIV-09 secure image integration — BLOCKED, 2026-10-21

Source now attempts secure JPEG/PNG loading with exact **Glycin 3.0.8**, matching
gtk/gdk 0.10.3, GLib/GIO 0.21.5 and adw 0.8.1. Cargo.lock records the new client
chain (common 1.0.3/utils 4.0.4). Glycin's gdk feature requires **native GTK >=4.16**;
the manifest explicitly raises the old 4.6 feature baseline. Existing GTK 4.18.6 /
adw 1.7.6 satisfy it. Rust stays stable 1.99.1; declared MSRV 1.85 was not separately
compiled. No GStreamer, extra production codec, editing or playback crate was added.

`src/image_loader.rs` contains no widgets: GIO source -> forced Bwrap sandbox ->
Glycin metadata/MIME -> oriented frame. It accepts actual detected JPEG/PNG MIME,
not extension as decode proof. Glycin `apply_transformations(true)` applies EXIF
once; the view never rotates again. No Auto/NotSandboxed/fallback decoder is used;
base-directory exposure remains disabled. Errors return controlled text.
`src/image_view.rs` runs the full async load/transform on GIO's blocking pool in a
worker-owned GLib context. Only frame.texture and widget updates run on the GTK
thread. Cancellable on close/destroy and weak widgets discard late results.
Catalog discovery/order/selection remain independent and unchanged.

The GTK view uses an expanding Picture/Overlay with **ScaleDown + can_shrink**.
The intended Fit policy is aspect-preserving, centered containment with no
upscaling above intrinsic size (one source pixel per logical layout unit); resize
uses GTK's layout. Physical 100% pixels at HiDPI/fractional scale are NOT validated.
Loading/errors have labels; a successful frame would replace them with its actual
texture. This is compiled source/API configuration, **not observed image/Fit
success**. No Fit/100% controls, zoom/pan or navigation were added.

Limits: compressed regular file <=64 MiB, nonzero dimensions <=32768 per edge and
<=32 MP, actual returned frame <=512 MiB. Tests reject oversized sparse input,
invalid dimensions, missing files and pre-cancelled work without starting decode.
These are admission/post-frame checks, not a hard total-RSS guarantee: loader
metadata can allocate first, transformations/copies and multiple windows add
memory, and Glycin's own sandbox memory limit remains upstream policy. TOCTOU,
blocked filesystem I/O and cancellation timing require further validation.

**Independent blockers:**

- Configured official Debian 13 metadata still offers glycin-loaders 1.2.1+ds-2
  (1+ protocol), not Glycin 3.x's 2+. No 2+ executable/config is installed.
  Official GNOME tag 2.0.0 was inspected at commit
  `724f18c5f3519885995428a7e2e6e5df9ca2ab27` (tarball 2.0.0 includes client 3.0.0 /
  utils 4.0.0 / loaders 2.0.0). Source checkout is research, not a built/installed
  JPEG/PNG loader. Broader upstream codec/editor bundles were not installed.
- `bwrap --unshare-all --die-with-parent --new-session --ro-bind /usr /usr
  --symlink usr/lib /lib --symlink usr/lib64 /lib64 -- /usr/bin/true` again failed
  with `setting up uid map: Read-only file system`. This is the Codex namespace
  restriction, not proof that native Debian/Glycin is incompatible. No mount,
  seccomp, privilege or sandbox bypass was attempted.

Only necessary LCMS/seccomp SDK packages were downloaded from official signed APT
metadata and extracted to the existing user prefix: liblcms2-dev 2.16-2+deb13u2 /
libseccomp-dev 2.6.0-2; pkg-config 2.16 / 2.6.0. Rust/GTK activation and system
packages were not reset/upgraded. The additive environment helper is
`/workspace/pivra-environment/piv09/prepare-image-sdk.sh`.

**Verification:** fmt/check/build/test/Clippy pass; **49 Rust tests = 25 unit + 24
integration**, 0 failed/ignored. These do not include successful image decode.
18 original assets and repeatable encoding/TIFF-tag checks pass; see
[image fixture provenance](tests/fixtures/images/README.md). The pixel/orientation
gate was attempted with valid PNG and JPEG separately; each returned **77/BLOCKED**
with `No image loaders are configured`. EXIF 1–8 pixels, actual GTK image/Fit,
resize, successful parallel decoding and close during active decoding remain
BLOCKED/NOT RUN. Broken-file decode errors and MIME mismatch success branches
were prepared but cannot be certified from a missing-loader error.

Empty-window X11/Sway/Weston regressions pass. File-window tests explicitly support
`--expect-image-error` for this blocked environment: they require a real controlled
loader error and print **BLOCKED image decode/pixels**, while independently testing
window/catalog lifecycle. Without that explicit flag stderr remains strictly
empty; image failures are never silently accepted. X11/Sway normal closure passed;
Weston proved real mapped surface/configure/SHM-buffer commits only. Physical
GNOME/Wayland, Nautilus DESKTOP-06 and Weston normal close remain NOT RUN.
Desktop entry/GIO launch and discovery-warning/failure UI passed with the same
explicit blocked-runtime distinction. The entry is not installed or made default;
its JPEG/PNG Exec/MIME contract and video defaults remain unchanged.

After activation/build, use these **blocked-runtime** checks:

```sh
GTK_A11Y=none GSK_RENDERER=cairo GDK_BACKEND=x11 \
  xvfb-run -a dbus-run-session -- python3 tests/gui_smoke.py target/debug/pivra --files --expect-image-error
dbus-run-session -- python3 tests/wayland_smoke.py target/debug/pivra --files --expect-image-error
PIVRA_WESTON_PREFIX=/workspace/pivra-environment/weston/usr \
  dbus-run-session -- python3 tests/weston_smoke.py target/debug/pivra --files --expect-image-error
GTK_A11Y=none GSK_RENDERER=cairo GDK_BACKEND=x11 \
  xvfb-run -a dbus-run-session -- cargo +stable run --locked --offline --example image_probe
# Same command with: -- --jpeg-first
```

Resume PIV-09 in a namespace-capable native environment with a matching minimal
2+ loader, then verify real JPEG/PNG, EXIF 1–8 pixels, app presentation/Fit/resize,
errors and active-load lifecycle on X11/native Wayland. The current source is
**not a DONE image viewer**. PIV-10 has not started. PIV-05 video risks, DESKTOP-06
and the independent Phase 0 clean-native-install gate remain open.


### PIV-09 continuation: loader prepared, sandbox still BLOCKED — 2026-10-21

This update supersedes the earlier missing-loader observation; application source,
dependencies and existing tests were preserved. The official unmodified GNOME
**2.0.0** tag, commit `724f18c5f3519885995428a7e2e6e5df9ca2ab27`, was built with its
own frozen lockfile (`cargo +stable build --locked -p glycin-image-rs -j 2`).
The resulting **glycin-image-rs 2.0.0** uses utils 4.0.0/common 1.0.0; the client
remains Glycin **3.0.8**, family **2+**. No source/protocol/version patches or hacks.

Only this necessary image-rs executable was installed under the isolated user
prefix. Its unmodified upstream bundle inherently compiles several Rust formats
and editor helpers; no independent HEIF/JXL/RAW/SVG loader or system codec package
was installed. **Only image/jpeg and image/png loader entries are configured;
no editor entries** are registered. No Edit feature or extra format support is
claimed. Build/version/hash evidence is in
[the loader manifest](docs/probes/PIV09_LOADER_BUILD.json).

Reproducible preparation (existing SDK/activation required, no system installation):

```sh
bash tests/probes/prepare_glycin_loader.sh
source /workspace/pivra-environment/piv09/image-activate.sh
cargo +stable run --locked --offline --example loader_probe
```

The helper verifies exact source HEAD and a clean source checkout, uses the pinned
upstream lock, builds only the image-rs package, preserves modified user configs,
and creates `loader-build.json`/`image-activate.sh` in the environment directory.
It was rerun successfully with cached outputs. The activation sources existing
GTK setup and sets GLYCIN_DATA_DIR to
`/workspace/pivra-environment/piv09/loader-prefix/share`. Loader executable:
`.../loader-prefix/libexec/glycin-image-rs`; config:
`.../share/glycin-loaders/2+/conf.d/pivra-jpeg-png.conf`. GTK activation itself was
not changed. Use **image-activate.sh** for the following PIV-09 checks.

**Discovery PASS:** the real Glycin 3.0.8 supported_mime_types API returns exactly
JPEG/PNG; the probe also checks config groups and executable existence/mode.
This is configuration discovery, **not a successful loader IPC handshake/decode**.
With that config, actual PNG/JPEG attempts now advance past the missing-loader
error and issue Bwrap commands selecting the correct executable. Both still
return **77/BLOCKED** because the sandbox child exits before running the loader.
No positive pixel assertion was removed or weakened.

**Precise Codex cause, traced:** clone with CLONE_NEWUSER and the other requested
namespaces succeeds; `/proc/<child>/uid_map` then fails open(O_RDWR) with **EROFS**.
The /proc mount is read-only. UID/GID1000, effective capabilities0, NoNewPrivs1,
Seccomp2/one filter; user namespace limit2147483647. The high limit is not a
functional-sandbox guarantee. This is not a general native Linux/Glycin failure.
No mount, sysctl, seccomp or privilege change was attempted.

**Permitted alternate route checked:** the available Docker daemon's built-in
seccomp/default isolation was retained. A checksum-pinned official Debian trixie
image plus the existing bwrap/runtime libraries was tested with user1000:1000,
all capabilities dropped, no-new-privileges, read-only rootfs, no network, and no
host bind mounts. Final Bwrap result: namespace creation is not permitted (exit1).
The exact Docker seccomp/LSM/kernel subpolicy was not identified; no profile was
relaxed to investigate further. No privileged container, added capability, host
PID namespace or sandbox escape was used. This route therefore cannot provide
safe image testing with its present permissions either.

A separate **approved native Linux/VM environment** must already permit ordinary
unprivileged user namespaces, UID/GID map writes through its normal procfs and
the nested mount/PID/network/etc. namespaces required by Bwrap. Its existing
seccomp/LSM policy must permit that workflow; no host security changes are part of
this task. It needs the same client/GTK libraries, matching 2+ loader/config and
private writable runtime directories. First require the minimal sandboxed true
probe to succeed, then loader discovery and the existing actual image_probe.
Current tools expose no other verified namespace-capable native environment.

Continuation results: **49 Rust tests PASS**, all mandatory Cargo checks, both
fixture checks and desktop validation PASS; **8 scoped GUI/helper runs PASS**
(X11 empty/file + desktop + discovery-error, Sway empty/file, Weston empty/file).
File tests explicitly expect a secure-loader failure, not image success. The
separate loader discovery probe PASS; the two real PNG/JPEG gates BLOCKED.
EXIF1–8, color/pixel correctness, actual app Fit/centering/resize, successful
parallel images and close during active decode remain **NOT RUN/BLOCKED**.
Weston normal close, physical GNOME/Wayland and Nautilus DESKTOP-06 remain NOT RUN.

**PIV-09 remains BLOCKED; PIV-10 has not started.** No application code rewrite,
production dependency change, system package change or security bypass occurred
in this continuation. Earlier SDK/Phase0/video risks remain as separately recorded.


### PIV-09 Arch Linux evidence and remaining gates — 2026-10-21

**Evidence source: user-reported local tests, not Codex execution.** Tested local
commit: `459fbea40cc3048eabc14d8c996db1ab7023c726` on real Arch Linux / GNOME
Wayland. Reported versions: Rust/Cargo 1.99.1, GTK4 4.24.1, libadwaita 1.10.0,
Glycin 2.2.3 with compatible 2+ loaders, Bubblewrap 0.13.1. Glycin 2.2.3 is the
user's reported native environment version; the project Cargo dependency remains
locked to client 3.0.8. Exact Arch package/loader executable inventories and raw
logs were not supplied; no additional package or ABI claims are inferred.

The user reports user namespaces and Bwrap isolation PASS; PNG and JPEG
`image_probe` exit 0; EXIF orientation 1–8 pixel checks, Unicode paths and controlled
invalid-image errors PASS. In the real Pivra GNOME/Wayland window, photos were
visible, centered, aspect-preserving Fit worked, resizing refit the image and
normal close completed without error. These are **user-reported PASS**, including
manual geometry; they are not automated application geometry measurements.
The user also reports fmt/check/build/test/Clippy (locked where applicable) PASS,
49 tests, zero failures and a clean worktree. This does not establish a clean
native installation reproduction or close the separate Phase 0 gate.

Codex reran `bash tests/check.sh` using the existing image activation: fmt,
check/build/test/Clippy locked/offline all PASS; **49 tests = 25 unit + 24 integration**,
0 failed/ignored. Both fixture manifests and CLI/catalog/natural-order/desktop
regressions remain intact. Eight existing GUI/helper calls passed their scoped
checks: four X11/Xvfb, two Sway, two Weston. File modes explicitly expected the
real secure-loader error; no decoded photo success is claimed for those runs.
Weston normal close remains NOT RUN. The user's real GNOME window close result
is a separate PASS; real Nautilus DESKTOP-06 was not reported.

New executable gate (not included in the user's `459fbea` results):

```sh
# In a graphical session with working secure Glycin 2+ loaders:
GDK_BACKEND=wayland cargo run --locked --example image_gui_probe
```

`image_gui_probe` imports the actual production `image_view` and `image_loader`;
it does not add application switches or another decoder. It schedules two real
view loads before pumping the main context (large PNG and EXIF-6 JPEG), requires
both texture results, and uses actual GTK WidgetPaintable/GSK rendered snapshots
to measure image alpha bounds against intrinsic-size ScaleDown geometry and
centering. It repeats after a resize and after closing the first window, requiring
the other to remain visible. This is automatic offscreen snapshot geometry of the
real view, not compositor screenshot or physical-display proof. Concurrent job
scheduling does not prove that the decoder execution intervals overlap.

The new probe compiles and passes Clippy here; its Codex Xvfb run returns **77 /
BLOCKED** at real secure decode. Geometry and successful two-image assertions
were **not executed successfully**. No synthetic texture, mock decoder, assertion
removal, NotSandboxed or sandbox fallback was used to produce a passing result.

Remaining PIV-09 acceptance gaps only:

- Run the new two-image load/automatic geometry/resize/independent-close gate in
  the working native environment and record its result at its actual commit.
- Verify close while a secure decoder is demonstrably active, with a second
  loaded window surviving; verify no stale UI callback/crash and bounded worker/
  loader cleanup. Existing pre-cancel tests and post-load normal close do not
  establish this. A fast small fixture or closing immediately without observing
  active decode is insufficient; no production delay or unsafe loader was added.

**PIV-09 TESTING**, not DONE. The Codex-specific sandbox restriction remains;
the user-reported native successes resolve the earlier global absence of runtime
evidence. PIV-10 remains untouched.


### PIV-09 final test stage — active-decode close still TESTING

**User-reported Arch Linux / GNOME Wayland evidence, not Codex execution:** at
`7d1acf4932b6b7dfa21e42c00777b965cee54bcc`, `image_gui_probe` exited 0.
The reported real-view snapshot checks passed for 1600×1200 and 32×64 images in
800×600 windows, two independently loaded images, resize to 480×320 and independent
normal close after successful decode. These supersede the earlier missing
geometry/two-image gate, not the separate Codex sandbox restriction. Two decoder
execution intervals were not measured to overlap.

The remaining gate is implemented in `examples/active_decode_close_probe.rs` and
`tests/active_decode_close.py`. It requires a real secure loader owning a partially
written RGB output, uses pidfd SIGSTOP/recheck to remove the completion race, closes
the actual view, resumes the decoder, and checks joined worker, released widgets,
no stale UI mutations, unchanged survivor pixels and bounded child cleanup.
A partially written real decode is paused for this controlled cancellation test;
it is not an unpaused timing benchmark. Production retains mandatory Bwrap and
its existing cancellation/result guards; the new internal completion observer is
a no-op in normal builds.

Exact native command, proof criteria, JSON interpretation and observation limits:
[active decode test instructions](docs/PIV09_ACTIVE_DECODE_TEST.md). No sudo or
sandbox policy changes are requested. Missing `/proc` observation rights, a
missed partial-frame interval or a different loader memory layout yields NOT RUN,
never PASS. The native positive path has not yet been run by the user or Codex.
Codex's new probe exits77 at baseline secure decode; 49 Rust + 6 Python observer
tests pass. Existing eight GUI/helper calls pass only their established window/
catalog/error scopes; Weston normal close remains NOT RUN.

**PIV-09 TESTING; only the active-decode close/cleanup result remains outstanding.
PIV-10 has not started.**


### PIV-09 active-decode observer diagnosis — 2026-10-21

The user reports the old native test still exited77/NOT RUN despite detecting the
real image-rs loader (8,285 process samples), the expected memfd/mapping (30 buffer
observations), size 96,000,000 bytes, first 12 bytes `1153bf1153bf1153bf1153bf` and
last 12 bytes zero, without proc access errors. These are **user-reported**, not
Codex observations. One initial 12-byte sample supports the format but does not
certify the full 96-byte conditions at an active stopped checkpoint.

The driver previously started observing after window presentation and performed
extra process/security discovery before STOP. These are verified timing risks,
not a proven sole explanation for the native failure. The updated test arms its
observer before target launch, pins PID/fd once, polls the cached output, handles
transient fd races independently and checks security only after STOP. All original
acceptance conditions remain; no production decoder or sandbox policy changed.
Per-stage DIAG counters, exact samples/timestamps and first uncompleted stage now
separate discovery, prefix/tail, stop, mapping/security and cleanup failures.

After updating the `dev` checkout, the **single native GNOME Wayland command** is:

```sh
bash tests/run_active_decode_close.sh
```

It builds the matching Rust probe and writes `/tmp/pivra-active-decode-close.json`.
[Full diagnostic interpretation](docs/PIV09_ACTIVE_DECODE_TEST.md).
Codex: **49 Rust + 17 Python observer tests PASS**, fmt/check/build/Clippy PASS;
real probe **NOT RUN/77 at baseline sandbox**. Native v2 active close is untested;
two-decoder time overlap remains NOT MEASURED. **PIV-09 TESTING**, never DONE here;
PIV-10 has not started.


### PIV-09 V3 — discover existing pooled loaders

The user reports V2 native exit77 at `loader_found` after 5,476 scans. A separate
5,445-scan diagnosis saw bwrap PIDs 142105/142122 and loader PID 142123 from the
first scan onward. **User-reported Arch observations, not Codex execution.** V2's
baseline PID subtraction hid that loader; V3 removes this concrete discovery bug.

The locked Rust Glycin 3.0.8 client uses a global process pool with default 30-second
idle retention. Source supports reuse; actual Arch reuse is not yet proven. V3
inventories pre-target process identities and frame inodes, scans both old/new
loaders, and requires a **fresh target frame** with every original pixel/STOP/
ownership/mapping/sandbox/pending-load guard. Old matching partial or completed
frames are rejected. Loader origin and target entered/returned telemetry are saved.

Target memory release is verified before normal survivor/app close. An idle pooled
baseline process is not itself a leak; all observed child identities must still
exit after normal app shutdown within the original 10-second total cleanup budget.
Production sources/dependencies and mandatory Bwrap remain unchanged.

Codex: **49 Rust + 30 Python observer tests PASS**, mandatory Cargo checks PASS;
real V3 probe **NOT RUN/77 at baseline sandbox**. Actual native reuse/active close
remains unverified; **PIV-09 TESTING**, PIV-10 untouched. Update `dev`, then run:

```sh
bash tests/run_active_decode_close.sh
```

[Current design, origin fields and cleanup interpretation](docs/PIV09_ACTIVE_DECODE_TEST.md).


### PIV-10 previous/next navigation — TESTING, 2026-10-22

Use **Left/Right** to move through the existing natural mixed-media order in each
window. Navigation stops at the ends; empty/single-item catalogs keep their
selection. Video entries are included and show `Video playback is not available
yet.`; only JPEG/PNG rendering is implemented. Broader image candidates show an
unsupported-format message. Source media and desktop/default player associations
are unchanged. Ctrl/Alt/Super arrows are left for other bindings.

The window keeps one ImageView. New selections cancel the old request, clear its
paintable and reset ScaleDown Fit. Both cancellation and current-token identity
gate late callbacks. Errors do not disable navigation. Directory discovery remains
a one-time snapshot; file watching/refresh, wrapping settings and later tasks are
not part of this change. No new toolbar, zoom tool or playback engine was added.

Codex: **54 Rust + 30 existing Python observer tests PASS**; mandatory Cargo checks
PASS. Twelve GUI/helper calls pass their stated scopes, including real X11 keys,
Sway navigation and deterministic controlled-error callback races. These do not
claim real photo pixels: strict new pixel navigation exits77/BLOCKED here.

In the working native GNOME/Wayland environment, update `dev` and run:

```sh
bash tests/run_navigation_probe.sh
```

This requires real secure JPEG/PNG results, validates current texture pixels and
Fit/center snapshots across navigation, error recovery and independent windows.
Also confirm physical Left/Right after changing focus between the picture/header
controls in normal Pivra. [Commands, actual results and exact remaining gates](docs/PIV10_VALIDATION.md).
PIV-10 remains TESTING pending those native results; PIV-11 has not started.


## PIV-11 final native acceptance — DONE (2026-10-22)

At `f96a0db029fa0e5157318d4bdfd4ef494d3baf33`, user ran `bash tests/run_folder_watch_probe.sh`: exit0/PASS, real filesystem events and secure Glycin refresh, previous frame retained until explicit user refresh action. In normal GNOME/Wayland application, user observed the refresh banner and confirmed clicking Yenile displays the modified image after a loading animation. Previous new-window impression was not confirmed by terminal logs (two separate cargo run commands); actual GtkWindow object identity not manually instrumented. Native acceptance met; PIV-11 DONE, PIV-12 TODO. The mixed Turkish/English UI strings are scheduled for PIV-39 i18n; Phase 0 clean install INCOMPLETE and DESKTOP-06 Nautilus NOT RUN remain independent.


## PIV-12 native ve manuel kabul kapanışı — 24.10.2026 — DONE

Kullanıcı `b12970f6a3a7ea2dddc890a747977d5d68437a63` üzerinde `bash tests/run_media_failure_probe.sh` çalıştırdı: **exit0**, gerçek secure Glycin, bozuk/yarım/boş/yanlış format JPEG/PNG hataları, sağlam kareye geri dönüş, katalog/seçim/monitor/bildirim izolasyonu, Yenile ve close cleanup PASS. Bozuk dosya hata logları beklenen test senaryolarıydı. Gerçek GNOME/Wayland manuel testinde kullanıcı iki Pivra penceresi açtı; birincide kırmızı→yeşil→bozuk→mavi gezinmesini, diğer pencerenin etkilenmemesini ve birinci kapatılınca ikincinin çalışmaya devam etmesini **PASS** bildirdi. Aktif gerçek secure decode sırasında silme/kapatma ve decoder süreçlerinin zamansal çakışması **NOT MEASURED**; bu ölçümler PASS sayılmaz. PIV-12 DONE; PIV-13 TODO. Faz 0 bağımsız temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN, PIV-39 tr/en i18n planı açık.


## PIV-13 native kabul ve kapanış — 24.10.2026 — DONE

Kullanıcı Arch Linux / GNOME Wayland üzerinde `d60c33078c08720524413d204f7131b34df2ff5b` ile `bash tests/run_view_modes_probe.sh` çalıştırdı: **exit0/PASS**; gerçek Glycin JPEG/PNG/EXIF 1–8 decode ve Fit/Fill/Actual render geometrisi, kırpma, merkez, resize, iki bağımsız pencere, yeni medyada Fit sıfırlama, bozuk medyadan toparlanma ve kaynak bütünlüğü doğrulandı. Kullanıcı normal GNOME uygulamasında modları manuel kontrol ettiğini bildirdi. %100 modunda büyük fotoğrafın sadece kesitinin görünmesini ve fareyle sürükleyerek gezinemediğini gözledi; ilki Actual davranışı, ikincisi henüz uygulanmamış **PIV-14 pan** kapsamıdır, PIV-13 hatası değildir. Gerçek HiDPI fiziksel 1:1 ölçek testi NOT RUN; Actual = bir kaynak piksel / GTK mantıksal birimi. Önceki Codex ortamı exit77 sonuçları geriye dönük PASS değildir. **PIV-13 DONE; PIV-14 TODO**. Faz 0 temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN, PIV-39 i18n planı açık.


## PIV-14 Arch native ve gerçek fare kabulü — 24.10.2026 — DONE

Kullanıcı `6c55eab27d4c5bcf980246c2c69fb08671d2d2be` üzerinde `bash tests/run_zoom_pan_probe.sh` çalıştırdı: **exit 0**. Gerçek Glycin JPEG/PNG/EXIF baseline ve üretim render/mod/geometri, iki pencere izolasyonu testleri PASS; betiğin `view_modes_probe --zoom-pan` yolunu çağırdığı doğrulandı. Bozuk PNG testindeki decoder hata satırı beklenendi. Kullanıcı ayrıca fiziksel GNOME/Wayland fare testi için **olumlu sonuç** bildirdi: %100 görüntü üzerinde sürükleme, imleç ankrajlı yakınlaştırma, Fit sıfırlama ve resize akışı için bir problem bildirilmedi. Ayrı ölçülmeyen HiDPI fiziksel piksel eşlemesi, gerçek decoder süreç zaman çakışması ve FPS/gecikme ölçümleri PASS sayılmaz. **PIV-14 DONE; PIV-15 TODO.** Faz 0 temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN ve PIV-39 i18n kararı açık kalır.


## PIV-15 Arch native ve GNOME manuel kabul kapanışı — 24.10.2026 — DONE

Kullanıcı `3313113246baca8b2e19073b0ed20c64f569a50c` üzerinde `bash tests/run_resampling_probe.sh` çalıştırdı: **exit 0**, gerçek Glycin JPEG/PNG/EXIF 1–8, secure PNG checker Nearest palette ve Smooth interpolasyon, aynı decoded texture/paintable, yeni fotoğrafta tercih, iki pencere izolasyonu, kaynak bütünlüğü PASS. Beklenen bozuk PNG hata logları test verisindendi. Kullanıcı normal Pivra'da küçük checker görselinde Smooth/Nearest arasında görünür piksel farkını ve değiştirirken zoom/konumun korunduğunu **manuel PASS** olarak onayladı. Fill görüntüyü alanı dolduracak şekilde ölçeklediğinden bu davranış doğrudur; %100 hatası doğrulanmadı. Çok küçük test görsellerinde %3200 manuel zoom sınırı yetersiz hissedilebilir; mevcut PIV-14 sınırı korunur, artırma önerisi ayrı değerlendirmedir, PIV-15 kabulünü engellemez. Gerçek GPU/HiDPI eşlemesi, FPS/RSS ölçümü ve decoder overlap **NOT MEASURED**; Codex exit77 kayıtları geriye dönük PASS sayılmaz. **PIV-15 DONE, PIV-16 TODO.** Faz 0 bağımsız temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN ve PIV-39 i18n planı açık kalır.


## PIV-16 native ve manuel kapanış — 24.10.2026 — DONE

Kullanıcı gerçek Arch/GNOME ortamında `0822e4b2028d414304ef3b2078bc5bc6d6da8852` üzerinde `bash tests/run_async_cancel_probe.sh` çalıştırdı: **exit 0**. Gerçek JPEG/PNG/EXIF baseline PASS; secure async latest-pixel, A→B→A, hata toparlanma, iki pencere ve kaynak bütünlüğü PASS. `LoadStats`: running=0, pending=0, started=11, completed=11, max_running=3, dropped_pending=97, suppressed=6, timed_out=0. Ek PIV-09 aktif secure decode close checkpoint'inde partial memfd/frame, pidfd/mapping/sandbox, worker join, stale mutation=0, survivor pixels ve child cleanup PASS. `Cleanup SIGCONT unavailable: No such process` süreç zaten çıktığı için fatal değildi; toplam test exit0. Burst içinde decoder active interval/overlap **NOT MEASURED**. Kullanıcı gerçek uygulamada hızlı Sol/Sağ, A→B→A ve yükleme sırasında görüntüleme tercihlerini denedikten sonra olumlu manuel yanıt verdi; ayrıntılı ayrı ölçüm iddiası yok. **PIV-16 DONE; PIV-17 TODO.** Faz 0 temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN ve PIV-39 i18n planı açık.


## PIV-18 — full-only fallback (TESTING)

The current JPEG/PNG provider has no certified early-preview path. Full decode
remains single-pass, secure and asynchronous; no second full decode or fake
preview was added. Lifecycle state is tracked through completion/cancellation.
Preview/full model transitions are tested separately from real presentation.
Actual early-preview and upgrade viewport continuity remain unavailable, not PASS.
[Capabilities, limits and native command](docs/PIV18_VALIDATION.md).


## PIV-17 Arch native ve manuel kabul kapanışı — 24.10.2026 — DONE

Kullanıcı `07f47b5fb21200d8337933893128b700e5b8329d` üzerinde `bash tests/run_preload_probe.sh` çalıştırdı, **exit 0**. Gerçek secure JPEG/PNG/EXIF baseline, cache hit/texture reuse/Fit/filter, next-before-previous preload, LRU eviction, atomic replace/delete identity, hesaplanan bellek bütçesi, injected memory pressure, source integrity, worker-entry promotion ve foreground priority, window close/drain **PASS**. Örnek: hits=3, misses=6, evictions=5; enabled başlangıcı foreground=1/background=2; disabled foreground=2. Native secure secondary probe promoted=1, cancelled=2, started=completed=4, cache entries/bytes=0 after drain. PIV-09 gerçek active decoder memfd/pidfd/mapping/sandbox close checkpoint, job join, survivor pixels, children cleanup PASS. `Cleanup SIGCONT unavailable` hedef PID zaten çıkmışken fatal değildi. Kullanıcı gerçek GNOME arayüzünde hızlı Sol/Sağ gezinmenin sorunsuz olduğunu bildirdi; ayrı gerçek RSS/VRAM/FPS/hız kazanımı, OS gerçek memory-pressure sinyali ve decoder overlap **NOT MEASURED/NOT RUN**. **PIV-17 DONE; PIV-18 TODO.** Faz 0 temiz kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN, PIV-39 i18n açık.


## TH-04/TH-05 thumbnail system — TESTING (2026-10-25)

Thumbnail selection keeps stable cells and horizontal scroll. The first completed
folder scan aligns the selected item after real GTK allocation. Cache loading is
progressive across the full photo catalog, prioritizes the viewport, and keeps a
bounded per-window texture LRU. Missing thumbnails use an optional, API-checked
native GNOME factory sidecar; cache pixels still require secure Glycin/Bwrap.
No independent JPEG/PNG decoder or fullscreen feature was added.

Arch GNOME Wayland acceptance (private TEST media/config/cache, then two real
production windows; native decoder mode, not the controlled Codex stub):

```bash
bash tests/run_thumbnail_system_probe.sh
```

Missing GNOME factory development API/helper returns77/NOT RUN; it is not a
successful generation test. Normal cache-only viewing remains available. Native
acceptance at implementation time was pending; user acceptance is recorded below. See [root cause and validation](docs/TH05_THUMBNAIL_SYSTEM_VALIDATION.md)
for the real SHAs, controlled/native evidence split and remaining limits.


TH-04/TH-05 native closure (2026-10-25): the user reported successful Arch GNOME
Wayland acceptance after `1b11074`,120/120 real secure thumbnails, failed0 and
manual two-window acceptance. TH-04/TH-05 DONE; historical Codex NOT RUN records
remain distinct. PIV-18 is still TESTING; TH-06 acceptance is separate.


TH-06 implementation (subsequently accepted by the user): the thumbnail panel now meets the bottom/side edges, with the
floating toggle above the open panel. Photo requests keep the last ready pixels
and zoom/pan until the new texture is ready; failures name the target and clearly
identify the retained previous photo. Secure Glycin, cache and prefetch remain in
place. Controlled GTK tests passed; native TH-06 visual acceptance is pending.
See [TH-06 evidence and manual checks](docs/TH06_VALIDATION.md). Arch GNOME:

```bash
bash tests/run_thumbnail_system_probe.sh
bash tests/run_image_transition_probe.sh
```


TH-06 regression follow-up: visible/selected thumbnails now have residency
priority, and sandbox-decoded cache pixels are retained as bounded RGBA8 cell
textures. The 64-texture/16 MiB limits remain; main-photo cache accounting and
secure loading are unchanged. The native thumbnail command above now runs an
explicit synthetic 81/120 large-cache retention test before real secure/native
acceptance. Physical GNOME acceptance of this fix is pending; TH-06 stays TESTING.


TH-06 is now DONE by the user's Arch GNOME acceptance after0571986 (2026-10-25).
TH-07 adds native fullscreen to the current photo window: **F** or a double-click
on the main photo toggles fullscreen; **Esc** exits. Header controls hide in
fullscreen. Thumbnail/T/navigation and viewing preferences remain available;
fullscreen itself does not reload the main photo. TH-03 keeps its separate
acceptance status. TH-07 is now DONE by user-reported Arch GNOME native and
physical acceptance (2026-10-25); PIV-18 remains TESTING.

Arch GNOME Wayland acceptance (private TEST media/config/cache, secure decoder
gate, automatic checks, then two normal production windows for physical review):

```bash
bash tests/run_fullscreen_probe.sh
```

[Fullscreen implementation, tested evidence and remaining native checks](docs/TH07_FULLSCREEN_VALIDATION.md).


Phase 2 closure audit (2026-10-25): **PIV-18 TESTING; Phase 2 is not closed**.
The full-only path works, but real early preview is unimplemented and foreground
loading has no dedicated indicator. Deferring these requires an explicit scope
decision; rejected JPEG prototypes were not restored. TH-07 is DONE by user
native acceptance; TH-03 restart/persistence acceptance remains separate.

[Phase 2 and0.1.0 readiness report](docs/PHASE2_RELEASE_READINESS.md) separates
88 Rust/62 Python and controlled GTK/compositor PASS results from native user
reports and remaining gates. Clean native installation, real Nautilus acceptance
and release packaging remain open. The confirmed release plan is one AUR
package `pivra` from an approved main tag, with `yay -S pivra`/`yay -Syu` after
publication; no `pivra-git`, merge, tag, release or package was created here.
