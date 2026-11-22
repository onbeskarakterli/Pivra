# Mixed GPU / stock ownership control — 2026-11-23

Base: `2d6a580cdb71e6810014b0fa72b1235634cf28ba`, branch `dev`.
**Native AMD acceptance BLOCKED / v0.2.6 NO-GO.**

## Native evidence and root-cause boundary

The latest owner report is seven nonvideo gates PASS and video 8/9 PASS, with
only mixed-lifecycle failing. Normal RX 7900 XT GPU playback, colors, audio and
seek are user-reported PASS. These are not Codex hardware measurements. No new
complete raw trace accompanied that report.

The retained native files were downloaded again and their hashes verified:

- `mixed-lifecycle.log`: `6683ec6fba6014da4bb18d3af5d8882c2f87a5026bc6881d7a12dfd2fe5b2b8c`.
- `mixed-lifecycle.gdb.jsonl`: `d3a62ee7be27a7cb04a2fa1b72a7cccf0fb3b2dac0631162f14c0732c27795af`.

All 117 GDB events were parsed. The inferior-result event records exit 101.
The historical GDB launcher's exact process exit is not reconstructed from the
sidecar. Call 18 is `eglTerminate(0x55d875099f70)`, return 1, through stock
GTK/GStreamer retirement while another Pivra GPU window uses that same display.
Call 19 fails to destroy the stock context, return 0; make-current failures and
38 failed subsequent context creations follow. See the retained
[original analysis](V026_FINAL_SPRINT.md#1-native-egl-ownership-failure-established-correction-still-blocked).
The old trace does not directly read the stock wrapper's foreign flag, and its
exact application source SHA remains unknown. It is not relabelled as a new run.

GTK 4.24.1 `gtk/media/gtkgstsink.c` wraps the borrowed `wl_display` in a native
`GstGLDisplayWayland`, but leaves its derived/cached `GstGLDisplayEGL` owning.
GStreamer 1.28.9 `gstgldisplay_egl.c` documents this exact contract:
`from_gl_display()` caches a nonforeign wrapper; callers sharing an externally
owned EGLDisplay must mark that wrapper foreign. Its finalizer otherwise calls
`eglTerminate()`. The setter is public since GStreamer 1.26; GTK 4.24.1 already
requires GStreamer >=1.28. No private field writes or ABI assumptions are needed.

Pivra's owned sink already applies this policy in `native/video/wayland_display.h`.
The missing ownership declaration is inside GTK's private stock sink. Public
GtkMediaFile APIs cannot safely reach that internal object. No demonstrated
Pivra window, pending-frame or shader defect justifies changing the accepted
normal playback path. No obsolete player retention, EGL interception, global
GTK override, CPU conversion, timeout increase or assertion relaxation is used.

## Narrow upstream correction, isolated only

[Stock ownership patch](gtk-4.24.1-stock-egl-ownership.patch): retain the native
Wayland display type, obtain its cached EGL wrapper, check its handle against
GDK's EGLDisplay, call public `gst_gl_display_egl_set_foreign(..., TRUE)` **before**
wrapped/worker context creation, then release the temporary reference. Missing or
mismatched handles return the existing initialization failure. Include the EGL
header also for Wayland-only builds. GDK retains ownership of final display
termination. GTK's existing object cleanup still releases pipelines/contexts.

The previously accepted [EGL configuration patch](gtk-4.24.1-modern-egl-config-review.patch)
is unchanged. It solves the separate implicit GLES1 config-selection defect.
Both corrections are needed for the owner's isolated Wayland experiment.
**The system GTK, Pivra production engine, shaders, fallback, controls, memory
budgets and thumbnail/preload optimizations are unchanged.** An upstream or
distribution GTK correction is still required for unmodified-system mixed
playback; the private test build is not a packaged solution.

## Reproducible A/B and safeguards

```sh
GDK_BACKEND=wayland bash tests/run_gtk_egl_isolated.sh --stock-lifecycle --workspace "$HOME/.cache/pivra-gtk-stock-ownership"
```

Run from `dev` on the native desktop. Requires the existing development tools,
GTK 4.24.1 build prerequisites and GStreamer >=1.28 headers/runtime/plugins.
There is no automatic system installation. Use a new dedicated workspace; the
builder rejects unmanaged nonempty/system/application directories. It downloads
the pinned GTK archive, verifies SHA-256, builds without `meson install`, and runs:

1. **Control:** accepted EGL configuration fix, original stock sink ownership.
2. **Candidate:** identical EGL configuration fix plus stock ownership fix.

Only the reviewed stock sink differs between sources; shader/renderer or added
source changes are rejected. The same Pivra/adapter binary is used in fresh
processes for both variants. Runtime library resolution, actual loaded GTK hash,
source hashes, build options, each raw exit/log and all nine gate results are
retained. GTK 4.24.1's GStreamer backend is **built into libgtk**: HAVE_GSTREAMER
is mandatory. The old `modules/media/libmedia-gstreamer.so` filename cannot prove
its availability. The unchanged mixed probe also refuses GtkNoMediaFile.

Control failures remain visible. Only nine genuine candidate PASS results with
verified runtime library identity yield exit 0. Unexpected make-current warnings,
EGL/GPU errors, fallback or dirty resource counters still fail existing gates.
Reports and raw logs remain in the printed private temporary directories. Native
acceptance requires candidate 9/9, no context warnings, correct GLMemory/color/
audio/seek, and clean counters after navigation and close. A software pass cannot
establish RX 7900 XT acceptance.

The separate destructive mechanism control now retains **two real GTK contexts**.
The legacy wrapper must invalidate both; direct borrowed and native-Wayland+
foreign wrappers must preserve both, including successful make-current and final
object cleanup. The negative process exits immediately after establishing the
destructive mechanism; this is never counted as a successful application close.

## Executed results

See [machine-readable results](V026_MIXED_GPU_STOCK_FIX.json) for measured results
and library/source provenance. Codex uses Debian 13, Mesa llvmpipe, headless Sway
and Xvfb, not the owner's physical AMD system. A private GStreamer 1.28.9 build
supplies core/base/GL/play; installed compatible plugin directories supply MP4
parsing and libav decoding. This is not a clean native Arch distribution test.
The initial private-link setup and missing-MP4-plugin attempts failed before a
valid mixed comparison and are retained in local logs; neither is counted PASS.

Native AMD 9/9, Intel/NVIDIA, actual VRAM and hardware decode/zero-copy: **NOT RUN**.
The accepted normal playback/shader evidence and all historical v0.2.5 records
remain intact. No main, tag, release, AUR, version, package or lockfile changes.

| Gate | Executed result |
| --- | --- |
| Rust fmt / check --locked / strict all-target Clippy / release | PASS, exit 0; zero Clippy warnings |
| Rust tests | PASS, 217; zero failures |
| Python root / thumbnail / FD05A / video color / GPU | PASS, 69 / 59 / 9 / 13 / 92 = 242; no skips, real GDB controls included |
| Real two-context retirement controls, GTK 4.24.1 + Gst 1.28.9 | Legacy invalidation reproduced; direct borrowed and native-Wayland foreign policies PASS |
| X11/Xvfb, EGL-only control | PASS 9/9 |
| X11/Xvfb, ownership candidate | PASS 9/9 |
| Sway/Wayland, EGL-only control | FAIL 8/9: mixed exit 101; four make-current warnings, full-range GPU fails, no pending owned workers |
| Sway/Wayland, ownership candidate | FAIL 8/9: mixed SIGSEGV / exit -11 after unknown stock retirement; zero make-current/config/GPU-failure markers |
| Additional candidate mixed runs under GDB | Five exited normally, including one with perf/private XDG homes; no failing stack captured |
| Physical RX 7900 XT, this ownership candidate | NOT RUN; native 9/9 pending |

The Sway candidate SIGSEGV remains **OPEN / UNCLASSIFIED**. Passing debugger runs
are not substituted for its failed ordinary gate. Its timing sensitivity alone
does not prove a stale callback, invalid pending GL work, Mesa fault, harness
fault, or patch independence. No speculative production fix is made for it.
Wayland mixed cleanup and full-range restoration are therefore still unproven
for this candidate. The native command preserves the failing gate and all
resource assertions; it is a diagnostic candidate, not release approval.

Local raw A/B reports: `/tmp/pivra-gtk-stock-lifecycle-6n87__wr` (Sway),
`/tmp/pivra-gtk-stock-lifecycle-wretubs3` (X11). Private build/quality logs:
`/workspace/pivra-environment/mixed-stock`. These are execution artifacts, not
tracked binaries or packaging inputs. No entire upstream GTK suite was run.
