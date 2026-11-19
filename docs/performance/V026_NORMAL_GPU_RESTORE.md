# Normal GPU playback restoration candidate — 2026-11-20

**Native AMD normal playback: PENDING. Mixed native lifecycle: BLOCKED. No release preparation or readiness claim.**

Implementation: `e416839efa094fe1ae9057c61af0c60511e55525`.
Starting development source: `d9d2fdadd2cbbe03b9399024202e5f96c75c83ff`.
Released v0.2.5 comparison: `726f6948cb391e620a4733348ab164ad3a7ea295`.
[Machine-readable evidence](V026_NORMAL_GPU_RESTORE.json).

## What the comparison establishes

The owner reports that v0.2.5 normal GPU playback passed on an AMD RX 7900 XT,
current stock playback works, and current GPU acceptance reports no shared GL
context. The prior native trace also proves stock GTK/GStreamer terminates an
EGLDisplay shared with Pivra during mixed playback. These are distinct observations:
that trace does not prove why a first GPU video fails in a fresh process.

Comparing the two immutable source trees shows these relevant changes:

- v0.2.5 used `GstGLDisplayWayland` for GStreamer's native window/context
  integration. Development changed this to a direct `GstGLDisplayEGL` wrapper
  to prevent termination of GTK's display. The ownership fix changed the native
  display type as well as ownership.
- Development added current-context guards/restoration, parent-filter `gl_stop`,
  context unrefs, bounded asynchronous retirement, and GTK-thread presentation
  cleanup. Reverting those would reintroduce resource/lifecycle problems.
- Transfer-function math, eligibility and playback control semantics were not
  changed by those lifecycle fixes. This patch does not change them either.

The candidate restores the **native Wayland display type** used by v0.2.5, while
keeping GTK's EGLDisplay explicitly borrowed. GStreamer's public
`gst_gl_display_egl_from_gl_display()` returns the cached derived EGL wrapper;
`gst_gl_display_egl_set_foreign(TRUE)` marks it borrowed before worker creation.
The native EGL handle must match GTK's actual handle. No private production
structure writes or retained obsolete players are used.

The foreign setter is available since GStreamer 1.26. A Linux ELF weak reference
preserves the existing >=1.24 contract: without that symbol, the previous safe,
direct borrowed-EGL wrapper remains in use. A missing-symbol control exercises
that branch, and the release ELF records an undefined **WEAK**, not mandatory,
setter symbol. An actual GStreamer 1.24 installation was **NOT RUN**.

Creation/realization errors now include their actual GDK error text in
`PIVRA_GPU_CONTEXT_FAILURE`, before the generic no-shared-context error. Context
owner/release diagnostics retain actual EGLDisplay handles and add display type.
All newer context guards, cleanup, shader math and stock routing remain.

This is a bounded restoration candidate, **not a demonstrated AMD FAIL-to-PASS**.
All three sources open normal video in this software-GL environment. Native
failure attribution remains uncertain until the fresh-process comparison is run
on the affected machine. An initial stale-GDK-pointer hypothesis was rejected:
real public-EGL unbinding plus GDK lookup normalizes the pointer to NULL on both
baseline and development. No restoration guard was weakened on that hypothesis.

## Controlled results actually executed

Environment: Debian 13 SDK, Rust/Cargo 1.99.1, GTK 4.18.6, GStreamer 1.26.2,
Xvfb/X11 and headless Sway/Wayland, Mesa **llvmpipe**. No AMD hardware, GTK 4.24.1
native run, physical audio or MPV visual comparison was performed here.

| Gate | X11/Xvfb | Sway/Wayland |
| --- | --- | --- |
| Fresh-process normal comparison (v0.2.5, old dev, candidate) and initializer controls | PASS, 8/8 | PASS, 8/8 |
| Separate existing mixed lifecycle suite | PASS, 9/9 | PASS, 9/9 |
| Native AMD first-video playback | NOT RUN | NOT RUN |
| Previously failing native AMD mixed lifecycle | NOT RUN; remains BLOCKED | NOT RUN; remains BLOCKED |

Normal comparison checks actual GPU backend selection, GLMemory-to-GLMemory
shader rendering, retained sync metadata, static BT.709 ramp output against the
independent BT.1886 reference (2 LSB tolerance), play/pause/loop/volume/mute/seek,
and production window creation/close with context/filter/frame/presentation and
worker counters drained. Software GL is a shader/lifecycle control, not hardware
acceptance. The baseline and pre-fix trees are exported with `git archive` into
private temporary directories; no branch checkout or changes to tracked files.

The Wayland initializer reports the following actual types and ownership:

| Source/control | Display type | Borrowed derived EGLDisplay |
| --- | --- | --- |
| Released v0.2.5 | GstGLDisplayWayland | false |
| Pre-fix d9d2fda | GstGLDisplayEGL | true |
| Candidate | GstGLDisplayWayland | true |
| Candidate with 1.26 setter symbol deliberately unavailable | GstGLDisplayEGL | true |

The control also verifies exact native-handle identity, incoming-unbound
initialization, and that the original GDK context can be rebound after teardown.
The legacy ownership observation is retained as evidence, not adopted again.

| Other executed check | Result |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo check --locked` | PASS |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS, zero warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 217 tests |
| Python root / thumbnail / FD05A / video color / video GPU | PASS, 69 / 59 / 9 / 13 / 69 = 219 |
| Native C initializer, ownership, shader and lifecycle controls | PASS in the above software sessions |
| `cargo build --release --locked` | PASS |
| Shell syntax and `git diff --check` | PASS |

The real GDB tests ran with the existing GDB 16.3 setup. The final GPU Python
suite includes negative checks that stock playback/exit zero/missing GPU markers
cannot pass normal GPU acceptance. Historical pre-fix FAIL results are preserved
as observations without requiring the old bug to pass; baseline and candidate
failures still fail the command. Manual product windows also require actual GPU
backend/draw/sync evidence, not just successful exit.

## Remaining native boundary and preserved scope

The stock GTK private sink still has the separately documented EGL ownership
problem: see [native acceptance fixes](V026_NATIVE_ACCEPTANCE_FIXES.md) and the
existing upstream review patch. This candidate corrects Pivra's own integration,
not stock GTK's private wrapper. No system GTK modifications, global overrides,
CPU color conversion, arbitrary sleeps, deadline increases, warning suppression,
or alternate stock/HDR routing are introduced. Prior native failures are open.

Thumbnail scheduling, cache/preload budgets, image decode security and all
v0.2.6 RAM optimizations are unchanged. No version, dependencies, packaging,
main, release tag, AUR or release-source configuration changes are made. The
accepted historical preload-budget observation remains open. Hardware decoding,
VRAM usage, end-to-end zero-copy and Intel/NVIDIA support are not established by
these results.

## One native AMD command

In the normal Arch/GNOME session on updated dev, with backend/test overrides unset:

```sh
bash tests/run_gpu_playback_probe.sh --video "/absolute/path/to/BT709-video.mp4" --manual
```

Use the previously accepted original video. The command builds and compares
immutable v0.2.5, pre-fix dev and current dev in separate processes, then opens
the actual v0.2.5 and current Pivra windows sequentially. Check normal colors
against MPV, audio, playback and seek; close each window to continue. Avoid
mixed stock/GPU navigation in this isolated normal-playback check. Keep
`report.json` and logs from the printed temporary report directory. It records
executable SHA-256 values, product versions and source references. Automated
process success leaves visual acceptance pending the user's explicit report.

The existing `bash tests/run_video_context_lifecycle.sh` remains a **separate**
mixed/native gate, requiring all nine tests without warnings or GPU errors.
Normal playback success cannot close that gate. No further release preparation
is authorized by these controlled results.
