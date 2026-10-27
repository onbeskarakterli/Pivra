# Arch packaging draft — NOT ready for AUR publication

Only `pivra` 0.1.0-1 / x86_64 is designed here. There is no approved main release
commit/tag/archive digest yet. No package was built or installed by makepkg.
Read [implementation evidence](../../docs/releases/PIVRA_V0.1_ARCH_PACKAGE_IMPLEMENTATION.md)
and [dependency evidence](../../docs/releases/PIVRA_V0.1_ARCH_DEPENDENCIES.md).

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

## Unverified Arch dependencies and build contract

`gnome-desktop-4`, `glycin` and all other Arch names in the recipe are candidate
package names based on the prior design, **not** verified pacman ownership.
The Arch API/mirrors were inaccessible and pacman/makepkg are absent here.
Do not assume glycin supplies the required JPEG/PNG **2+** loader config and
Exec solely because of its name/version. Verify with pacman -Si/-Fl/-Ql/-Qo
and real secure decode. Only then finalize depends and generate .SRCINFO.
Rust crate glycin 3.0.8 is not the OS loader package version.

The helper links the real `gnome-desktop-4`, gio-2.0, gobject-2.0 and
 gdk-pixbuf-2.0 API. Missing headers/module/linkage fail build(); Cargo's optional
helper warning is not accepted as package success. The required separately
compiled helper is installed alongside pivra at `/usr/bin/pivra-thumbnail-factory`.
No OUT_DIR glob, environment override or libexec path workaround is installed.
Makepkg compiler/linker flags are retained. Rust/pkgconf/Python are build tools;
C compiler/binutils and other standard tools are expected from base-devel.
The lcms2 and libseccomp shared-library contract is checked on the release ELF.

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
- Current absolute OUT_DIR fallback can affect binary byte reproducibility;
  sibling installation solves discovery, not bit-identical builds.
- Real GNOME menu/Nautilus/icon acceptance, secure JPEG/PNG/EXIF decode and
  GNOME generation / independent Freedesktop fallback.
- Actual pacman install/upgrade/remove: package-owned /usr files only;
  photos, ~/.config, ~/.cache, shared thumbnails and user MIME defaults survive.
- Icon provenance/owner visual acceptance beyond controlled rendering.

B5 remains **NO-GO** until real Arch installation/removal and native gates pass.
No AUR publication, tag, release, main change, security change or decoder fallback
is part of this draft.
