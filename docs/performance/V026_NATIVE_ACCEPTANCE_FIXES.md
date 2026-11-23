# v0.2.6 native acceptance follow-up — 2026-11-20

**NO-GO. Native AMD lifecycle remains BLOCKED.**
Implementation: `ff9d6fc74a34b65d43c4c06e1c62235ac55cc49a`.
Base: `7f99570c263000b0ebb2124af0bc2b4c4c1d82a3`.
[Machine-readable results](V026_NATIVE_ACCEPTANCE_FIXES.json).

## Catalog: harness mode mismatch, not a requirement to reject repaired cache

The user reports 6 PASS / 2 FAIL on actual Arch/GNOME. The catalog failed at
`examples/support/thumbnail_catalog.rs:447`, `missing cache must be explicit`,
expected 1, actual 0. The full report and exact native source SHA were not supplied.

`v026_acceptance.py` invoked `thumbnail_overlay_probe --catalog-regression` in
both software and native sessions. It omitted `--native-thumbnails` in native
mode. That flag controls the example's decoder and its generation expectations;
merely running on real GNOME does not enable the secure decoder in this example.
Consequently, a native session was running a hybrid test: synthetic cache decoding
with the real production provider. Its "120 decodes" message alone is not proof
of 120 real secure decodes.

The production GNOME path intentionally attempts bounded generation when a cache
entry is Missing/Stale or a recoverable corrupt PNG. A successful current cache
HIT has zero failed cells. Therefore expecting one failed cell after a missing
cache is obsolete **when generation is allowed and succeeds**. Actual zero is
consistent with repair; it is not independently sufficient proof of repair.
No production cache/provider change is justified by this excerpt.

The fix separates the two contracts explicitly:

- **Native session:** command includes `--native-thumbnails`; the real provider
  environment is preserved. The existing one-attempt secure prerequisite remains.
  Missing/corrupt caches must recover with zero failed cells. New assertions also
  require a real cell texture; for GNOME they require a valid current normal-cache
  publication and exactly one uncancelled successful secure decode of that repaired
  path. The result cannot pass merely by keeping an old cell with failed=0.
- **Controlled software session:** a private, trusted failure-only generator fixture
  returns exit 4; the per-child GNOME metadata-provider session uses it without
  changing the parent environment. Missing=1 and missing+corrupt=2 assertions are
  unchanged. No mock generation or synthetic pixels are labelled native success.

Each catalog row now records the command and `evidence_kind`, with a matching
terminal mode banner. Positive/negative Python controls verify the native flag,
unchanged native provider environment, controlled refusal and parent-env isolation.
Source replacement/deletion, stable decode audit, retention and window/navigation
checks remain in the catalog suite. No production cache, worker, UI or color path
is changed. The six other native-gate commands and environments are unchanged;
they were not rerun on physical hardware by Codex.

## GPU: confirmed upstream ownership boundary, no application workaround claimed

The new user excerpt reports `PIVRA_GPU_FAIL no shared GL context` and
`No EGL configuration available`. This is consistent with the existing complete
native evidence, not a new color/shader investigation:

1. Stock MediaFile teardown calls eglTerminate successfully on the EGLDisplay
   also used by the live owned GPU contexts.
2. The next stock context destruction fails, then 38 subsequent creations return
   NULL without another eglInitialize. The actual inferior exits 101.
3. These events, handles, caller stacks and provenance are retained in
   [the preceding sprint evidence](V026_FINAL_SPRINT.md) and Task 01C reports.

Pivra's owned sink already borrows that display with
`gst_gl_display_egl_new_with_egl_display()`; its foreign-display/ownership controls
continue to pass. The remaining incorrect wrapper belongs to stock GTK's private
GstPlay/sink. Public GtkMediaFile API does not expose it for a scoped correction.
The smallest source fix is the existing
[GTK 4.24.1 upstream review patch](gtk-4.24.1-stock-egl-ownership.patch): use the
GDK EGLDisplay and the foreign/borrowed EGL wrapper in the stock Wayland branch,
with its EGL include guard. It is **not applied to system GTK or claimed native
validated**. It needs upstream/distribution integration and the exact AMD nine-gate
rerun. An application-owned passthrough backend would be a larger architectural
change requiring separate metadata/HDR acceptance, not a safe implicit substitution.

No global hooks, retained obsolete players, system GTK patch, CPU conversion,
stock/HDR rerouting, BT.1886 change or timeout increase is introduced. This task
does not implement or claim an application-side fix for the private stock sink.
The GPU runner now treats `No EGL configuration available` as a failing gate
condition even if the process exits 0, and stores its count separately. A real
child-process negative test covers that case; no warning is suppressed.

## Executed verification

Codex SDK: Debian 13, GTK 4.18.6, GStreamer 1.26.2, Rust 1.99.1, llvmpipe.
This is not the user's GTK 4.24.1 / physical AMD environment.

| Check | Result |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo check --locked` | PASS |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS, zero warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 217 tests |
| Python root / thumbnail / FD05A / video color / GPU | PASS: 69 / 59 / 9 / 13 / 63 = 213 |
| X11/Xvfb controlled GNOME catalog with explicit failure fixture | PASS |
| Sway/Wayland floating-window controlled catalog, same fixture | PASS |
| X11 and Sway targeted video lifecycle/current-context/shader controls | 9/9 PASS each, llvmpipe only |
| Real secure catalog prerequisite, one attempt in Codex/Xvfb | NOT RUN / exit 77: Bwrap loader exited early with status 1 |
| `cargo build --release --locked` | PASS, development profile; no new makepkg claim |
| New real Arch/GNOME catalog acceptance and AMD lifecycle | NOT RUN in Codex; native acceptance pending/BLOCKED |

The targeted run constructed catalog commands with `catalog_case()` inside the
existing private X11/Sway session helper. X11 pointer was verified at (1279,1023).
It then ran `bash tests/run_video_context_lifecycle.sh --software` in each session.
No entire GTK suite or broad GPU experiments were run. After the one secure
prerequisite failed, it was not retried and no sandbox fallback was enabled.
No new source archive approval, version/package/lockfile change, main promotion,
tag, release or AUR action occurs.

## Single native command

From updated dev in the normal Arch/GNOME session:

```sh
bash tests/run_v026_acceptance.sh
```

The catalog row must show `CATALOG_MODE secure-decoder-and-production-provider`
and its command must include `--native-thumbnails`. Keep report.json and the
catalog/video logs. Expect native cache-repair PASS only with actual publication /
secure decode proof. Until the stock EGL ownership correction is available and
all nine real AMD video gates pass, v0.2.6 remains NO-GO. The known unreproduced
preload-budget observation and other existing release limitations remain open.
