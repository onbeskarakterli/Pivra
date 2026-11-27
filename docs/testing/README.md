# Testing Pivra

Use private copies of fixtures and private XDG locations for GUI runs, especially
rename, deletion, Trash, cache and external file changes. Keep execution bounded.
The complete result for this cycle is the
[v0.2.8 validation report](../releases/PIVRA_V0.2.8_RC_VALIDATION.md).

For a frozen candidate run the static and Rust gates:

```sh
cargo fmt --all -- --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
```

Then run the Python discovery groups and controlled GTK scenarios listed in
the report. The existing runners include `tests/toolbar_ux_smoke.py --session
x11` for toolbar and preference restart, its `--development` modes for
specific windows and actions, `tests/slideshow_smoke.py --session x11` for
slideshow and video, and `tests/run_video_context_lifecycle.sh --session x11
--software` for graphics-context lifecycle checks. Their controlled image
pixels and software GPU rendering have limited evidence scope.

For physical GNOME acceptance, launch with ordinary installed dependencies
and disposable media copies. Verify native Open, context popups, focus,
Nautilus, metadata, rename, Trash/Undo, zoom and thumbnail controls. Exercise
real still and animated formats, secure loader cancellation, slideshow timing
and transitions, video audio/seek/controls, sleep inhibition and GPU fallback.
Check the default XWayland path and native Wayland separately. The
[GTK upstream handoff](../upstream/GTK_4.24.1_WAYLAND_EGL_OWNERSHIP.md) describes
the known native Wayland risk. Record case, compositor and versions, plus
actual exit or visible outcome. A missing runtime or sandbox prerequisite is
a skipped gate.

Optional traces:
- Open and subsequent photo latency: `bash tests/trace_open_latency.sh
  /absolute/path/to/photo.jpg`. The [latency guide](../diagnostics/OPEN_FILE_LATENCY.md)
  explains what it can observe and why the physical delay remains open.
- Thumbnail admission and retry: see [thumbnail diagnostics](../diagnostics/THUMBNAIL_REQUEST_DIAGNOSTICS.md).
- The existing [performance baseline](../performance/V026_BASELINE.md) records
  worker, cache and process-resource measurement boundaries.

Package build and installed desktop verification are separate native gates.
Use the [packaging guide](../packaging/README.md).
