# Arch packaging — native RC accepted; publication not authorized

Only `pivra` 0.1.0-1 / x86_64 is designed here. There is no approved main release
commit/tag/archive digest recorded in this checkout yet. Clean makepkg,
checksum validation, installed integrity and removal/reinstallation at
`e8fc9ec011ad6759bc6eef018b9a6f71465f80f1` are now **user-reported native PASS**.
They were not independently repeated by Codex. Older NOT RUN/B5 NO-GO text
below records its date; current status is in the
[B6 audit](../../docs/releases/PIVRA_V0.1_B6_READINESS_AUDIT.md).
This checkout still has no approved `release-source.conf` or `.SRCINFO`; the
fail-closed source gate remains intentional. No AUR publication is claimed.
The unchanged PKGBUILD's earlier draft/native-pending comments are historical;
F2 native acceptance is recorded below. This documentation task does not alter
the recipe or add the successful native workspace's config/metadata to Git.
Read [implementation evidence](../../docs/releases/PIVRA_V0.1_ARCH_PACKAGE_IMPLEMENTATION.md)
and [dependency evidence](../../docs/releases/PIVRA_V0.1_ARCH_DEPENDENCIES.md).

## B6-F2 current acceptance

**Technical GO** for RC `7e26e780acffdb1f2db0f37e8cf6485eba0baa63` and archive
`828a97503173f4fef568345e2d8b2334fcf35b6b587928692ac2261ef3b68277`.
User-reported native Arch source verification, actual makepkg `.SRCINFO`
generation/consistency (0.1.0-1/full-SHA URL/both hashes), clean build, install
and Qkk 1309 files / 0 altered PASS. The native package is approximately 22 MB
`pivra-0.1.0-1-x86_64.pkg.tar.zst`. These were not independently run by Codex;
F1's Codex NOT RUN status below remains dated evidence, not a current missing
native gate. No source config or generated metadata is added to this repo.
Preload observation remains open and explicitly accepted for v0.1.0.
No main/tag/GitHub/AUR/CI action or final publication approval. A different
source commit requires fresh archive/metadata validation; retain source/notices.

## Files and deliberate source gate

- `PKGBUILD`: Bash-syntax-checked draft; mandatory native GNOME4 helper build,
  offline/locked release Cargo build, checks and staged installation.
- `release-source.conf.example`: empty approval inputs, deliberately separate
  from the recipe. The actual `release-source.conf` does **not** exist yet.
- `collect-licenses.py`: standard-library-only source/notice collector; copies
  the exact checksum-verified locked crate archives as corresponding source.

The recipe returns nonzero when the actual source config is absent or invalid.
This also blocks `makepkg --printsrcinfo`; an empty `.SRCINFO` is not supplied.
There is no SKIP digest, invented release checksum, moving dev URL or silently
selected snapshot. The collector's fixed SHA-256 in the recipe is real and
must be updated whenever the collector changes.

Later, **only after owner approval of the immutable main release commit**:

1. Download and inspect its GitHub commit archive; establish the real SHA-256
   from those bytes, verify the approved tag-to-commit relation if a tag exists,
   and check source contents/Cargo.lock/LICENSE. Do not publish dev as a release.
2. Copy the example to release-source.conf and fill all three approved fields:
   full 40-character commit, exactly
   `https://github.com/onbeskarakterli/Pivra/archive/<full-commit>.tar.gz`, and
   the actual 64-character archive SHA-256. The example stays blank in this task.
3. Verify Arch package names/providers below and finish native acceptance.
   Then generate `.SRCINFO` from the complete recipe using
   `makepkg --printsrcinfo > .SRCINFO`; compare source/checksums/depends.
4. In a clean Arch build environment, run `makepkg --verifysource` and a clean
   checked build (`makepkg --cleanbuild --syncdeps --check` or clean-chroot
   equivalent). Do not run makepkg/Cargo/decoder as root.

These steps are a future gate, not instructions that were executed here.

## Arch evidence and build dependency decisions — release-08

The user reports actual Arch Linux x86_64 / GNOME Wayland validation:
GTK4 4.24.1, libadwaita 1.10.0, gnome-desktop-4 51.0, glycin 2.2.4 owning the
JPEG/PNG loader files, Bubblewrap 0.13.1, Rust/Cargo 1.99.1 and GCC 16.2.1.
The user also reports a real `-Wall -Wextra -Werror` helper build and
`libgnome-desktop-4.so.2` linkage. This is user-provided native evidence,
not an independently repeated Codex/Arch test. gnome-desktop-4 and glycin remain
mandatory runtime dependencies. Existing toolkit minimums are retained rather
than raised to the user's installed versions. Rust crate glycin 3.0.8 is not
the OS loader version. Actual package install/remove acceptance is still open.

`makedepends=('cargo' 'pkgconf' 'python>=3.11')` requests Arch's Cargo provider,
rather than hard-coding `rust>=1.85`. This follows the Arch Rust package
provider convention ([Rust package guidelines](https://wiki.archlinux.org/title/Rust_package_guidelines)).
The current ArchWiki fetch returned HTTP 403; no fresh pacman provider query
is claimed. A Cargo provider can be
supplied by the system Rust package or an already configured alternative;
the recipe does not initialize rustup or download a toolchain. The project's
Cargo.toml still requires Rust >=1.85 and edition 2024. A usable toolchain must
exist before prepare/build; requesting `cargo` is not a lower-MSRV claim.

Dependency classification:

- User-validated Arch runtime names: gtk4, libadwaita, gnome-desktop-4, glycin,
  bubblewrap. Keep the existing >=4.16 / >=1.2 toolkit bounds.
- Remaining native runtime providers: glib2, gdk-pixbuf2, libseccomp, lcms2,
  fontconfig, cairo, graphene, glibc, gcc-libs. Previous ELF/API evidence justifies
  these libraries, but their exact Arch split-package ownership was not supplied
  in this message or independently verified here. They are retained pending
  pacman/namcap review, not relabeled as confirmed. Duplicate transitive coverage
  alone is not evidence a directly linked library is unnecessary.
- hicolor-icon-theme: icon integration dependency; exact Arch hook/theme
  ownership remains pending. Not a new rendering/decoder engine.
- cargo, pkgconf, Python: build tools only. C compiler/binutils/basic tools are
  expected from base-devel, not new viewer runtime requirements.
- desktop-file-utils: check dependency; no new GUI/test runtime packages added.
- Removed redundant hard pin: `rust>=1.85` in makedepends, replaced by `cargo`.
  No runtime library was arbitrarily removed and no separate unverified Cargo
  package implementation, Qt/KDE stack or service was added.

The helper links the real `gnome-desktop-4`, gio-2.0, gobject-2.0 and
gdk-pixbuf-2.0 API. Missing headers/module/linkage fail build(); Cargo's optional
helper warning is not accepted as package success. The required separately
compiled helper is installed alongside pivra at `/usr/bin/pivra-thumbnail-factory`.
No OUT_DIR glob, environment override or libexec path workaround is installed.
Makepkg compiler/linker flags are retained. Cargo-provider/pkgconf/Python are build tools;
C compiler/binutils and other standard tools are expected from base-devel.
pkg-config --cflags and --libs are now queried separately. Header/preprocessor
flags precede native/thumbnail_factory.c; makepkg LDFLAGS and pkg-config libraries
follow the source/output arguments. Both queries fail closed. The lcms2 and
libseccomp shared-library contract is checked on the release ELF.

CARGO_HOME and CARGO_TARGET_DIR are private to `$srcdir`; Cargo fetches only in
prepare(), then build/check are offline and locked. Use a clean build tree:
notice output refuses to overwrite an existing per-crate directory. A repeated
build in a reused build directory is not the supported acceptance procedure.
No system package installation, user config/cache deletion or automatic MIME
default change is performed by the recipe. There is no service or .install hook.
System desktop/icon hook behavior still needs Arch verification.

## Notices, complete sources and package size

The source inventory covers **all 184 locked registry packages**, including
foreign-target/build/dev entries, conservatively preserving upstream named
license/notice/AUTHORS/readme/manifests plus complete exact `.crate` archives.
This retains inline license/copyright terms (including cases without a file
called LICENSE), rather than inventing holders or substituting a license label.
Source archives total **18,910,822 bytes** for the current lockfile, before
compression; collecting all locked packages intentionally increases the package.
No dependency versions or root Cargo.lock are changed.

`package()` installs LICENSE and third-party notices under `/usr/share/licenses/pivra/`;
exact dependency archives, the approved Pivra archive and the actual packaging
scripts/config are installed under `/usr/share/doc/pivra/corresponding-source/`.
The native OS toolkit/loaders are dependencies, not bundled SDK binaries.
The [license audit](../../docs/releases/PIVRA_V0.1_DEPENDENCY_LICENSE_AUDIT.md)
remains authoritative for MIT/BSD/Apache/Unicode/IJG and LGPL duties. Artifact
source completeness, rebuilding/relinking and actual native package notices
still require final delivery review; merely copying a GPL text is insufficient.

## Remaining gates

- Approved main source/archive/hash; actual generated .SRCINFO.
- Verified Arch runtime and helper API package ownership; makepkg clean build.
- Distribution builds now suppress the OUT_DIR helper fallback as described
  below; whole-artifact bit reproducibility still requires two clean builds.
- Real GNOME menu/Nautilus/icon acceptance, secure JPEG/PNG/EXIF decode and
  GNOME generation / independent Freedesktop fallback.
- Actual pacman install/upgrade/remove: package-owned /usr files only;
  photos, ~/.config, ~/.cache, shared thumbnails and user MIME defaults survive.
- Icon provenance/owner visual acceptance beyond controlled rendering.

Historical gate: B5 required real Arch installation/removal and native gates.
Those package checks are now user-reported PASS at `e8fc9ec`; final release
source approval/publication remains open (see the B6 audit).
No AUR publication, tag, release, main change, security change or decoder fallback
is part of this draft.

## Release-08 targeted validation

Bash syntax and a controlled argument-capture test validate the exact helper
command ordering and preservation of CC/CPPFLAGS/CFLAGS/LDFLAGS. The capture
uses a mock compiler/pkg-config under /tmp: it is **not** native API/link proof.
The user's Arch helper compile/link result above remains separately attributed.
Missing source approval still returns the same BLOCKED/exit 1; source/URL/hash
validation and collector checksum lines are unchanged. ShellCheck is NOT RUN
because it is absent. Rust sources, tests, Cargo.lock, icons and security code
are unchanged; the existing Rust suite is rerun and reported in this delivery.
No makepkg -si, installation, main merge, tag or AUR publication occurs.

Validation results (2026-10-28, baseline 83af397): Bash syntax PASS;
exact helper flag ordering PASS; cflags/libs query-failure negative scenarios
PASS; absent-source-approval negative scenario PASS; unchanged source/hash gate
comparison PASS; `CARGO_INCREMENTAL=0 cargo test --locked` PASS — 186 tests,
zero failures. ShellCheck NOT RUN (not installed). No fresh GNOME/Wayland or
native helper compile is claimed in Codex; the reported Arch result is the
user's evidence. Release source approval, actual .SRCINFO/makepkg artifact,
remaining native provider ownership and install/remove/upgrade gates remain
open. B5 remains NO-GO.

## Distribution-only helper-path policy — after Arch OUT_DIR finding

The user reports real Arch makepkg PASS and 186 Rust tests PASS, but an absolute
`.../out/pivra-thumbnail-factory` path embedded in the installed viewer. This
report is user-provided native evidence, not an independent Codex package build.

`PIVRA_DISTRIBUTION_BUILD=1` is now exported in both build() and check(). In
build.rs this explicit value emits an empty PIVRA_BUILT_THUMBNAIL_FACTORY and
skips the optional development-side helper compilation. The PKGBUILD's separate
mandatory real-GNOME4 C helper build remains in place. check() uses the same
policy so test compilation cannot silently restore a development-path binary.
The flag is registered with cargo:rerun-if-env-changed; switching back to a
normal development build reruns the policy without requiring cargo clean.

Normal builds (unset or non-1 flag) keep the existing optional API-checked
helper and absolute development fallback. Runtime generation_helper() is
unchanged: explicit PIVRA_THUMBNAIL_GENERATOR override, then current_exe sibling,
then a nonempty compiled development fallback. Installed `/usr/bin/pivra` thus
still finds `/usr/bin/pivra-thumbnail-factory`; no libexec or security fallback
is added. No loader, provider priority, sandbox, C helper or cancellation code
changes are made.

After release Cargo build, the recipe reads the actual viewer bytes and fails
if `/out/pivra-thumbnail-factory` is found. This detects the concrete Cargo
OUT_DIR helper leak; it is not a guarantee that all debug/source paths or every
possible reproducibility difference have disappeared. Whole-package source
materials intentionally retain source code and packaging documents mentioning
the helper; scanning those is not the same as scanning the executable.

Targeted tests compile the actual build.rs with controlled pkg-config/cc tools,
then compile the exact generation_helper() body read from production source.
They verify development fallback bytes/precedence, empty distribution fallback,
absence of its path in that test binary, sibling/override behavior, missing-API
behavior and explicit opt-in. These controlled tools do not claim GNOME API,
real factory generation or secure decode PASS. A real Cargo distribution-mode
debug viewer is also built and byte-scanned in Codex; it is not an Arch release
artifact. Current results are recorded in the task delivery report.

### Required Arch revalidation (not performed in Codex)

1. Use the newly approved source revision/hash in the real local packaging
   workflow; do not bypass the existing source approval or SHA-256 checks.
   Clean-build the updated recipe on Arch with mandatory helper compilation and
   package checks enabled. Verify check() does not re-embed the fallback.
2. Inspect **the viewer executable**, first in the release target, then extracted
   from the actual pivra package, and finally installed at /usr/bin/pivra.
   The recipe's byte guard must pass in each relevant artifact. For the installed
   binary a read-only equivalent is:

   ```bash
   python - /usr/bin/pivra <<'PY'
   import pathlib, sys
   binary = pathlib.Path(sys.argv[1]).read_bytes()
   if b'/out/pivra-thumbnail-factory' in binary:
       sys.exit('FAIL: Cargo OUT_DIR helper path embedded')
   print('PASS: no Cargo OUT_DIR helper path')
   PY
   test -x /usr/bin/pivra-thumbnail-factory
   ```

3. Remove/move only the private build tree or test in a fresh VM without it;
   unset PIVRA_THUMBNAIL_GENERATOR and other SDK overrides. Launch installed
   Pivra from the menu/terminal and prove GNOME generation through its installed
   sibling with private test media/cache. A present executable is not generation
   PASS. Keep normal error/cancellation/other-window regressions.
4. Repeat install/remove/upgrade and data-preservation acceptance; compare two
   clean artifacts if claiming full bit reproducibility. This small path fix
   does not close all B5/native release gates.

Local result (2026-10-28): fmt/check PASS; cargo test --locked 186 PASS;
thumbnail Python suite 41 PASS (four new build/resolver policy tests included).
Real Cargo distribution-mode debug build and byte scan PASS. The exact recipe
artifact guard accepts a clean controlled file and rejects a leaked-path file;
Bash syntax and missing-source-approval negative checks PASS. Actual updated
Arch release package bytes and installed-helper generation remain NOT RUN here.

## v0.2.5 GPU distribution preparation

The current development release candidate is 0.2.5. Version preparation is
authorized; immutable source approval and actual makepkg validation remain separate. GPU distribution builds are mandatory and check native GStreamer
headers, runtime plugins, actual Arch file ownership and the resulting ELF.
Both LGPL texts and corresponding modified adapter sources are installed.
See [the candidate report](../../docs/releases/PIVRA_V0.2.5_RC_VALIDATION.md)
and [rebuild/relink instructions](../../docs/diagnostics/VIDEO_ADAPTER_REBUILD.md).
For clean native Arch **DESTDIR staging only**, run
`bash packaging/arch/stage-video-candidate.sh` from the repository root.
The immutable release-source.conf/hash guard and options=('!debug') remain;
this staging command is not makepkg, AUR or publication approval.

### Current Arch OpenGL ownership correction

Native user evidence identifies `/usr/lib/gstreamer-1.0/libgstopengl.so` as
owned by **gst-plugins-base-libs**, not gst-plugins-base. Both glupload and
glcolorconvert register in this same plugin (also verified against upstream
GStreamer 1.26.2 `subprojects/gst-plugins-base/ext/gl/gstopengl.c` and local
`gst-inspect-1.0`). The ownership gate now requires that owner for both elements,
checks their exact plugin basename, and rejects the former owner. Existing
playback/demux/parser/decoder/audio file expectations remain strictly checked;
local plugin filenames are verified, but local Debian ownership is never used
as evidence for Arch ownership. Current official Arch file lists, GitLab recipe
and mirror access remain blocked by HTTP 403 here; native ownership of later
checks is not claimed verified. The next native staging run must execute them.

The recipe already required gst-plugins-base-libs>=1.24, so no additional package
is needed for GL. gst-plugins-base remains installed to preserve the existing
stock codec/audio dependency set; it is no longer described as the GL provider.
No codec dependency is removed on the basis of this single ownership correction.
The old archived split-recipe evidence in the release preparation report remains
historical; this native finding supersedes its OpenGL ownership entry.

## 0.2.5 final candidate version handoff

Cargo.toml, the Pivra entry in Cargo.lock and PKGBUILD now declare 0.2.5; pkgrel=1
and options=('!debug') are preserved. The recipe installs the 0.2.5 release notes
and candidate report, not the old 0.2.4 notes. It defines one pkgname=pivra and
must not produce pivra-debug. A real makepkg package list is still a native gate.

No release-source.conf, application archive checksum or .SRCINFO is invented.
Keep local approval inputs/generated metadata out of the application repository.
After separate source approval, in an isolated release workspace:

```sh
makepkg --verifysource --force
makepkg --printsrcinfo > .SRCINFO.tmp && mv -- .SRCINFO.tmp .SRCINFO
makepkg --cleanbuild --force
makepkg --packagelist
```

Inspect the actual .SRCINFO: pkgver=0.2.5, pkgrel=1, only pkgname=pivra, the
approved immutable source and both real SHA-256 values, !debug, native dependencies
and license. Inspect actual package contents/ELF/notices/sources; no pivra-debug
archive is expected. These commands are not claimed executed in Codex.

README and release notes intentionally say release candidate now. On separately
authorized publication, change them to current stable v0.2.5, update the source
clone example to the stable line and run final preflight before approving the
final immutable archive. Do not publish dev's candidate/unreleased wording as
current stable, and do not silently change a previously approved source hash.
