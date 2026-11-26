# Pivra

**Pivra** is a native photo and video viewer for Linux, built with **Rust, GTK4, and libadwaita**. Browse a folder, zoom in on details, or start a fullscreen slideshow in a GNOME-friendly interface.

**Current stable release: v0.2.7.** Published on GitHub and AUR. The `dev` branch contains additional, **unreleased** changes; its unchanged version number does not make those changes part of v0.2.7.

**[v0.2.7 release](https://github.com/onbeskarakterli/Pivra/releases/tag/v0.2.7)** · **[Release notes](docs/releases/PIVRA_V0.2.7_RELEASE_NOTES.md)** · **[Arch Linux AUR](https://aur.archlinux.org/packages/pivra)**

## Install

### Arch Linux — AUR

Install **[`pivra`](https://aur.archlinux.org/packages/pivra)** with an AUR helper:

```bash
yay -S pivra
# or
paru -S pivra
```

The AUR package builds from a pinned source revision. GitHub also provides source archives; prebuilt packages are available only when attached to a release.

### Linux — build stable v0.2.7 from source

Use the published tag for stable installation:

```bash
git clone --branch v0.2.7 --depth 1 https://github.com/onbeskarakterli/Pivra.git
cd Pivra
make deps
make build
sudo make install
```

You need Rust 1.85+, GTK 4.16+, libadwaita 1.2+, GStreamer GL 1.24+ development/runtime components, and the required native libraries. `make deps` uses your distribution's package manager and may request administrator access and confirmation. `make build` compiles the viewer and GNOME thumbnail helper. A manual installation is not tracked by your package manager.

Native installation has been tested on Arch Linux. Fedora and Debian/Ubuntu dependency recipes are provided but remain unverified; openSUSE requires manual dependency installation. Older distributions may lack the required library versions. See the [Linux build guide](packaging/linux/README.md) for additional build and staging details; retain the `v0.2.7` checkout when following it for stable installation.

## Included in stable v0.2.7

- **Photo viewing:** JPEG/PNG, EXIF orientation, secure Glycin/Bubblewrap decoding, and GIF/animated WebP/APNG playback, subject to installed loaders.
- **Viewing controls:** Fit, Fill, 100%, cursor-anchored zoom, drag-to-pan, Smooth/Nearest filtering, and fullscreen.
- **Folder browsing:** natural filename ordering, photo/video navigation, a scrollable/resizable/pinnable thumbnail strip, folder monitoring and Refresh, and Trash with Undo.
- **Playback:** video and audio, play/pause, volume/mute, seeking, approximate frame stepping, and video zoom/pan. Eligible BT.709 SDR H.264 videos use GPU BT.1886 color correction; other metadata/codecs retain stock playback.
- **Slideshow:** F5 starts fullscreen playback; Space pauses/resumes, arrows navigate, and Escape exits. Photo timing has presets and manual entry; videos advance on completion. Looping, photo fade/no-transition options, and saved slideshow preferences are included.
- **Window controls:** grouped viewing/zoom/slideshow toolbar controls, independent windows, saved window dimensions/maximized state, adjacent-photo preloading and a bounded image cache. Open replaces media in an **already active viewer** within the same window.

**v0.2.7 Open limitation:** the Open button in an empty startup window is not connected to the viewer. Open a file through your file manager or run `pivra /path/to/photo.jpg` to start viewing. The empty-window fix is available only on `dev`.

## Unreleased development changes

The current `dev` branch additionally implements:

- Open from an empty window, and Ctrl+O in empty and active windows.
- An optional media information panel with available EXIF/video metadata.
- A zoomed-photo minimap with click/drag panning and temporary navigation-overlay hiding.
- F2 single-file rename and All/Photos/Videos thumbnail filtering.
- Display-sleep inhibition during active video playback and toolbar zoom-state synchronization.

These features are **not included in the published v0.2.7 release**. Validation and remaining native GNOME checks are recorded in [post-v0.2.7 development notes](docs/development/NEXT_CYCLE_UI.md). The optional DNG/LibRaw experiment is also development-only and is not part of the published AUR package.

## Known limitations

- Pivra is a viewer; image editing, batch conversion, and complete HDR/ICC workflows are not available. Format support depends on installed Glycin loaders and GStreamer codecs; individual files may fail.
- Still images are bounded to 64 MP, 32,768 pixels per edge and 64 MiB input size. Animated frames have additional limits.
- The GPU color adapter uses OpenGL and has native AMD acceptance. Intel/NVIDIA are untested; Vulkan adapter interop, native HDR visual preservation, hardware decoding and end-to-end zero-copy are not established.
- A stock GTK Wayland/EGL lifecycle issue remains open. Pivra defaults to XWayland/X11 when `DISPLAY` is available and `GDK_BACKEND` has not been explicitly set. If needed, launch with `GDK_BACKEND=x11 GSK_RENDERER=gl pivra /path/to/video.mp4`. This is a per-process workaround, not a system GTK change or a guarantee for other GPUs.

See the [v0.2.7 release notes](docs/releases/PIVRA_V0.2.7_RELEASE_NOTES.md) for release limitations and the [validation record](docs/releases/PIVRA_V0.2.7_RC_VALIDATION.md) for evidence and unresolved observations.

## Development and historical records

- [Development roadmap](docs/PIVRA_DEVELOPMENT_ROADMAP.md) and [feature specification](docs/PIVRA_FEATURE_SPEC_TR.md): longer-term plans, not a list of shipped features.
- [Post-v0.2.7 development notes](docs/development/NEXT_CYCLE_UI.md): current unreleased functionality and focused checks.
- [v0.2.7 validation and publication record](docs/releases/PIVRA_V0.2.7_RC_VALIDATION.md): historical candidate checks and subsequent owner-confirmed publication.
- [GPU adapter rebuild and LGPL source obligations](docs/diagnostics/VIDEO_ADAPTER_REBUILD.md) and [GTK upstream handoff](docs/upstream/GTK_4.24.1_WAYLAND_EGL_OWNERSHIP.md): engineering details.
- Historical [v0.2.5 release records](docs/releases/PIVRA_V0.2.5_RELEASE_NOTES.md) and [unreleased v0.2.6 candidate records](docs/releases/PIVRA_V0.2.6_RELEASE_NOTES.md): retained evidence, neither is the current release.

## License

Pivra is licensed under **[GPL-3.0-only](LICENSE)**. Third-party components retain their own licenses and notices.
