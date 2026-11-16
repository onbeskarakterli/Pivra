# v0.2.6 Task 01B — stock/GPU lifecycle regression

Status: **controlled regression PASS; native AMD acceptance pending**.
Reference: `9adca85940c09a31d200211ad2ab1314277f510a` on `dev`.
This is a focused lifecycle correction, not a performance optimization or a
release/version change. The [Task 01 baseline](V026_BASELINE.md) remains historical.
[Machine-readable results](V026_GPU_CONTEXT_LIFECYCLE.json) retain the actual
controlled exits, requested/observed APIs and final drain checkpoints.

## Native evidence and attribution boundary

The user reported successful secure-image/thumbnail baseline measurements on
physical Arch/GNOME, followed by four `gdk_gl_context_make_current() failed`
warnings, `PIVRA_GPU_FAIL no shared GL context`, and the full-range checkpoint
failing with `filters=1 workers=0 pending=0`. The performance log reached 4,096
events. Original v0.2.5 AMD acceptance remains valid historical evidence.

Codex did **not** reproduce those exact driver warnings on llvmpipe. It did
reproduce the transition and several concrete application ownership/threading
defects below. They are corrected and tested; attributing the exact AMD warning
to one of them, or declaring native GPU acceptance complete, requires the native
rerun. No native log attachment beyond the user's excerpts was available here.

## Demonstrated defects and minimum corrections

1. **Presentation could execute on the producer thread.**
   `g_main_context_invoke_full()` may execute inline when the calling thread can
   acquire the default context. The lifecycle probe pumps GTK manually, leaving
   intervals without an owner. A real GThread control reproduced inline
   publication: expected two queued presentations, observed zero, before the GTK
   thread dispatched anything. This invalidates the assumption that the callback
   always runs on GTK's thread. Normal `GApplication` loop ownership can mask this
   race, consistent with earlier successful normal playback.

   `gtkgstpaintable.c` now attaches an explicit GSource to the default context,
   preserving default priority and the two-presentation bound. A source cannot
   publish opportunistically on its producer. Reset increments an epoch and
   clears the retained image after pipeline NULL; old queued deliveries release
   their texture/buffer references without republishing. The control tests both
   stale rejection and a successful new-epoch delivery, with and without a main
   context owner.

2. **Wayland EGL display ownership was wrong.**
   Wrapping `wl_display` did not mark the derived `GstGLDisplayEGL` as foreign.
   The C control asserted `egl->foreign_display` and failed on the old code.
   GStreamer's EGL finalizer calls `eglTerminate` for a non-foreign EGLDisplay;
   that is unsafe when GTK owns the same display and other windows still use it.
   The sink now wraps `gdk_wayland_display_get_egl_display()` with
   `gst_gl_display_egl_new_with_egl_display()`, whose borrowed-display contract
   preserves GTK ownership. X11's existing EGL/GLX selection is retained.

3. **The sink leaked its owned GStreamer GL context reference.**
   `gst_gl_display_create_context()` returns a full reference. Sink dispose
   cleared the wrapped/GDK/display references but omitted `gst_context`.
   The weak-reference retirement control failed (0 notifications rather than 1)
   before the correction. Dispose and initialization-error cleanup now release
   that reference exactly once.

4. **The transfer filter skipped parent GL cleanup.**
   Its `gl_stop` released its shader but did not chain to `GstGLFilter::gl_stop`.
   That parent owns the FBO, vertex/index buffers and VAO used by
   `gst_gl_filter_render_to_target_with_shader()`. With the sink reference fixed,
   the stronger real-playback drain still failed with
   `filters=0 workers=0 pending=0 contexts=4 frames=0 presentations=0`.
   Pipeline retirement and sink disposal were observed, so this was not merely
   an application waiting for a window to close. Chaining the parent cleanup on
   the GL thread made all four contexts retire. Repeated reopen/close now also
   returns the tracked resources to zero. Shader math, caps, range conversion,
   synchronization metadata and GPU transport were not changed.

5. **Initialization clobbered the caller's current GDK context.**
   The C control made a caller context current before constructing the sink;
   the old implementation failed to restore it. Initialization now retains and
   restores it on success and failure, balances wrapped-context activation, and
   clears GDK's TLS before restoring native current state. Context creation uses
   the required GstGLDisplay object lock. An injected unsupported GST_GL_API
   verifies failure cleanup and restoration; it is not a native hardware test.

Public API contracts reviewed against GStreamer 1.26.2:
[EGL display ownership](https://github.com/GStreamer/gstreamer/blob/1.26.2/subprojects/gst-plugins-base/gst-libs/gst/gl/egl/gstgldisplay_egl.c),
[context creation/transfer](https://github.com/GStreamer/gstreamer/blob/1.26.2/subprojects/gst-plugins-base/gst-libs/gst/gl/gstgldisplay.c),
[parent filter cleanup](https://github.com/GStreamer/gstreamer/blob/1.26.2/subprojects/gst-plugins-base/gst-libs/gst/gl/gstglfilter.c).
The production fix uses existing GTK/GStreamer APIs within the current minimum
versions, and introduces no custom GTK or new library dependency.

## Harness and diagnostics

The initial controlled environment had no GTK stock media module:
`GtkNoMediaFile` could satisfy the old stock-route counters without playing a
stock video. The probe now requires a real stock backend (exit 77 if absent),
checks the actual video picture's frame dimensions, and checks the full-range
selection title. A nonvisual `video-content` widget name identifies that picture;
thumbnail or unrelated pictures cannot satisfy its frame barrier.

Wait deadlines remain 10 seconds; no speculative sleep or timeout increase was
introduced. Final waits assert zero Pivra-owned contexts, mapped GL frames,
queued presentations, filters, discovery workers and pending discoveries.
The counters do not count every driver allocation, GDK renderer context, or VRAM.

`PIVRA_TRACE_VIDEO=1` adds bounded monotonic lifecycle events with owner/thread
identity, pipeline/sink retirement and context reference diagnostics. Failure
output includes the initialization phase, native platform/API/handle and error.
Pipeline bus errors close the owned pipeline and reset presentation, even if a
caller retains the errored MediaFile. No automatic CPU correction is substituted.

Unchanged filter-count polling is deduplicated before performance emission.
The runner preserves terminal logs, raw exit/signal codes and diagnostic counts.
Even exit 0 is FAIL if a context/GPU failure is logged or the 4,096-event cap is
reached. Only the explicitly injected API-failure control expects eight failures
and requires exactly that count; ordinary playback has no error exemption.

## Validation and reproducibility

Recorded 2026-11-16 in Codex: Debian 13, GTK 4.18.6, GStreamer 1.26.2,
Mesa 25.0.7 llvmpipe, Rust/Cargo 1.99.1. Xvfb uses 1280×1024; Sway is headless
with a pixman compositor. Rendering was real OpenGL 4.5 on X11 and OpenGL ES 3.2
on Wayland, explicitly permitted **software controls**, not hardware acceptance.

For genuine stock playback, the official Debian `libgtk-4-media-gstreamer`
4.18.6+ds-2 package was verified (SHA-256
`74b69f1d116e108944311dccadb1289df5897a9e4b8abf817f665a52c66773fe`)
and extracted to a user directory. Its module was found through GTK_PATH for
GUI runs only; system GTK was not installed or replaced. A preliminary attempt
using GIO_EXTRA_MODULES globally polluted non-GTK helper stderr and failed three
CLI assertions plus one helper-stderr assertion. With that setting removed,
the unchanged tests passed. The final runner does not set GIO_EXTRA_MODULES.

The benchmark creates private H.264 files with ffprobe-verified limited BT.709,
sRGB, unknown transfer/primaries and full-range BT.709 metadata. It runs the exact
four-file sequence, real stock rendering, two windows, rapid navigation, repeat
close/reopen, deletion, injected malformed input, presentation controls, and a
256-level full-range ramp against an independent BT.1886 formula (2 LSB tolerance).
Controlled memory-texture tests exercise callback ownership only, not decoding.
No user media or existing user configuration/cache is modified.

| Gate | Result |
|---|---|
| fmt / locked check / strict all-target Clippy | PASS, zero Clippy warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 212 tests |
| Python suites: root / thumbnail / FD05A / video color / video GPU | PASS, 64 / 59 / 9 / 13 / 38 = 183 |
| `cargo build --release --locked` and native C `-Wall -Wextra -Werror` compilation | PASS |
| X11: five ownership controls, mixed lifecycle, malformed-input cleanup, limited presentation/controls, full-ramp shader/controls | PASS, 9 gates |
| Sway: same nine gates | PASS, 9 gates |
| Native Arch/AMD with this correction | NOT RUN in Codex; user acceptance required |
| Intel/NVIDIA, Vulkan, native HDR visual output, hardware decode, end-to-end zero-copy | NOT RUN / not established |

Exact controlled commands, after activating the existing SDK and setting the
isolated official stock-module GTK_PATH:

```sh
dbus-run-session -- bash tests/run_video_context_lifecycle.sh --session x11 --software --api opengl3
dbus-run-session -- bash tests/run_video_context_lifecycle.sh --session sway --software --api gles2
```

Runner regression tests exercise real subprocess FAIL/NOT RUN exits, deadlines,
logged failures on exit 0, diagnostic exhaustion and exact injected-error counts.
The upstream GTK test suite was not run. Source version, dirty flag, requested API,
software-control flag and each exit are recorded in the generated report; tests
of an instrumented working tree must not be attributed to the untouched baseline.
The native logs record the actual renderer/GL version. The mixed lifecycle emitted
647 performance events on X11 and 612 on Sway, below the unchanged 4,096 cap.
The separate Task 01 thumbnail controller assertion was not changed or retested;
this report does not close that gate or repeat native still-image acceptance.

## Native Arch acceptance

From the updated `dev` checkout in the normal GNOME/Wayland session:

```sh
bash tests/run_video_context_lifecycle.sh
```

No backend-enabling environment variables or GTK fork are required. The runner
builds the adapter/probes using installed development APIs, generates private
fixtures, prints its retained log directory, and never installs a package.
Expected: nine PASS results, exit 0, real GtkGstMediaFile for the two stock clips,
AMD hardware GL in the GPU logs, GLMemory/sync metadata retained, no make-current
warning or playback GPU failure, and zero tracked resources after every final
close. The injected API-error case deliberately records eight failures; the
malformed-input case records a controlled decode error and verifies cleanup.

If the original AMD failure persists, preserve `report.json`, `mixed-lifecycle.log`
and the ownership logs. The last `LIFECYCLE_STAGE`, monotonic context-init stage,
owner/thread identities and error reason locate the preceding transition without
exhausting the performance stream. Native visual comparison with MPV remains a
separate acceptance requirement. Task 02 optimizations are not included.
