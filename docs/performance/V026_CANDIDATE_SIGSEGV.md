# Ownership-candidate SIGSEGV investigation — 2026-11-23

**OPEN / UNCLASSIFIED. No fix claimed.** Diagnostic source:
`c4ca4d85a05fa81efeddc6f5229d28040c56d787`. The owner's subsequent RX 7900 XT
9/9 result establishes a successful native run, not resolution of this crash.

## Rechecked evidence

The original local result remains available in
`/tmp/pivra-video-context-2zhh_7c1/report.json` and `mixed-lifecycle.log`.
The report records raw exit **-11 (SIGSEGV)**, 139 diagnostic events, zero
make-current warnings, zero EGL configuration errors and zero GPU failure
markers. The log SHA-256 is
`c6f444810ed914c541aea5446558f3b5752c14d3cd406c29bcf24b612b5c2ad7`.
No stack/core was captured. Core dump configuration was unavailable through
this managed environment's `/proc/sys/kernel/core_pattern`.

Actual runtime: private GTK 4.24.1 with both review patches, GStreamer core/base/
GL/play 1.28.9, Debian 13, Mesa 25.0.7 llvmpipe/headless Sway. Existing SDK/system
plugins supply demux/parser/libav. This differs from native Arch/Mesa 26.3.1 and
is not a clean all-1.28.9 plugin distribution test. That is a possible confounder,
not a demonstrated cause or grounds to dismiss the crash.

The mixed stage last reported `unknown stock route ready`, contexts=1,
frames=1, presentations=0. The next navigation action then logged:

```text
LIFECYCLE_ACTION next-media
... stage=action-before
... stage=video-retire-begin
... stage=stock-stream-retired
... stage=video-retire-end
```

There is **no following action-after or full-range GPU begin**. This narrows the
observed interval to returning from unknown-stock retirement / completing the
next-media action, before the full-range wait begins. It does not identify the
faulting thread/instruction. Cross-thread log order is not a full happens-before
proof. `stock-stream-retired` is an object-lifetime marker, not proof that every
GStreamer/GDK reference or callback has finished.

Static review: `examples/integrated_video_probe.rs::action()` calls the production
window action. `src/image_view.rs::stop_video()` disconnects UI error handlers,
detaches the paintable, pauses/unrealizes the stream and drops stock references;
owned streams additionally clear their own pipeline. The test observes the real
window, not a mock decoder. Existing weak-reference/generation checks and GL
presentation cleanup have not been changed. Absence of context warnings or
pending owned presentations does not prove absence of other stock/GDK work.
No production defect or harness defect is established by this static review. The probe's
`frame()` readiness check reads GtkPicture visibility/paintable dimensions; it
does not itself perform a texture download. That does not exclude later GTK or
driver work, and supplies no basis to equate this crash with the old DMA-BUF
SIGBUS.

Five bounded GDB mixed executions exited normally, with complete cleanup stages;
one also used PIVRA_PERF and private XDG homes. Their log SHA-256 values are:

| Execution | SHA-256 |
| --- | --- |
| Initial | `dff15eb718e5a77542e5dbcecef2610032bb0d709c9f867957f53256f3114e33` |
| Bounded 0 | `d43c14cf65866124ab7b414950bfd99fb587804ec0bc7770489400703987f4c1` |
| Bounded 1 | `1564ad3adb79c2d0e13aeccd0ef997ec8abe6e31b0edc03f9698cfdd311a03e1` |
| Bounded 2 | `5258b07f09f193a62ae030b8ac10ff4e23d1b09a8d0280684f9f075a730d615e` |
| Perf/private XDG | `c5b0df9a2aa18957822a7e51262082a22a322bdf7ec4b365a9808f47099eada9` |

These were executed in the preceding task. This documentation follow-up rechecks
those logs, not five newly executed runs. No more timing retries were performed.
A debugger can change scheduling; these passes cannot replace the failed ordinary
gate. Latest native PASS is recorded separately, without a confirmed crash fix.

## Next decisive evidence, if it recurs

Retain the exact failing binary/GTK/GStreamer/plugin hashes, raw exit, thread and
full stack. Use the existing `lifecycle.py --debug-mixed` / `context_trace.gdb`
with the same private candidate runtime environment, keeping
`--return-child-result`, sidecar and terminal signal records. No inferior EGL/GL
calls or state changes are needed. The A/B runner's printed build/report paths
identify that environment. Do not silently run system GTK instead of the candidate.

With a failing stack, distinguish a pending GDK/GStreamer callback or texture
lifetime from an action/GTK harness issue or plugin/driver fault. If the crash
cannot be reproduced under GDB, preserve the ordinary log and use a user-enabled
core collector on the native system; Pivra must not change system core policy.
Do not declare a specific race, ABI conflict, Mesa fault, or patch independence
without that evidence. Do not increase timeouts, suppress signals or change
production GL ownership speculatively.

Release impact: this remains a separate unresolved observation. It is **not**
the older `/colorstate/convert-dmabuf/srgb/srgb` SIGBUS and is **not** implicitly
accepted by the owner's historical preload-budget risk decision. Explicit owner
review is required before treating it as a nonblocking v0.2.6 risk.
