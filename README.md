# Pivra
A fast, lightweight GNOME image viewer with seamless photo and video navigation, smart zoom, and basic editing.

## Development status

**PIV-01: DONE. PIV-02: GTK4/libadwaita shell implemented and validated on 2026-10-21, on `dev`.** No image/video functionality, media engine or CI is included. PIV-03 has not started.

Launching without arguments opens a blank `Pivra` window with a native libadwaita header and system theme. Each process owns an independent application/window (`io.github.onbeskarakterli.Pivra`, GIO NON_UNIQUE); file associations and Nautilus file-opening integration are future work. Closing the last window ends that process normally. If GTK cannot initialize a graphical session, the program prints a controlled error to stderr and exits 1. Help/version flags and invalid-argument exit 2 work without initializing GTK.

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
