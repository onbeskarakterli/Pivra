# v0.2.6 Task 01C — native AMD failure and stock EGL ownership

Status: **native acceptance BLOCKED / pending; not declared fixed**.
2026-11-19: attached matching native logs now establish real inferior exit 101,
successful shared-display eglTerminate and 38 later failed context creations.
See [final sprint evidence](V026_FINAL_SPRINT.md). The older exit-255 excerpt
below remains a separate historical debugger-classification question.
Latest native GDB follow-up: exit 255 at an observed `eglDestroyContext` stack
does not identify an inferior failure or prove display termination; see
[debugger stop classification](V026_GDB_STOP_CLASSIFICATION.md).
Base: `0ec645b69ab92e8426e8e6056f0a49cdda3263bb`, branch `dev`.
Verified implementation commit: `3988c40c14d61a4ac3f0b6f362049733ac844043`.
Recorded 2026-11-17. No version, release, packaging or main-branch changes.

## User-reported evidence

Arch/GNOME Wayland, AMD Radeon RX 7900 XT, GTK 4.24.1:
`bash tests/run_video_context_lifecycle.sh` returned eight PASS and one FAIL.
`mixed-lifecycle` exited **-6 / SIGABRT**. Four make-current warnings started
during `unknown stock route`; `full-range GPU` then reported
`PIVRA_GPU_FAIL no shared GL context`, followed by a libepoxy assertion about
the absence of a current GLX/EGL context. These are user-reported native results,
not Codex observations. The original complete native log/core was not attached.
The Task 01B llvmpipe results did not establish an AMD fix.

## Source evidence and reproduced mechanism

The exact GTK version is important. Official GTK 4.24.1 media sources moved from
`modules/media/` to `gtk/media/`; the earlier failed source lookup used the old
location. The files were read directly from the upstream 4.24.1 tag:

- [Stock sink](https://raw.githubusercontent.com/GNOME/gtk/4.24.1/gtk/media/gtkgstsink.c),
  SHA-256 `5a5591e2e72b2b93f49174cd25e3e975a9022d73e9e84990af93eb47ea6a9f04`.
- [Stock MediaFile](https://raw.githubusercontent.com/GNOME/gtk/4.24.1/gtk/media/gtkgstmediafile.c).
- [GDK current-context handling](https://raw.githubusercontent.com/GNOME/gtk/4.24.1/gdk/gdkglcontext.c).

In the stock sink's Wayland branch, lines 879–900 wrap `wl_display` with
`gst_gl_display_wayland_new_with_display()`. A derived GstGLDisplayEGL defaults
to **non-foreign**, although its EGLDisplay can be the same one GTK owns.
Stock sink dispose, lines 1078–1091, now releases `gst_context` as well as the
display/wrapped-context references. GTK 4.18.6 omitted that context release;
its leaked reference can postpone display finalization, hiding this failure
mechanism in the development environment. This version difference matters more
than a software-vs-AMD conclusion alone.

GStreamer 1.26.2's `gstgldisplay_egl.c:107–116` calls `eglTerminate()` on a
non-foreign EGLDisplay at finalization. Its `gst_gl_display_egl_from_gl_display()`
contract explicitly warns callers to mark externally owned displays as foreign.
GTK 4.24.1 stock close/dispose calls `gtk_gst_media_file_destroy_play()`, which
stops/unrefs GstPlay and its pipeline. This provides a concrete route from
**stock stream retirement → EGLDisplay termination → failed subsequent bindings**.

`stock_egl_retirement.c` reproduces this ownership mechanism in a private child
process using real GDK/GStreamer/EGL APIs, without building or replacing GTK:

```text
RETIREMENT_CONTROL policy=gtk424-wl-wrapper gtk_context_valid=false egl_error=0x3001
MECHANISM_REPRODUCED external wrapper invalidated GTK context; native AMD acceptance NOT established
RETIREMENT_CONTROL policy=borrowed gtk_context_valid=true egl_error=0x3000
PASS borrowed wrapper preserves GTK context
```

0x3001 is EGL_NOT_INITIALIZED; 0x3000 is EGL_SUCCESS. The control asserts that
the wrapper and GTK use the **same actual EGLDisplay**. The negative policy
ends its child process after observing invalidation, without rendering or
claiming successful cleanup on that deliberately invalid display. This is a
mechanism reproduction, not a native playback/cleanup PASS. No warning handler,
assertion suppression, system GTK mutation or global override is used.

The intact nine-gate sequence still passes on unmodified GTK 4.18.6/llvmpipe,
including a Wayland run **without forcing GST_GL_API**. This does not reproduce
the exact full GTK 4.24.1/AMD pipeline failure. The upstream source and control
establish an ownership defect and a plausible explanation of its native timing;
attribution to the user's actual Arch binary needs its runtime provenance and
the EGL termination caller stack. Distribution patches have not been assumed.

## Verified Pivra changes

- `current_context.h`: obtain the native EGL/GLX handle **before** querying GL
  version/API. With no binding, return GST_GL_API_NONE without calling
  `gst_gl_context_get_current_gl_api()` (which calls glGetString).
- Owned sink initialization uses that guard on its native branches. Previous
  binding capture/restoration uses the display's native platform and checks the
  actual restored handle. An unbound/stale previous GDK pointer is not restored.
  A failed restoration remains a terminal error, not silent success or CPU fallback.
- Bounded binding snapshots record monotonic time, thread, GDK object and native
  EGL/GLX handles before/after stock opening, video retirement and probe actions.
  Stock-stream weak retirement callbacks record identity/thread only; they do
  not call GTK/GL on a player finalization thread. Existing queued-source/epoch
  guards, buffer ownership and worker/context drain assertions remain intact.
- The probe reports runtime GTK version and relevant actually loaded library
  paths. The runner hashes accessible library files for provenance, preserves raw
  exit codes, reports the last transition/binding snapshots and prints a bounded
  failure tail while retaining the complete log.
- Unexpected context-initialization/restoration errors now fail even on exit 0.
  Only the existing explicit invalid-API control expects exactly eight injected
  failures; other GTK warnings and GPU errors are never exempted.
- Optional GDB observation records eglTerminate/eglDestroyContext caller stacks
  and all-thread backtraces on fatal signals. It changes no GTK implementation
  and performs no inferior GL calls. Under GDB, exit 134 and the separately
  recorded inferior SIGABRT are distinguished from a direct subprocess exit -6.

These corrections guard Pivra-owned operations and improve attribution. They
**cannot repair a stock GTK sink terminating GTK's shared display**, or guarantee
that GTK's own renderer will recover from that termination. GDK and libepoxy
warnings/assertions are left enabled. All BT.1886 math, range/matrix selection,
GLMemory synchronization, GPU shader execution and stock metadata routing remain
unchanged; no CPU color path, artificial delay or larger timeout was added.

## Constraints and upstream blocker

Changed files are limited to the owned-context guard/observability and its tests:

| Area | Files |
|---|---|
| Production | `native/video/current_context.h`, `native/video/gtkgstsink.c`, `native/video/bridge.c`, `src/gpu_video.rs`, `src/image_view.rs` |
| Existing probe/controls | `examples/integrated_video_probe.rs`, `tests/video_gpu/context_lifecycle.c`, `tests/video_gpu/lifecycle.py`, `tests/video_gpu/test_lifecycle.py` |
| New diagnostic controls | `tests/video_gpu/current_api_policy.c`, `tests/video_gpu/stock_egl_retirement.c`, `tests/video_gpu/context_trace.gdb` |
| Evidence | This report/JSON, Task 01B follow-up note, proposed upstream patch |

[Proposed upstream-only patch](gtk-4.24.1-stock-egl-ownership.patch) changes the
stock Wayland wrapper to borrow GTK's EGLDisplay and makes its EGL include work
in Wayland-only builds. It applies with `patch --dry-run --fuzz=0 -p1` to the
exact upstream file above. It is an **unapplied review artifact**, not a runtime
dependency, bundled custom GTK or instruction to replace system GTK. No patched
GTK build/native acceptance is claimed for it.
The final patch uses zero context to avoid whitespace-only context records in the
repository; its exact source SHA-256 above must be checked before use. This
documentation normalization changes no patch operation or application behavior.

The public GtkMediaFile API does not expose its internal GstPlay pipeline/sink
to Pivra for scoped GstContext injection. Fixing that object's private ownership
from Pivra would require a GTK class/global hook, retaining obsolete players,
or replacing the stock route. None is introduced: they would violate this task's
constraints or weaken resource cleanup. The upstream fix needs review and
validation in an official/distribution GTK build, or a separately authorized
application-owned passthrough design. Native acceptance remains open meanwhile.

## Checks and environment

Codex: Debian 13, GTK 4.18.6, GStreamer 1.26.2, Mesa 25.0.7 llvmpipe,
Rust/Cargo 1.99.1, existing SDK and isolated official stock media module from
Task 01B. No installation or full upstream GTK suite. Source-version/dirty-tree
evidence must not be confused with a test of an untouched release commit.

Required verification covers fmt, locked check, strict all-target Clippy, Rust
tests, the five existing Python suites, strict native C compilation, release
build, X11/Sway lifecycle and shader/control probes. The new C unit tests call
ordering with stubbed getters; the ownership-current gate also tests a real
unbound and rebound context. Neither is a substitute for native AMD rendering.

The exact results and remaining limitations are recorded in
[the result manifest](V026_NATIVE_AMD_CONTEXT_FAILURE.json). The previous Task 01
thumbnail-controller assertion is outside this task and is not silently closed.

| Executed check | Result |
|---|---|
| `cargo fmt --check`, `cargo check --locked` | PASS |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS, zero warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 212 tests |
| Python root / thumbnail / FD05A / video color / video GPU suites | PASS, 64 / 59 / 9 / 13 / 43 = 188 tests |
| Strict C compilation and current-API call-order policy | PASS |
| `cargo build --release --locked` | PASS |
| X11/Xvfb OpenGL control | 9 PASS, software control |
| Sway/Wayland default API control (GST_GL_API unset) | 9 PASS, software control |
| Wayland legacy/borrowed EGL retirement control | Mechanism reproduced / borrowed policy preserves context |
| Proposed GTK patch | Dry-run PASS, not applied or native build-tested |
| GDB mode in Codex | NOT RUN, exit 77, GDB absent |
| New revision on native AMD/GTK 4.24.1 | NOT RUN; acceptance BLOCKED/pending |

## Native command

The standard nine-gate command is unchanged. To retain the EGL destruction stack
and fatal-signal backtrace for this particular native failure, use:

```sh
bash tests/run_video_context_lifecycle.sh --debug-mixed
```

This optional mode requires native GDB; if absent it returns NOT RUN / 77 instead
of claiming a debugger result. Debuginfod downloads are disabled, GTK warnings
remain enabled and the original deadlines remain. GDB was unavailable in Codex;
its native execution is NOT RUN here. Without the flag the ordinary runner still
works, with added binding/provenance diagnostics.

Expected eventual acceptance is still **9 PASS, no unexpected make-current/GPU
failure or libepoxy assertion, actual AMD GPU presentation, and zero tracked
resources after teardown**. It has not been obtained for this revision. Preserve
`report.json` and `mixed-lifecycle.log`, especially GDB_EGL_TERMINATE,
GDB_INFERIOR_SIGNAL, the last LIFECYCLE_STAGE and PIVRA_GPU_BINDING records.

The separate destructive policy control, not an acceptance gate, is:

```sh
bash tests/run_video_context_lifecycle.sh --retirement-control
```

It requires Wayland and writes `retirement-report.json` with
`native_acceptance=false`. Intel/NVIDIA, native HDR, Vulkan interop,
hardware decoding and end-to-end zero-copy remain unverified.
