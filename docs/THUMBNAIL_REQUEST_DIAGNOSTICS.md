# Thumbnail request stalls: evidence, bounded retry and diagnostics

Baseline: dev `66ca07c4ac0ddbacb9998100e76f730b54b734ae`.
User evidence: cold GNOME folders sometimes leave thumbnails absent; Nautilus
can populate them before a Pivra restart. The exact native failure is still
**unconfirmed** without a trace from the updated build.

## Confirmed code findings

- Existing PIVRA_TRACE_GNOME_THUMB messages were downstream of scheduler and
  provider admission. An empty helper log did not prove no request existed.
  RUST_LOG alone does not initialize a subscriber for these scheduler stages.
- Actual GNOME helper timeout previously became a generic Invalid outcome and
  terminal Failed cell, losing the distinction between temporary generation
  failure and bad/unsupported data. The existing decoder/cache error detail was
  often discarded by the strip. This terminal behavior is reproducible with a
  controlled helper; it is not proof that the user's exact files time out.
- Busy is fail-fast admission (global two-worker/path permit limit), not a
  completed generation attempt. Existing Pending/re-admission is preserved.
- Hidden prewarming is deliberately a bounded initial viewport batch (up to
  24 entries), not a promise to generate an entire hidden folder.

## Policy

Only typed TimedOut / ProviderTemporarilyUnavailable work gets **one** automatic
retry. A visible transient failure returns to Pending and yields 250 ms through
the existing scheduler. A second transient failure remains terminal. Hidden
transient completion shares the same per-entry retry budget before handing off
to a later visible admission. Unknown/format/permission/unsupported/source/limit
failures stay terminal; no string guess classifies an unknown decoder failure
as temporary. A GNOME helper deadline is explicitly distinguished from external
cancellation, and mapped to TimedOut. Cancellation and stale callbacks retain
their prior token/epoch behavior and do not spend a failure retry.

Busy never changes the failure budget. Capacity release permits still-required
selected/visible Pending work to be admitted again; no duplicate decoder queue
is created. Paint, scroll and re-selection do not reset retry rights. Existing
real source/cache invalidation or explicit invalidation may establish a new
request. Global two-worker and single-large-job admission, texture 64 / 16 MiB,
source identity, Bwrap, provider priority and helper cleanup remain unchanged.

## Capture on Arch

Rebuild/install the binary containing these changes first. Then use private
test copies in a cold folder; do not delete the user's shared thumbnail cache.

```bash
log=$(mktemp /tmp/pivra-thumbnail-trace.XXXXXX)
PIVRA_TRACE_THUMBNAILS=1 PIVRA_TRACE_GNOME_THUMB=1 \
  /usr/bin/pivra '/absolute/path/to/a/test-photo.png' 2>"$log"
printf 'Trace: %s\n' "$log"
```

Either trace variable enables the unified PIVRA_THUMB stages. Logs contain paths;
redact them when sharing. Each process emits at most 4096 events plus a limit
notice, with bounded/escaped fields and no retained in-memory event history.
Restart to capture a new session if the limit is reached. Existing optional
helper-stderr capture remains private to the helper invocation; sandbox limits
were not changed. No logger/decoder dependency was added.

Stages: catalog -> scheduler-state (closed/unmapped/building/initial/validation/
retry yield) -> admit-selected/visible/background -> worker-start -> resolve-start
(provider and active permits) -> admission-busy or GNOME/cache/helper stages ->
resolve-result -> worker-result -> hit, bounded failure-retry or terminal-failure.
Hidden outcomes and stale/identity discards are also logged. Sequence numbers,
monotonic elapsed milliseconds, paths, strip identity and worker thread IDs help
correlate multiple windows. Pixel buffers are not logged.

Record the last stage for each missing item, actual helper availability/result,
provider, whether the strip was mapped, and whether a timeout/retry occurred.
Keep the strip open until visible items finish; scroll to another region, then
return. Confirm recovery needs no Nautilus/restart. Check the other window,
close during work, malformed/permanently unreadable data, and resource cleanup.
All of this real Arch/secure decode acceptance is **NOT RUN in Codex**.

## Controlled regression scope

`thumbnail_overlay_probe --thumbnail-scheduler-regression` uses private cached
fixtures, actual GTK/GIO/permit/scheduler objects, a synthetic cache decoder and
a controlled helper subprocess. It requires XDG_CURRENT_DESKTOP=GNOME and a
nonexistent helper-fixture filename under the private XDG_CONFIG_HOME. The
probe creates it exclusively, never overwrites an existing helper. It times out
once using the production 20-second deadline, then copies a prebuilt TEST cache;
this is explicitly not GNOME generation or actual pixel decode evidence.
It verifies admission blocking, drain, viewport/scroll completion, one temporary
retry, no permanent-error retry on re-selection, and bounded memory/permits.

The existing headless Sway driver supports `--navigation-probe
--thumbnail-scheduler` and creates those private test prerequisites. X11 uses
its existing Xvfb/D-Bus environment with a private test root. Unit tests cover
error classification, retry exhaustion and bounded/escaped logging. Existing
catalog/cancellation/81-and-120 retention and Python helper tests are preserved.
Actual run results and commit IDs are in the delivery report; no native result
is inferred from these controlled tests.

## This change's measured results

Rust fmt/check/test PASS: 190 Rust tests, zero failures. Python thumbnail/helper
suite: 41 PASS. Controlled scheduler test PASS under X11 and headless Sway;
controlled catalog, cancellation race and 81/120 retention PASS under X11.
The real helper deadline was exercised by a sleeping TEST helper: timeout at
about 20 seconds, one retry after at least 250 ms, then synthetic-cache recovery.
The permanent TEST decode error ran once and stayed terminal after re-selection.
No native GNOME/Glycin success is inferred. Clippy all-targets passed with the
existing unused load_stats and identical pin-icon branch warnings.

Hidden batch ownership is tracked (at most 24 paths) until its outcomes and
retry counters are published. Visible admission can process other paths, but
cannot duplicate that batch's still-unpublished requests. After stale hidden
completion releases ownership, the visible scheduler is explicitly woken.
This closes an observable scheduling gap without changing worker/memory limits;
the user's original Arch failure still needs the updated native trace.
