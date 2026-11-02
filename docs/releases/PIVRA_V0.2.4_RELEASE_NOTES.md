# Pivra v0.2.4 — Development release notes (unreleased)

This is a development candidate on `dev`, not a published stable release.

## Changes

- On repeated deletion or Undo-error notifications, dismiss the previous toast and show the new one with HIGH priority to avoid libadwaita's normal-priority toast queue.
- Preserve the per-item Trash/Undo handling.
- Remove standalone example-target `dead_code` warning noise for `ImageView::video_widget()` with a narrowly scoped, documented exception; production video behavior is unchanged.

## Validation

- User reported `cargo test --locked --all-targets`: 32 passed, 0 failed before the v0.2.4 version bump (2026-11-02).
- After the version bump, native build and tests require revalidation.
- Manual GNOME repeated-Delete test failed on the initial 0.2.4 candidate. After applying the HIGH-priority toast fix, the user confirmed that repeated deletion notifications behave correctly on native GNOME (2026-11-02): **PASS**.
- Release packaging/source archive checks and publication authorization are separate.

## Release boundaries

No automatic change to `main`, GitHub release/tag, or AUR. Publish only after final source and native acceptance.
