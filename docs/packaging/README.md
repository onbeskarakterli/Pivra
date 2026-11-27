# Linux builds and Arch package validation

The published stable package is v0.2.7. Source installation from the stable tag
is described in the [project README](../../README.md). For local development
builds, the Makefile wraps [Linux dependency checks](../../packaging/linux/dependencies.sh)
and [build/install/stage commands](../../packaging/linux/build-install.sh).

```sh
make check
make build
make test
make stage DESTDIR=/tmp/pivra-stage PREFIX=/usr
```

`make deps` installs distro dependencies using the system package manager and
may require privileges. Arch is the distribution with recorded native package
acceptance. Fedora and Debian/Ubuntu recipes remain unverified and openSUSE
requires manual dependency setup. Build prerequisites include Rust 1.85+,
GTK 4.16+, libadwaita 1.2+, and GStreamer GL 1.24+. Check actual native headers
and runtime providers with the scripts for the selected distribution.

## Arch candidate gates

The [PKGBUILD](../../packaging/arch/PKGBUILD) and staging preflight target
0.2.7. The [local validator](../../packaging/arch/validate-release.py) still
hard-codes `VERSION = '0.2.6'` and rejects an approved source at another
version. Version preparation must align all three components before native
v0.2.8 validation. The README files in the packaging tree retain earlier
version-specific instructions. Current results are in the
[v0.2.8 validation record](../releases/PIVRA_V0.2.8_RC_VALIDATION.md).

1. Prepare the target version and staging commands during the authorized
   release phase. Verify Arch native ownership with
   `bash packaging/arch/verify-video-dependencies.sh`. The current staging
   script is `bash packaging/arch/stage-video-candidate.sh` and has a fixed
   0.2.7 preflight target that requires updating with future version preparation.
2. Review and approve the exact immutable commit and real GitHub archive
   SHA-256. Preserve the source approval gate, the single `pivra` package and
   `options=('!debug')`. Keep local `release-source.conf`, `.SRCINFO` and build
   artifacts outside the application repository.
3. Use the matching version of `python3 packaging/arch/validate-release.py
   --approved-source SHA --sha256 HASH` on native Arch. Review the tool's
   source-verification, generated metadata, package list, checked build and
   extracted-content evidence. Run as an ordinary user.
4. In the intended test system, verify artifact metadata and inventory with
   `pacman -Qip` and `pacman -Qlp`. Install the reviewed local artifact and
   check `pacman -Qkk pivra`, `pivra --version`, application-menu launch,
   installed icon, desktop integration and sibling thumbnail helper. Inspect
   the package list for unintended debug/test artifacts and compare install
   and removal behavior using disposable media and config.

Distribution builds must include the mandatory adapter and GNOME thumbnail
helper, exact corresponding sources and license notices. Follow the
[adapter rebuild and relink guide](../diagnostics/VIDEO_ADAPTER_REBUILD.md) and
[release workflow](../releases/README.md). Local package builds and verified
installation are evidence for release review. Publication requires a separate
approval and result.
