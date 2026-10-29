# B6-F1/F2 — Pivra 0.1.0 immutable release candidate and acceptance

2026-10-29 (Europe/Istanbul). **F2 technical GO for the accepted RC;
publication is not authorized and has not occurred.** F1's pending native
metadata/package checks below are retained as dated history, now superseded
by explicit user-reported F2 PASS evidence.

## B6-F2 native RC acceptance

The user confirms the exact source/archive below on actual Arch/GNOME:

- `makepkg --verifysource --force`: PASS, both files verified.
- `makepkg --printsrcinfo`: PASS / exit 0; `.SRCINFO` matches 0.1.0-1,
  immutable full-SHA URL and both expected hashes (archive below; collector
  `e0bad72c99f4629b0668f59450e53a75022ea746efe57191ab8da34fb5cf1201`).
- `makepkg --cleanbuild --force`: PASS / exit 0; produced
  `pivra-0.1.0-1-x86_64.pkg.tar.zst`, approximately 22 MB.
- pacman install PASS; `pivra --version` prints `Pivra 0.1.0`.
- `pacman -Qkk pivra`: 1309 total files, 0 altered files.
- Problematic JPEG opens; thumbnail loading has no missing entries reported;
  previous/next navigation and size/maximize persistence PASS.
- Prior native Nautilus JPEG double-click/PNG Open With PASS retained; JPEG
  default Pivra, PNG default Loupe. Prior B5 removal/reinstall PASS remains
  historical evidence, not a newly inferred RC uninstall test.

All native results are **user-reported**, not run by Codex. No native package,
.SRCINFO or complete transcript was attached for independent inspection.
Current application/recipe equals the accepted RC and prior F1 automation
remains applicable (strict zero-warning Clippy, 197 Rust, Python 53+33+9,
all four controlled X11/Sway gates PASS). F2 rechecked identity/checksum and
consistency, not the full automated/native suites again.

**GO recommendation is technical readiness for the fixed RC only.**
`Preload exceeds cache entry budget` remains OPEN/unfixed and explicitly
accepted by the owner as a v0.1.0 risk. Final owner permission to merge, tag or
publish is absent; no such action is initiated. A different source commit
requires its own archive/metadata review, not reuse of this checksum.
See the [F2 assessment](PIVRA_V0.1_B6_READINESS_AUDIT.md#b6-f2--current-technical-decision-go-publication-not-authorized).

## Exact reviewed reference and actual archive

- Starting application/build/test source:
  `eabc0e51eb9434c72a0b08ab1c7fbfbba5eaf2a7`.
- **Final reviewed dev RC snapshot:**
  `7e26e780acffdb1f2db0f37e8cf6485eba0baa63`.
- Git tree: `189125429dfc78760e8e5d661249012a5efd95a1`.
- Immutable full-commit source URL:
  <https://github.com/onbeskarakterli/Pivra/archive/7e26e780acffdb1f2db0f37e8cf6485eba0baa63.tar.gz>.
- Actual downloaded archive: **627,913 bytes**.
- **SHA-256:**

```text
828a97503173f4fef568345e2d8b2334fcf35b6b587928692ac2261ef3b68277
```

The full-SHA archive was actually downloaded from that HTTPS URL. Its digest
was computed, written to SHA256SUMS and verified with `sha256sum --check`.
All **211** regular file contents and executable bits match Git blobs/modes at
that exact commit. Archive root is
`Pivra-7e26e780acffdb1f2db0f37e8cf6485eba0baa63`, as the PKGBUILD expects;
no extra files, symlinks, absolute entries or path traversal were accepted.
The archived manifest says version 0.1.0 and GPL-3.0-only.
This validates the bytes actually obtained, not a guarantee that GitHub can
never change archive encoding. Any later checksum mismatch must fail closed.

The preliminary acceptance commit `f91c298ac38feef8d2c99482055b5eb79c316420`
was archived before the owner's risk answer arrived. It is **not the final RC**.
The final snapshot includes the subsequent explicit risk decision and fresh
test record. No history was rewritten. All application/build/test/assets/recipe
blobs are unchanged between `eabc0e5` and final RC `7e26e78`; F1 commits are
only documentation. The later delivery/receipt commit is also docs-only and
is not silently substituted for this archive or its checksum. If another
source commit is eventually selected for main/release, generate/verify its
own archive digest and regenerate metadata. No tag or final publication approval
is implied by an RC full-SHA reference.

## User-reported native acceptance

Recorded in README, release notes and the B6 audit as user results, not Codex:

- JPEG registered; Pivra selected as default.
- PNG registered; **Loupe remains default**.
- Nautilus JPEG double-click: PASS.
- Nautilus PNG Open With → Pivra: PASS.
- Actual JPEG and PNG rendering: PASS.
- Existing B5 package install/remove: PASS; earlier exact package source remains
  `e8fc9ec011ad6759bc6eef018b9a6f71465f80f1`.
- W1 strict all-target Clippy: user-reported PASS, zero warnings; also independently
  executed by Codex at W1 and again for F1.

The new physical-test message did not separately pin its binary SHA; no such
SHA is invented. Codex did not change desktop MIME defaults or repeat physical
Nautilus/image rendering. Independent native rebuilding/installation of the
new final RC is not established by the older package report.

## Owner risk decision — open, explicitly accepted for v0.1.0

The owner answered on 2026-10-29: **“Accept as a documented v0.1.0 release risk”**
for `Preload exceeds cache entry budget`. Accordingly it is not a v0.1.0
release blocker by owner decision. The single main-image observation remains
**open/unfixed**, without a reproducible cause or fix claim. It is retained in
release notes. This acceptance does not cover later releases, make the
observation disappear, or authorize main/tag/GitHub/AUR publication.

The W1 Sway synthetic pixel mismatch remains a separate historical,
unclassified single-run test observation; its isolated W1 rerun and all F1
first-run gates passed. No geometry/synchronization/assertion fix is claimed.

## Separate RC workspace and package consistency

Actual controlled workspace, outside the application repository:

```text
/workspace/pivra-release-candidates/pivra-0.1.0-rc-7e26e780acffdb1f2db0f37e8cf6485eba0baa63/
  REVIEWED_COMMIT
  pivra-0.1.0.tar.gz
  SHA256SUMS
  VERIFICATION.json
  arch/
    PKGBUILD
    collect-licenses.py
    pivra-0.1.0.tar.gz
    release-source.conf
```

The recipe and collector are exact files from RC `7e26e78`. The private source
config explicitly says **RC-ONLY / NOT approved final main source or publication**.
It uses the exact full SHA, canonical commit URL and actual archive checksum
above, with no moving dev reference, SKIP or invented digest.

Loading the actual unmodified PKGBUILD with this RC config succeeded. Its real
Bash declarations were checked: `pivra`, pkgver=0.1.0, pkgrel=1, x86_64,
GPL-3.0-only, exact source URL/alias and archive digest; collector digest remains
`e0bad72c99f4629b0668f59450e53a75022ea746efe57191ab8da34fb5cf1201`.
This is **Bash/source-configuration validation**, not makepkg or SRCINFO proof.

**F1 Codex `.SRCINFO` generation/consistency: NOT RUN.** `makepkg` and `pacman`
are absent in Codex. **F2 native generation/consistency: PASS, user-reported**
as detailed above; these are separate evidence classes. No `.SRCINFO` was synthesized, copied from a template or
created empty. No machine-local config/archive/checksum receipt/generated
packaging metadata is tracked in the application repo. Local artifacts are
not a published release asset, approved AUR submission or retention guarantee.

### Exact native Arch commands (metadata only)

Run from the native Pivra checkout. This creates a new private workspace; it
uses `git show` and does not alter the checkout or require branch switching.
No installation, makepkg build or publication is performed. The source config
is only for this authorized RC experiment, regardless of the template's
final-main approval wording; it is not a final-main approval record.

```bash
(
set -euo pipefail
if ! command -v makepkg >/dev/null; then
    echo 'NOT RUN: makepkg is required' >&2
    exit 77
fi
git fetch origin dev
rc_commit='7e26e780acffdb1f2db0f37e8cf6485eba0baa63'
rc_sha256='828a97503173f4fef568345e2d8b2334fcf35b6b587928692ac2261ef3b68277'
rc_workspace=$(mktemp -d -t pivra-0.1.0-rc-XXXXXXXX)
git cat-file -e "$rc_commit^{commit}"
git show "$rc_commit:packaging/arch/PKGBUILD" > "$rc_workspace/PKGBUILD"
git show "$rc_commit:packaging/arch/collect-licenses.py" > "$rc_workspace/collect-licenses.py"
curl --fail --location --proto '=https' --tlsv1.2 \
    "https://github.com/onbeskarakterli/Pivra/archive/$rc_commit.tar.gz" \
    -o "$rc_workspace/pivra-0.1.0.tar.gz"
printf '%s  %s\n' "$rc_sha256" "$rc_workspace/pivra-0.1.0.tar.gz" | sha256sum --check -
cat > "$rc_workspace/release-source.conf" <<CONFIG
# RC-ONLY; NOT approved final main source or publication.
release_commit='$rc_commit'
release_url='https://github.com/onbeskarakterli/Pivra/archive/$rc_commit.tar.gz'
release_sha256='$rc_sha256'
CONFIG
chmod 600 "$rc_workspace/release-source.conf"
cd "$rc_workspace"
makepkg --printsrcinfo > .SRCINFO
RC_EXPECTED_COMMIT="$rc_commit" RC_EXPECTED_SHA256="$rc_sha256" python3 - <<'PY'
import hashlib
import os
from pathlib import Path
values = {}
for line in Path('.SRCINFO').read_text().splitlines():
    if ' = ' in line:
        key, value = line.strip().split(' = ', 1)
        values.setdefault(key, []).append(value)
assert values['pkgbase'] == ['pivra']
assert values['pkgname'] == ['pivra']
assert values['pkgver'] == ['0.1.0'] and values['pkgrel'] == ['1']
assert values['arch'] == ['x86_64'] and values['license'] == ['GPL-3.0-only']
url = 'https://github.com/onbeskarakterli/Pivra/archive/' + os.environ['RC_EXPECTED_COMMIT'] + '.tar.gz'
assert values['source'] == ['pivra-0.1.0.tar.gz::' + url, 'collect-licenses.py']
assert values['sha256sums'] == [os.environ['RC_EXPECTED_SHA256'],
    'e0bad72c99f4629b0668f59450e53a75022ea746efe57191ab8da34fb5cf1201']
assert hashlib.sha256(Path('collect-licenses.py').read_bytes()).hexdigest() == values['sha256sums'][1]
assert set(values['makedepends']) == {'cargo', 'pkgconf', 'python>=3.11'}
assert values['checkdepends'] == ['desktop-file-utils']
assert set(values['depends']) == {'gtk4>=4.16', 'libadwaita>=1.2', 'glib2',
    'gdk-pixbuf2', 'gnome-desktop-4', 'glycin', 'bubblewrap', 'libseccomp',
    'lcms2', 'fontconfig', 'cairo', 'graphene', 'glibc', 'gcc-libs', 'hicolor-icon-theme'}
print('PASS native .SRCINFO matches the exact reviewed RC recipe/source/checksums')
PY
printf 'RC-only metadata workspace: %s\n' "$rc_workspace"
)
```

These exact metadata commands are provided for native execution and **not
reported as executed here**. `makepkg --printsrcinfo` does not create a tested
package. F2 now records native RC makepkg/install/integrity PASS from the user. The
native commands above remain a reproduction recipe, not a newly executed
Codex test; a separate F2 removal run is not inferred from install success.
Do not copy the generated source config/.SRCINFO into the application repository
without an explicitly approved tracked-file policy.

## Complete fresh F1 check results

Existing SDK: Rust/Cargo 1.99.1, GTK4 4.18.6, libadwaita 1.7.6; no environment
reset/install. Tested application/build/test files equal the final RC blobs.

| Check actually run | Result |
|---|---|
| `cargo fmt --check` | PASS / exit 0 |
| `cargo check --locked` | PASS / exit 0, no warnings |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS / exit 0, zero warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 197 tests, no failures |
| Python thumbnail discovery | PASS, 53 tests |
| Python observer/report discovery | PASS, 33 tests |
| Python memory-driver discovery | PASS, 9 tests |
| Both fixture generators `--check` | PASS, integrity not decode proof |
| Desktop validation, SVG XML/ID/hicolor paths | PASS, not new physical icon evaluation |
| Cargo metadata/version/license/lock integrity | PASS, 185 packages; unchanged lock SHA below |
| Actual license/source collector | PASS, 184 checksum-verified crate archives/notices |
| PKGBUILD Bash syntax + loading actual RC-only config | PASS |
| Distribution-mode debug binary + OUT_DIR-helper-path byte check | PASS, not an Arch release artifact |
| Current tracked-tree credential/artifact pattern scan | PASS limited check; no matches, no generated executables/metadata/CI tracked |
| Download + all Git file/mode comparisons + SHA256SUMS check | PASS, exact archive above |
| Native makepkg / `.SRCINFO` / new RC package | NOT RUN in Codex |
| Physical GNOME/Nautilus/real secure pixels | NOT RUN in Codex; separate user reports above |

Python command patterns: `python3 -B -m unittest discover -s tests/thumbnail
-p 'test_*.py'`, same for `tests` and `tests/fd05a`. Prerequisite provider and GUI
examples were built first. No failed Python prerequisites in F1.

| Controlled GUI gate | X11/Xvfb | Sway/Wayland |
|---|---|---|
| `thumbnail_overlay_probe --catalog-regression` | PASS / exit 0 | PASS / exit 0 |
| `view_modes_probe --synthetic --zoom-pan` | PASS / exit 0 | PASS / exit 0 |
| `async_cancel_probe --controlled` | PASS / exit 0 | PASS / exit 0 |
| `window_failure_race_probe` | PASS / exit 0 | PASS / exit 0 |

All are first F1 runs. X11 pointer verified before testing at X=1279/Y=1023,
held Xlib connection, screen 1280×1024, private XDG directories. Sway uses the
existing Wayland driver, XWayland disabled/DISPLAY absent. `G_DEBUG=fatal-criticals`
and Cairo renderer remain. These are synthetic/controller/race tests, not
real native decode, hardware input or sandboxed active-process cleanup.

## Source/distribution review and remaining gates

- Version/manifest/recipe remain 0.1.0, GPL-3.0-only; Cargo.lock SHA-256:
  `41c86b8c1df465f4cdacb81b9287ecd0930c170b6cb9403106e5192004544e34`.
  LICENSE equals official GNU GPLv3 text; SHA-256:
  `3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986`.
- JPEG/PNG still-image scope, skipped unsupported navigation entries, session
  GNOME-first/Freedesktop selection, secure mandatory Glycin Bwrap and native
  factory sandbox/resource limits remain unchanged. No video engine/fallback
  or other feature is introduced. Source validation and providers are preserved.
- Recipe dependencies, mandatory separately compiled native GNOME4 sidecar,
  sibling `/usr/bin/pivra-thumbnail-factory`, shared native runtime linkage,
  source/notices/recipe/config packaging reviewed unchanged. No bit-identical
  reproducibility claim or new Arch package-ownership query is made here.
- Original `.desktop` and icons match application ID and hicolor paths.
  JPEG/PNG MIME registration does not install or forcibly switch defaults.
- The archive contains tracked project source, original fixtures/assets and
  documentation, not the private SDK, local OUT_DIR, generated executable or
  machine-local RC config/.SRCINFO. Current-tree secret-pattern review is not
  a comprehensive historical secrets or CVE scan.
- Corresponding-source collection is necessary, not blanket approval of all
  future binary bundles; retain actual third-party notices, native LGPL
  source/replacement duties and archive/build inputs when distributing.
- **F1 status was PARTIAL; F2 technical status is GO** for the accepted RC.
  Native `.SRCINFO` generation/consistency, build/install/integrity are now
  user-reported PASS, still not independently executed by Codex. Explicit
  permission for main/tag/publication remains separate. A different stable
  source requires its own archive digest and regenerated/reviewed metadata. No main merge, tag, GitHub Release, AUR or CI was done.
- Preload is an **accepted v0.1.0 risk, not a blocker by owner decision**, and
  remains open/unfixed. Missing exact Rust 1.85/32-bit testing and bounded
  accounting not equaling global RSS/VRAM remain known validation limitations.

F1 application changes: none. Documentation commits record native acceptance,
then the owner's risk decision/frozen RC, then this actual archive/metadata
receipt. The receipt commit's own SHA is delivered separately; it is not
substituted for the final RC reference above. Stop here; no release is performed.
