# Building Pivra from source on Linux (experimental)

The `dev` branch currently reports **Pivra 0.2.3 (unreleased)** through Cargo. The stable `main` and published AUR package remain on the 0.2.1 release line. Native builds from `dev` do not change the installed AUR package without a separate approved installation or release. Experimental DNG decoding requires explicit opt-in and optional native helper dependencies; it is not enabled by default.


## Common Makefile interface (development branch)

From the repository root:

```bash
make check             # Inspect local build prerequisites
make deps              # Install distro packages using pacman/dnf/apt (sudo)
make build             # Build Rust viewer and GNOME native helper
make test              # Rust tests
make stage DESTDIR=/tmp/pivra-stage PREFIX=/usr
sudo make install PREFIX=/usr
```

`make deps` requires the user's package-manager confirmation and does not
install packages without being explicitly requested. Arch, Fedora and
Debian/Ubuntu package names are provided as a starting point, but **none of the
non-Arch dependency recipes has been tested**. openSUSE is intentionally
check-only pending confirmation of its current package naming. Older
Ubuntu/Debian releases may not supply GTK 4.16 or Rust 1.85; in that case
the script reports a failure rather than claiming compatibility.

`make build` reuses the previously Arch-tested
`packaging/linux/build-install.sh` implementation. The staged install
does not modify the host. `make install` installs files manually; it is not
tracked by apt/dnf/pacman/zypper and does not ship complete third-party
corresponding-source material required for redistributing binary packages.


This workflow is for Linux distributions other than Arch Linux and is **not yet
validated on Fedora, Ubuntu, Debian, or openSUSE**. The only completed native
distribution acceptance so far is on Arch/GNOME. The source script is on
`dev`; it is not part of the immutable `v0.1.0` tag.

## Requirements

- Linux with Rust/Cargo (Rust >= 1.85, edition 2024), C compiler, Bash,
  `pkg-config`, and `install`
- GTK4 development files >= 4.16; libadwaita development files >= 1.2
- Development files exposing `gnome-desktop-4`, `gio-2.0`, `gobject-2.0`,
  `gdk-pixbuf-2.0`, `lcms2`, and `libseccomp` pkg-config modules
- At runtime: compatible Glycin loaders, Bubblewrap and unprivileged sandbox
  support, GNOME thumbnail facilities for GNOME-first thumbnail generation
- Internet connectivity for the initial Cargo dependency download, unless
  crates are already cached

Package *names* differ by distribution; install the relevant development
packages using the distribution's package manager. Check availability:

```bash
pkg-config --modversion gtk4 libadwaita-1 gnome-desktop-4 lcms2 libseccomp
```

## Build, test, stage, install

From a checkout of the `dev` branch (or a later release that includes this
script):

```bash
bash packaging/linux/build-install.sh build
PIVRA_DISTRIBUTION_BUILD=1 cargo test --locked
DESTDIR="$PWD/staging" PREFIX=/usr bash packaging/linux/build-install.sh install
find staging/usr -type f
```

This staging operation changes no system files. To install system-wide
after reviewing the staged output:

```bash
sudo env PREFIX=/usr bash packaging/linux/build-install.sh install
```

For a personal installation, use `PREFIX="$HOME/.local"` and omit sudo,
but desktop integration depends on the user's XDG environment.

The script builds both `pivra` and its sibling
`pivra-thumbnail-factory`, installs the desktop entry and both application
icons, and installs Pivra's GPLv3 license. It intentionally does not install
or update system package-manager dependencies and does not alter file
associations. Re-running `install` overwrites its destination files;
it is **not** a package-manager tracked install.

**Do not redistribute a binary package based solely on this helper.** The
Arch packaging recipe includes additional third-party notices and
corresponding-source material. Binary redistributors must review and fulfill
GPL/LGPL and third-party license obligations, include required notices and
corresponding sources, and establish a suitable uninstall/upgrade story.

## Experimental DNG/LibRaw packaging (dev only)

DNG is opt-in and is not part of the published AUR package. To build and
stage the isolated LibRaw decoder alongside Pivra on a supported development
machine:

```bash
PIVRA_BUILD_EXPERIMENTAL_RAW=1 bash packaging/linux/dependencies.sh check
PIVRA_BUILD_EXPERIMENTAL_RAW=1 bash packaging/linux/build-install.sh build
PIVRA_INSTALL_EXPERIMENTAL_RAW=1 PREFIX=/usr DESTDIR=/tmp/pivra-raw-stage \
  bash packaging/linux/build-install.sh install
PIVRA_EXPERIMENTAL_DNG=1 /tmp/pivra-raw-stage/usr/bin/pivra /path/to/photo.DNG
```

The experimental build additionally requires C++, LibRaw development files
(`pkg-config libraw`), Bubblewrap and `timeout`. The staged installation
places `pivra-raw-decoder` and `pivra-raw-runner` next to the viewer, and
uses the mandatory Bubblewrap sandbox at runtime. Ordinary builds do not build
or install those companions. Staging without RAW opt-in removes any stale
experimental companions in that *staging directory only*.

Only DNG has been validated so far with a real Ricoh GR III sample. Other RAW
extensions are not enabled; malformed-input and interruption tests do not
constitute a full security audit. Do not promote this opt-in workflow to a
public release without cross-distribution checks, license review and a full
regression pass.

## Known limitations

- Only JPEG and PNG still images are supported in v0.1.0.
- No video playback, editing, or GIF/WebP/APNG animation.
- The accepted but unresolved `Preload exceeds cache entry budget`
  observation is documented in the v0.1.0 release notes.
- This experimental script has not been exercised on non-Arch systems.
  Report distribution, GTK/libadwaita versions and build error logs before
  claiming compatibility.
