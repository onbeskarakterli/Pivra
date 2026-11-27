# Pivra v0.2.7 — validation and publication record

**Current status: Published stable v0.2.7 on GitHub and AUR, confirmed by the project owner.** The publication report is distinct from Codex-controlled tests and the native acceptance reports below. No publication, artifact download, hash calculation or package test was performed in this documentation-only update.

## Publication and source scope

- Published release: [GitHub v0.2.7](https://github.com/onbeskarakterli/Pivra/releases/tag/v0.2.7); published package: [AUR pivra](https://aur.archlinux.org/packages/pivra).
- Slideshow is included in stable v0.2.7. Empty-startup-window Open was a limitation of that release; Open in an active viewer was implemented.
- Empty-window Open was fixed after publication on `dev`. The information panel, minimap, Ctrl+O, rename, filters and video inhibition are also unreleased; see [development notes](../development/NEXT_CYCLE_UI.md).
- The source version remains 0.2.7 on `dev`; that does not establish source equivalence with the published tag. This update does not substitute current dev for the approved release source or alter any source archive/hash.
- The historical checklist did not capture every final archive/`.SRCINFO` verification artifact. Owner-confirmed publication does not turn those unchecked items into independently verified PASS results.

## Historical candidate checklist — 23 Nov 2026

The following entries preserve the pre-publication evidence and outstanding checks as recorded at that time. References to pending publication or a candidate are historical, not the current release status. The owner approved publication on 23 Nov 2026; v0.2.5 was the preceding stable release, and v0.2.6 remained an unreleased candidate.

### Version / documentation consistency

- [x] `Cargo.toml` package version and `Cargo.lock` pivra entry: 0.2.7.
- [x] `packaging/arch/PKGBUILD` pkgver: 0.2.7; pkgrel=1 and !debug retained.
- [x] `README.md` and `packaging/linux/README.md` describe the v0.2.7 candidate and stable v0.2.5 separately.
- [x] Target-version candidate validation and release notes created; Arch package template installs them.
- [ ] `make release-preflight VERSION=0.2.7` candidate and final publication gates. Candidate should use `python3 scripts/release_preflight.py --version 0.2.7 --candidate`; final gate requires an explicitly approved stable handoff and evidence.

### Validation evidence

- [x] Owner-reported source (pre-metadata update): `cargo fmt --all -- --check`, `cargo check --locked --all-targets`, strict `cargo clippy`, `cargo test --locked --all-targets` (40 pass), Python unittest discovery (83, one skip), and release build passed.
- [x] Owner-reported in-window Open functional acceptance; last-selected folder behavior reported fixed.
- [x] Owner-reported XWayland slideshow media playback and slideshow video interval regression pass on prior slideshow-feature code.
- [x] Final v0.2.7 XWayland BT.709 slideshow video-interval regression: PASS (owner native Arch output, 23 Nov 2026; pause/seek/EOS/error/manual/close, no crash).
- [ ] Re-run full native checks on final 0.2.7 metadata/source.
- [x] Owner-reported final installed Pivra v0.2.7 GNOME acceptance: photo/video navigation, slideshow, notifications and Open are working without observed issues (23 Nov 2026). This is a manual user acceptance, not a newly run automated GUI suite.
- [x] Native Arch DESTDIR staging and v0.2.7 candidate preflight: PASS (owner output, 23 Nov 2026). GPU ELF, dependency ownership/runtime, 184 crate source inventory, 383 application files, installed licenses and thumbnail helper validated. Stage: `/tmp/pivra-gpu-stage.anFsJLMo/usr`.
- [ ] Approved immutable source archive and SHA-256, real `makepkg` verification, package installation and version checks.
- [x] Owner-reported Arch `makepkg` built package `pivra-0.2.7-1-x86_64.pkg.tar.zst` (23 Nov 2026); `pacman -Qip` confirms version 0.2.7-1, x86_64, GPL-3.0-only and expected dependency metadata. Compressed 21.62 MiB, installed 28.42 MiB. Package inventory and installed integrity checked separately; `.SRCINFO` remains pending.
- [x] Owner-reported `pacman -Qlp` package contents: `/usr/bin/pivra`, `/usr/bin/pivra-thumbnail-factory`, desktop entry, both icons and v0.2.7 release notes present (23 Nov 2026).
- [x] Owner native Arch install: `sudo pacman -U` upgrade PASS; `pivra --version` = `Pivra 0.2.7`, `pacman -Q pivra` = `pivra 0.2.7-1`, `pacman -Qkk pivra` = 1331 total files, 0 altered (23 Nov 2026).
- [ ] `.SRCINFO` and source-archive approval plus final release documentation handoff pending; published artifacts not updated.
- [x] Explicit owner approval (23 Nov 2026) for `dev` → `main`, version tag, GitHub Release and AUR update; execution and verification tracked separately.

## Release limitations and open observations

- **Stock native GTK Wayland** EGL/teardown regression remains open, with XWayland default workaround where available.
- In published v0.2.7, empty-startup-window Open has no attached viewer; opening in an already active viewer is implemented. The later dev-only fix does not close this historical release limitation.
- `Preload exceeds cache entry budget` historical observation is not asserted fixed.
- Non-Arch, Intel/NVIDIA and end-to-end hardware decode acceptance are not established.

Historical v0.2.6 and v0.2.5 release documents retain their original facts and must not be re-labelled as v0.2.7 evidence.
