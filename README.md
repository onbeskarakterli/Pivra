# Pivra

Pivra is a native Linux photo and video viewer built with Rust, GTK4 and
libadwaita. It is designed for GNOME, with folder browsing, zoom and fullscreen
slideshow. Native package and desktop acceptance has been recorded on Arch Linux.

**Current stable release: v0.2.7.** Available on GitHub and AUR.
[GitHub release](https://github.com/onbeskarakterli/Pivra/releases/tag/v0.2.7) ·
[AUR package](https://aur.archlinux.org/packages/pivra) ·
[Stable release notes](docs/releases/PIVRA_V0.2.7_RELEASE_NOTES.md)

## Install stable v0.2.7

On Arch Linux, install `pivra` with your AUR helper:

```sh
yay -S pivra
```

To build the stable source, use the published tag:

```sh
git clone --branch v0.2.7 --depth 1 https://github.com/onbeskarakterli/Pivra.git
cd Pivra
make deps
make build
sudo make install
```

`make deps` uses your package manager and can request administrator access.
Source builds need compatible Rust and native GTK, libadwaita and GStreamer
libraries. Other distribution recipes have pending native validation.
See the [Linux build guide](docs/packaging/README.md).

## Use Pivra

Open a photo or video in your file manager, or run
`pivra /path/to/media` to browse its folder. Stable v0.2.7 supports secure
Glycin photo decoding, EXIF orientation, cursor zoom and panning, a
scrollable thumbnail strip, video/audio controls, and a fullscreen slideshow.
Format and animation support depends on installed loaders and codecs.
The stable Open button works in an active viewer. Starting from an empty
window requires a file-manager or command-line open.

| Shortcut | Action |
| --- | --- |
| Left / Right | Browse the folder |
| F6 / F7 / F8 | Fit / Fill / 100% |
| Ctrl + wheel, drag | Zoom at the pointer, pan |
| T | Toggle the thumbnail strip |
| F or F11 | Toggle fullscreen |
| F5 | Start fullscreen slideshow |
| Space | Pause/resume slideshow or video playback |
| Escape | Leave slideshow or fullscreen, or close the ordinary viewer |
| Delete | Move the selected file to Trash if supported by the desktop |

Slideshow photos use the chosen interval. Video slides play to completion.
Stable offers Fade and No transition.

## Development on dev

**Upcoming release candidate: v0.2.8.** The feature set is frozen and
under validation. Empty-window Open, Ctrl+O, the context menu, the information
panel, minimap, F2 rename, thumbnail type filters, video sleep inhibition and
additional slideshow transitions belong to this development cycle.
[Draft release notes](docs/releases/PIVRA_V0.2.8_RELEASE_NOTES.md) ·
[Validation and open gates](docs/releases/PIVRA_V0.2.8_RC_VALIDATION.md)

## Limitations

Pivra is a viewer. Editing, batch conversion and complete HDR/ICC workflows
remain outside the current release. The still-image budget is 64 MP, 32,768
pixels per edge and 64 MiB input, with additional limits for animation. DNG
requires the optional experimental decoder and explicit opt-in. The stable AUR package does not ship that helper.

Eligible BT.709 SDR video uses the OpenGL color adapter, with recorded native
AMD acceptance. Intel/NVIDIA, Vulkan interoperability and native HDR
preservation remain unverified or unsupported as detailed in the release notes.
A GTK Wayland/EGL lifecycle issue remains open. Pivra defaults to XWayland when
available unless you specify a backend. See the [GTK investigation](docs/upstream/GTK_4.24.1_WAYLAND_EGL_OWNERSHIP.md).

The [documentation index](docs/README.md) links to development, architecture,
testing, packaging, release policy and earlier release records.

Pivra is licensed under [GPL-3.0-only](LICENSE). Third-party notices apply.
