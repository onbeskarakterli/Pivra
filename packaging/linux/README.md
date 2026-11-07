# Building Pivra from source on Linux (experimental)

The `dev` branch reports **Pivra 0.2.5**, the upcoming release candidate. The latest
published stable release remains v0.2.4; the version bump does not publish or
update AUR. Native Arch GPU-enabled staging and physical AMD OpenGL playback
were accepted by the user before this bump. The new 0.2.5 artifacts require
candidate staging again. Experimental DNG remains explicit opt-in, not a default
format-support claim.


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
tracked by apt/dnf/pacman/zypper. It now stages locked crate sources/notices, an
application source snapshot, the modified adapter sources and LGPL texts. This
material must be retained and reviewed by binary redistributors.


This workflow is for Linux distributions other than Arch Linux and is **not yet
validated on Fedora, Ubuntu, Debian, or openSUSE**. The only completed native
distribution acceptance so far is on Arch/GNOME. The source script is on
`dev`; it is not part of the immutable `v0.1.0` tag.

## Requirements

- Linux with Rust/Cargo (Rust >= 1.85, edition 2024), C compiler, Bash,
  `pkg-config`, and `install`
- GStreamer GL >= 1.24, video/allocators/pbutils development APIs and runtime
  playbin3/GL upload/conversion/demux/parser/decoder/audio elements
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
icons, the current-version release notes, GPL-3.0-only license, LGPL adapter
notices and corresponding sources. It intentionally does not install
or update system package-manager dependencies and does not alter file
associations. Re-running `install` overwrites its destination files;
it is **not** a package-manager tracked install.

**Do not redistribute a binary package based solely on this helper.** The source frontend and Arch recipe collect corresponding sources and notices;
the Arch recipe additionally preserves its approved immutable source and recipe.
The staging snapshot is not a replacement for immutable release-source approval. Binary redistributors must review and fulfill
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

## Candidate checks and publication handoff

Run `bash packaging/arch/stage-video-candidate.sh` on clean native Arch for the
single candidate staging step. It does not run makepkg or install system files.
After explicit immutable-source approval, use a separate release workspace for
makepkg source verification, real .SRCINFO generation and package checks.
`python3 scripts/release_preflight.py --version 0.2.5 --candidate --staging-root
/path/to/stage` validates this candidate. Final preflight omits --candidate and
requires README/release notes to identify v0.2.5 as current stable; that handoff
must be explicitly authorized before final source approval/publication.

## Known limitations

- Intel/NVIDIA GPU paths are untested; Vulkan adapter interop is not implemented.
- Native HDR visual preservation, hardware decoding and end-to-end zero-copy
  remain unproven; eligible BT.709 SDR correction uses the OpenGL path.
- Other metadata/codecs retain stock playback; image decoding remains Glycin/Bwrap.
- Optional DNG is experimental. Editing and future format/workflow extensions
  are not implied by the version bump.
- Physical distribution acceptance is Arch/GNOME; non-Arch installation recipes
  remain unverified. The Codex SDK build is not native distro-install acceptance.
- The historical accepted `Preload exceeds cache entry budget` observation is not
  claimed fixed; see the existing release/audit records.

See the [v0.2.5 release notes](../../docs/releases/PIVRA_V0.2.5_RELEASE_NOTES.md),
[candidate report](../../docs/releases/PIVRA_V0.2.5_RC_VALIDATION.md) and
[LGPL rebuild instructions](../../docs/diagnostics/VIDEO_ADAPTER_REBUILD.md).
