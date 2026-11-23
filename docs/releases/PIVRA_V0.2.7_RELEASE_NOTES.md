# Pivra v0.2.7 — release notes

**Status: Stable-release source prepared for approval.** Release publication requires a tag and GitHub/AUR handoff.

## Changes in v0.2.7

- Fullscreen slideshow: F5 start, Space pause/resume, Left/Right manual navigation and Escape exit; photo intervals and completion-based video advancement.
- User-configurable slideshow intervals (presets and manual 1–3600 seconds) and photo fade/no-transition selection; preferences are persisted.
- Revised header toolbar with grouped viewing, wheel-zoom and slideshow controls.
- Slideshow status notifications replace prior notifications and are explicitly dismissed after two seconds.
- Open button at the left of the title bar. When viewing media, Open changes the selection within the existing window, restarts folder monitoring and persists the last selected directory. Empty-window Open is not yet wired to the viewer and remains an open limitation.
- XWayland/X11 is selected by default when DISPLAY is available unless GDK_BACKEND is explicitly overridden, to avoid the known native GTK Wayland/EGL ownership issue.

## Validation and limitations

- Owner reported working slideshow playback on XWayland and working in-window Open.
- Earlier XWayland slideshow video-interval regression passed; native Wayland regression reached video EOS but GTK/EGL teardown failed (exit 134). This is not claimed fixed.
- Owner reported the first release preparation gate passed: 40 Rust tests, 83 Python tests (one skipped), formatting, check, strict Clippy and release build; this was before the v0.2.7 version/packaging metadata updates.
- Native Arch staging, package build, local installation and package integrity passed (1,331 files, zero altered); the owner approved final GNOME use. Final tagged source archive/checksum and AUR publication remain separate.
- Native Wayland video lifecycle remains an upstream GTK issue; Intel/NVIDIA and cross-distribution validation are outstanding.
- Final source tag, GitHub Release and AUR publication must reference the exact approved commit; this document alone is not proof they have been published.

See [v0.2.7 validation gates](PIVRA_V0.2.7_RC_VALIDATION.md).
