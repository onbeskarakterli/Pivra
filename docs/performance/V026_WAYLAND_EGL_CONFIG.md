# Native Wayland EGL configuration boundary — 2026-11-20

**Wayland GPU fix: BLOCKED, not demonstrated. v0.2.6 publication remains NO-GO.**
Implementation: `ef55fd0a18b66571bf05bfc39ab57501ac5a7fee`.
Base: `444de1b108425ee3f27f5170ec9cbbdedb7e3aa8`.
[Machine-readable evidence](V026_WAYLAND_EGL_CONFIG.json).

## Native evidence belongs to the user

On Arch/GNOME, GTK 4.24.1, Mesa 26.3.1, GStreamer 1.28.9, RX 7900 XT, the
owner reports the **same binary/video** failing on native Wayland with
`GskGLRenderer: No EGL configuration available`, then
`PIVRA_GPU_CONTEXT_FAILURE stage=create-gdk-context`, then stock recovery.
With `GDK_BACKEND=x11 GSK_RENDERER=gl`, actual RX 7900 XT OpenGL 4.6, GLMemory,
smooth video, audio and BT.1886 colors PASS. This is physical user acceptance,
not a Codex measurement. The X11 EGL-versus-GLX platform was not supplied;
OpenGL 4.6 alone does not establish which native binding it used.

## Established boundary and exact source request

Official GTK 4.24.1 source establishes:

- `gdk_surface_create_gl_context()` calls `gdk_display_prepare_gl()` **before**
  constructing the requested context. Display-only creation has the same gate.
- Wayland's `gdk_wayland_display_init_gl()` calls `gdk_display_init_egl()` on
  `EGL_PLATFORM_WAYLAND_EXT` with the compositor's `wl_display` and `allow_any=TRUE`.
- `gdk_display_create_egl_config()` emits exactly **No EGL configuration available**
  when its count query returns false **or** returns zero candidates. These are
  distinct conditions; the native excerpt does not distinguish them.
- The initial SDR request is `EGL_SURFACE_TYPE=EGL_WINDOW_BIT`,
  `EGL_COLOR_BUFFER_TYPE=EGL_RGB_BUFFER`, RGB minima 8/8/8, alpha minimum 8,
  terminated by `EGL_NONE`. The count call uses configs=NULL, config_size=-1.
  There is no explicit `EGL_RENDERABLE_TYPE`. EGL's default is
  `EGL_OPENGL_ES_BIT` (GLES **1**, not GLES 2/3 or desktop OpenGL).
- A later optional HDR query uses RGB minima 9/9/9, alpha 8, and component type
  DONT_CARE when `EGL_EXT_pixel_format_float` exists. The same error text can
  originate there. An optional HDR miss is not by itself a fatal SDR failure.
- `gdk_display_prepare_gl()` caches its display-level error. A new application
  context or repeated calls cannot repair that cached initialization failure.
- X11 has separate EGL/GLX selection and native visual/config scoring. X11
  success establishes working rendering/decoding on that route, not matching
  Wayland EGL configuration availability.

At the reported failure, Pivra's `wayland_display.h` and Gst context sharing have
**not executed**: those require a successfully created/realized GDK context.
Pivra has not supplied its own EGLConfig/API attributes to this GTK query.
This rules out fixing this *particular stage* by changing the owned Gst EGL
wrapper, shader, range conversion or presentation. It does not clear the
separate historical mixed-lifecycle ownership defect.

Mesa 26.3.1 source independently confirms the implicit GLES 1 mask. It also
ignores config_size when configs=NULL, so **a negative count-query size is not
established as the cause** on that source. Actual EGLBoolean, eglGetError and
counts from the affected machine are still missing. Missing GLES 1-compatible
configs, RGBA8 availability, an EGL error or differing installed GTK build/state
must not be conflated. No Mesa defect is asserted.

References reviewed (official upstream tag sources; installed Arch distro
patches/build provenance have not been independently inspected):

- [GTK display/configuration](https://github.com/GNOME/gtk/blob/4.24.1/gdk/gdkdisplay.c#L1338)
- [GTK surface gate](https://github.com/GNOME/gtk/blob/4.24.1/gdk/gdksurface.c#L1212)
- [GTK Wayland initialization](https://github.com/GNOME/gtk/blob/4.24.1/gdk/wayland/gdkglcontext-wayland.c)
- [GTK X11 initialization](https://github.com/GNOME/gtk/blob/4.24.1/gdk/x11/gdkdisplay-x11.c#L3017)
- [Mesa configuration defaults/selection](https://github.com/chaotic-cx/mesa-mirror/blob/mesa-26.3.1/src/egl/main/eglconfig.c)

## Safe application correction delivered

The owned adapter previously discarded the actual GDK GError and replaced it
with a generic sink-construction error. `create_sink_full(..., GError **)` now
transfers the original creation/realization failure into GtkMediaStream and the
automatic fallback reason. Failed GDK acquisition returns before constructing
an unusable sink. The compatibility wrapper remains for existing probes;
sharing/plugin/link failures also return explicit errors. The real production
failure-injection regression requires the original controlled GDK reason to
survive to the stock-fallback log. Automatic stock recovery is retained.

**This improves error propagation; it does not change GTK's rejected EGL query.**
There is no verified application-only Wayland playback fix in this commit.
Public GTK APIs do not expose these private display/config attributes, and
setting allowed APIs on a context cannot help before a context exists.
Global EGL/libepoxy interposition, private GDK structure/vtable writes, retries,
forced X11 and a replacement backend were deliberately not introduced. No
system library, shader, version, packaging, security, cache/preload/thumbnail
policy, main, tag or AUR changes occur.

## Conditional upstream correction, not installed

[The review patch](gtk-4.24.1-modern-egl-config-review.patch) removes the implicit
GLES 1 requirement, explicitly asks for DONT_CARE, then rejects candidates
without desktop OpenGL or GLES 3 support before the existing scoring. It keeps
window, alpha, depth, HDR and GTK context API validation; it does not require
both APIs via an incorrect OR-ed minimum bitmask. It also uses size=0 for the
count-only query as a portability clarification, not a claimed Mesa fix.

This patch is relevant **only if** native comparison establishes the implicit
GLES 1 filter mismatch. It is an upstream review artifact, **not applied to
system GTK, not linked into Pivra, not compiled/run as GTK 4.24.1, and not a
native validated fix**. `git apply --check` against the exact downloaded tag
file PASS. If the native counts instead show an EGL error or RGBA8 failure,
that requires a different upstream investigation. A package/GTK upgrade or
patched-GTK distribution is not authorized or claimed here.

## Executed controlled verification

SDK: Debian 13, GTK 4.18.6, GStreamer 1.26.2, Rust 1.99.1, llvmpipe.

| Check | Result |
| --- | --- |
| `cargo fmt --check` / `cargo check --locked` | PASS |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS, zero warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 217 tests |
| Python root / thumbnail / FD05A / video color / video GPU | PASS, 69 / 59 / 9 / 13 / 80 = 230; real GDB available |
| Production surface/fallback/window tests, X11 and Sway | PASS, 9/9 each, software GL |
| Existing ownership/mixed/limited/full shader controls, X11 and Sway | PASS, 9/9 each, software GL |
| `cargo build --release --locked` | PASS |
| Private Wayland EGL configuration C probe, `-Wall -Wextra -Werror` | PASS execution; native failure NOT reproduced |
| Upstream patch applicability / shell syntax / diff whitespace | PASS |
| Patched GTK compilation and native RX 7900 XT Wayland after these changes | NOT RUN |

Software Wayland counts: implicit request=10, size-zero=10, explicit GLES1=10,
GLES2=10, desktop GL=10, DONT_CARE=10, no-alpha control=40. The sample renderable
mask is 0x4d. Optional HDR queries both return zero with EGL_SUCCESS. Thus the
SDR failure is not reproduced and the HDR zero alone cannot prove it. No native
Wayland, VRAM, hardware-decode, zero-copy or cross-GPU success is inferred.

An additional Python rerun without the existing GDB PATH reported 73 tests,
one skip (the native GDB group was not expanded). The final 80-test GPU run used
the actual existing GDB 16.3 and passed without skips; it is the gate recorded
above. No upstream GTK suite was run.

## One native Wayland command

On updated dev, in the normal GNOME session with test/backend overrides unset:

```sh
GDK_BACKEND=wayland bash -c 'bash tests/run_wayland_egl_config.sh && bash tests/run_video_initialization.sh'
```

The first command runs bounded queries on a **private new Wayland connection**.
It owns and terminates only that separate EGLDisplay, never GTK's display. It
records EGL errors, API/config counts, selected samples, size and alpha controls
and optional HDR counts. It does not patch GTK or render video. A private
connection cannot reproduce a corrupted/cached GTK display; that remains a
limitation. Keep its report and log, plus the second command's production report.

`IMPLICIT_ES1_FILTER_MISMATCH` supports upstream review of the attached patch;
`EGL_QUERY_ERROR`, `RGBA8_REQUIREMENT_MISMATCH` and
`GTK_SDR_CONFIG_FAILURE_NOT_REPRODUCED` require different evidence. Query exit 0
means data was collected, **not that Wayland playback passed**. The production
suite still fails its GPU gates if only stock recovery works. Final RX 7900 XT
normal playback/visual/audio acceptance is required. v0.2.6 remains unpublished
and blocked; the requested final native Wayland fix has not been achieved.
