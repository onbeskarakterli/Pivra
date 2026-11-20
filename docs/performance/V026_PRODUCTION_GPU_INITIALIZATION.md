# Production GPU initialization and stock recovery — 2026-11-20

Follow-up native evidence: X11/XWayland GPU playback PASS on RX 7900 XT,
while native Wayland fails inside GTK's EGL configuration query before the
owned Gst sink. See [the Wayland EGL boundary report](V026_WAYLAND_EGL_CONFIG.md).
The application surface/fallback work below does not fix that private GTK query.

**v0.2.6 publication: BLOCKED pending actual AMD GPU playback or safe stock
recovery acceptance.** Implementation: `0923a6f4fc0395f93e97c56749ac7c9dd08ab602`.
Base: `fc3547e6eef94e2a47c4daf9a5b3780d8f9e530a`.
[Machine-readable results](V026_PRODUCTION_GPU_INITIALIZATION.json).

## Evidence and limits of attribution

The owner now reports both installed v0.2.5 and current development failing
automatic GPU playback, including direct `cargo run --locked --release`.
Stock GTK playback works. Metadata correctly admits BT.709; failure precedes
filtering/presentation and reports `PIVRA_GPU_FAIL no shared GL context`.
Reported native platform: GTK 4.24.1, Mesa 26.3.1, GStreamer 1.28.9,
Linux 7.2.15, Wayland, AMD RX 7900 XT. These are **user reports**, not Codex runs.
The earlier historical AMD v0.2.5 acceptance is not evidence that it works on
this current platform. This does not establish a v0.2.6-only regression.

The production source has a concrete lifecycle omission: ImageView presents
MediaFile through `GtkPicture` and `ActualPaintable`, neither of which forwards
`GtkMediaStream::realize()` as `GtkVideo` does. `open_selected()` set the filename
before any surface binding. The native paintable therefore had no surface,
and always created its GL context through the display-only factory. The
initialization was also not explicitly gated on the native window's realization.
The accepted standalone probe used that offscreen path too, so its success did
not establish the production surface contract.

Both released and development sources had this omission. The patch corrects
it, but **the exact native AMD GL/GStreamer failure stage remains unproven**:
there is no new affected-machine context-stage trace or hardware rerun here.
No additional EGL ownership hypothesis, system patch or driver change is made.

## Implemented behavior

- Production waits for the window's `realize` signal if necessary. This is a
  signal barrier, without startup sleeps or retries. Navigation/close disconnect
  pending handlers; generation guards reject obsolete discovery results.
- `open_selected_on_surface()` realizes the media stream on the actual window
  surface **before** setting its filename. The sink uses
  `gdk_surface_create_gl_context()` for this path. Standalone diagnostic users
  retain the display-only API. The C control asserts the actual GL context's
  surface equals the production surface.
- Surface references are held safely and realization is paired with unrealization.
  Close/unrealize/destroy detach the picture, disconnect the error handler and
  clear owned pipelines. Existing GPU context guards and bounded cleanup remain.
- A synchronous GPU initialization error retires the failed owned stream and
  opens stock GTK once. An asynchronous owned-pipeline error queues one recovery
  outside the error/bus callback. Both generation and exact stream identity must
  still match before replacement. Duplicate notifications, navigation and close
  cannot reopen an obsolete file. Stock errors are terminal visible errors,
  without a retry loop; synchronous stock errors are handled too.
- Recovery preserves position, volume/mute, loop state, viewing transform and
  playback intent. GTK's `gtk_media_stream_gerror()` pauses the stream before
  notifying error; the code preserves the last non-error play/pause intent so
  that this internal pause cannot accidentally pause recovered playback.
- Failures report `PIVRA_VIDEO_FALLBACK stage=initialization|pipeline backend=stock
  reason=...; BT.1886 correction unavailable`. The preceding context failure
  retains its specific stage/reason. Optional tracing reports
  `PIVRA_GPU_INITIALIZE source=window-surface` versus `offscreen-display`.

**Stock recovery uses GTK's ordinary colors; it is not BT.1886 GPU success or a
CPU transfer-function workaround.** The BT.1886 shader, metadata eligibility,
HDR/non-target routing, media controls, EGL ownership policy, thumbnail/cache/
preload budgets and decoder security are unchanged. No dependency, version,
package, release-source, main, tag or AUR changes occur.

## Executed validation

Codex SDK: Debian 13, GTK 4.18.6, GStreamer 1.26.2, Rust/Cargo 1.99.1,
Mesa llvmpipe, X11/Xvfb and headless Sway. These are **software GL controls**.

| Check | Result |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo check --locked` | PASS |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS, zero warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 217 tests |
| Python root / thumbnail / FD05A / video color / video GPU | PASS, 69 / 59 / 9 / 13 / 73 = 223 |
| New production initialization/recovery suite, X11 and Sway | PASS, 9/9 in each |
| Existing GPU context lifecycle suite, X11 and Sway | PASS, 9/9 in each |
| `cargo build --release --locked` | PASS |
| Shell syntax / `git diff --check` | PASS |
| Current native AMD GPU/fallback acceptance, physical audio and MPV appearance | NOT RUN in Codex |

The new suite uses real production ImageView/MediaFile/GTK windows and a full
production-window gate. It covers initial surface binding, GPU success,
synchronous failure with real stock frames, asynchronous recovery while paused
and while playing, a second independently active GPU window, seek/frame-step/
mute/volume controls, navigation, stale recovery, close with recovery queued,
deleted-source visible errors and recovery to another video. GPU counters and
weak references must drain after close. Snapshots must contain a real render
node/texture. Destructive cases copy inputs into private temporary directories.
Fixtures contain video without an audio track; control-state checks do not
establish physical audio output. No native visual success is claimed.

The existing suite separately checks limited/full-range shader output against
the independent BT.1886 reference, sRGB/unknown stock routing, foreign display
ownership, active binding restoration, bounded presentation and cleanup. Its
software mixed PASS does not close the earlier native mixed-lifecycle failure.

## Negative-control boundary retained

The first attempted failure control set `GST_GL_API=invalid-test-api` globally.
That also breaks stock GTK's GL setup and reproduced a make-current warning /
libepoxy abort during fallback. It is **not a passing recovery result**, and the
known shared-EGL stock/backend boundary remains open. No production workaround
or suppression was added for that separate all-GL-broken scenario.

The final initialization control instead uses a **test-only** shared library
that rejects `gdk_surface_create_gl_context()` only when the caller is the
Pivra probe executable (verified by device/inode with `dladdr` and `/proc/self/exe`).
GTK's renderer and stock sink remain unmodified. There is no product injection
switch or new runtime dependency. The driver requires exactly two context/GPU
failures for two deliberately rejected media opens, as well as actual stock
rendered frames and working controls/navigation. Zero exit alone cannot pass.

An intermediate assertion assumed mute always started false; shared test
preferences invalidated that assumption. Cases now have separate private XDG
directories and verify that the key toggles the actual starting mute state.
Another intermediate report counted the diagnostic tag quoted in an error
message as a second error; the error text no longer repeats the tag. Exact
unexpected-warning/error count checks remain in place.

## Native acceptance

From updated dev in the normal Arch/GNOME session, without backend/test overrides:

```sh
bash tests/run_video_initialization.sh
```

Keep the printed report directory, `report.json` and per-case logs. The normal
and full-production-window rows require actual owned GPU playback; stock
recovery cannot satisfy them. The initialization-failure row deliberately
tests recovery with stock GTK frames and must not be reported as GPU success.
Hardware and software results are labelled separately. The command does not
alter real media or the user's preferences. A normal-playback FAIL with a
successful stock-recovery row still requires review of the actual native logs;
the publication block is not removed automatically.

Finally confirm the original user's video in normal Pivra for audio, visual
color, controls and navigation. GPU success should show the window-surface
initialization plus GLMemory draw/sync traces; a fallback must explicitly say
BT.1886 correction is unavailable. Until the affected AMD machine confirms
actual GPU playback or safe normal-app fallback, v0.2.6 remains blocked. The
previous mixed-lifecycle EGL issue and historical preload-budget observation
remain open. No VRAM, hardware-decoding, end-to-end zero-copy or cross-GPU
compatibility claim is made.
