# Pivra v0.2.5 — Final release candidate preparation

Current base: `c15e0af082ff52cef577ec226d1d12be20c8e0cf`, dev.
Cargo.toml, the Pivra Cargo.lock entry and PKGBUILD now consistently declare
**0.2.5**; pkgrel=1 and options=('!debug') remain. This is an **upcoming candidate**,
not a published stable release. No main merge, tag, AUR update or release occurs.
The GPU backend and production thumbnail behavior are unchanged by version work.

The final preparation record is appended below. Earlier 0.2.4 version statements,
open cache failure and initial OpenGL owner table are historical: cache closure is
recorded in c15e0af; current Arch OpenGL ownership is gst-plugins-base-libs (90f13b5).
Native AMD and Arch staging are user-reported PASS before this bump, not a claim
that the newly versioned source has already passed actual makepkg acceptance.

## Historical pre-version preparation record

Base integration: `4dcd84c30013cacc884bd99de9244087d27f635d`.
Status: **development candidate / PARTIAL**, not publication approval.
The actual Cargo/lock/recipe version remains **0.2.4**, as instructed; no version
bump, tag, main merge, AUR upload or release-source approval is performed.

## Accepted native evidence

User-reported physical AMD Radeon RX 7900 XT acceptance of the integrated
backend: three BT.709 H.264 videos (720×1142, 1080×1712, 1080×1920) selected the
GPU backend automatically. GLMemory processing and synchronization, MPV-like
colors, smooth playback, sound, controls and navigation PASS. No custom GTK or
manual backend enablement. These are native user results, not Codex execution.
No repeated full GTK investigation or full GTK test-suite run is needed here.

## Arch dependency evidence and executable contract

Build and runtime dependencies added to the existing recipe:

| Arch package | Required files/capabilities | Evidence |
|---|---|---|
| `gstreamer>=1.24` | core .so/headers, gst-inspect-1.0 | Official archived Arch split recipe; current native ownership gate required |
| `gst-plugins-base-libs>=1.24` | GL/video/allocators/pbutils headers, .pc and shared libraries; playbin3 | Same |
| `gst-plugins-base` | libgstopengl.so: glupload/glcolorconvert | Same |
| `gst-plugins-good` | qtdemux, autoaudiosink | Same |
| `gst-plugins-bad-libs` | libgstvideoparsersbad.so: h264parse | Same; do not guess the much larger gst-plugins-bad split |
| `gst-libav` | libgstlibav.so: avdec_h264 | Same |

These are real Arch package names, not Debian package names. Arch integrates
headers into the runtime library packages, so separate speculative `-dev` packages
are not added. Existing `cargo`, `pkgconf`, Python and base-devel/cc/ar/binutils
build tools remain necessary; desktop-file-utils remains a check dependency.
Keep gtk4>=4.16, libadwaita>=1.2, gnome-desktop-4, glycin and existing security
libraries. libgstallocators may be removed from DT_NEEDED by as-needed because
the adapter currently consumes GL textures; its public headers remain a build
requirement. libgstplay/GstPlayer is not required by this adapter.

Source actually inspected:
`https://raw.githubusercontent.com/archlinux/svntogit-packages/packages/gstreamer/trunk/PKGBUILD`
SHA-256 `f09757d6695944a49f52be6c516c184371249d173ffe221df0ca723b49efc560`.
This is a historical archived recipe, **not current repository ownership proof**.
Current archlinux.org JSON, GitLab packaging and official mirror access returned
proxy HTTP 403 in Codex. Current installed ownership must pass
`packaging/arch/verify-video-dependencies.sh` on Arch; a changed split fails
explicitly instead of being silently assumed compatible. The native recipe calls
this gate before building. Current pacman ownership remains NOT RUN in Codex.

Distribution mode (`PIVRA_DISTRIBUTION_BUILD=1`) now makes unavailable GStreamer
APIs or `PIVRA_DISABLE_GPU_BUILD=1` a hard build error. The reversible runtime
`PIVRA_VIDEO_BACKEND=stock` path remains available. Normal development can still
explicitly build stock-only. Both package frontends check runtime elements and
inspect the actual resulting ELF for adapter markers and GTK/GStreamer shared
SONAMEs; absolute library dependencies and RPATH/RUNPATH are rejected. There is
no runtime dependency on the SDK used to compile this candidate, a local GTK fork
or a target/module environment variable. The usual native system shared libraries
and plugins must exist on the installed machine.

## Licenses and corresponding source

See [adapter rebuild/relink instructions](../diagnostics/VIDEO_ADAPTER_REBUILD.md).
Both frontends install LGPL notices and modified adapter sources. Arch retains
its exact approved source archive, crate-source collector and recipe/configuration.
The portable staging frontend now collects all checksum-verified locked crate
archives/notices plus a deterministic application-source snapshot and manifest;
installation refuses artifacts changed after source collection. A failed build
invalidates the previous successful-source pointer. This does not replace the
Arch immutable source approval/hash contract or prove bit-identical builds.

## Validation

Current Codex checks and explicit native limitations are recorded below after
execution. Controlled software-GL tests do not add new physical GPU acceptance.

## Short native Arch staging command

From a clean reviewed dev checkout with the dependency packages installed:

```sh
bash packaging/arch/stage-video-candidate.sh
```

This verifies current package ownership and runtime plugins, builds in mandatory
GPU distribution mode, stages viewer/helper/icons/desktop/licenses/source under a
fresh temporary DESTDIR and verifies the actual staged ELF. It does not install
or publish anything and **does not claim a makepkg binary package was built**.
Retain the printed staging path and logs. Fedora/Debian installation branches
have capability gates; their native installation is not certified by Arch tests.

## Historical pre-bump procedure (superseded by the final handoff below)

1. Review final dev SHA and native staging logs. Keep the accepted GPU backend.
2. Separately authorize and reconcile the 0.2.5 version in Cargo.toml/Cargo.lock,
   recipe, README and final release notes. Do not relabel a 0.2.4 binary as 0.2.5.
3. Re-run strict preparation/final gates. `scripts/release_preflight.py --version
   0.2.5 --candidate` checks preparation while showing the actual unchanged version;
   without --candidate the final gate must still fail until the bump/docs are done.
4. After an immutable source commit is approved, download its actual archive,
   calculate/independently verify SHA-256 and complete private release-source.conf.
   No digest or release source is invented in this task.
5. On Arch run `makepkg --verifysource --force`, `makepkg --printsrcinfo`, and
   `makepkg --cleanbuild --force`; retain metadata and archive checksums. Verify
   ELF/source/notices in the extracted actual package, then perform clean package
   installation/removal and native playback acceptance. Keep options=('!debug').
6. Source collection still needs exact crate archive checksums and complete source
   availability. Preserve notices and native dependency provenance in the package.
7. Main promotion, tags, AUR and publication require separate explicit authorization.

## Remaining limits

Intel/NVIDIA untested; Vulkan adapter not implemented; native HDR visual
preservation unverified; hardware decoding/end-to-end zero-copy unproven.
Full native Arch makepkg/install/relocated execution of this updated recipe and
current package ownership remain open until executed. No release approval follows
from a successful development-candidate preparation check.

## Executed results (Codex, 2026-11-07)

Private Debian 13 SDK, GTK 4.18.6, GStreamer 1.26.2, Mesa llvmpipe.
Only private SDK development files/plugins were added; system GTK was unchanged.

| Check | Result / evidence |
|---|---|
| fmt, cargo check --locked | PASS |
| Strict all-target Clippy | PASS, exit 0, zero warnings after completing the private GNOME header SDK and refreshing the build-script cache |
| CARGO_INCREMENTAL=0 cargo test --locked | PASS, 208 tests |
| Python root / GPU / color / FD05A | PASS: 47 / 32 / 13 / 9 tests |
| Python thumbnail suite | **52 PASS, 1 FAIL**; details below |
| New packaging unit coverage | 14 tests within the root count: mandatory headers/opt-out negatives, ELF/loader-path negatives, deterministic source snapshot, preflight distinction, source approval, controlled Arch ownership and missing runtime decoder |
| Real Cargo distribution opt-out negative | PASS: PIVRA_DISTRIBUTION_BUILD=1 with PIVRA_DISABLE_GPU_BUILD=1 fails with `GPU backend REQUIRED`; exit 101 is expected here |
| C metadata gate, -Wall -Wextra -Werror | PASS |
| Real discoverer routing | PASS, 9 cases: limited/full BT.709 GPU; sRGB, unknown, PQ, HLG, VP9, subtitles and PNG excluded |
| X11/Xvfb lifecycle | PASS: rapid navigation/deletion/repeated close/two windows and zero workers/pending/filters after close |
| Headless Sway lifecycle | PASS, exit 0, 1.583 s, process max RSS 397872 KiB, user 2.033 s, system 0.259 s |
| 256-level decoded full-range shader ramp/control regression | PASS, 2 LSB tolerance, X11 |
| Mandatory GPU release build and strict GNOME helper C build | PASS through portable distribution frontend |
| Actual ELF / staged copy | PASS: expected GTK, gstgl, gstvideo, gstpbutils, gstreamer shared SONAMEs, no RPATH/RUNPATH/absolute NEEDED or OUT_DIR helper fallback |
| Staged executable version | PASS: `Pivra 0.2.4`, intentionally not relabeled 0.2.5 |
| Corresponding-source integrity | PASS: all 184 crate digests match Cargo.lock/inventory, all 282 application files match the staging source manifest |
| Changed-artifact negative | PASS: a tampered viewer is rejected before destination files are written |
| Portable PREFIX=/usr DESTDIR staging | PASS: viewer/helper, desktop/icons, 184 checksum-verified crate source archives/notices, both LGPL texts, modified adapter sources and application source snapshot |
| Staged executable real window / automatic selection / draw / WM close | PASS on X11 with explicit software-test permission; normal renderer selection, no target/module variables |
| Bash syntax, desktop-file-validate | PASS |
| ShellCheck | NOT RUN: unavailable |
| Current pacman ownership, makepkg/.SRCINFO, actual updated Arch package install | NOT RUN: Arch tooling unavailable; official current package endpoints blocked by proxy 403 |

PQ/HLG routing uses **metadata-only synthetic exclusion fixtures**, with H.264
VUI transfer/primaries verified using ffprobe before Pivra discovery. GStreamer
reported `bt2100-pq` and `bt2100-hlg`; these are not HDR visual acceptance tests.
`tests/video_gpu/backend/routing.py` reproduces the nine-case gate with an
existing GTK display and ffmpeg/ffprobe. Pixel and lifecycle tests explicitly
permitted software GL and do not replace the accepted physical AMD evidence.
The small synthetic lifecycle RSS is a process measurement including GTK/Mesa,
not VRAM or a native high-resolution playback benchmark. Sway closed while
`workers=1, filters=0`; afterward all resource counters were zero.

### Unclosed regression gate: existing GNOME cache test

`tests/thumbnail/test_cache.py:60` expects `OUTCOME Stale` after atomically
replacing a file with identical bytes and preserving mtime. The actual query
returns `HIT metadata/container valid ... current=true`; assertion at line 40.
The same exact test fails independently against a separately built unmodified
`4dcd84c30013cacc884bd99de9244087d27f635d` source tree. Production cache code and
this test are unchanged by this work. Existing `src/thumbnail.rs` explicitly
uses GIO URI/MTime/Size authority and avoids a source-ctime-vs-thumbnail-mtime
rule so Trash restoration does not invalidate good thumbnails. This evidence
explains the contract conflict but does not declare it resolved or decide a new
cache freshness policy. No assertion was relaxed. This remains an open release
regression gate, requiring a separate cache-policy/test decision.

Four initial helper-policy test errors were fixed in their controlled build
harness: it now models the GPU object/archive build separately from GNOME helper
compiler success/failure, keeping every existing sibling/path/warning assertion
and adding four-GPU-object/archive assertions. This is not native GPU evidence.
The real distribution release build and negative Cargo build validate the actual
GPU contract separately. An initial cached missing-GNOME-header build warning
was resolved by private SDK completion; the final strict Clippy run is clean.

### Assessment

**GO for native Arch development-candidate staging; NO-GO for release.**
The packaging implementation and controlled GPU gates pass, and the three-video
AMD acceptance is recorded. The pre-existing cache test FAIL, current native Arch
ownership/makepkg/installation, explicit 0.2.5 version/source approval and actual
immutable archive checksum/metadata remain open. Candidate-mode preflight success
is preparation evidence, never a claim that these release gates are closed.

## Follow-up — native staging acceptance and GNOME cache gate closure

The user subsequently reports physical Arch native staging PASS: GStreamer
ownership/runtime plugins, mandatory GPU release build/ELF, collection of 184
locked crate sources and 282 application files at that staging revision, LGPL
notices, DESTDIR staging and preparation preflight. This is **user-reported
native acceptance**, not an independent Codex run. No actual makepkg/publication
is reported; the version remains intentionally 0.2.4.

The one cache FAIL above is now historical. See the focused
[GNOME cache freshness decision](../THUMBNAIL_CACHE_FRESHNESS.md): identical-byte
replacement with unchanged URI/mtime/size remains a valid persistent HIT, per
`d9bbcc1`'s intended Trash-restore fix. The obsolete test expectation was corrected
and changed-content/mtime/size, restored-file, metadata and in-flight identity
regressions were strengthened. Production cache logic is unchanged; assertions
are not bypassed. Codex results: thumbnail Python 59 PASS; all relevant Python
163 PASS; Rust 210 PASS; fmt/check and strict all-target Clippy PASS, zero warnings.

**The last known cache regression gate is closed.** Explicit v0.2.5 version/source
preparation may now be considered as a separate authorized task. No version bump,
immutable release-source checksum, makepkg artifact, main/tag/AUR action or
publication approval follows from this closure. Existing GPU/platform/HDR and
zero-copy limitations remain as documented.

## Final version handoff and source/package audit

- Version source is 0.2.5 everywhere; only the Pivra lockfile version changed,
  with the complete dependency graph and registry checksums preserved.
- README says upcoming v0.2.5 candidate and accurately retains v0.2.4 as the
  latest published stable release. New 0.2.5 notes describe the implemented GPU
  selection/transfer/lifecycle work and explicit limitations, not speculative
  Vulkan/HDR/zero-copy support.
- Arch pkgname remains only pivra, pkgver=0.2.5, pkgrel=1, options=('!debug').
  Actual makepkg output enumeration is pending, so no package/debug-archive result
  is invented. The recipe installs 0.2.5 release notes and the candidate report.
- Native APIs/plugins/ELF/ownership remain mandatory. The portable frontend also
  installs version-matched notes and binds source collection to artifact version;
  installing an old 0.2.4 build after the bump fails before writing the destination.
- Both LGPL texts, upstream notices, modified static adapter sources, complete
  application source and 184 locked crate archives remain required. Rebuild/relink
  instructions use the complete-source approach; no relink-obstructing dependency
  or opaque-only object distribution is introduced. The full license/source
  collector is unchanged and its real recipe checksum remains valid.
- No release-source.conf or .SRCINFO is created/committed. The immutable-source
  guard remains active even for metadata generation. makepkg/pacman are not
  available in Codex, so real .SRCINFO/package/install testing is NOT RUN.
- Candidate preflight now requires the exact target version, current release notes
  and README candidate/stable identity. Stage validation checks actual ELF **and
  executable --version**, plus release notes and source/notice presence. It does
  not authorize publication. Final preflight rejects candidate-only wording.

### Next native verification

From clean updated dev with native dependencies installed:

```sh
bash packaging/arch/stage-video-candidate.sh
```

This rebuilds/stages 0.2.5 and runs candidate preflight; it does not install or
run makepkg. Preserve its output before separately authorizing immutable-source
approval and actual package validation. Existing native AMD acceptance is retained;
there is no need to repeat the GPU investigation for a version-only change.

### Separate final publication procedure

1. Review this final candidate dev SHA and new native staging evidence.
2. On explicit publication authorization, update README to
   `**Current stable release: v0.2.5.**`, remove the upcoming-candidate headline,
   switch its source-clone example to the stable line, and change the release-note
   title/status to stable. Do not retain misleading unreleased/development labels
   once published. Review this documentation handoff before final source approval.
3. Run final preflight without --candidate against the actual staged binary.
   This does not substitute for immutable source/actual package gates.
4. Approve the immutable source commit separately, obtain its real archive and
   independently verify SHA-256. Fill local release-source.conf in a controlled
   release workspace; never manufacture a digest or silently change an old one.
5. Run `makepkg --verifysource --force`, then
   `makepkg --printsrcinfo > .SRCINFO.tmp && mv -- .SRCINFO.tmp .SRCINFO`;
   verify version/license/depends/!debug/source/checksums and one pkgname=pivra.
   Run a clean checked makepkg build, inspect --packagelist (no pivra-debug),
   package files/ELF/source/notices and native install/remove acceptance.
6. Main promotion, tag, GitHub Release and AUR remain separate authorized actions.

Intel/NVIDIA untested; Vulkan GPU adapter absent; native HDR visual preservation,
hardware decoding and end-to-end zero-copy unproven. No new platform guarantees.

### Final candidate execution record

Codex private Debian SDK, GTK 4.18.6 / GStreamer 1.26.2, 2026-11-07:

| Check | Actual result |
|---|---|
| `cargo fmt --check`, `cargo check --locked` | PASS |
| Strict all-target Clippy, -D warnings | PASS, exit 0, zero warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | 210 PASS, zero failures |
| Python root / thumbnail / FD05A / GPU / color | 54 / 59 / 9 / 32 / 13 PASS; **167 total**, zero failures |
| Packaging tests within root count | 21 PASS; exact-target versions, stable/RC wording, notes and source-approval negatives included |
| Strict C metadata policy | PASS |
| All examples compile | PASS |
| GPU-required release frontend build and GNOME helper | PASS |
| New PREFIX=/usr DESTDIR staging | PASS: actual `Pivra 0.2.5`, native ELF contract and desktop validation |
| Source/license collection | PASS: 184 locked crate archives, 284 application-source files, notices/LGPL/static-adapter/relink instructions |
| Old-version install negative | PASS: prior 0.2.4 artifacts rejected before destination writes |
| X11/Xvfb | PASS: nine verified metadata routes, two-window/rapid-navigation/delete/close lifecycle, 256-level pixels/seek, synthetic view/zoom/pan, controlled async cancellation and window failure races |
| Staged 0.2.5 real window on X11 | PASS: automatic renderer/backend selection, synchronized texture draw, normal WM close/filter retirement |
| Sway/Wayland | PASS: lifecycle, pixels/seek, synthetic view/zoom/pan, controlled async cancellation and window failure races |
| Bash syntax / desktop-file-validate | PASS |
| Native 0.2.5 Arch makepkg/.SRCINFO/install | NOT RUN, tooling absent and immutable source approval pending |
| ShellCheck | NOT RUN, unavailable |

Controlled rendering uses explicitly permitted software GL; it is not new native
hardware or HDR acceptance. Physical AMD and native Arch staging acceptance
remain the user's pre-bump evidence. No complete GTK suite was run.

Cargo.lock audit confirms only Pivra's own version changed; all other package
versions/registry checksums are identical. The license collector is unchanged.
Neither .SRCINFO nor a release-source config/archive hash is synthesized. Static
recipe assertions confirm one pkgname=pivra and exact options=('!debug'); a real
makepkg package/debug-archive result cannot be claimed until that separate test.

After committing the clean reviewed tree, run candidate preflight against the
actual staging tree; final preflight must still reject RC-only documentation.
The final source SHA is the resulting Git commit reported to the owner, not a
predicted value written into source approval. **GO for native 0.2.5 candidate
staging; publication is not authorized.**
