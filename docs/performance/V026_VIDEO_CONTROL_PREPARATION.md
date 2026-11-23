# Video control preparation barrier — 2026-11-21

Implementation: `aeb66c9dfdd872ff432be48afaa45d671ca311d0`.
Base: `757706bd5227ff2afe9e1ab55b32022f8539347f`.
[Machine-readable results](V026_VIDEO_CONTROL_PREPARATION.json).

**Native patched-GTK GPU/frame gate: user-reported PASS. Control gate: TESTING,
native rerun pending. Publication remains BLOCKED.**

## Evidence and classification

The owner reports actual RX 7900 XT / GNOME Wayland success with the isolated
GTK correction: context creation, GLMemory, frame presentation and production
window PASS. The real-video snapshot comparison reports max channel difference
0. These are user-reported results, not independent Codex hardware tests.
The following control check fails: `control gate requires a prepared, seekable
finite video`. The old message did not record which individual predicate failed.

The probe has a demonstrated synchronization defect: it stops its first-frame
loop after successful raw/wrapper snapshots, then immediately asserts
`prepared && seekable && duration > 0`. Frame availability does not establish
that media preparation has been delivered to GTK. The production adapter
publishes preparation through a separate main-context `ASYNC_DONE` callback.
A real GTK state-order control now reproduces a render node existing while
prepared=false, seekable=false, duration=0, followed by a later preparation
callback. The old immediate guard rejects that valid ordering.

This proves the harness gap, **not which field was false on the owner's machine**.
The real software decoder runs here already had prepared=true, seekable=true,
duration=2,000,000 when inspected. No actual production metadata defect has been
demonstrated from the supplied generic native error; no engine change is made.

GTK 4.24.1 `gtk_media_stream_stream_prepared()` assigns seekability and duration
before setting prepared=true, under frozen property notification. The current
adapter publishes that information once. Therefore an already-prepared stream
with seekable=false or duration<=0 must fail clearly, rather than be treated as
an indefinitely pending preparation race. A real nonseekable/unknown-duration
input remains outside this finite-video control gate.

## Test-only correction

- `video_color_probe` uses a shared helper to wait for preparation after its
  unchanged pixel comparison. Color/render assertions and seek completion
  requirements remain intact.
- The helper dispatches GLib events, with a deadline wakeup source. It reuses
  the **existing 15-second frame deadline**; there is no new grace period,
  arbitrary sleep, metadata override or decoder retry. The existing five-second
  seek-completion limit is unchanged.
- All three original readiness conditions are still mandatory. Prepared but
  nonseekable/zero-duration streams fail immediately. Unprepared streams expire
  at the shared deadline. Existing or newly delivered media errors fail even
  when capability fields would otherwise look ready.
- Logs show `CONTROL_PREPARATION initial`, state transitions and readiness,
  including prepared, seekable and duration_us. Failure reports retain those
  actual values and distinguish prepared capabilities from a preparation
  deadline or media error. Only state changes are logged.
- The helper removes its pending deadline source on completion/error.

No production Rust/C source, GPU engine, shader, EGL patch, performance/cache/
preload policy, dependencies, system library or packaging file changes.

## Regression coverage and actual results

The new controlled GTK MediaFile subclass uses a deliberately synthetic 1x1
texture. Its seven cases cover delayed preparation after an actual snapshot
node, already-ready completion, never-prepared expiry, prepared nonseekability,
prepared zero duration, pre-existing error and an error delivered while waiting.
These are **GTK state-order tests, not successful real decode/GPU evidence**.

| Executed check | Result |
| --- | --- |
| Seven controlled GTK cases on SDK 4.18.6, clean 4.24.1 and patched 4.24.1, each on X11 and Sway | PASS, 42 checks after final fast-fail refinement |
| SDK X11/Sway preparation, real video/color/control and production-window regressions | PASS, 6 process gates |
| Isolated clean/patched GTK 4.24.1, X11/Sway: context, preparation, real video/color/controls and production window | PASS, 16 process gates |
| `cargo fmt --check` / `cargo check --locked` | PASS |
| Strict all-target Clippy, `-D warnings` | PASS, zero warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 217 tests |
| Python root / thumbnail / FD05A / video color / GPU | PASS, 69 / 59 / 9 / 13 / 86 = 236 |
| `cargo build --release --locked` | PASS; production source unchanged |
| New native AMD control run | NOT RUN in Codex; PENDING |

All new GUI/real-video results above use llvmpipe. Real clips in these automated
checks are the private finite H.264 ramp fixtures, not the owner's video. They
retain the existing 256-level BT.1886 reference tolerance and actual completed
seek checks. The isolated GTK libraries and their EGL source patch hashes are
unchanged from the preceding task; they were reused, not rebuilt or replaced.
The final prepared-capability fast-fail refinement was rechecked with all seven
state cases across the six GTK/backend combinations; the healthy video path is
unchanged by that refinement.

## Native follow-up and remaining gates

Reuse the existing private workspace and the same actual video:

```sh
bash tests/run_gtk_egl_isolated.sh --workspace "$HOME/.cache/pivra-gtk-egl-config-4.24.1" --video "/absolute/path/to/BT709.mp4" --manual
```

The cached GTK source/patch must still match the preceding verified hashes.
Keep the new report and `patched-gpu-color-controls.log`:

- `initial prepared=false` followed by all-true/positive readiness and completed
  seek PASS confirms preparation ordering explained this control failure.
- `prepared=true seekable=false` or `duration_us=0` remains FAIL. That would
  require investigation of the actual file/backend metadata query, not another
  timeout increase or removal of the gate.
- A deadline with prepared=false or a media error remains FAIL with exact state.

Native GPU/frame success is recorded; native control success after this change
has not been claimed. The separate upstream/backport delivery requirement for
GTK, historical mixed-EGL/SIGBUS observations and other open release limitations
remain. No main, tag, GitHub Release or AUR action occurs.
