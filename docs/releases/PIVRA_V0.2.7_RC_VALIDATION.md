# Pivra v0.2.7 release candidate validation

**Status: Owner approved v0.2.7 release on 23 Nov 2026; source/tag/GitHub/AUR publication verification pending.** Target: `0.2.7`. Historical stable before this handoff: v0.2.5.

## Version / documentation consistency

- [x] `Cargo.toml` package version and `Cargo.lock` pivra entry: 0.2.7.
- [x] `packaging/arch/PKGBUILD` pkgver: 0.2.7; pkgrel=1 and !debug retained.
- [x] `README.md` and `packaging/linux/README.md` describe the v0.2.7 candidate and stable v0.2.5 separately.
- [x] Target-version candidate validation and release notes created; Arch package template installs them.
- [ ] `make release-preflight VERSION=0.2.7` candidate and final publication gates. Candidate should use `python3 scripts/release_preflight.py --version 0.2.7 --candidate`; final gate requires an explicitly approved stable handoff and evidence.

## Validation evidence

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

## Open risks

- **Stock native GTK Wayland** EGL/teardown regression remains open, with XWayland default workaround where available.
- Empty-startup-window Open currently has no attached viewer; opening in an already active viewer is implemented.
- `Preload exceeds cache entry budget` historical observation is not asserted fixed.
- Non-Arch, Intel/NVIDIA and end-to-end hardware decode acceptance are not established.

Historical v0.2.6 and v0.2.5 release documents retain their original facts and must not be re-labelled as v0.2.7 evidence.
