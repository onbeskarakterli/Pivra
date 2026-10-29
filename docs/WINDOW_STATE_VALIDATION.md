# Window size and maximize persistence — v0.1.0

The production window installs `settings::window_state` before presentation.
`$XDG_CONFIG_HOME/pivra/window.conf` (normally `~/.config/pivra/window.conf`)
contains a versioned, bounded text record of normal width/height in logical
pixels and a separate maximize flag. It uses the same GIO PRIVATE atomic
replacement mechanism as the existing preferences; no dependency was added.

On close-request, `GtkWindow::default_size()` saves the normal size, not the
maximized/fullscreen allocation. GTK explicitly recommends this API for state
across restarts (installed Gtk-4.0.gir, Window.get_default_size documentation).
GTK's maximize property remains independent of fullscreen; both normal→full
and maximized→full close/restart paths are covered by the real WM probe.
Fullscreen, position and monitor identifiers are never persisted. Multiple
windows retain independent live state; the last normally closed window writes
the shared preference. Write failures are nonfatal and reported on stderr.

Records larger than 256 bytes, malformed/version-mismatched fields, invalid
booleans, extra fields, overflow, and dimensions outside 320–16384 × 240–16384
are rejected. Missing/invalid records use the existing 960×640 defaults.
If dimensions exceed the first available monitor's logical geometry minus a
96-pixel desktop margin, safe defaults are used and reduced if necessary; the
maximize flag is retained. With no monitor information, defaults are used.
Wayland does not expose a portable exact desktop workarea through GDK: the
compositor remains responsible for final workarea constraints and placement.
There are no window positioning requests or display-security changes.

## Results — Codex, 2026-10-28

- `cargo fmt --check`: PASS.
- `cargo check --locked`: PASS.
- `cargo test --locked`: PASS, 196 tests (including four new persistence tests).
- `cargo clippy --locked`: PASS, exit 0; existing unused `load_stats` and
  identical pin-icon branch warnings remain.
- `bash -n tests/run_window_state_probe.sh`: PASS.
- X11/Xvfb + Xfwm4, actual production no-file windows: PASS. Six **separate
  processes** verify resize, restart, maximize, restore, fullscreen roundtrip,
  close while fullscreen, maximize→fullscreen close, and restoration of normal
  dimensions and maximize state. An additional run with `--external-resize`
  uses xdotool to resize the actual X11 window instead of assigning its default
  size, and verifies that GTK updates the remembered normal dimensions: PASS.
- Headless Sway/Wayland, `--normal-only`: PASS for four separate processes,
  resize/restart/fullscreen and normal-size preservation. The initial full
  probe hit its configure deadline on maximize: this compositor did not honor
  the floating-window maximize request. Maximize/restore is therefore **not
  verified under Sway**; the normal-only mode explicitly reports NOT RUN for
  that part, rather than claiming a complete pass.
- Probe synchronization checks real allocated bounds. Initial test development
  exposed backend differences: raw Wayland surfaces include external shadows,
  while X11 content width excludes CSS borders. The probe uses GTK allocation
  for portable normal-size checks; it retains exact size assertions.
- Existing navigation probe: PASS on X11 and headless Sway in
  `--expect-image-error` scope (selection, arrows, boundaries, window isolation).
  Real secure image pixels remain BLOCKED by the Codex Bwrap environment.
- Existing `view_modes_probe --synthetic --zoom-pan`: PASS on X11; synthetic
  render/gesture coverage, not a secure-decoder or physical-input claim.
- Physical Arch/GNOME Wayland and real secure decode: NOT RUN here.

## Native GNOME command

After activating the existing development environment, run:

```bash
GDK_BACKEND=wayland bash tests/run_window_state_probe.sh
```

The command builds the probe, creates private XDG config/cache/data directories,
uses the actual production window builder, runs fresh processes, and removes
only its private temporary data. It does not modify normal user preferences or
photo files. Do not pass `--normal-only` for the full GNOME maximize acceptance.
For physical input acceptance, additionally resize a normal Pivra window,
close/reopen, maximize/close/reopen, then restore; repeat closing from fullscreen.
Confirm normal size and maximize restoration and that fullscreen itself does
not persist. Placement remains the compositor's choice.
