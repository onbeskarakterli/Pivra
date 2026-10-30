# Pivra

**Pivra** is a native image viewer for Linux, built with **Rust, GTK4, and libadwaita**. It is designed to feel at home on GNOME: open a photo, move through the folder, zoom in on details, and browse thumbnails without getting in the way.

Pivra is free and open-source software. The current release, **v0.1.0**, focuses on JPEG and PNG photos. Video playback, editing, and additional image formats are planned for later versions.

**[Latest release](https://github.com/onbeskarakterli/Pivra/releases/tag/v0.1.0)** · **[Arch Linux AUR](https://aur.archlinux.org/packages/pivra)** · **[Development roadmap](docs/PIVRA_DEVELOPMENT_ROADMAP.md)**

## Installation

### Linux — build from source

The current `main` branch provides a common Makefile-based build and installation workflow. You will need a compatible Linux desktop, Rust 1.85 or newer, GTK 4.16+, libadwaita 1.2+, and the necessary native libraries.

```bash
git clone https://github.com/onbeskarakterli/Pivra.git
cd Pivra
make deps
make build
sudo make install
```

- `make deps` uses your distribution's package manager to install build/runtime prerequisites and may ask for administrator access and confirmation.
- `make build` compiles Pivra and its GNOME thumbnail helper.
- `make test` runs Rust tests (optional).
- `make check` reports missing build prerequisites.
- `make stage DESTDIR=/tmp/pivra-stage PREFIX=/usr` stages a test installation without changing system files.

**Compatibility:** The common build and staged installation have been tested on Arch Linux. Fedora and Debian/Ubuntu package-install recipes are provided but have **not** yet been tested on those distributions. openSUSE currently requires manually installing dependencies. Older distributions may not provide the required GTK or Rust versions. Manual installations are not tracked by the system's package manager.

For more details, see the [Linux build guide](packaging/linux/README.md).

### Arch Linux — AUR

Pivra is available as **[`pivra` on AUR](https://aur.archlinux.org/packages/pivra)**. With an AUR helper:

```bash
yay -S pivra
# or
paru -S pivra
```

You can also install the verified prebuilt Arch package from the [v0.1.0 GitHub release](https://github.com/onbeskarakterli/Pivra/releases/tag/v0.1.0), or build it using the AUR PKGBUILD.

## Features and roadmap

**Available** = supported in the current v0.1.0 release. **Planned** = on the development roadmap, **not yet available**. Planned features are goals, not promises of a specific release date. Some early interface work was completed ahead of its original roadmap phase.

### Photo viewing and navigation

| Feature | Status |
|---|---|
| Open local JPEG and PNG images | ✅ Available |
| Secure image decoding with Glycin and Bubblewrap | ✅ Available |
| Apply EXIF orientation to supported photos | ✅ Available |
| Browse other supported photos in the same folder, in natural filename order | ✅ Available |
| Previous/next navigation; skip unsupported video entries | ✅ Available |
| Fit, Fill, and 100% display modes | ✅ Available |
| Cursor-anchored zoom and drag-to-pan | ✅ Available |
| Smooth/Nearest image filtering | ✅ Available |
| Fullscreen viewing | ✅ Available |
| Scrollable, selectable, resizable, pinnable thumbnail strip | ✅ Available |
| Adjacent-photo preloading and bounded in-memory image cache | ✅ Available |
| Folder-change monitoring and manual Refresh | ✅ Available |
| Independent windows and saved window size/maximized state | ✅ Available |
| Early low-resolution previews while full images load | ⏳ Planned |
| Image minimap when zoomed beyond the viewport | ⏳ Planned |
| Optional EXIF/metadata information panel | ⏳ Planned |
| Photo/video/all filters and multi-selection | ⏳ Planned |
| Slideshow with configurable timing, looping, and transitions | ⏳ Planned |
| Customizable keyboard shortcuts and touchpad gestures | ⏳ Planned |
| Expanded file actions, trash/undo, wallpaper, and side-by-side comparison | ⏳ Planned |

### Video playback

| Feature | Status |
|---|---|
| Play videos in the same photo/video browsing sequence | ⏳ Planned |
| Automatic playback and clean stop when switching files | ⏳ Planned |
| Playback/pause, volume, mute, seek bar, and keyboard controls | ⏳ Planned |
| Remember volume and playback position within a window | ⏳ Planned |
| Frame-by-frame stepping and timeline hover previews | ⏳ Planned |
| Embedded/external SRT/ASS subtitles and audio-track selection | ⏳ Planned |
| Subtitle appearance settings | ⏳ Planned |
| Video zoom/pan and still-frame capture | ⏳ Planned |
| Auto-hiding playback controls and display sleep inhibition | ⏳ Planned |

### Formats, display quality, and performance

| Feature | Status |
|---|---|
| GIF, animated WebP, and APNG playback | ⏳ Planned |
| Additional still formats such as TIFF, SVG, BMP, and ICO | ⏳ Planned |
| RAW, HEIC, AVIF, JPEG XL, and PSD support where feasible | ⏳ Planned |
| Multipage images and supported layered-file viewing | ⏳ Planned |
| ICC color management and conditional HDR support | ⏳ Planned |
| Further large-file responsiveness and memory/performance work | ⏳ Planned |
| Native DEB/RPM packaging and wider distribution validation | ⏳ Planned |

### Editing and batch tools — later phase

Editing will live in a separate **Edit** mode so the normal viewer remains simple.

| Feature | Status |
|---|---|
| Non-destructive editing workflow | ⏳ Planned |
| Crop, rotate, flip, and straighten | ⏳ Planned |
| Resize by dimensions or target file size | ⏳ Planned |
| Exposure, color, sharpness, noise, and detail controls | ⏳ Planned |
| Auto-enhance and red-eye correction | ⏳ Planned |
| Before/after comparison and undo | ⏳ Planned |
| Safe Save As, metadata handling, and original-file protection | ⏳ Planned |
| Background batch conversion with pause/cancel and collision handling | ⏳ Planned |

The full [60-step development roadmap](docs/PIVRA_DEVELOPMENT_ROADMAP.md) and [feature specification](docs/PIVRA_FEATURE_SPEC_TR.md) describe the implementation plan in detail.

## Current limitations

Pivra v0.1.0 displays **JPEG and PNG still images only**. It does not yet play videos, animate GIF/WebP/APNG files, or edit images. It uses a full-resolution image-loading pipeline without an early preview stage.

A previously observed `Preload exceeds cache entry budget` error remains unresolved and is documented as a known v0.1.0 risk. See the [release notes](docs/releases/PIVRA_V0.1_RELEASE_NOTES.md) for details.

## License

Pivra is licensed under **[GPL-3.0-only](LICENSE)**. Third-party components retain their own licenses and notices.
