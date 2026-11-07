# v0.2.6 Task 01 — performance and ownership baseline

Status: **PARTIAL — measured filesystem/model baseline; native secure-image and
hardware resource baseline still required.** No optimization, budget adjustment,
version change or color/playback change is included.

Stable reference: v0.2.5, `726f6948cb391e620a4733348ab164ad3a7ea295`.
Measurements below used that HEAD **plus Task 01 instrumentation in the working
tree**, release-mode examples. They are not measurements of an untouched stable
binary. [Machine-readable evidence](V026_BASELINE.json) preserves the source,
dirty flag, measured executable hashes, exits and summaries. Commands in that
copy replace temporary absolute directories with placeholders. The final Git
commit containing this document identifies the diagnostic implementation; it
must not be substituted for the measured source reference.

## Environment and execution

Codex, 2026-11-07: Debian 13, Linux 6.18.57 x86_64, glibc 2.41,
5 visible logical CPUs, Rust/Cargo 1.99.1, GTK 4.18.6, GStreamer 1.26.2,
Mesa llvmpipe software rendering. `/proc` CPU clock: 100 ticks/second.
This is not the user's Arch/AMD machine. The existing SDK was activated with:

```sh
source /workspace/pivra-environment/video-integration/packaging-activate.sh
```

No system GTK replacement or additional dependency installation was performed.
Exact baseline collection command after building the runner's five examples:

```sh
bash tests/run_performance_baseline.sh --native --output /tmp/pivra-v026-native
```

It returned **1**: six scan cases passed, the secure image attempt failed,
dependent image/thumbnail measurements could not proceed, and the hardware video
gate was unavailable. Raw stdout/stderr and `report.json` were retained in that
directory. These files are execution artifacts, not installed application files.

On native Arch/GNOME, from a clean checkout, collect with one command:

```sh
bash tests/run_performance_baseline.sh --native
```

Requires the existing GPU-enabled development dependencies, working secure Glycin
loaders/user namespaces, a graphical session and `ffmpeg` for private video
fixtures. The command prints the report path. `--output NEW_DIRECTORY` is
optional; an existing output directory is rejected. No privilege, cache flush,
system installation, real-media modification or decoder bypass is requested.

## Instrumentation contract

`PIVRA_PERF=1 cargo run --locked -- /path/to/media` explicitly enables compact
JSONL on stderr. Without that setting, no `/proc` sampling or diagnostic output
occurs. Once enabled, the flag is cached for the process. Each process emits at
most 4,096 events and retains no sample history. There is no video frame hook,
pixel download or color conversion in this instrumentation. Poll-heavy probes
can consume the cap; `diagnostics_truncated` reports that limitation.

Timing uses Rust `Instant` and Python `time.monotonic`. Events distinguish:

- Directory scan total, natural-sort duration and admitted entry count.
- Image request-to-ready callback, secure load and secure `next_frame` duration.
  `error_or_cancel` and `abandoned` durations are not successful decode latency.
  Ready means texture delivery, **not first visible/composited frame**.
- GNOME cache query hit/miss/stale/busy/cancelled; whole cache/generation request;
  existing thumbnail request/admission/helper/completion stages; Freedesktop
  generation duration. Stage `value=1` is one occurrence, not a cumulative count.
- Preload queue, active/background state, started/completed/cancelled counts;
  RAM cache hit/miss/eviction/entry counts and estimated charge; image workers
  and pending depth when existing stats are sampled.
- Video metadata discovery time and observable live native GL-filter count.
  Filter count is **not GPU allocation count**.

Each event includes process RSS from `/proc/self/status` and cumulative process
user+system CPU ticks. JSON `null` means unavailable. Snapshots are point-in-time,
not continuous monitoring; per-window stats share the PID and are not additive
process totals. No paths, media names or URIs are included in performance JSON.
Existing separately enabled detailed diagnostics keep their own behavior.

The collector samples the direct child every 20 ms, saves relative monotonic
timestamps/RSS/CPU ticks in `CASE.resources.jsonl`, and records `wait4` CPU
seconds/high-water RSS. The first stored baseline predates the resource-series
export and contains aggregate samples only. New reports include min/max/last
diagnostic counter snapshots. Short scans may finish before an RSS sample; zero sampled
RSS means **no sample captured**, not zero memory. `wait4` can include launcher
pre-exec mappings and reaped descendants. Neither number is a decoder-tree live
sum, PSS, GPU VRAM, or a steady-state application-memory measurement. CPU ticks
are not CPU utilization percentages. Actual device VRAM is **NOT MEASURED**.

## Fixture and scenario definitions

- 100 / 1,000 / 5,000 files, 4:1 photo/video entries. Photos are valid constant
  RGB 64×32 PNGs; MP4 entries in these directories are deliberately empty and
  used **only** for enumeration/model navigation. Insertion order is shuffled
  with seed 2601, names are natural-sort numeric stems. Each process scans 11
  times and alternates 2,000 model navigation operations per scan. These timings
  do not represent video decode or image delivery.
- Native ImageView probe: real secure 1600×1200 PNG delivery, nine further
  forced decodes (warm OS cache), ten asserted RAM-cache hits, rapid alternating
  loads, two windows, survivor after close, twenty open/close cycles and a real
  8000×8000 RGB PNG at the 64 MP boundary. Forced decode deliberately invalidates
  the RAM entry via the existing observed-load API; RAM-hit tests use normal load.
- Native catalog probe measures real photo-cell construction for the above three
  directory sizes and the first idle-turn barrier. This is not a completed render
  or a percentile responsiveness claim. It runs only after the secure image gate.
  A separate two-cell production thumbnail pipeline must finish both thumbnails;
  failed cells make the probe fail, not PASS. Metadata-only lookup is separate.
- Existing integrated video probe covers real GPU/stock routing, repeated opening,
  rapid navigation, two windows, deletion and cleanup. Real generated H.264
  BT.709/sRGB/unknown/full-range 192×96 files are used, not the empty scan entries.
  The color/control probe includes seeking and snapshot assertions. Its readbacks
  are test observations, not a playback-throughput benchmark or production path.

Fixtures live under a private temporary directory. XDG config/cache/data for the
child are isolated in the report directory. No filesystem cache flush is done:
**first open/scan is not proven disk-cold**. Local physical disk-cold testing,
thumbnail scroll throughput at scale, decoder-child RSS/PSS, device VRAM and a
long-duration soak remain separate measurements. Twenty cycles are bounded
regression coverage, not evidence of long-running leak freedom.

## Measured results

Microseconds, monotonic. First scan is reported separately; repeat median/p95
uses 10 samples, nearest-rank p95 (therefore the maximum with this sample count).
Small samples and shared-host scheduling do not establish an SLA.

| Files | Diagnostics | First scan | Repeat median | Repeat p95 | Sort median / p95 |
|---:|---|---:|---:|---:|---:|
| 100 | off | 206 | 129.5 | 190 | not instrumented |
| 100 | on | 249 | 159.5 | 811 | 19 / 21 |
| 1,000 | off | 1,698 | 1,419.5 | 1,536 | not instrumented |
| 1,000 | on | 1,482 | 1,434.5 | 1,519 | 298 / 313 |
| 5,000 | off | 8,573 | 7,643.5 | 28,745 | not instrumented |
| 5,000 | on | 24,677 | 11,855.5 | 28,925 | 2,927 / 3,377 |

| Files | Trace off: process CPU seconds / wait4 peak bytes | Trace on: CPU seconds / wait4 peak bytes |
|---:|---:|---:|
| 100 | 0.002668 / 13,631,488 | 0.003662 / 13,762,560 |
| 1,000 | 0.017255 / 13,762,560 | 0.017000 / 13,762,560 |
| 5,000 | 0.088360 / 14,286,848 | 0.122015 / 14,286,848 |

The 5,000-entry scan sampled parent RSS peaks of 1,929,216 bytes (off) and
1,982,464 bytes (on); those are coarse process samples, not total GUI memory.
The much larger `wait4` high-water numbers must not be interpreted as a growing
texture cache. Diagnostics-on model navigation median for 2,000 operations was
32 / 30 / 50 µs at 100 / 1,000 / 5,000 entries; diagnostics-off was 31 / 32 / 32 µs.

The secure-image attempt returned exit 1: `Loader process exited early with
status '1'`, the executed command was Bwrap, and content-only MIME was correctly
`image/png`. Its failed load span was 44,500 µs; this is **not a decode PASS**.
Direct independent namespace control:

```sh
bwrap --unshare-user --ro-bind / / /usr/bin/true
```

returned 1: `bwrap: setting up uid map: Read-only file system`. This verifies a
development-environment restriction, without pretending the wrapper's generic
exit alone identifies every internal loader failure. No further secure-decode
benchmark attempts or unsafe fallback were made. Successful image latency,
thumbnail generation/hit rate, 64 MP decode, native multi-window memory plateau
and native catalog construction are **NOT RUN** beyond this failed prerequisite.
Recorded zero cache counters are from the failed path, not a useful cache baseline.

Hardware video probe: raw exit 101, `software driver rejected`; classified
**NOT RUN** for the hardware requirement. The diagnostic cap was reached by
stats polling; counts after truncation are unavailable. Seek then remained NOT RUN.
No CPU/GPU performance inference is made from that failed hardware prerequisite.

## Existing resource and ownership audit

| Component | Existing policy / ownership / cleanup |
|---|---|
| `discovery.rs`, `catalog.rs`, `window.rs` | Nonrecursive extension admission, per-entry metadata/readability checks, natural sort. Scan cancellation checked per entry and around sort; the sort comparator itself is synchronous. Production scans are dispatched off GTK; the native benchmark's explicitly timed widget construction stays on GTK. No decoded pixels in the catalog. |
| `image_requests.rs` | Per-view maximum 3 active jobs, one replaceable pending request, 5 s admission deadline. Cancellation/token identity suppresses stale results; cancellation is not proof of immediate termination of native work. Weak references avoid applying results to closed widgets. |
| `image_loader.rs` / ImageView | Mandatory secure Bwrap; source ≤64 MiB, ≤64 MP, edge ≤32,768, frame buffer ≤512 MiB. Foreground texture, paintable and cached clones can share GObject ownership. Closing invalidates/cancels requests and disconnects owned callbacks; decoder pool lifetime need not equal window lifetime. |
| `image_cache.rs` | Per-view 128 MiB nominal budget, 64 MiB entry limit, at most 3 LRU entries. Current pin and protected neighbor paths resist eviction; insertion is rejected if no room. Charge = width×height×32, two worst-case RGBA32F representations: an accounting estimate, not an allocation. Memory pressure halves the budget and evicts unpinned entries (or clears if necessary). |
| `preload.rs` | At most two neighbor paths, one background load, foreground priority and promotion. Failure history bounded to four paths. Memory warning disables preload/cancels pure background and pressures the cache. Foreground large images can be displayed outside this RAM cache. Epoch/source identity and completion ownership prevent stale publication. |
| `thumbnail_strip.rs` | 64 retained textures / 16 MiB estimated charge; visible and selected entries protected. LRU eviction may legitimately require re-decode. Cancellable generation, source identities, epochs and weak window refs guard completion. Widget/cell count is distinct from retained texture count. |
| `thumbnail_manager.rs`, `thumbnail.rs` | Session GNOME-first provider choice; per-file failure does not switch provider. Max 2 global unique-path permits, provider busy admission; Busy does not consume transient retry. One retry for eligible transient failures, no unbounded retry. GNOME helper AS=2 GiB, CPU=15 s, FSIZE=256 MiB, deadline=20 s; guards reap children/release permits. |
| `freedesktop_generator.rs` | Separate thumbnail ceiling 50 MP, >8,388,608 pixels is a large job, only one large job, 2 GiB reservation budget with observed 512 MiB headroom, estimate pixels×32+128 MiB; decode ≤512 MiB, download ≤224 MiB. RAII releases reservation/admission. Host/cgroup headroom is not a hard RSS cap. |
| `gpu_video.rs`, `native/video/` | Discoverer deadline 2 s, max 2 discovery workers, 64-entry identity cache. Eligible path remains metadata-gated H.264/8-bit/≤8 MP. Playbin state NULL, bus/timer removal and GL ownership release on teardown; seek coalescing unchanged. GL buffer pool 2–8 buffers; bounded 2 queued presentations; RGBA surface cap 32 MiB. Actual allocations depend on negotiated caps/driver and may be shared. Live-filter count proves only filter lifetime. |

64 MP image charge is **2,048,000,000 bytes estimated**, so it is intentionally
outside the 64 MiB cache-entry/preload budget; this is not a claim of 2 GB RSS or
VRAM. Device VRAM, hardware decoding and end-to-end zero-copy remain unproven.
No new evidence of a resource leak was obtained. The previously accepted,
unreproduced `Preload exceeds cache entry budget` observation remains OPEN;
this task does not resolve it.

## Validation actually executed

All checks used the SDK above; no upstream GTK suite was run.

| Check | Result |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo check --locked` | PASS |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS, zero warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 212 tests, zero failures |
| Python discovery: `tests`, `tests/thumbnail`, `tests/fd05a`, `tests/video_gpu`, `tests/video_color` | PASS: 64+59+9+32+13 = 177 tests |
| Five release diagnostic examples / debug GUI examples | PASS, compiled |
| Six filesystem/model benchmark cases | PASS |
| X11/Xvfb: synthetic zoom/pan, controlled async cancel, window failure races, integrated video lifecycle | PASS, four gates |
| Sway/Wayland: same four gates plus video ramp/control assertions | PASS, five gates |
| X11 and Sway `thumbnail_overlay_probe --catalog-regression` | FAIL in both, exit 101, assertion below |
| Secure image performance gate | FAIL raw exit 1; dependent native measurements NOT RUN |
| Hardware video, device VRAM, native HDR, Intel/NVIDIA performance | NOT RUN |

Python commands used `python3 -B -m unittest discover -s DIRECTORY -p 'test_*.py'`.
The root suite includes the four new collector tests. Shell entry syntax was
checked with `bash -n tests/run_performance_baseline.sh`.
The final non-native runner smoke (`--output /tmp/pivra-v026-final-smoke`)
also returned 0: six scan cases PASS, native scenarios explicitly NOT RUN;
all six timestamped resource-series files were created. Its new timings do not
replace the earlier reported measurement set. An invocation without SDK
activation failed to locate `rustc`; the activated root-suite rerun passed 64/64.

GUI runs used private XDG runtime/config/cache directories, `dbus-run-session`,
Xvfb 1280×1024×24 or headless Sway with pixman compositor, GTK `ngl` renderer,
llvmpipe and explicit `PIVRA_GPU_ALLOW_SOFTWARE_TEST=1`. These controlled software
GL successes do **not** establish native AMD throughput, hardware decode or
secure image decode. Commands inside those sessions:

```sh
target/debug/examples/view_modes_probe --synthetic --zoom-pan
target/debug/examples/async_cancel_probe --controlled
target/debug/examples/window_failure_race_probe
target/debug/examples/integrated_video_probe BT709.mp4 SRGB.mp4 UNKNOWN.mp4 FULL709.mp4
target/debug/examples/video_color_probe RAMP.mp4 --check-bt1886-ramp --check-controls
target/debug/examples/thumbnail_overlay_probe --catalog-regression
```

Catalog failure, before catalog performance checks:

```text
examples/thumbnail_overlay_probe.rs:214:9
assertion `left == right` failed
  left: Bubble
 right: Capture
```

The test expects the first motion controller it finds to use Capture; it finds
Bubble. The controller/test implementation is unchanged by this task. The X11
pointer was moved outside before launch; the assertion happens before auto-hide
testing. No assertions were removed or adjusted. This controller-discovery
regression needs separate diagnosis; no unmeasured claim about its native impact.

## Ranked findings and Task 02 recommendations

1. **Measured scaling cost:** 5,000-entry filesystem admission/sort (7.64 ms
   repeat median, 28.75 ms tail with diagnostics off) dominates the measured
   2,000 model navigation operations (~32 µs). Sort alone measured 2.93 ms median
   with diagnostics on. Investigate syscall/admission vs sort separately on Arch
   before changing algorithms; these numbers do not prove a GUI freeze.
2. **Measured observer cost/noise:** diagnostics-on 5,000-entry median was
   11.86 ms versus 7.64 ms off in this run. Separate enabled/off measurements
   and increase sample count/alternate order before attributing a precise overhead
   percentage. Poll-heavy video probes exhausted the bounded log; retain raw logs
   and use a fresh process for each scenario.
3. **Not ranked without data:** native thumbnail generation, visible-set churn,
   cache hit rate, decode/first-visible latency and GPU memory. These may dominate
   real workloads, but the sandbox prevents a trustworthy comparison here.

Task 02 should start by collecting the native report and resolving the independent
catalog test gate. Then compare cold thumbnail creation with warm shared-cache
and RAM reuse; correlate admission/Busy/cancellation/completion, visible scrolling
and cache evictions before changing scheduling. Preserve current/visible pinning,
bounded transient retry and foreground priority. Sample decoder children/PSS and
driver-supported per-process GPU counters for longer repeated-window runs before
calling retention a leak or changing budgets. No Task 02 optimization is authorized
or implemented by this baseline commit.
