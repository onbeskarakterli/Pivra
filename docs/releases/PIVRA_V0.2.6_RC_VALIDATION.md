# Pivra v0.2.6 release checklist and validation

Updated 2026-11-23. Reviewed dev source:
`c4ca4d85a05fa81efeddc6f5229d28040c56d787`.
**Preparation PARTIAL / unrestricted publication NO-GO.** The isolated native
ownership-fix gate has passed; stock GTK compatibility, independent risk decisions
and immutable-source/package approval are still open. This is not publication
authorization. Stable README information remains v0.2.5 until an approved handoff.

## Current acceptance, with evidence provenance

| Gate | Result | Provenance / limit |
| --- | --- | --- |
| RX 7900 XT, stock-ownership GTK Wayland lifecycle | 8/9; mixed fails | Latest owner-reported physical result |
| RX 7900 XT, isolated ownership-patched GTK Wayland | 9/9 PASS | Latest owner report, not Codex execution |
| RX 7900 XT, X11 lifecycle | 9/9 PASS | Latest owner report |
| Normal production Wayland GPU, video/audio/color/seek | PASS | Latest owner report; not every mixed transition |
| Seven nonvideo native gates | PASS | Earlier owner report; not rerun by this documentation task |
| Codex strict Clippy/fmt/check/release / Rust / Python | PASS; 217 Rust, 242 Python | Executed at diagnostic commit c4ca4d8; historical controlled evidence |
| Codex real GTK 4.24 / Gst 1.28.9 X11 control/candidate | 9/9 each | llvmpipe, not hardware acceptance |
| Codex real GTK 4.24 / Gst 1.28.9 Sway control/candidate | 8/9 each | Control EGL failure; candidate SIGSEGV remains OPEN |
| New v0.2.6 source hash / makepkg / package installation | NOT APPROVED / NOT RUN here | Previous versions' acceptance is not inherited |

Native environment supplied previously: GTK 4.24.1, Mesa 26.3.1-arch1.2,
GStreamer 1.28.9, Arch/GNOME Wayland, RX 7900 XT. New native raw logs, exact
checkout and loaded-library hashes were not supplied. Record these before
upstream submission/package source approval; do not invent them. The existing
A/B runner compares common EGL-config-fixed trees with original versus fixed
stock ownership; it does not prove a pristine GTK baseline.

## Release decision and compatibility policy

The stock GTK private sink can terminate an EGLDisplay still used by other
windows when switching GPU → stock → GPU video. Normal playback PASS and stock
fallback do not establish that every invalid-context path is harmless. The
ownership correction is already merged in inspected upstream main/gtk-4-24
refs at `95b7ec19b01c460c46d58ec52cd0c4eb2d730395`. The source commit labelled
4.24.2 contains it, but a published tag/fixed Arch package and native acceptance
of that exact official source are not verified. The separate EGL selection fix
is absent from inspected gtk-4-24 gdkdisplay.c. Distribution delivery/context
validation remains necessary; Pivra cannot reach the
private stock sink through public GtkMediaFile API. Neither GTK patch is bundled
or installed by the package. See the [upstream report](../upstream/GTK_4.24.1_WAYLAND_EGL_OWNERSHIP.md).

A conditional compatibility-limited release is technically defensible **only if**
the owner explicitly accepts the affected stack/transition, discloses the
limitation before installation, and supports the tested per-process workaround:

```sh
GDK_BACKEND=x11 GSK_RENDERER=gl pivra /path/to/video.mp4
```

This requires XWayland/X11 and is accepted on the tested AMD system; it is not a
universal or global backend change. Mixed metadata/folders and independent
windows are ordinary use cases, so **default recommendation is hold an
unrestricted Wayland release** until a fixed distribution stack is natively validated or risks explicitly accepted.
A patched private build passing 9/9 alone does not approve the stock package.

| Risk / decision | Current status | Required decision/evidence |
| --- | --- | --- |
| Stock GTK mixed-lifecycle shared EGL termination | OPEN, confirmed | Native-validated fixed GTK package/backport, or explicit v0.2.6 owner acceptance with compatibility disclosure/X11 workaround |
| Earlier private software candidate SIGSEGV | OPEN, UNCLASSIFIED | Owner review; later native/debugger PASS is not a fix. [Investigation](../performance/V026_CANDIDATE_SIGSEGV.md) |
| `Preload exceeds cache entry budget` | OPEN, unreproduced | Prior v0.1 acceptance is not automatically v0.2.6 acceptance |
| Intel/NVIDIA, Vulkan adapter, native HDR, hardware decode / zero-copy | NOT VALIDATED / Vulkan adapter not implemented | Keep limitations; no cross-GPU/HDR/VRAM claims |

No risk acceptance is inferred from this task or from native test success.

## Checklist before source approval and publication

- [x] Version-bearing files reviewed: Cargo.toml, Pivra Cargo.lock entry,
  PKGBUILD pkgver, validate-release.py VERSION, staging/preflight references
  all target 0.2.6; pkgrel=1; GPL-3.0-only.
- [x] One pkgname=pivra; `options=('!debug')` unchanged; no debug subpackage.
- [x] Both GTK patches and their diagnostic/validation infrastructure preserved.
- [x] Existing upstream ownership fix, issue #8456 / MR !10582, and current
  source refs verified; no duplicate submission or published-version claim.
- [x] User-reported native 9/9 isolated Wayland / 9/9 X11 recorded separately
  from Codex/software controls and original stock-ownership 8/9.
- [x] README/notes/Arch/Linux documentation disclose compatibility limits and
  preserve stable v0.2.5 and historical test records.
- [x] Existing source URL/SHA-256 approval, .SRCINFO validation, mandatory GPU
  build/ELF, source collection and LGPL rebuild/relink requirements reviewed.
- [ ] Owner decides stock GTK mixed-lifecycle compatibility policy; prefer native
  validation of a distributor's official ownership fix, also checking the separate
  EGL selection issue, over accepting known broken 4.24.1 transitions.
- [ ] Owner separately reviews SIGSEGV and preload observations; no silent closure.
- [ ] Attach native A/B raw logs, actual source/GTK/library identities and versions.
- [ ] Rerun applicable candidate tests/staging on the exact final reviewed source;
  changed documentation alone does not create fresh package validation evidence.
- [ ] Approve the exact immutable source commit and downloaded archive SHA-256.
- [ ] Actual native makepkg --verifysource, --printsrcinfo, clean build, package
  contents/ELF/LGPL corresponding-source/notices/relink verification.
- [ ] Explicitly authorized native package install/remove/reinstall and user-data
  preservation checks for this source; no installation is performed by this task.
- [ ] On authorized stable handoff, change README to current stable v0.2.6 and
  release notes to stable-source wording, then re-review the final source/hash.
- [ ] Separate authorization for main promotion, tag, GitHub Release and AUR.

## Reproduction and regression commands

Focused upstream ownership A/B, private GTK builds only:

```sh
GDK_BACKEND=wayland bash tests/run_gtk_egl_isolated.sh --stock-lifecycle --workspace "$HOME/.cache/pivra-gtk-stock-ownership"
```

The full existing native acceptance entry remains:

```sh
bash tests/run_v026_acceptance.sh
```

Run on the intended desktop/GPU with secure loaders. Its **stock GTK** video
lifecycle failure remains a real FAIL; no waiver option or forced X11 is added.
Private patched GTK success is separate evidence. It uses private fixtures/XDG
directories and does not install packages. Preserve logs/resource series.

`python3 scripts/release_preflight.py --version 0.2.6 --candidate` checks
source/document/package consistency of a clean dev tree. It does not interpret
native risk decisions or authorize publication. The noncandidate preflight
must continue to reject an upcoming-only README or missing real distribution ELF.
`--clean-old-packages` stays opt-in and narrowly scoped; this task does not run it.

## Historical implementation/acceptance records (unchanged results)

The following entries retain their original dates/test scope. The current table
above supersedes their old native-pending wording, without turning prior runs
into new PASS results.

Reviewed implementation: `25d4b9eddd169b2d25385dccbcdde136b465ae5f`.
Codex: fmt/check/strict Clippy PASS, 217 Rust / 210 Python tests PASS, controlled
X11 and Sway GUI gates PASS (video 9/9 each, llvmpipe). Mandatory GPU release
build, ELF contract, private staging, corresponding source/notices, desktop entry
and clean candidate preflight PASS. These are source/package-consistency controls,
not Arch makepkg, physical AMD, or publication approval. Final evidence updates
are documentation-only and do not make the implementation's native gate PASS.


### Native acceptance follow-up — 2026-11-20

The user reports **6 PASS / 2 FAIL**, catalog expected one failed cell but got
zero, and GPU `no shared GL context` / `No EGL configuration available`.
This is not a successful native acceptance. At
`ff9d6fc74a34b65d43c4c06e1c62235ac55cc49a`, the driver now enables the real
`--native-thumbnails` catalog mode and isolates controlled generation failures.
Native repair assertions require publication/current identity and another secure
cache decode, not just failed=0. The stock GTK EGLDisplay defect remains upstream
and **is not fixed by this harness change**.

Latest Codex follow-up: 217 Rust / 213 Python PASS; fmt/check/strict Clippy/release
build PASS; targeted catalog and 9/9 video controls PASS on X11 and Sway/llvmpipe.
The real secure catalog prerequisite returned NOT RUN/77 on its single Codex
attempt. The six other native gate definitions are preserved, without claiming
new hardware PASS. [Detailed follow-up](../performance/V026_NATIVE_ACCEPTANCE_FIXES.md).

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


## Source and package approval workflow (only after blockers close)

1. Review and push the exact clean dev commit to be packaged. Do not use dev HEAD
   as a moving URL. Obtain owner approval for the exact 40-hex commit/digest.
2. Follow `packaging/arch/validate-release.py --help` to fetch/check the immutable
   archive and locally prepare the source configuration. A computed digest is
   evidence to review, not authorization. Never reuse an earlier version's hash.
   After explicit approval, the existing automated entry is:
   `python3 packaging/arch/validate-release.py --approved-source APPROVED_40_HEX_COMMIT --sha256 APPROVED_64_HEX_DIGEST`.
   Those are placeholders, not actual approved values. Passing --approved-source
   attests owner approval; do not use it to obtain approval implicitly.
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

## Documentation-preparation verification — 2026-11-23

Newly executed for this documentation-only follow-up: 69 root + 92 GPU-tool
Python tests PASS (161, zero skips); cargo fmt --check PASS; version/license/
single-package/!debug/script-reference and changed Markdown link audits PASS;
Bash syntax and both original pinned GTK patch apply-checks PASS. Missing
release-source.conf still fails closed with the expected exit 1.

No production source, tests, release scripts, PKGBUILD, Cargo manifests/lockfile
or GTK patches changed. Full Rust/Clippy/release, GPU hardware probes and makepkg
were not rerun for these prose edits; their earlier evidence remains labelled by
source/version above. Candidate preflight on the final clean commit is a
consistency check only, never an acceptance or publication waiver.
