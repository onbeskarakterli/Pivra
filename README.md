# Pivra

Pivra is a native Linux photo and video viewer built with Rust, GTK4 and
libadwaita. It is designed for GNOME, with folder browsing, zoom and fullscreen
slideshow. Native package and desktop acceptance has been recorded on Arch Linux.

**Current stable release: v0.2.8.** Available on GitHub and AUR.
[GitHub release](https://github.com/onbeskarakterli/Pivra/releases/tag/v0.2.8) ·
[AUR package](https://aur.archlinux.org/packages/pivra) ·
[Stable release notes](docs/releases/PIVRA_V0.2.8_RELEASE_NOTES.md)

## Install stable v0.2.8

On Arch Linux, install `pivra` with your AUR helper. The published package
version is `0.2.8-1`.

```sh
yay -S pivra
```

To build the stable source from GitHub, use the release tag:

```sh
git clone --branch v0.2.8 --depth 1 https://github.com/onbeskarakterli/Pivra.git
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

Open a photo or video from the **Open File** button, with Ctrl+O, from your
file manager, or with `pivra /path/to/media` to browse its folder. Right-click
media for file, viewing and playback actions. The information panel shows
available metadata, and the thumbnail strip can filter photos and videos.
Pivra supports secure Glycin photo decoding, EXIF orientation, cursor zoom
and panning with an interactive minimap, and fullscreen slideshow. Format,
animation and playback support depend on installed loaders and codecs.

| Shortcut | Action |
| --- | --- |
| Left / Right | Browse the folder |
| Ctrl+O | Open media in the current window |
| F6 / F7 / F8 | Fit / Fill / 100% |
| Ctrl + wheel, drag | Zoom at the pointer, pan |
| T | Toggle the thumbnail strip |
| I | Toggle media information |
| F2 | Rename the current file |
| F or F11 | Toggle fullscreen |
| F5 | Start fullscreen slideshow |
| Space | Pause/resume slideshow or video playback |
| Escape | Leave slideshow or fullscreen, or close the ordinary viewer |
| Delete | Move the selected file to Trash if supported by the desktop |

Click a video once to play or pause, or twice to toggle fullscreen.
Slideshow photos use the chosen interval and advance on a left-click.
Video slides play to completion. Choose Fade, a slide or zoom effect, or
No transition.

## Development on dev

For development builds, see the [development guide](docs/development/README.md).
The [validation record](docs/releases/PIVRA_V0.2.8_RC_VALIDATION.md) tracks
the evidence and distribution status for v0.2.8.

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

Intermittent Open File and next-photo delays were reported. The investigation
was paused after the delays could not be reproduced reliably.
See the [Open latency report](docs/diagnostics/OPEN_FILE_LATENCY.md).

The [documentation index](docs/README.md) links to development, architecture,
testing, packaging, release policy and earlier release records.

Pivra is licensed under [GPL-3.0-only](LICENSE). Third-party notices apply.
