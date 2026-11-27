# Linux builds and Arch package validation

This guide covers the v0.2.8 Arch recipe. The [project README](../../README.md)
describes source installation from the stable tag. For local builds, the
Makefile wraps [Linux dependency checks](../../packaging/linux/dependencies.sh)
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

The [PKGBUILD](../../packaging/arch/PKGBUILD) targets the published
**0.2.8-1** package. The staging script reads the Cargo version and the Arch
validator checks the exact requested version. The
[v0.2.8 validation record](../releases/PIVRA_V0.2.8_RC_VALIDATION.md) records
native acceptance and publication. The local candidate commands below
describe the v0.2.8 preparation workflow on the matching numeric source.

### Build and verify the local v0.2.8 candidate

Use a native Arch x86_64 test machine with the PKGBUILD's dependencies.
If dependencies are missing, install the prerequisites with
`bash packaging/linux/dependencies.sh install` and
`sudo pacman -S --needed desktop-file-utils libarchive`. Run the validator
as an ordinary user from the reviewed `dev` checkout. Substitute the exact
40-character commit being tested for `<candidate-commit>`.

```sh
python3 -B scripts/release_preflight.py --version 0.2.8 --candidate
bash packaging/arch/verify-video-dependencies.sh
python3 -B packaging/arch/validate-release.py --version 0.2.8 --local-candidate '<candidate-commit>'
```

The validator archives the specified local Git commit, computes its SHA-256,
and creates a private workspace with a `candidate-source.conf` and a local
`source` entry. It runs `makepkg --verifysource --force`,
`makepkg --printsrcinfo`, `makepkg --packagelist`, and
`makepkg --cleanbuild --force --check`. It checks the single
`pivra-0.2.8-1-x86_64` package, executable and helper, desktop entry and
icons, release docs, LGPL notices and corresponding sources. The printed
`Package:` path identifies the verified artifact. The printed workspace retains
`.SRCINFO`, package contents and logs. `build.log` can be followed during the
build. `bash packaging/arch/stage-video-candidate.sh` remains available as a
separate DESTDIR check.

Set `package` to the exact printed path and review it before installing.
The local package has the same name as the published package and replaces
an existing Pivra installation on the test machine.

```sh
package='/absolute/path/from-the-validator/pivra-0.2.8-1-x86_64.pkg.tar.zst'
pacman -Qip "$package" &&
pacman -Qlp "$package" &&
sudo pacman -U -- "$package" &&
/usr/bin/pivra --version &&
pacman -Qkk pivra &&
desktop-file-validate /usr/share/applications/io.github.onbeskarakterli.Pivra.desktop &&
test -x /usr/bin/pivra-thumbnail-factory
```

Expect `Pivra 0.2.8` and zero altered files. Verify the icon and GUI launch
from the installed application menu, and open disposable media from Nautilus.

### Verify an approved publication source separately

Keep local files and generated `.SRCINFO` out of Git. The local Git archive's
SHA-256 belongs to that artifact. Once the owner approves the actual GitHub
commit archive and its independently verified SHA-256, run
`python3 -B packaging/arch/validate-release.py --version 0.2.8 --approved-source SHA --sha256 HASH`.
That mode requires the immutable GitHub URL and generates separate
`release-source.conf` and `.SRCINFO` files. The recipe rejects both configs
being present together and keeps `options=('!debug')` in both modes.

Distribution builds must include the mandatory adapter and GNOME thumbnail
helper, exact corresponding sources and license notices. Follow the
[adapter rebuild and relink guide](../diagnostics/VIDEO_ADAPTER_REBUILD.md) and
[release workflow](../releases/README.md). Local package builds and verified
installation are evidence for release review. Publication requires a separate
approval and result.
