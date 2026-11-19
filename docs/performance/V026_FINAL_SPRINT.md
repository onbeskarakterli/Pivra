# v0.2.6 final development sprint — measured changes, native blocker open

Implementation commit: `25d4b9eddd169b2d25385dccbcdde136b465ae5f`.
Recorded 2026-11-19. Base: `1e25d0a850c1af174ec1a697e1bda2eb9ac27cab`, dev.
**PARTIAL / NO-GO for release.** Version 0.2.6 is a development candidate; stable
v0.2.5 is not changed. [Machine-readable evidence](V026_FINAL_SPRINT.json).

## 1. Native EGL ownership: failure established, correction still blocked

The two user-supplied native files contain exactly the same 117 GDB events.
They establish an **inferior exit 101**, not an unfinished debugger breakpoint.
GDB 18.1 / GTK 4.24.1 and GStreamer libraries `libgstgl-1.0.so.0.2809.0` are
recorded. Exact tested Git SHA is not recoverable without the original report;
it must not be inferred solely from the debugger protocol revision.

At call 18, during sRGB stock MediaFile retirement, the main thread calls
`eglTerminate(0x55d875099f70)` and observes return 1. The stack passes through
libgstgl, unref, GTK, libgstopengl/libgstplayback and MediaFile drop. That actual
display is also recorded for both owned GPU workers. The other GPU window is
still alive. Call 19 tries to destroy the stock context `0x55d8764d73a0` and
returns 0. Four make-current warnings follow. All 38 subsequent context creations
return NULL on the same display, without another observed eglInitialize.
The probe finally panics at `examples/integrated_video_probe.rs:41` with
`FAIL full-range GPU: filters=1 workers=0 pending=0 contexts=1 frames=2 presentations=0`.

This runtime ordering supports the stock borrowed-display ownership defect
identified in [Task 01C](V026_NATIVE_AMD_CONTEXT_FAILURE.md). The stock display
object's exact `foreign_display` field is not directly measured. This is not
attributed to the BT.1886 shader, nor is a later change declared an AMD fix.

The owned sink already uses `gst_gl_display_egl_new_with_egl_display()` to borrow
GTK's display. The private stock GstPlay sink cannot be reached through public
GtkMediaFile API for a scoped ownership change. The existing
[upstream review patch](gtk-4.24.1-stock-egl-ownership.patch) is not applied to the
system. Global hooks, retaining obsolete players, replacing stock/HDR rendering,
CPU conversion and system GTK patching are not introduced. An upstream/distribution
ownership fix needs native review/validation; an application-owned passthrough
renderer would be a separate architectural change requiring color/routing tests.
**Native AMD acceptance remains BLOCKED until all nine real gates pass.**

## 2. Measured RAM/work reduction

### Targeted preload invalidation

Previously any path notification called a helper that cleared all cached images,
failed history and queued work, and cancelled unrelated pure background decoding.
Now only affected cache paths/history/queued entries are invalidated. An affected
background job is cancelled; current-file changes still rotate the epoch and
cancel background work. Unknown-path notifications and explicit full refresh/close
retain full invalidation. Full FileKey lookup and token checks remain mandatory.

Three release-mode controlled processes repeatedly invalidated an unrelated path,
then reopened the current image and its two cached neighbors, 12 times. Only
private synthetic loader fixtures are used; the production cache/controller/GDK
objects are real. They do not demonstrate native decoder speed.

| Metric, per process | Before (three runs) | After (three runs) |
| --- | --- | --- |
| Extra controlled decode calls | 48, 47, 48 | 0, 0, 0 |
| Whole 12-cycle elapsed microseconds | 67,161; 140,850; 56,365 | 121,456; 31,728; 30,834 |
| Cached entries at end | 3 | 3 |

The repeated work was eliminated in this workload. Timing is noisy: medians
67,161 → 31,728 µs with only three process trials. No general latency/RSS
percentage or native speedup is inferred. Other caches, 3-image/128 MiB budgets,
64 MiB entry limit and memory-pressure behavior are unchanged. Current, changed,
deleted, promoted, obsolete and independently closing-window cases remain tested.
The historical `Preload exceeds cache entry budget` observation remains OPEN.

### Packed generated RGBA thumbnails

Generated RGBA8/sRGB buffers that already fit the requested dimensions and have
an exact packed stride/length now return their first MemoryTexture directly.
Other layouts, colors/formats and resizes retain the previous normalization path.
The helper validates layout/overflow before texture construction. Pixel/alpha,
padded-stride and resampling regressions use real GDK memory textures.

A release probe processes 64 private 192x160 RGBA buffers, 11 rounds in each of
three fresh processes. The source buffers occupy 7,864,320 bytes. Both revisions
verify every pixel and alpha value after measurement.

| Metric | Before | After |
| --- | --- | --- |
| Per-process round median, µs | 31 / 31 / 30 | 8 / 8 / 8 |
| Per-process p95 of 11 rounds, µs | 759 / 136 / 108 | 106 / 86 / 95 |
| RSS added above already-created inputs | 229,376 bytes each | 225,280 bytes each |

**No meaningful RAM reduction is established by 4 KiB.** The initial suspicion
of a duplicate full pixel buffer was not confirmed: GTK can already return shared
GBytes from its downloader. The supported gain is avoiding redundant downloader /
texture-object work, not claiming a removed 7.5 MiB allocation or a VRAM decrease.
The thumbnail count/16 MiB charge and main-photo conservative accounting remain.

## 3. GPU reference lifetime

Reset previously invalidated the epoch but left two queued presentations holding
textures/buffers until the main loop dispatched them. Reset now destroys queued
sources immediately outside a short mutex; the original queue limit of two is
unchanged. A currently executing callback is removed from the cancellable-source
list and retains its source references through return, so a reentrant close from
an invalidate signal cannot free its active invocation. Epoch suppression still
rejects stale frames. New-epoch presentation continues normally.

The actual C control measured `pending_after_reset=2` before and `0` after,
without pumping GLib. Weak-reference checks verify texture and paintable retirement
when the final owner closes with pending work, including a reentrant close.
This is an application-reference count result, **not measured GPU VRAM freed**.
Drivers/renderers can retain their own references/caches. GLMemory transfer,
synchronization, shader math, range/matrix eligibility, stock routing and the
existing GStreamer 2–8 buffer pool are unchanged. No speculative pool/cache-budget
retuning is performed without hardware measurements.

## 4. GUI test contract corrections

- Motion controllers now coexist. The probe targets the named thumbnail visibility
  controller, including its held instance after detachment. Capture is still asserted.
- Preload's old sibling cast found the intervening video Picture; it now finds the
  error Label, like the existing window-race test. Error assertions remain intact.
- Git blame shows thumbnail photo margins added by `7d7e056c` after the outer-size
  assertion. Tests now require the exact configured photo viewport, all four 3 px
  margins and exact outer allocation including those insets. Production geometry
  and styling are unchanged, and selected/unselected cells are checked.
- Headless Sway tiles windows by default and ignores requested floating sizes.
  This acceptance fixture explicitly floats its Pivra windows for resize tests;
  the original standalone video fixture default remains unchanged.
- Xvfb without `-noreset` was reproduced resetting the cursor from (1279,1023) to
  (640,512) after the last client disconnected. The initial verification alone
  was insufficient. Private Xvfb now uses `-noreset`, with checks after disconnect
  and again before the catalog launch, plus a real-Xvfb Python regression.
  No production hide timer, pointer behavior, timeout or assertion is weakened.

Initial failing controls (old sibling/geometry/tiling assumptions, X11 auto-hide)
are not PASS records. The final corrected runs passed both backends. They use
synthetic image/thumbnail loaders where explicitly named; decoder success is
not inferred from them.

## 5. Validation and measurement limits

Existing Debian 13 SDK, GTK 4.18.6 / GStreamer 1.26.2, Rust 1.99.1, llvmpipe.
No system GTK/library install or full upstream GTK suite. Commands from the repo:

```sh
cargo fmt --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
CARGO_INCREMENTAL=0 cargo test --locked
# For DIRECTORY: tests, tests/thumbnail, tests/fd05a, tests/video_color, tests/video_gpu
python3 -B -m unittest discover -s DIRECTORY
bash tests/run_v026_acceptance.sh --session x11 --software
bash tests/run_v026_acceptance.sh --session sway --software
bash tests/run_performance_baseline.sh --output NEW_PRIVATE_DIRECTORY
bash packaging/linux/build-install.sh build
PREFIX=/usr DESTDIR=NEW_PRIVATE_STAGE bash packaging/linux/build-install.sh install
```

| Executed check | Final result |
| --- | --- |
| fmt, locked check, strict all-target Clippy | PASS, zero warnings |
| Rust tests | PASS, 217 |
| Python root / thumbnail / FD05A / video color / video GPU | PASS, 67 / 59 / 9 / 13 / 62 = 210; real GDB and Xvfb controls executed |
| Strict C context/current-API/shader controls | PASS |
| X11 and Sway: memory/cache/preload, catalog, zoom/pan, cancellation, window races | PASS in each session |
| X11 and Sway: nine video lifecycle gates, including shader/control checks | 9/9 PASS each; software only |
| Filesystem/model collector, 100/1000/5000, diagnostics off/on | Six PASS; native scenarios explicitly NOT RUN |
| Mandatory GPU distribution release build, ELF and sibling helper | PASS |
| Locked crate/adapter/application source and notices; private DESTDIR staging | PASS, 184 crate sources |
| Clean candidate preflight | PASS on the implementation commit; source/package consistency only |
| Native AMD / new Arch makepkg / immutable source approval | BLOCKED / NOT RUN / NOT APPROVED |

Current diagnostics-off scan repeat medians: 131 / 1,509.5 / 10,387 µs at
100 / 1000 / 5000 files, p95 186 / 1,893 / 11,733 µs. Scan code is unchanged.
These are regression measurements, not improvements against the historical
baseline from a different run/date. The file-system cache is not disk-cold.
RSS is process RSS, not PSS or decoder-tree memory. Device VRAM, Intel/NVIDIA,
native HDR, hardware decode, zero-copy and sustained soak measurements are NOT RUN.
No secure Bwrap retry/bypass is made in the restricted Codex environment.

To reproduce just the new microbenchmark: `cargo run --release --locked --example
thumbnail_memory_probe`. For controlled cache churn, use `target/release/examples/
preload_probe --cache-churn-benchmark` in a graphical session after building that
example. Old/new binaries and hashes are retained in the local evidence directory;
the committed manifest contains their actual hashes and raw summary rows.

## 6. Release gates and next native command

Cargo.toml, only Pivra's lock entry, PKGBUILD, target-version source validator,
staging/preflight references and notes are prepared for 0.2.6. `options=('!debug')`
is intact. README retains current stable v0.2.5 and labels the candidate blocked.
LGPL source/relink/notices and opt-in old-package cleanup are preserved. No
release-source.conf, .SRCINFO, fake archive digest, tag, main or AUR change occurs.
[Source/hash approval and makepkg workflow](../releases/PIVRA_V0.2.6_RC_VALIDATION.md)
remains separate; none of the staging artifacts are approved release inputs.

On the native Arch/GNOME machine:

```sh
bash tests/run_v026_acceptance.sh
```

The command uses private fixtures/config/cache, refuses native mode with software
test permission, keeps FAIL/NOT RUN visible and performs no package installation.
Native performance/64 MP collection runs only after the video lifecycle gate
passes, avoiding another repetition of the known failure. Until the stock EGL
ownership blocker is corrected and all nine AMD gates pass, publication is NO-GO.

Post-commit staging used the exact implementation commit above: 184 locked crate
sources / 324 application files collected. The staged archive's changed Rust,
native video and acceptance-runner files were compared byte-for-byte with the
reviewed tree. The staged binary reports Pivra 0.2.6; desktop validation, GPU ELF
contract and clean candidate preflight passed. This evidence follow-up changes
only documentation; neither the staging snapshot nor its checksum is an approved
immutable release source. Native makepkg and physical acceptance remain separate.
