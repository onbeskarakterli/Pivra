# GTK 4.24.1 implicit GLES 1 filter — isolated correction, 2026-11-21

Follow-up native user report: corrected GTK context/GLMemory/frame/production
gates PASS on RX 7900 XT, and real-video raw/wrapper max channel difference is
0. A control-preparation assertion then failed; see the
[test barrier follow-up](V026_VIDEO_CONTROL_PREPARATION.md). Native control
acceptance remains pending; the historical results below are preserved.

**Root cause: CONFIRMED by user-reported native API/config comparison and
upstream source. GTK correction: source-unit, build and software-runtime
validated. RX 7900 XT Wayland playback after correction: PENDING. Release BLOCKED.**

Implementation/test tools: `a5639ff88839737f284f6606391f3a79731f407d`.
Starting dev: `4413778903aad17b4a006f2d186a2f60446280fa`.
[Machine-readable evidence](V026_GTK_EGL_ISOLATED_FIX.json).

## Native evidence and cause

The owner reports on actual Arch/GNOME Wayland, GTK 4.24.1, Mesa 26.3.1,
RX 7900 XT:

| Same SDR/window/buffer requirements, API selector | Native configurations |
| --- | --- |
| GTK implicit `EGL_RENDERABLE_TYPE` | 0 |
| Explicit `EGL_OPENGL_ES_BIT` (GLES 1) | 0 |
| Explicit `EGL_OPENGL_ES2_BIT` | 40 |
| Explicit `EGL_OPENGL_BIT` | 40 |
| Explicit `EGL_DONT_CARE` | 40 |

EGL reports `EGL_SUCCESS`. These are **user-reported hardware results**, not
independent Codex measurements. Unreported size/alpha/HDR control counts are not
invented. The same Pivra/video already has user-reported X11/XWayland hardware
acceptance for GPU/GLMemory, BT.1886 colors, smooth video and audio.

Official GTK 4.24.1 `gdk_display_create_egl_config()` requests window RGB buffers,
RGB minima 8/8/8 and alpha minimum 8 for SDR. It omits `EGL_RENDERABLE_TYPE`.
The EGL default is `EGL_OPENGL_ES_BIT`, requiring GLES **1** compatibility.
This filters out the native modern-only configurations, although desktop GL
and GLES configurations are available. This is an unintended GTK selection
requirement; the observations do not establish a Mesa defect.

Wayland must initialize EGL through this path before
`gdk_surface_create_gl_context()` can construct Pivra's context. GTK's cached
display error then explains the renderer warning and `create-gdk-context`
failure. Pivra's Gst display wrapper has not run at that stage. X11 has a
separate EGL/GLX selection route, so its success does not contradict this cause.

## Separate upstream-compatible correction

[GTK patch](gtk-4.24.1-modern-egl-config-review.patch), deliberately separate
from production Pivra:

1. Explicitly specify `EGL_RENDERABLE_TYPE=EGL_DONT_CARE` when gathering candidates.
2. Before existing scoring, reject configurations with neither desktop OpenGL
   nor GLES 3 support. A bitmask requiring both APIs would incorrectly reject
   valid desktop-only or GLES3-only drivers; the tests cover both independently.
3. Preserve window/RGB/alpha/depth requirements, scoring and context version/API
   validation. ES1-only, ES2-only and OpenVG-only candidates cannot satisfy GTK's
   modern-context requirement.

The unrelated count-only config_size normalization was removed from the earlier
review patch. It remains GTK's original value; the evidence identifies the API
filter. No driver preference, global environment override, EGL dispatch
interposition or private-object mutation is added to production Pivra.

**Only GTK source changes.** All Pivra production sources, BT.1886 shader,
metadata routing, stock recovery, RAM/cache/preload/thumbnail policies,
Cargo dependencies/lock, packaging, README/version and security code are
unchanged. No patched GTK is bundled, installed by the test helper, or loaded
by default. An upstream GTK fix or an explicit distribution backport is the
production dependency; this commit does not pretend to deliver it to users.

## Source and build provenance

- Source: `https://codeload.github.com/GNOME/gtk/tar.gz/refs/tags/4.24.1`
- Actual archive SHA-256:
  `6b03cc3eac17e6d7b252c7ac96559e58aa7c990874ca47da696d81d0add7521d`
- Clean `gdk/gdkdisplay.c`:
  `1dfc7c6da4cff4883e7835e430501426eae196229eceed6e6a3de0e950f72f67`
- Patched `gdk/gdkdisplay.c`:
  `bad9c80634ab2112c9d361708a78747f01058c6e7c6cf96a6ad966353b0fba10`

Both complete GTK 4.24.1 libraries were built in private workspace directories.
Meson options were the same apart from the unused private installation prefix:
X11/Wayland enabled, Vulkan disabled, media-gstreamer auto, tests/testsuite/
examples/demos/introspection/documentation/accesskit/CUPS disabled, debugoptimized.
No `meson install` was run for GTK; no system GTK was replaced.

The existing SDK has GTK 4.18.6/GStreamer 1.26.2. To satisfy real GTK 4.24.1
requirements without changing it, private dependencies were prepared under
`/workspace/pivra-environment/egl-isolated`: GLib 2.90.0 (including its pinned
GVDB source), Pango 1.58.0, Fontconfig 2.17.1, Wayland 1.24.0 and protocols 1.48.
Meson/Ninja and checksum-verified Debian gperf/SassC/libsass build tools were
also private. SDK `.pc` relocation copies and the multiarch ffi include path
were confined to this workspace. No package installation, ldconfig, profile
edit, security change or version-requirement bypass occurred.

GStreamer 1.26 does not meet GTK 4.24.1's 1.28 media-module requirement, so those
two private GTK builds have **no stock media module**. Normal Pivra's owned GPU
backend still uses the existing GStreamer SDK. Stock fallback was separately
tested with the unchanged GTK 4.18.6 SDK; fallback inside the private 4.24.1
build is **NOT RUN**, not PASS. Native Arch with matching GStreamer 1.28 headers
may build that module; the helper records whether it actually exists.

## Independent tests actually executed

### Exact GTK selection function: 16 source-level controls

`gtk_egl_source_test.py` extracts the verbatim selection function from the
checksum-verified clean and patched sources and compiles each against a bounded
configuration-capability fixture. This is a **source unit**, not a real driver
or fake GPU/frame result.

Each variant passes eight expected-behavior cases: modern-only configs,
desktop-only GL, GLES3-only, ES2-only, OpenVG-only, an existing ES1-compatible
modern config, ES1-only, and an actual query-error return. The unmodified
function fails to choose from modern-only fixtures; the patched function
selects eligible GL/GLES3 while rejecting lower/irrelevant APIs. Exact SDR
buffer/alpha requirements are asserted, and real query errors stay failures.

### Real GTK/Pivra runtime: 12 controlled checks

Environment: Debian 13 SDK, Mesa 25.0.7 llvmpipe, OpenGL ES 3.2, actual privately
built GTK **4.24.1**, private GLib/Pango dependencies and GStreamer 1.26.2.
This is not the owner's AMD/Mesa 26.3.1 environment.

For both baseline and patched libraries, on Sway/Wayland and X11/Xvfb:

- Actual GTK display preparation, native window, GL renderer and current GL context: PASS.
- Existing GPU video color/control probe: PASS, GLMemory-to-GLMemory BT.1886,
  retained synchronization, all 256 ramp levels within the existing 2 LSB
  reference tolerance, play/pause/seek/loop/volume/mute state.
- Existing full production window's normal GPU open/frame/close and owned
  resource drain: PASS.

All 12 cases PASS. Clean software GTK also passes because this driver exposes
ES1-compatible configs. **The native failure is not reproduced by llvmpipe**;
the source-level modern-only controls establish the decision change, not a
fabricated hardware FAIL-to-PASS.

The runner validates source hashes, existing renderer/color source equality,
the private library location and absence of the historical GTK BT.1886 color
patch. It preserves ldd output and validates actual `GTK_RUNTIME 4.24.1` and
`GTK_LOADED` maps. Clean/patched library SHA-256 values and Meson build options
are retained in the evidence. Only the application-owned shader corrects video.
Test-only snapshot download checks pixels; no production CPU transfer path is
introduced.

### Pivra regressions on the unchanged SDK

| Check | Result |
| --- | --- |
| `cargo fmt --check` / `cargo check --locked` | PASS |
| Strict all-target Clippy, `-D warnings` | PASS, zero warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 217 tests |
| Python root / thumbnail / FD05A / video color / GPU | PASS, 69 / 59 / 9 / 13 / 86 = 236 |
| Existing production initialization/stock recovery suite, X11 and Sway | PASS, 9/9 each, software GL |
| `cargo build --release --locked` | PASS |
| C source/window probes `-Wall -Wextra -Werror`, shell syntax and diff checks | PASS |
| Native RX 7900 XT corrected Wayland visual/audio/control acceptance | NOT RUN in Codex; PENDING |

No complete GTK upstream suite was run. The historical DMA-BUF download SIGBUS,
mixed stock EGLDisplay termination and preload-budget observations remain open;
this EGL-selection patch does not claim to resolve those separate issues.

## One native Arch acceptance command

From updated dev, use the original eligible BT.709 video and a new private
workspace. Meson >=1.8, Ninja, C compiler, pkg-config, Rust/Cargo and the GTK
4.24.1 build dependencies must already be available. The helper installs none.

```sh
bash tests/run_gtk_egl_isolated.sh --workspace "$HOME/.cache/pivra-gtk-egl-config-4.24.1" --video "/absolute/path/to/BT709.mp4" --manual
```

The helper verifies the source archive, prepares clean/patched trees, builds
only GTK/media-library targets, and selects libraries **only for explicit test
subprocesses**. It does not change normal Pivra startup, system GTK or packaging.
An unmanaged nonempty workspace or unreviewed source is refused. The automated
preparation uses the Meson commands validated here; a full fresh native Arch
invocation itself was NOT RUN in Codex. Compilation/prerequisite errors retain
their logs rather than being treated as runtime success.

The runtime comparison records clean failures as baseline evidence. Patched
Wayland and X11 checks must pass; stock playback cannot satisfy GPU gates. The
manual patched window uses normal automatic video admission. Compare with MPV
and confirm RX 7900 XT, GLMemory, colors, smoothness, sound and all controls,
then close the window. Process exit alone leaves physical acceptance PENDING.
Keep both build logs and the printed runtime `report.json`/logs.

Even successful private-library testing is **not** approval to ship a GTK fork
or publish Pivra. Native physical acceptance, upstream/backport delivery and
explicit release approval remain separate gates. Vulkan/HDR visual behavior,
Intel/NVIDIA, hardware decode and end-to-end zero-copy are not established.
