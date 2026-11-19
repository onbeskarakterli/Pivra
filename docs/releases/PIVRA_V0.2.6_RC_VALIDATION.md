# Pivra v0.2.6 candidate validation

Status: **PARTIAL / NO-GO for publication**. Version preparation is not native acceptance.

- Cargo.toml, only Pivra's Cargo.lock entry and Arch pkgver: 0.2.6.
- pkgrel=1 and `options=('!debug')`: preserved; no separate pivra-debug package.
- Stable README information remains v0.2.5; v0.2.6 is upcoming and blocked.
- Immutable source commit/checksum: NOT APPROVED; release-source.conf/.SRCINFO
  are machine-local and not committed. Old v0.2.5 source approval is not reusable.
- Native AMD: previous user-provided mixed lifecycle FAIL/exit 101; new revision
  acceptance pending. llvmpipe PASS cannot close it.
- Native Arch makepkg, package installation/removal and physical visual tests of
  this version: NOT RUN in Codex. Prior v0.2.5 results remain historical.
- Exact controlled results and before/after limitations:
  [V026_FINAL_SPRINT.md](../performance/V026_FINAL_SPRINT.md).

## Native acceptance

```sh
bash tests/run_v026_acceptance.sh
```

Run from native Arch/GNOME on the intended GPU with normal stock GTK and secure
loaders. This command uses private XDG/fixture directories, performs no install,
and prints report paths. It refuses native mode with software-test permission.
It checks image/cache controls, the catalog and existing GUI races, the nine
video lifecycle gates, then the existing native performance/64 MP collection
only if video lifecycle passed. FAIL/NOT RUN remains visible; no CPU color or
sandbox fallback is enabled. Preserve logs and resource-series files.

## Source and package approval workflow (only after blockers close)

1. Review and push the exact clean dev commit to be packaged. Do not use dev HEAD
   as a moving URL. Obtain owner approval for the exact 40-hex commit/digest.
2. Follow `packaging/arch/validate-release.py --help` to fetch/check the immutable
   archive and locally prepare the source configuration. A computed digest is
   evidence to review, not authorization. Never reuse an earlier version's hash.
3. In a separate private recipe directory, copy PKGBUILD, collect-licenses.py and
   the approved local release-source.conf. Run `makepkg --verifysource --force`,
   `makepkg --printsrcinfo > .SRCINFO`, then `makepkg --cleanbuild --force`.
   No install flag is used. Verify only pkgname=pivra, pkgver=0.2.6, pkgrel=1,
   options=!debug and the approved URL/checksums in actual generated metadata.
4. Inspect package paths, GPU-enabled ELF dependencies, sibling GNOME helper,
   LGPL notices, adapter/application/locked-crate corresponding source and rebuild
   instructions. Run candidate preflight against the staged/package-extracted tree.
5. Record real Arch results separately. Installation, main promotion, tag, GitHub
   release and AUR publication each remain outside this task's authorization.

Existing `--clean-old-packages` removes only explicitly requested, untracked old
archives under packaging/arch; it must never touch user media/config/cache or
approved source evidence. Candidate staging does not automatically delete them.
