# v0.2.7 slideshow — feature branch validation

> Historical record. Dates, versions, test outcomes and approval states below
> describe the work captured at the time. See the [documentation index](../README.md)
> for current development and release status.

Date: 2026-11-23. **TESTING — physical GNOME acceptance pending.**
Subsequent owner-reported native EOS SIGSEGV: **OPEN; native acceptance BLOCKED**.
See the [focused investigation and GDB command](V027_SLIDESHOW_EOS_SIGSEGV.md).
The original results below remain historical controlled-environment evidence.
The subsequent [video-completion guard](V027_VIDEO_COMPLETION_GUARD.md) replaces
level-only EOS sampling and keeps video errors selected/paused; its new native
acceptance is also pending.
Branch: `feature/v0.2.7-slideshow`, based on
`72405a9fa31b5ff45a98532fbd4008de5ecdc29d`.
Implementation: `823455ad96844e7ac3d130a5fd119d855cf961ee`.
The following test/documentation commit records validation separately. This is
not a change to the v0.2.6 release candidate: Cargo/package versions remain
0.2.6, current stable remains v0.2.5, and dev/main are not promoted.

## Implemented contract (feature specification §3, 42–45)

- F5 starts from the selected media. Ordinary F/F11 intent and slideshow
  fullscreen intent are separate; the actual window is fullscreen while either
  requires it. Escape stops slideshow and restores ordinary fullscreen intent.
- Photos receive five seconds by default, starting when the accepted full image
  is ready. The existing window menu offers 3/5/10/15/30 seconds, saved through
  the existing private atomic settings infrastructure in
  `$XDG_CONFIG_HOME/pivra/slideshow-interval.conf` (usual XDG fallback applies).
  Invalid preferences restore five seconds. No new toolbar or dependency.
- Space freezes the remaining photo interval and pauses video. Resume preserves
  the remainder. Left/Right and existing previous/next actions navigate manually
  with a fresh interval; navigating while paused retains pause.
- Slideshow always wraps its own catalog traversal, including videos, regardless
  of normal navigation settings. Video advancement requires error-free EOS,
  never the photo timer. Existing repeat-current-video is temporarily suppressed
  and its preference restored on stop. A single video repeats by seeking or
  reloading through the normal path when seeking is unavailable.
- Photo-to-photo transitions freeze the outgoing paintable into one transient
  render-node paintable, then fade it over the incoming accepted image using
  the GTK frame clock for approximately 300 ms. There is no CPU readback,
  additional decoder or change to ordinary navigation effects. Rapid loads,
  failure, video switching, stop and close clear that paintable/tick callback.
- Existing selection/catalog monitoring, token-gated loading, cache and preload
  are reused. A changed selected file re-enters the existing reload path. Failed
  or unsupported photos are skipped once per slideshow session; an empty/all-bad
  photo catalog stops with an explicit message rather than retrying indefinitely.
  Video playback errors pause with their visible error; manual arrows remain
  available. They do not count as successful EOS or trigger automatic skipping.
  A repaired failed item is eligible again after restarting slideshow.
- Each window owns its clock, failed-item set, weak stream reference and one
  chained 100 ms GLib one-shot observer while active. Stop/close removes the
  source, transition and inhibitor. A retained action cannot restart a closed
  window. No per-frame diagnostic logging is introduced.
- GtkApplication's public IDLE inhibitor is requested for the active window and
  deliberately retained while paused. Unsupported inhibition reports a concise
  diagnostic; physical display-sleep behavior has not been tested here.

Production GPU code, GTK patches, shader, decoder security, package recipes,
Cargo dependencies/lockfile and v0.2.6 release documents are unchanged.

## Executed Codex checks

Environment: Rust/Cargo 1.99.1, reused GTK 4.18.6/GStreamer 1.26.2 SDK,
Xvfb + xfwm4, headless Sway, software GL/llvmpipe. No physical GNOME/AMD claim.

| Check | Result | Evidence/scope |
|---|---|---|
| `cargo fmt --check` | PASS | Rust formatting |
| `cargo check --locked` | PASS | Existing native build environment |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS | Zero warnings |
| `cargo test --locked` | PASS | 226 tests; nine new model/settings regressions |
| `cargo build --locked --bin pivra` | PASS | Version remains 0.2.6 intentionally |
| Existing Python suites | PASS | 242 tests: root 69, thumbnail 59, fd05a 9, video_color 13, video_gpu 92; no skips |
| New slideshow GTK probe, X11 and Sway | PASS | Production windows/actions/GIO, explicitly controlled photo pixels, real stock video EOS |
| Catalog regression, X11 and Sway | PASS | Existing assertions; Sway floating-window rule required for resize test |
| Synthetic zoom/pan, controlled async cancellation, window failure races | PASS | All three existing probes on both X11 and Sway |
| Existing ordinary fullscreen regression | PASS on final X11/Sway runs | Full monitor allocation, chrome, paintable/mode preservation and other-window isolation retained |
| Real secure photo slideshow prerequisite | NOT RUN / 77 | One native probe invocation: secure Glycin loader exited 1 under the existing sandbox; no bypass/repeated attempts |
| Physical GNOME, hardware GPU slideshow, actual display inhibition | NOT RUN | Owner acceptance required |

The new clock tests use injected monotonic durations (no sleeps): loading versus
readiness, one-shot deadlines, pause remainder, manual reset, interval changes,
video EOS, bounded looping/single/empty catalogs and failure exclusions. GTK
tests additionally cover F5 from normal/fullscreen windows, independent F11,
actual compositor acknowledgement, automatic/manual switching, a real visible
fade and its release, a changed selected file while paused, corrupt photo skip,
video EOS with ordinary repeat enabled, stopped-timer observation, late actions
after close, two-window independence and live deletion. The generated video is
five seconds, longer than the three-second test photo interval; success also
requires an actual error-free EOS trace, so error skips are not video success.

The native helper runner's legacy dev-only branch guard initially caused six
thumbnail Python failures. It now permits only dev and this exact feature branch;
provider tests and assertions are unchanged, and all 59 tests passed afterward.
The older fullscreen probe expected normal Escape to propagate although the
baseline already closes its window. It now verifies close on a separate window
and checks other windows survive. Its error-label lookup now finds the Label by
type instead of assuming an immediate sibling (already a Picture at baseline).
Visibility and error-message assertions remain intact.

Sway test setup matters: an initial tiled catalog run failed its 320×300 resize
assertion; the same gate passed with the existing floating-window setup. Initial
ordinary-fullscreen runs also observed an 800×600 surface despite fullscreen
flags and a tiled-layout restore deadline. The final identical floating-layout
control passed on both an isolated pre-feature source snapshot and the feature
source, with the full-monitor assertions intact. These earlier failures are
retained as intermittent headless-compositor evidence, not relabelled as PASS
or used to justify speculative production changes. X11 catalog tests placed and
verified the virtual pointer at (1279,1023) before opening test windows.

Local logs (ephemeral): `/tmp/pivra-slideshow-rust-final-all.log`,
`/tmp/pivra-slideshow-python-thumbnail-final.log`,
`/tmp/pivra-slideshow-regressions.log`,
`/tmp/pivra-slideshow-catalog-sway-final.log`,
`/tmp/pivra-slideshow-fullscreen-baseline-control.log`,
`/tmp/pivra-slideshow-fullscreen-feature-control.log`,
`/tmp/pivra-slideshow-native-prerequisite.log`, and
`/tmp/pivra-slideshow-{x11,sway}-long-video.log`.

## Reproduce

Controlled developer sessions (existing SDK must already be activated):

```bash
python3 -B tests/slideshow_smoke.py --session x11
python3 -B tests/slideshow_smoke.py --session sway
```

The runner creates private XDG directories/session bus, fixtures and compositor;
X11 requires xfwm4/xprop for real fullscreen acknowledgements. Controlled photo
pixels are explicitly labelled and never count as secure decoder evidence.

**One native Arch/GNOME automated acceptance command**, on this branch:

```bash
bash tests/run_slideshow_probe.sh
```

It uses real secure Glycin, production windows and a private generated video,
corrupt file and PNG fixtures. It changes no system installation or user
media/preferences. It requires ffmpeg with libx264 and a working GTK/GStreamer
media backend. Exit 77 means the real decoder prerequisite was not run; exit 0
requires successful assertions and the actual video-EOS trace. Failed logs and
fixtures are retained at the printed temporary path; successful temporary data
is removed. This command does not replace physical visual/sleep acceptance.

For physical manual acceptance, run `cargo run --locked -- /path/to/photo` in a
folder of **private copies** of photos, a corrupt photo and a video longer than
the chosen interval. Verify F5 starts at that photo, menu intervals, 300 ms
photo-only fade, video to completion with sound, Space pause/resume, arrows and
final-to-first wrap. Verify F11 intent survives entering/exiting slideshow,
Escape stops without closing the window, and ordinary seeking/zoom/pan/thumbnail
controls still work afterward. Repeat in two windows; close one and modify/delete
private files in the other. Check screen inhibition remains effective while
paused and is released on stop/close. No physical acceptance is claimed yet.

## Remaining boundaries

- Existing stock GTK mixed GPU/stock EGLDisplay lifecycle limitation and prior
  candidate SIGSEGV remain documented in the v0.2.6 diagnostics. Slideshow uses
  the unchanged renderer and does not resolve or suppress those failures. The
  existing isolated GTK patches or documented per-process X11 workaround may
  be needed on affected systems; this feature never installs/replaces GTK.
- Secure decode, physical timing/appearance, video GPU lifecycle and GNOME sleep
  inhibition need native acceptance. Headless software GL is not AMD validation.
- Inhibition depends on desktop support; unsupported systems keep slideshow
  functional with an explicit diagnostic. Broken-media exclusions last until
  slideshow restart; no unbounded automatic retry is added.

No release, tag, AUR action, version bump or dev/main update is authorized by
this feature test record.
