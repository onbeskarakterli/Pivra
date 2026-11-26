# Pivra v0.2.7 — release notes

**Status: Published stable release.** v0.2.7 is available on GitHub and AUR. Publication is confirmed by the project owner; this documentation update does not perform or independently repeat publication checks.

[GitHub v0.2.7](https://github.com/onbeskarakterli/Pivra/releases/tag/v0.2.7) · [AUR pivra](https://aur.archlinux.org/packages/pivra)

These notes describe the published `v0.2.7` source, not the newer `dev` branch. The package/source version on `dev` still reads 0.2.7, but subsequent features are unreleased.

## Changes in v0.2.7

- Fullscreen slideshow: F5 start, Space pause/resume, Left/Right manual navigation and Escape exit; photo intervals and completion-based video advancement.
- User-configurable slideshow intervals (presets and manual 1–3600 seconds) and photo fade/no-transition selection; preferences are persisted.
- Revised header toolbar with grouped viewing, wheel-zoom and slideshow controls.
- Slideshow status notifications replace prior notifications and are explicitly dismissed after two seconds.
- Open button at the left of the title bar. When viewing media, Open changes the selection within the existing window, restarts folder monitoring and persists the last selected directory. In the published v0.2.7 build, empty-window Open is not wired to the viewer and remains a release limitation. It was fixed after publication on `dev`; that fix is not retroactively part of v0.2.7.
- XWayland/X11 is selected by default when DISPLAY is available unless GDK_BACKEND is explicitly overridden, to avoid the known native GTK Wayland/EGL ownership issue.

## Validation and limitations

- Owner reported working slideshow playback on XWayland and working in-window Open.
- Earlier XWayland slideshow video-interval regression passed; native Wayland regression reached video EOS but GTK/EGL teardown failed (exit 134). This is not claimed fixed.
- Owner reported the first release preparation gate passed: 40 Rust tests, 83 Python tests (one skipped), formatting, check, strict Clippy and release build; this was before the v0.2.7 version/packaging metadata updates.
- Native Arch staging, package build, local installation and package integrity passed (1,331 files, zero altered); the owner approved final GNOME use. This is historical native acceptance evidence; subsequent GitHub/AUR publication is confirmed by the owner. No archive hash is added or changed by this update.
- Native Wayland video lifecycle remains an upstream GTK issue; Intel/NVIDIA and cross-distribution validation are outstanding.
- GitHub and AUR publication are owner-confirmed. Historical source-approval, checksum and package-validation records in the linked validation document remain distinct from that report; no new native or artifact-verification run is claimed.

## After v0.2.7 publication — dev only

The empty-window Open fix (`63c9a15`), Ctrl+O, media information panel, photo minimap, F2 rename, thumbnail type filters, video display-sleep inhibition and subsequent toolbar refinements belong to the next development cycle. See [post-v0.2.7 development notes](../development/NEXT_CYCLE_UI.md) for implementation and validation limits. They do not change this release's historical scope or empty-window Open limitation.

See the [v0.2.7 validation and publication record](PIVRA_V0.2.7_RC_VALIDATION.md).
