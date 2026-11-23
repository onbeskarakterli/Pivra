# Upstream handoff: existing stock Wayland EGL ownership fix

Prepared 2026-11-23 from Pivra `dev` diagnostic commit
`c4ca4d85a05fa81efeddc6f5229d28040c56d787`. **Not submitted upstream.**
Reproducible report title: **Wayland: retiring a GStreamer media sink invalidates
GL contexts belonging to other live windows**. A new duplicate issue/MR is not
recommended: the fix already exists upstream.

## Existing upstream fix, independently verified during this task

GTK commit [95b7ec19b01c460c46d58ec52cd0c4eb2d730395](https://gitlab.gnome.org/GNOME/gtk/-/commit/95b7ec19b01c460c46d58ec52cd0c4eb2d730395),
**gstreamer: mark EGL display as foreign**, adds the cached-wrapper lookup,
foreign setter and temporary unref to the Wayland stock sink. The original
commit message says the EGL display is owned by GTK and references
[issue #8456](https://gitlab.gnome.org/GNOME/gtk/-/issues/8456) and
[MR !10582](https://gitlab.gnome.org/GNOME/gtk/-/merge_requests/10582).
This is existing upstream work, not a Pivra submission or a claim of authorship.

Read-only Git/source checks on 2026-11-23 established:

| Upstream reference | Exact source commit / result |
| --- | --- |
| gtk-4-24 HEAD | `75f53888db1cdab9cbe11b14cf3cef55a4ed307f`; fix present |
| main HEAD | `4b117a8daad9bc54ff6d11e130561d744b6c8056`; fix present |
| Both current stock sink blobs | SHA-256 `3f7457b3fbaba9f70132eb4cd9f7c2b4a811c387e308cbc7800465c398ec7d16` |
| Commit labelled `4.24.2` in branch history | `2938ffc815387baf7c7894dfe6e345fa35761eaf`; verified descendant of fix |
| Published `4.24.2` tag / Arch fixed package | NOT VERIFIED; exact tag query returned no match; no installed-package test |
| gtk-4-24 HEAD gdk/gdkdisplay.c | SHA-256 `1dfc7c6da4cff4883e7835e430501426eae196229eceed6e6a3de0e950f72f67`, identical to original 4.24.1 selection source; the separate config patch is still absent there |

This narrows the next upstream action to confirming/distributing the existing
fix or requesting its packaging/backport, with the reproduction/evidence below.
Do not claim that a source commit labelled 4.24.2 proves a published tag/package,
or that the official five-line fix was the exact native-tested local patch.
Do not apply the local ownership patch on top of a source already carrying the
upstream fix. Prefer the original upstream commit for a distributor backport.
The separate EGL selection correction still needs its own review and native
context validation. Neither fix is installed by this task.

## Affected environment and evidence provenance

| Component | Native owner-reported environment |
| --- | --- |
| Desktop / backend | Arch Linux x86_64, GNOME Wayland |
| GPU | AMD Radeon RX 7900 XT |
| GTK | 4.24.1 |
| Mesa | 26.3.1-arch1.2 |
| GStreamer | 1.28.9 in the preceding native context report/trace |
| Latest native summary | Stock-ownership Wayland 8/9; ownership-patched Wayland 9/9; X11 9/9; normal production GPU/audio/color/seek PASS |

These are user-reported physical results. The newest summary supplies no raw
report, exact tested Pivra SHA, GTK library hashes, GStreamer package revision or
GNOME/kernel versions. It follows the diagnostic commit above, but that does not
prove the exact checkout or loaded library. Attach those fields before filing;
do not copy Codex build hashes into the native run's provenance.

The existing A/B runner applies the **same EGL config-selection patch to both
variants**, then changes only `gtk/media/gtkgstsink.c` ownership. Its control is
stock **sink ownership**, not a completely pristine GTK tree. The owner's short
label "stock GTK" does not establish a pristine baseline. This matters because
the separate implicit GLES1 config-selection problem can prevent context creation
before this ownership issue is exercised. Keep the two GTK fixes separate.

## Expected and observed behavior

Two independent GPU video windows are active. In one window, navigate from
BT.709 SDR to sRGB/unknown-metadata stock GtkMediaFile playback, then back to
BT.709 SDR. Retiring a stock media stream must release its own pipeline and GL
contexts without invalidating GDK's display or the other live window.

Observed with original stock sink ownership: four
`gdk_gl_context_make_current() failed` warnings, later GPU context creation
failure and stock fallback. Retained native GDB evidence records
`eglTerminate(0x55d875099f70)` returning 1 in the stock teardown stack through
libgstgl → GTK/media/plugin unrefs, while both Pivra GPU workers used that display.
The following context destruction returns 0; 38 subsequent context creations fail
on the same display. The inferior exits 101. This is an application exit, not an
intentional debugger stop. The trace's exact original application SHA is unknown.

Retained trace SHA-256:

- Main log: `6683ec6fba6014da4bb18d3af5d8882c2f87a5026bc6881d7a12dfd2fe5b2b8c`.
- 117-event GDB sidecar: `d3a62ee7be27a7cb04a2fa1b72a7cccf0fb3b2dac0631162f14c0732c27795af`.

## Source-level ownership contract and preserved local review patch

Upstream commit subject: **gstreamer: mark EGL display as foreign**. The local
review patch below is retained as the already native-tested diagnostic candidate.

GTK's Wayland stock sink wraps GDK's `wl_display` using
`gst_gl_display_wayland_new_with_display()`. The derived EGL display returned by
`gst_gl_display_egl_from_gl_display()` is cached and nonforeign by default.
GStreamer's public documentation explicitly requires marking it foreign when
another API owns the EGLDisplay. Its finalizer otherwise calls `eglTerminate()`.
Borrowing the native Wayland connection alone does not borrow EGL ownership.

The fix obtains the cached EGL wrapper before creating wrapped/worker contexts,
checks that it refers to GDK's actual EGLDisplay, marks it foreign using public
`gst_gl_display_egl_set_foreign(..., TRUE)`, and releases the temporary reference.
Retain the Wayland-native GstGLDisplay type; the Wayland connection, GDK display,
window system integration and X11 paths stay unchanged. Add the EGL header for
Wayland-only builds as well. Missing/mismatched handles fail initialization.
The public setter is available since GStreamer 1.26; GTK 4.24.1 already requires
GStreamer >=1.28. No private struct writes, symbol interception or global hooks.
GDK remains responsible for terminating its display after dependent contexts die.

Apply the already reviewed attachment, unchanged:
[gtk-4.24.1-stock-egl-ownership.patch](../performance/gtk-4.24.1-stock-egl-ownership.patch).
It changes only `gtk/media/gtkgstsink.c`; no Pivra dependency or shader is involved.
Its ownership mechanism matches upstream. It additionally validates the native
EGL handle/failure path and includes the EGL header for Wayland-only builds;
upstream uses an assertion on its wrapper lookup. They are not byte-identical
patches. Upstream's current X11/Wayland header guard was not changed by the
five-line commit; a Wayland-only build remains a review item.

| Pinned input | SHA-256 |
| --- | --- |
| Official GTK 4.24.1 GitHub archive | `6b03cc3eac17e6d7b252c7ac96559e58aa7c990874ca47da696d81d0add7521d` |
| Original stock sink | `5a5591e2e72b2b93f49174cd25e3e975a9022d73e9e84990af93eb47ea6a9f04` |
| Ownership-patched stock sink | `5aaf601a1dd892ba08f34832ff9751eecc4ca5c0304efd14f025f61c11fa1f3c` |
| Ownership patch attachment | `e4b4e994853636c91be489a58782eba3e55b2361dbe314115b5949290d728811` |
| Separate, unchanged EGL config patch | `61c36ea800954e2728c3414f0f126bc0c4025fb7d231ce9f2560c517bc2773f9` |

## Reproduction without replacing system GTK

From Pivra `dev`, with GTK 4.24.1 build prerequisites, GStreamer >=1.28
headers/runtime/MP4/H.264 plugins, Meson/Ninja, C compiler, Rust/Cargo and FFmpeg:

```sh
GDK_BACKEND=wayland bash tests/run_gtk_egl_isolated.sh --stock-lifecycle --workspace "$HOME/.cache/pivra-gtk-stock-ownership"
```

The workspace must be new or already managed by this runner. No `meson install`,
system replacement, GTK fork packaging or dependency installation occurs. Both
private GTK builds enable the real **built-in** GStreamer backend; an old media
module filename or GtkNoMediaFile is not accepted. The same application/adapter
binary is run with each actual runtime GTK library. Source/build/library hashes,
ldd, individual logs, signal exits and all nine gate results are retained in the
printed directories. Exit 0 requires all nine candidate gates PASS; control
failures are retained, not hidden. No software permission is set by this command.

For a smaller ownership-mechanism reproducer (no video decoding), use the
standalone C file [stock_egl_retirement.c](../../tests/video_gpu/stock_egl_retirement.c):

```sh
cc -O2 -Wall -Wextra -Werror -DGST_USE_UNSTABLE_API \
  $(pkg-config --cflags gtk4 gstreamer-gl-1.0 epoxy) \
  tests/video_gpu/stock_egl_retirement.c -o /tmp/pivra-egl-retirement \
  $(pkg-config --libs gtk4 gstreamer-gl-1.0 epoxy)
GDK_BACKEND=wayland /tmp/pivra-egl-retirement legacy
GDK_BACKEND=wayland /tmp/pivra-egl-retirement borrowed-native
```

Use fresh processes. GDK must already be able to create a context (the separate
EGL selection defect may require the isolated config-fixed build). `legacy`
intentionally reproduces invalidation of two real GDK contexts, reports
MECHANISM_REPRODUCED, then exits without unsafe GL cleanup on the invalid display.
`borrowed-native` retains the native Wayland wrapper and cached foreign EGL
wrapper; both context queries, make-current operations and normal unrefs must
succeed. This mechanism control alone is not a GTK media integration PASS.

## Comparison and review limitations

| Environment / variant | Result | Evidence type |
| --- | --- | --- |
| Native RX 7900 XT, original stock ownership, Wayland | 8/9; mixed fails | Owner report |
| Native RX 7900 XT, ownership-fixed Wayland | 9/9 | Owner report; fresh raw provenance still requested for submission |
| Native RX 7900 XT, X11 | 9/9 | Owner report |
| Codex GTK 4.24.1 / Gst 1.28.9 / Mesa 25.0.7 llvmpipe, Wayland control | 8/9; four warnings, mixed exit 101 | Independently executed at diagnostic commit |
| Same Codex stack, Wayland candidate | 8/9; mixed SIGSEGV/-11, no context-failure warnings | Independently executed; **OPEN**, not replaced by later native PASS |
| Codex X11, both variants | 9/9 each | Independently executed |
| Real two-context C policy control | Legacy invalidates both; foreign policy preserves both | Independently executed |

The candidate's earlier software SIGSEGV is not attributed or cleared. Include
[this investigation](../performance/V026_CANDIDATE_SIGSEGV.md) with any upstream
submission. No failing stack was captured; five GDB executions exited normally.
The existing unclassified GTK DMA-BUF SIGBUS baseline is a different signal/test,
not an explanation of this SIGSEGV. Intel/NVIDIA and Wayland-only build-matrix
validation are still NOT RUN. No entire GTK upstream suite was rerun here.

Before submission, attach native A/B report.json, both logs, source/library
hashes, Meson options, `pkg-config --modversion gtk4 gstreamer-gl-1.0`,
`pacman -Q gtk4 mesa gstreamer gst-plugins-base-libs`, and actual GL renderer/API.
The current upstream source check above supersedes the earlier plan to propose
a new ownership MR. An existing fix is merged in the inspected refs; no new
issue, comment or MR was posted by this task. Attach the Pivra native findings to
the existing issue/MR only after authorization and after obtaining the missing
native provenance. Fixed distribution-package availability and its physical
acceptance remain separate from the upstream source result.
