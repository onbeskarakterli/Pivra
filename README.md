# Pivra
A fast, lightweight GNOME image viewer with seamless photo and video navigation, smart zoom, and basic editing.

## Development status

**PIV-09: BLOCKED (2026-10-21).** Glycin 3.0.8 integration compiles, but compatible
2+ loaders are absent and this Codex session rejects bubblewrap UID-map setup.
Real JPEG/PNG pixels, EXIF orientation and application Fit have **not** passed.
PIV-01–08 remain DONE; PIV-10 has not started. Details below supersede the older
placeholder-only delivery descriptions, which record their respective milestones.

**PIV-01: DONE. PIV-02: GTK4/libadwaita shell implemented and validated on 2026-10-21, on `dev`.** No image/video functionality, media engine or CI is included. PIV-03 separates application, window, CLI and core data responsibilities without changing that behavior. PIV-04 adds deterministic fixtures and isolated Rust/X11/headless Wayland test infrastructure. PIV-05 records versioned GTK/Glycin/GStreamer capability probes and explicit runtime gaps; production dependencies are unchanged. PIV-06 adds local photo path/URI input and a decoder-free file placeholder. PIV-07 discovers unsorted neighboring image/video candidates on a worker and preserves the opened selection. PIV-08 naturally orders the unified catalog while preserving the opened selection. PIV-09 has not started.

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

See the [development roadmap](docs/PIVRA_DEVELOPMENT_ROADMAP.md), [feature specification](docs/PIVRA_FEATURE_SPEC_TR.md), and [commit tracker](docs/PIVRA_COMMIT_TRACKER.md). License selection remains open; no license is assumed. The PIV-01 results above describe the earlier CLI-only bootstrap; PIV-02 intentionally replaces its no-argument placeholder with the GTK window.


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
