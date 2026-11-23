# Native slideshow SIGSEGV — EOS investigation

2026-11-23. **OPEN / UNCLASSIFIED; native acceptance BLOCKED. No SIGSEGV fix claimed.**
Reviewed feature source: `d041378a087a6a129e1acf249421e05b1d2c193b`, on
`feature/v0.2.7-slideshow`. The owner reports exit 139 after secure rejection of
intentionally corrupt `3.png` and the actual `video-ended` event for `4.mp4`.
The reported `/tmp/pivra-slideshow.F67XQIpR/` directory, probe log and native
core are not present in this Codex filesystem. No native stack, faulting thread,
library provenance or source SHA accompanying that crash has been inspected.

Corrupt-image rejection is an expected test step, not evidence of the SIGSEGV
cause. An EOS log narrows the interval; it does not locate the crashing callback.
Neither the existing GTK EGL defect nor a new Pivra lifetime bug is established.

## Ownership audit

- `slideshow_window.rs::arm/poll`: EOS is observed by a main-context, weak-Rc,
  chained one-shot GLib source, **not a navigation call from an ended signal
  callback**. The source ID is consumed before polling; stop removes any queued
  source. The Session holds only a weak MediaFile reference. Its temporary strong
  observation reference is dropped before navigation and `stop_video()`.
- `ImageView::stop_video`: controls detach, the video Picture is hidden and its
  paintable detached, the error handler is disconnected, then the stream is
  paused/unrealized/dropped. Owned GPU streams additionally clear their pipeline;
  stock lifetime remains GTK's existing asynchronous policy. No change here.
- The playback-controls ended handler retains weak current-stream/UI references
  and ignores retired streams and slideshow repeat. The playing handler retains
  only an intent Cell; the prepared seek handler captures a timestamp, not a
  window. ActualPaintable's media invalidation handlers use weak paintables.
- The slideshow photo fade never captures a video frame: readiness plus absence
  of a current video stream are required. Stop/close clear its frame-clock tick
  and paintable. The existing load-generation checks continue rejecting stale
  completions; window close also invalidates loads and stops video.

This review finds no demonstrated new dangling callback. It is not a proof of
GStreamer/GDK/driver safety. Stock stream finalization and zero owned-GPU counters
do not prove that every native render operation or display reference has ended.

## Justified changes

**Confirmed probe synchronization gap:** its `video EOS loops to first photo`
barrier previously checked only the selection title. That title changes before
decode and render completion. It then immediately paused and navigated to/from
video again. The probe now separately waits for photo content with no video or
loading overlay, uses the existing persistent WidgetPaintable/after-paint frame
barrier, and submits its real render node through the window renderer. No fake
node, removed assertion, sleep or raised timeout is used. Original rapid manual
video/photo/video navigation remains, after an explicit stage marker.

This fixes what the test certifies; it does **not** establish that its old timing
caused the native SIGSEGV or justify changing production teardown.

New `--eos-regression` exercises three fresh sessions: select corrupt photo,
reject/skip it, play the five-second video through actual EOS, wrap to photo,
pause and submit a photo frame. It checks the independent second window,
owned filter/context/frame/presentation/worker/pending counters returning to zero,
repeat preference restoration and independent close. Starting a new session
before each corrupt selection preserves the normal bounded failed-item policy.
Controlled photo pixels are explicitly labelled; native mode uses secure Glycin.

Opt-in Session diagnostics now identify EOS stream pointer/type/prepared/seekable/
duration, temporary-reference release, display-selected entry/return and stop
entry/completion. They introduce no callbacks, extra timers, ownership policy,
GL queries, shader or renderer changes. Probe stages and loaded-library paths
identify which boundary was reached, without high-volume frame logging.
Final diagnostic timestamps use GLib's monotonic clock, matching existing native
lifecycle timestamps; the slideshow scheduling clock is unchanged.

The native runner's `--gdb` reuses the existing read-only EGL observer, with
auto-continuing observer breakpoints, signal/thread stacks and verified inferior
results. A missing inferior result is BLOCKED, a signal is FAIL even after success
markers, and exit 77 is NOT RUN. The runner retains logs, binary SHA-256, GDB JSONL,
result JSON and hashes of observed library files (or an explicit read error).
Successful GDB artifacts are also retained. It does not alter system core policy
or install/replace GTK. Error skips cannot count as EOS: each complete focused
cycle must contain exactly one actual EOS between video selection and photo
presentation. Shader/context failures remain failures even with application exit 0.

## Executed checks and evidence

Existing SDK: Rust 1.99.1, GTK 4.18.6/GStreamer 1.26.2, llvmpipe, Xvfb+xfwm4 and
headless Sway. Not physical Arch/GNOME or AMD validation.

| Check | Result |
|---|---|
| Original controlled slideshow before these changes | PASS, X11 and Sway; native SIGSEGV not reproduced |
| Three-cycle EOS/corrupt/photo focused gate | PASS, X11 and Sway |
| Updated full slideshow including original rapid-navigation case | PASS, X11 and Sway |
| Rustfmt / locked check / strict all-target Clippy | PASS, zero warnings |
| Locked Rust tests | 226 PASS, 0 failures |
| Python suites | 253 PASS: root 80, thumbnail 59, fd05a 9, video_color 13, video_gpu 92 |
| Native-mode GDB prerequisite in Codex | NOT RUN / 77: secure loader failed under existing sandbox; GDB independently confirmed inferior exit 77 |
| Native Arch SIGSEGV stack/core analysis | NOT RUN: reported artifacts not available here |
| Physical GNOME/AMD and real secure EOS regression | NOT RUN; owner retest required |

The eleven new Python cases cover genuine complete cycles, missing EOS, title-only
readiness, missing middle-cycle EOS, incomplete cleanup, signal after success
markers, absent debugger evidence, verified debugger signal/exit, exit 77, and
all existing shader/context failure markers. No existing assertions were relaxed.

The controlled Sway cycle log independently associates all three EOS pointers
with `stock-stream-retired` markers before photo presentation (stock_opened 1/2/3,
gpu_opened 0). This supports correct observed GObject retirement in that runtime;
it is not a claim of hardware GPU success or proof of completed driver teardown.

### Additional narrow GTK 4.24.1 control (not an EOS PASS)

The existing private GTK 4.24.1 EGL-config-only control and ownership-patched
candidate, GStreamer 1.28.9 and llvmpipe/Sway were tested with the **same** probe
and one focused case each. No GTK rebuild or upstream suite was run. Both reached
a missing-playback-plug-in error, so the actual EOS comparison was **NOT RUN**:

- Control: raw exit 101, `gdk_gl_context_make_current() failed` after stock stream
  retirement, then the existing `render_frame.rs` after-paint assertion failed
  (mapped/visible/content true, 1280×1024, mode fit).
- Candidate: raw exit 0 and rendered photos, but **the outer gate returned FAIL**
  because no actual video EOS occurred. Its apparent inner PASS is insufficient.

These failures are preserved rather than relabelled PASS. In this private setup,
`gst-inspect avdec_h264` can find the 1.26.2 SDK libav plug-in but the 1.28.9
GtkGstMediaFileBuiltin pipeline still reports no suitable plug-in. That is an
environment limitation/confounder; its underlying cause was not established.
The control shows a GL invalidation boundary on stock error teardown, consistent
with the older ownership investigation. It **does not reproduce the native EOS
SIGSEGV**, identify its faulting instruction, or clear Pivra/GTK/the patches.

Local ephemeral logs:
`/tmp/pivra-slideshow-eos-{ordinary,focused}-{x11,sway}-final.log`,
`/tmp/pivra-slideshow-eos-{fmt,check-final,clippy-final,rust}.log`,
`/tmp/pivra-slideshow-eos-python-*.log`,
`/tmp/pivra-slideshow-eos-native-gdb.log`, and
`/tmp/pivra-slideshow-eos-gtk24.log`. The latter retains the private control and
candidate directories. The secure prerequisite GDB artifacts are at
`/tmp/pivra-slideshow.Nuf7Xkah/` (one invocation, no sandbox bypass/retry).

## Native next step

From this feature branch, on the same Arch/GNOME runtime as the failure:

```bash
bash tests/run_slideshow_probe.sh --eos-regression --gdb
```

Requires GDB with Python plus the existing native build/media prerequisites.
Use the same existing isolated GTK launcher/environment if that was used for the
failed run; do not install GTK or silently change backend. The runner prints the
retained artifact directory. On SIGSEGV it captures all-thread stacks, EGL handle
observations and the real signal result. Share the full directory alongside the
original `F67XQIpR/probe.log` and any original core/backtrace if available.

If the focused gate passes, also rerun `bash tests/run_slideshow_probe.sh --gdb`
for the complete original rapid-navigation/close scenario. A debugger can change
scheduling, so normal `bash tests/run_slideshow_probe.sh` must pass too before
acceptance. No automatic repetition is added. A later successful run alone does
not retrospectively identify the original crash cause.

Until native evidence supports a specific correction, production GPU, stock
teardown and GTK ownership code remain untouched. v0.2.7 slideshow stays TESTING;
the SIGSEGV remains OPEN and native acceptance is BLOCKED.
