# Pivra 0.2.5 — immutable source review and native package validation

Status at preparation: **OWNER SOURCE APPROVAL PENDING**. This is not a release,
tag, main promotion, package installation or AUR publication authorization.

The owner reports native Arch staging PASS for a8ed590185478b466b62f3d0ff0153937e43f299,
including GPU-required compilation, ownership/plugins, ELF, source/notices and
preflight. Earlier physical AMD acceptance and the 210 Rust / 167 Python checks
are retained as evidence, not independently repeated physical tests in this task.

## Stable README handoff

README now carries the prepared current-stable v0.2.5 wording; release notes say
stable-release **source**, rather than falsely claiming publication. The README
explicitly separates the prepared handoff from actual GitHub/AUR availability.
Final preflight checks stable-source wording and the actual 0.2.5 binary. A PASS
is source/package consistency, never publication or owner approval. No product,
dependency, GPU implementation or version changes accompany this handoff.

## Exact source and real archive evidence

After reviewing and committing these preparation changes on dev:

1. Record the full immutable candidate SHA; do not substitute moving dev/main
   branches or a guessed tag. Do not approve the new commit on the owner's behalf.
2. Download `https://github.com/onbeskarakterli/Pivra/archive/<SHA>.tar.gz` with
   normal TLS verification; retain redirect URL, byte count and actual SHA-256.
3. Compare every archive file/blob and executable mode with `git ls-tree` for that
   commit. Check Cargo/lock/recipe version 0.2.5, license, !debug, source safeguards
   and the complete modified LGPL adapter and source-collection files.
4. Keep the evidence and proposed release-source.conf in a **separate private
   review workspace**, not the application checkout. The proposed values are for
   review; do not activate them in the application recipe before owner approval.
   A new source commit requires a new archive/digest and approval.
5. The final response supplies the actual SHA/digest/evidence path. They cannot be
   predicted or embedded in the same commit whose hash they describe.

No makepkg/pacman/bsdtar is available in the current Codex environment. Packages
are not installed automatically. Therefore actual .SRCINFO generation and native
makepkg verification/build are **NOT RUN here**, not fabricated. The complete
native procedure below generates .SRCINFO from the actual approved recipe;
there is no handwritten .SRCINFO pretending to be makepkg output.

## One native command, only after approving the exact SHA/digest

```sh
python3 packaging/arch/validate-release.py --approved-source FULL_SHA --sha256 REAL_SHA256
```

Passing --approved-source explicitly attests the owner's approval; do not run it
before that decision. From an ordinary user's updated dev checkout on Arch, this:

- Requires existing native tools; never installs dependencies or invokes sudo.
- Downloads only that immutable GitHub archive and rejects a digest mismatch.
- Extracts bounded regular source files safely into a new private workspace.
- Copies PKGBUILD and collector **from the pinned archive**, not moving dev.
- Writes an active release-source.conf only in that approval-scoped workspace.
- Runs actual `makepkg --verifysource --force` and `makepkg --printsrcinfo`.
- Checks .SRCINFO source/collector checksums, version/license, required dependencies,
  one pkgname=pivra and !debug. No separate pivra-debug package is accepted.
- Keeps build/output/config/cache in private locations and runs
  `makepkg --cleanbuild --force --check`; no --syncdeps or --install.
- Verifies the actual package list, .PKGINFO, ELF/CLI version, executable GNOME
  helper, desktop entry, both LGPL texts, modified adapter files, the exact
  approved application archive and all locked crate-source hashes/notices.
- Retains logs, .SRCINFO, package contents, actual package SHA-256 and outputs.

Review the resulting logs and package; package install/remove and published
GitHub/AUR availability remain separate decisions. No physical color/HDR claim
is inferred from package extraction or software tests. Intel/NVIDIA remain
untested, Vulkan adapter absent, HDR visual preservation/zero-copy unproven.

Existing recipe source/checksum guards, options=('!debug'), corresponding-source
collection and LGPL complete-source rebuilding/relinking instructions remain.
The recipe/example now permit an explicitly approved immutable candidate for
native validation; they do not require or authorize a main merge to perform it.

## Preparation checks executed in Codex

- 210 Rust tests PASS; 173 Python tests PASS (root 60, thumbnail 59, FD05A 9,
  GPU 32, color 13), including five new source-workflow policy tests.
- cargo fmt --check, cargo check --locked and strict all-target Clippy PASS,
  zero warnings; Bash recipe/example syntax PASS.
- No diff versus a8ed590 in src/, native/, Cargo.toml/lock, build.rs, data/,
  PKGBUILD or the license collector: the accepted product/packaging payload is
  preserved. New code is limited to validation tooling and its focused tests.
- This record does not claim native makepkg/.SRCINFO generation or installation.
  The downloaded source archive, real digest, Git tree comparison and proposed
  configuration are retained separately after the candidate commit is known.
