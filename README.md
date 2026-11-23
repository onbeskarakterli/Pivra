# Pivra

**Pivra** is a native image viewer for Linux, built with **Rust, GTK4, and libadwaita**. It is designed to feel at home on GNOME: open a photo, move through the folder, zoom in on details, and browse thumbnails without getting in the way.

**Current stable release: v0.2.5.** Pivra includes automatic GPU-accelerated BT.709 SDR video color correction on stock GTK and is free software under GPL-3.0-only.

**Upcoming release candidate: v0.2.6.** Performance/resource-lifetime work is prepared. Native RX 7900 XT acceptance reports ownership-patched GTK Wayland 9/9 and X11 9/9; stock GTK Wayland remains 8/9 with a known mixed-playback limitation. Release approval is pending. See the [candidate notes](docs/releases/PIVRA_V0.2.6_RELEASE_NOTES.md) and [validation gates](docs/releases/PIVRA_V0.2.6_RC_VALIDATION.md). This does not change the current stable release.

v0.2.6 is not published. The [v0.2.5 source review](docs/releases/PIVRA_V0.2.5_SOURCE_APPROVAL.md) and [release notes](docs/releases/PIVRA_V0.2.5_RELEASE_NOTES.md) remain the stable-version record; the [v0.2.6 checklist](docs/releases/PIVRA_V0.2.6_RC_VALIDATION.md) tracks the next release separately.

**Feature branch only — v0.2.7 slideshow: TESTING.** On
`feature/v0.2.7-slideshow`, F5 starts a fullscreen slideshow; Space pauses,
Left/Right navigate and Escape exits. Photos use a configurable interval and
photo-only crossfade; videos advance at completion. Ordinary F11 intent is
preserved. This feature is not part of stable v0.2.5 or the v0.2.6 candidate.
See [validation and native acceptance instructions](docs/slideshow/V027_SLIDESHOW_VALIDATION.md).

**[Latest published release](https://github.com/onbeskarakterli/Pivra/releases/latest)** · **[Arch Linux AUR](https://aur.archlinux.org/packages/pivra)** · **[Development roadmap](docs/PIVRA_DEVELOPMENT_ROADMAP.md)**

## Installation

### Linux — build from source

Use the approved v0.2.5 source archive for release builds; the workflow below describes a development checkout, not an immutable release input. You will need a compatible Linux desktop, Rust 1.85 or newer, GTK 4.16+, libadwaita 1.2+, GStreamer GL 1.24+ development/runtime components, and the necessary native libraries.

```bash
git clone --branch dev https://github.com/onbeskarakterli/Pivra.git
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

**Available** = implemented in the current `dev` source (v0.2.6 development source); publication is tracked separately from implemented functionality. **Partial** = some aspects implemented, with more work planned. **Planned** = not yet delivered. This is a feature-level snapshot, not a declaration that every numbered roadmap milestone is complete.

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
| Slideshow with configurable timing, looping, and transitions | 🧪 Feature branch TESTING (v0.2.7; native acceptance pending) |
| Customizable keyboard shortcuts and touchpad gestures | ⏳ Planned |
| Move media to Trash with Undo | ✅ Available |
| Wallpaper setting and side-by-side comparison | ⏳ Planned |

### Video playback

| Feature | Status |
|---|---|
| Play videos in the same photo/video browsing sequence | ✅ Available |
| Automatic playback and stop when switching files | ✅ Available |
| GPU BT.1886 display transfer for eligible BT.709 SDR H.264 video | ✅ Available (AMD OpenGL native accepted) |
| Playback/pause, volume, mute, seek bar, and keyboard controls | ✅ Available |
| Remember volume and playback position within a window | ✅ Available |
| Approximate frame stepping via seeking | ✅ Available |
| Audio-track selection | ⏳ Planned |
| Video zoom/pan | ✅ Available |
| Still-frame capture | ⏳ Planned |
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

The [detailed development roadmap](docs/PIVRA_DEVELOPMENT_ROADMAP.md) retains historical PIV milestone and release-gate records; its older status snapshots do not supersede the current feature snapshot and version-specific release notes. The [feature specification](docs/PIVRA_FEATURE_SPEC_TR.md) describes the longer-term goals.

## Current limitations

Pivra is a Linux/GNOME-focused viewer, not an image editor. Slideshow is under test on the v0.2.7 feature branch; HDR/ICC workflows and batch conversion are not yet available. Image decoding depends on installed Glycin format loaders and their supported codecs. Video playback depends on the system GTK/GStreamer media backend and available codecs. Individual formats and files may fail.

Still-image loading is bounded to 64 MP (maximum 32,768 pixels per edge) and 64 MiB input size. Animated frames have additional bounds. The automated tests and video-navigation stress test were exercised on the development build; compatibility across other Linux environments remains to be verified.

The v0.2.5 release scope is described in the [release notes](docs/releases/PIVRA_V0.2.5_RELEASE_NOTES.md). Source/package/publication approval is recorded separately; historical release records remain in docs/releases.

The GPU adapter targets the validated OpenGL path. Intel/NVIDIA are untested; Vulkan interop is not implemented; native HDR visual preservation and end-to-end zero-copy remain unproven. Other color metadata/codecs retain stock playback.

On the reported GTK 4.24.1 / Mesa 26.3.1 Arch/GNOME Wayland stack, switching between GPU-eligible and stock-routed videos can invalidate the shared EGLDisplay and break later GPU playback or another window. Normal production Wayland playback passed the owner’s tests; the mixed sequence remains a stock GTK limitation. A separate, isolated GTK ownership fix passed native 9/9 and is not bundled with Pivra. An earlier software candidate SIGSEGV remains unclassified. The equivalent ownership fix already exists upstream; fixed distribution-package acceptance is pending. See the [upstream handoff](docs/upstream/GTK_4.24.1_WAYLAND_EGL_OWNERSHIP.md) and [release risk checklist](docs/releases/PIVRA_V0.2.6_RC_VALIDATION.md).

Where XWayland/X11 is available, a per-process workaround is `GDK_BACKEND=x11 GSK_RENDERER=gl pivra /path/to/video.mp4`. Native RX 7900 XT X11 lifecycle acceptance is user-reported 9/9. This does not change the system backend or guarantee other GPUs.

## License

Pivra is licensed under **[GPL-3.0-only](LICENSE)**. Third-party components retain their own licenses and notices.
