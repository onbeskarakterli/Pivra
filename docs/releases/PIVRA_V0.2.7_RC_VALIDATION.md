# Pivra v0.2.7 release candidate validation

**Status: IN PROGRESS — no publication approval.** Branch: `dev`. Target: `0.2.7`. Stable release remains v0.2.5.

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
- [ ] X11/XWayland GUI toolbar, slideshow, image/video navigation, pause/resume, EOS, open/reopen, notification timeout, folder monitor and preference persistence regression on final `dev`.
- [x] Native Arch DESTDIR staging and v0.2.7 candidate preflight: PASS (owner output, 23 Nov 2026). GPU ELF, dependency ownership/runtime, 184 crate source inventory, 383 application files, installed licenses and thumbnail helper validated. Stage: `/tmp/pivra-gpu-stage.anFsJLMo/usr`.
- [ ] Approved immutable source archive and SHA-256, real `makepkg` verification, package installation and version checks.
- [ ] Confirm source version, binary `--version`, `.SRCINFO`, package version and release notes agree at 0.2.7.
- [ ] Explicit owner approval for `dev` → `main`, version tag, GitHub Release and AUR update.

## Open risks

- **Stock native GTK Wayland** EGL/teardown regression remains open, with XWayland default workaround where available.
- Empty-startup-window Open currently has no attached viewer; opening in an already active viewer is implemented.
- `Preload exceeds cache entry budget` historical observation is not asserted fixed.
- Non-Arch, Intel/NVIDIA and end-to-end hardware decode acceptance are not established.

Historical v0.2.6 and v0.2.5 release documents retain their original facts and must not be re-labelled as v0.2.7 evidence.
