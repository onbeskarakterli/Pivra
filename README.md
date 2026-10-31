# Pivra

**Pivra** is a native image viewer for Linux, built with **Rust, GTK4, and libadwaita**. It is designed to feel at home on GNOME: open a photo, move through the folder, zoom in on details, and browse thumbnails without getting in the way.

Pivra is free and open-source software. The **`dev` branch is version 0.2.3 (unreleased)**. The published stable release remains **0.2.1**. That stable release is a maintenance update to the 0.2 series, including removal of an obsolete video-playback warning and release consistency fixes. Mixed photo/video navigation, native video controls, animated-image playback, and Trash/Undo remain available. Editing and advanced media tools remain planned.

**[Latest published release](https://github.com/onbeskarakterli/Pivra/releases/latest)** · **[Arch Linux AUR](https://aur.archlinux.org/packages/pivra)** · **[Development roadmap](docs/PIVRA_DEVELOPMENT_ROADMAP.md)**

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

The AUR package builds Pivra from a pinned source revision. GitHub releases also provide source archives; prebuilt packages are listed only when explicitly attached to a release.

## Features and roadmap

**Available** = implemented in the current `dev` source (v0.2.3, unreleased); the published v0.2.1 release may have a different feature set. **Partial** = some aspects implemented, with more work planned. **Planned** = not yet delivered. This is a feature-level snapshot, not a declaration that every numbered roadmap milestone is complete.

### Photo viewing and navigation

| Feature | Status |
|---|---|
| Open local JPEG and PNG images | ✅ Available |
| Secure image decoding with Glycin and Bubblewrap | ✅ Available |
| Apply EXIF orientation to supported photos | ✅ Available |
| Browse other supported photos in the same folder, in natural filename order | ✅ Available |
| Previous/next navigation through photos and videos | ✅ Available |
| Fit, Fill, and 100% display modes | ✅ Available |
| Cursor-anchored zoom and drag-to-pan | ✅ Available |
| Smooth/Nearest image filtering | ✅ Available |
| Fullscreen viewing | ✅ Available |
| Scrollable, selectable, resizable, pinnable thumbnail strip | ✅ Available |
| Adjacent-photo preloading and bounded in-memory image cache | ✅ Available |
| Folder-change monitoring and manual Refresh | ✅ Available |
| Independent windows and saved window size/maximized state | ✅ Available |
| Early low-resolution previews while full images load | Not planned (project decision) |
| Image minimap when zoomed beyond the viewport | ⏳ Planned |
| Optional EXIF/metadata information panel | ⏳ Planned |
| Photo/video/all filters and multi-selection | ⏳ Planned |
| Slideshow with configurable timing, looping, and transitions | ⏳ Planned |
| Customizable keyboard shortcuts and touchpad gestures | ⏳ Planned |
| Move media to Trash with Undo | ✅ Available |
| Wallpaper setting and side-by-side comparison | ⏳ Planned |

### Video playback

| Feature | Status |
|---|---|
| Play videos in the same photo/video browsing sequence | ✅ Available |
| Automatic playback and stop when switching files | ✅ Available |
| Playback/pause, volume, mute, seek bar, and keyboard controls | ✅ Available |
| Remember volume and playback position within a window | ✅ Available |
| Approximate frame stepping via seeking | ✅ Available |
| Audio-track selection | ⏳ Planned |
| Video zoom/pan and still-frame capture | ⏳ Planned |
| Auto-hiding playback controls | ✅ Available |
| Display sleep inhibition during playback | ⏳ Planned |

### Formats, display quality, and performance

| Feature | Status |
|---|---|
| GIF, animated WebP, and APNG playback | ✅ Available |
| Still-image decoding up to 64 MP (subject to loader support and limits) | ✅ Available |
| Additional still formats such as TIFF, SVG, BMP, and ICO | ⏳ Planned |
| Experimental DNG/RAW via opt-in LibRaw sandbox (development only) | 🧪 Experimental (`PIVRA_EXPERIMENTAL_DNG=1`; separate native helper build) |
| HEIC, AVIF, JPEG XL, and PSD support where feasible | ⏳ Planned |
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

The [detailed development roadmap](docs/PIVRA_DEVELOPMENT_ROADMAP.md) retains historical PIV milestone and release-gate records; its older status snapshots do not supersede the current v0.2.3 development status above. The [feature specification](docs/PIVRA_FEATURE_SPEC_TR.md) describes the longer-term goals.

## Current limitations

Pivra is a Linux/GNOME-focused viewer, not an image editor. Slideshow, HDR/ICC workflows, and batch conversion are not yet available. Image decoding depends on installed Glycin format loaders and their supported codecs. Video playback depends on the system GTK/GStreamer media backend and available codecs. Individual formats and files may fail.

Still-image loading is bounded to 64 MP (maximum 32,768 pixels per edge) and 64 MiB input size. Animated frames have additional bounds. The automated tests and video-navigation stress test were exercised on the development build; compatibility across other Linux environments remains to be verified.

The `dev` branch is unreleased v0.2.3; these [v0.2.1 release notes](docs/releases/PIVRA_V0.2.1_RELEASE_NOTES.md) describe the last published stable release, not the development branch.

## License

Pivra is licensed under **[GPL-3.0-only](LICENSE)**. Third-party components retain their own licenses and notices.
