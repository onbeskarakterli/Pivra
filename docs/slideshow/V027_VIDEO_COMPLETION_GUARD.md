# Slideshow video completion guard — 2026-11-23

**TESTING — physical GNOME acceptance pending.** Feature branch only:
`feature/v0.2.7-slideshow`; reviewed base
`918a79e7181e8f4c8dc93912cdab7c1e6c0bc0dc`.
Before committing, concurrent documentation-only commit
`c55a082b04f061da9e2fbaeb0203a08b686a6638` was preserved by fast-forward. Its
top-bar/UX notes are unchanged; no new UI work is implemented here. Tested
application source is identical across those two bases.

## Findings and correction

The owner reports video advancing around the photo interval. Static review does
not establish the native event responsible: `Clock::due()` was already called
only for images. No image completion callback navigates directly. Selection and
catalog revisions reset the photo clock, and selected-file revisions may reload
through the normal path. Manual actions, removed selection, photo deadline,
photo failure and video completion are distinct navigation triggers.

Two production weaknesses were demonstrated in the existing decision logic:

1. An error-free `is_ended()` sample was accepted without checking preparation,
   seeking, prior playback or whether this was a new completion for this stream.
2. A video decoder error used the same failed-item/automatic-next path as a bad
   photo. A failed video could therefore disappear without successful EOS.

GTK 4.24.1 source review (`gtk/gtkmediastream.c`) confirms that `ended` is a
stored property, documented as a hint. `stream_ended()` pauses then sets it;
successful `play()` and `seek_success()` clear it. `seek()` first sets seeking
and completion is asynchronous; `stream_unprepared()` does not itself clear
ended. Pausing alone does not mark EOS. The built-in stock backend sets ended
from GstPlay's end-of-stream signal; Pivra's GPU backend sets it from
`GST_MESSAGE_EOS`. Neither backend nor GTK is patched by this correction.
This establishes the insufficiency of a bare level test, not the exact cause
of the owner's report without its transition log.

The new per-stream completion guard observes prepared/playing/seeking/ended/error
notifications **before first play**, so clips shorter than the 100 ms observer
interval are not lost. It requires prepared playback followed by a fresh ended
edge without seeking/error; consumption rechecks the same current stream/state.
An initially ended stream, replacement, error, in-flight seek or resumed playback
cannot consume a stale queued completion. Paused completion is held until resume.
There is no arbitrary minimum duration, timestamp-near-end tolerance or wall-time
substitute for backend EOS (a backend can reset its timestamp at EOS).

Signal handlers only record state. The existing main-context observer performs
navigation after leaving the signal emitter; weak references and explicit
handler disconnection on navigation/replacement/stop/close prevent retaining old
streams. Retirement also disconnects before the emitter is dropped during
asynchronous GPU-to-stock fallback. The ImageView change is a small optional
stream-observer hook; outside
slideshow its weak callback does not change playback. Stock and GPU streams use
the same public GtkMediaFile contract. GPU architecture/shaders remain untouched.

The photo clock is explicitly disarmed throughout video selection. Video errors
now **pause with their existing visible error**, preserving selection and manual
arrows, rather than pretending completion. Bad photos retain bounded skipping.
Removed catalog selections are reconciled separately. Trace output is opt-in:
`manual`, `photo-deadline`, `photo-error`, `video-eos`, `video-error-paused`,
`catalog-empty-or-removed`, `catalog-revision-reload`, plus ended notifications
with preparation/seeking/error/acceptance fields. No per-frame logging.

## Executed validation

Codex: Rust 1.99.1, reused GTK 4.18.6/GStreamer 1.26.2 SDK, Xvfb/xfwm4 or
headless Sway, Mesa llvmpipe. Controlled photo pixels are explicitly labelled;
video decode/EOS, GTK windows, controls, seeks and production selection are real.

| Check | Result |
|---|---|
| fmt / locked check / strict all-target Clippy | PASS, zero warnings |
| `cargo test --locked` | 230 PASS, 0 failures |
| Python suites | 256 PASS: root 83, thumbnail 59, fd05a 9, video_color 13, video_gpu 92 |
| New interval regression, stock metadata route | PASS on X11 and Sway |
| New interval regression, actual owned BT.709 filter route | PASS on X11 and Sway, explicitly permitted software GL; not AMD acceptance |
| Existing full slideshow and three-cycle corrupt/EOS/photo regression | PASS on X11 and Sway |
| Secure native slideshow / physical GNOME and AMD | NOT RUN in this task; previously established Codex Bwrap limitation retained without another bypass/retry |

The new GUI gate uses an eight-second clip and three-second photo interval.
It continuously asserts video selection through at least 3300 ms of playing,
also requiring actual position progress beyond 40%. It pauses for another
3300 ms, applies a real backward seek (verified through the backend timestamp
label), resumes and requires EOS before photo rendering. It then replaces only
its private video copy with invalid data and proves error/selection remain paused
past another photo interval, with manual escape and independent window close.
The outer evidence validator rejects early/missing EOS or absent hold/error/close
stages; stock fallback cannot satisfy the GPU-specific assertion.

Initial GPU fixture runs correctly failed that assertion: generic ffmpeg color
flags produced only matrix metadata (`0:3:0:0`) in this environment. The fixture
now uses the existing project's x264 VUI parameters; actual discovery reports
`colorimetry=bt709`, `PivraGpuMediaFile` and a live filter. Production eligibility
was not weakened. One initial root Python run hit the existing private-X11
pointer check (`Cannot verify private X11 pointer`); the unchanged complete
83-test rerun passed. No v0.2.6 test or pointer implementation was modified.

Final GUI logs: `/tmp/pivra-slideshow-skip-{stock,gpu,full,eos}-{x11,sway}-verified.log`.
Quality logs:
`/tmp/pivra-slideshow-skip-{fmt,check,clippy}-final.log`,
`/tmp/pivra-slideshow-skip-rust-final.log`, and
`/tmp/pivra-slideshow-skip-python-*.log` (ephemeral).

## Native acceptance

On Arch/GNOME, from this branch, exercise the actual eligible GPU route:

```bash
bash tests/run_slideshow_probe.sh --video-interval-regression --bt709
```

This requires secure Glycin and the existing GPU/media prerequisites. It uses
private fixtures/config only. The probe selects the supported GL renderer if
none was explicitly selected; it does not replace GTK, enable software GPU
permission or accept stock fallback as GPU success. For the unknown-metadata
stock route, omit `--bt709`. Add `--gdb` if a crash recurs; failure artifacts and
verified signal/stack evidence are retained. Do not change the user's existing
isolated GTK environment silently.

Finally use a private copy of a genuinely long video in normal Pivra: F5,
3-second interval, play beyond that interval, Space, seek, resume to EOS, arrows,
audio and Escape. Physical acceptance is still required. The earlier native
SIGSEGV and stock GTK EGL lifecycle boundary remain OPEN; these controlled
successes do not classify or close them. No dev/main, release/version, packaging,
AUR, GTK patch or GPU shader change is included.
