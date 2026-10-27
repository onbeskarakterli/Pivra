# v0.1 photo-only directional navigation

Left/Right and the existing on-image buttons use the same window actions.
Those actions now search the existing ordered catalog for the next JPEG/PNG
image in the requested direction. Video and unsupported image entries remain
in the catalog, but are skipped by directional navigation. Broken JPEG/PNG
candidates still reach the existing controlled error path; extensions select
candidates, not proof of successful content decoding.

The finite scan does not wrap. No wrap preference exists in current Settings.
At an edge, or with no eligible neighbor, selection and current load remain
unchanged. Action sensitivity uses the same target lookup. Thumbnail filtering,
natural sorting, secure decoder, cancellation and provider policy are unchanged.

The existing CLI rejects video arguments; that boundary is preserved. The
window layer's existing video unsupported state is also preserved. The GUI
probe explicitly constructs a video OpenedFile to test that window state and
asserts that the ordinary argument parser still rejects the video path.

## Verification (2026-10-28, Codex)

- `cargo fmt --check`: PASS.
- `cargo check --locked`: PASS.
- `cargo test --locked`: PASS, 192 tests.
- `cargo clippy --locked`: PASS (exit 0); existing unused `load_stats` and
  identical pin-icon branch warnings remain.
- `navigation_probe --expect-image-error`: PASS under X11/Xvfb and headless
  Sway/Wayland. Covers keyboard skip, actual production next-button clicked
  signal, bounds, rapid navigation, corrupt-file recovery, independent windows,
  direct window video state, and unchanged fixture bytes.
- New model tests: consecutive videos, unsupported image candidates,
  case-insensitive JPEG, unchanged catalog, video-only/empty/single catalogs,
  preserved selection and finite repeated boundary navigation.
- Real successful secure decode/pixel presentation and physical Arch/GNOME
  input: BLOCKED / NOT RUN here; no native success is claimed.

Native Arch command after activating the existing development environment:

```bash
bash tests/run_navigation_probe.sh
```

This native gate requires real secure decoded pixels. Also manually check
Left/Right and on-image arrows in a mixed photo/video folder.
