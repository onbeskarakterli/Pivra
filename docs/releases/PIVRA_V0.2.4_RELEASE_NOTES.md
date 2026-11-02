# Pivra v0.2.4 — Stable release notes

Published stable maintenance release for Linux/GNOME (2026-11-02).

## Changes

- Successive deletion and Undo-error notifications dismiss the previous toast and display the latest with HIGH priority, avoiding libadwaita's normal-priority notification queue.
- Per-file Trash/Undo handling remains intact.
- A targeted dead-code lint exception removes noise from standalone probe builds without changing production video functionality.
- Rustfmt source formatting cleanup.

## Validation

- Arch Linux native release build: PASS (user-reported).
- Rustfmt and strict all-target Clippy: PASS (user-reported).
- All-target Rust automated test suite: PASS, 0 failed (user-reported).
- Repeated deletion notification behavior in the actual GNOME UI: PASS after HIGH-priority fix (user-reported).
- Fixed-source archive SHA-256 verified for commit `2ce44567a81723a9cff3357f5130229574715c5e`: `7b125cad05328c1ff004febf6ab0c0e8cd48903ba104343d582a8d68cc334ef4` (user-reported).

## Distribution

- GitHub tag/release: `v0.2.4`.
- Arch AUR package: `pivra` `0.2.4-1` (user-reported website verification).
- This documentation revision on `dev` postdates the published tag; it does not amend the immutable v0.2.4 release source.
