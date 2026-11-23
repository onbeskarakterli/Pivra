# Task 01C follow-up — GDB stop classification and EGL handle ownership

Status: **native AMD acceptance remains BLOCKED** until all nine gates pass.
Base: `f1cc9da7cca657005716c9c36804c321d16df5a0`. Recorded 2026-11-17.

## What the new native excerpt establishes

The user reported eight PASS, one FAIL, raw exit 255, last transition
`LIFECYCLE_STAGE sRGB stock route begin`, and:

```text
#0 eglDestroyContext()
#1 libgstgl-1.0.so.0
#2 libgstgl-1.0.so.0
#3 libglib-2.0.so.0
```

The old GDB script intentionally broke at `eglDestroyContext` and printed `bt`.
This backtrace can therefore be observer output during normal teardown. The
excerpt contains no inferior exit event, signal event, EGLDisplay/EGLContext
values or complete debugger error. **It does not establish an application crash,
premature context destruction, or EGLDisplay termination.** The actual native
stop reason cannot be reconstructed from these four frames alone.

The earlier non-debug native SIGABRT remains unresolved; this report does not
dismiss it. Likewise, the separately reproduced stock-EGL ownership mechanism in
[the preceding investigation](V026_NATIVE_AMD_CONTEXT_FAILURE.md) remains a
control/source finding, not a root-cause verdict for this latest native trace.
`eglDestroyContext` and `eglTerminate` are different operations.

## Reproduction with a real debugger

GDB 16.3 was obtained from official Debian packages, SHA-256 checked against
the existing package index, and extracted under a user-owned SDK. System GTK
and system packages were not replaced. Controlled C functions with EGL names
test debugger behavior only; they do not render, decode, or emulate a GPU.

An unmanaged intentional breakpoint at the C control's `eglDestroyContext`,
followed by batch exit without continuation, returned **255**. The program did
not reach its subsequent marker and had not aborted. A real application returning
255 is a separate control and remains FAIL when its exit is confirmed.
This proves that raw debugger exit 255 is ambiguous, not that it explains the
user's particular run.

During development, Python `FinishBreakpoint` return observers triggered an
actual GDB 16.3 internal assertion in a software GTK/GStreamer run:
`find_program_space_for_breakpoint: Assertion 'thr != nullptr' failed`.
The debugger process aborted without an inferior result. The new runner correctly
kept application outcome UNKNOWN. These thread-bound temporary observers were
removed. Final return snapshots use shared permanent call-site breakpoints,
numeric thread IDs and return PCs, without retaining a frame/thread object.
The full software lifecycle subsequently completed under GDB.

## Runner changes

- EGL entry observers return `False` from `Breakpoint.stop()`, automatically
  continuing. Unwind/ABI extraction errors are recorded without cancelling a
  CLI `bt; continue` command list. No GTK/GL function is called in the inferior.
- Signals and inferior exits are observed separately. The control driver quits
  outside stop callbacks, avoiding reentrant `quit` from `gdb.events.stop`.
- A flushed `mixed-lifecycle.gdb.jsonl` sidecar records bounded entry/return events,
  monotonic timestamps, PID, GDB thread and native LWP, typed event kind and one
  final inferior result. Human-readable copies remain in the complete log.
- `report.json` preserves raw `debugger_exit`, `exit_scope`, the verified inferior
  result, debugger version and separate application/debugger status. Missing,
  duplicated, malformed or mismatched terminal evidence is BLOCKED/UNKNOWN,
  never an application PASS or an inferred application crash.
- Confirmed inferior SIGABRT/SIGSEGV/SIGBUS/SIGILL is FAIL. A debugger SIGABRT
  without inferior evidence is BLOCKED. An unclassified/nonfatal debugger signal
  stop is not assumed to be fatal application termination. Real nonzero inferior
  exits remain FAIL. Existing GL warning, GPU error and resource assertions remain.
- `all_gates_pass` requires exactly nine PASS rows. BLOCKED returns 77, keeping
  acceptance open; no intentional-but-unfinished debugger stop can close it.

## Handle ownership and ordering evidence

With `PIVRA_TRACE_VIDEO`, Pivra now logs its GstGLContext object, sink, actual
native context/display, GDK share context/object, reference snapshot, PID and
Linux thread ID at registration/release. Weak retirement notifications remain
observational and do not take ownership or call GTK/GL on finalizer threads.
The only production edits in this follow-up are these optional diagnostic fields.

GDB records EGLDisplay, create-share/context arguments, caller modules/functions
and observed returns for `eglInitialize`, `eglCreateContext`, `eglDestroyContext`
and `eglTerminate`. Debug info is preferred for arguments; Linux x86-64 ABI
register fallback was exercised with minimal symbols. AArch64 fallback is
implemented but untested; unsupported ABIs yield unknown data, not guessed handles.
Return values that cannot be observed remain unknown. Calls are capped per symbol;
per-frame make-current breakpoints are not installed.

The report joins `(PID, EGLDisplay, EGLContext)` with registration/generation
timestamps and disposal records, separating Pivra workers from GDK share contexts.
It records the last display request separately from a successfully observed
initialize/terminate return. Later reinitialization and reused native handles
are covered by regression controls. Unmapped GTK/GStreamer contexts remain
unmapped; a module stack alone is not proof of logical ownership.

An EGL destruction request can be legitimate and deferred while a context is
current. Reference snapshots precede release; absence of a weak retirement
notification is not proof of a live reference. GDK native destruction may occur
before its weak notification is delivered. The report therefore emits facts and
owner candidates, **not automatic leak/use-after-free conclusions**.

The supplied native excerpt has no numeric handles, so no exact native owner or
prior display termination can be established retrospectively. The new trace is
needed for that correlation. A successful controlled run can contain the same
libgstgl/libglib destruction stack shape and still drain all resources normally.

## Verification and limitations

The controlled environment remains Debian 13 / GTK 4.18.6 / GStreamer 1.26.2 /
Mesa llvmpipe. GDB is real, not a mocked debugger. It reported read-only `/proc`
memory-file warnings in Codex but successfully debugged its own children using
the available mechanism; no ptrace/security policy was changed.

Tests cover automatic continuation through intentional breakpoints, normal exit,
nonzero exit including 255, actual SIGABRT, nonfatal signal stops, missing protocol,
debugger crashes, argument/handle extraction with minimal symbols, handle reuse,
display reinitialization and unknown ownership. EGL-named stub tests establish
debugger integrity only. Actual GTK/GStreamer runs separately retain shader,
playback, routing and resource-drain checks.

Executed against the base plus this follow-up tree:

| Check | Result |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo check --locked` | PASS |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS, zero warnings |
| `CARGO_INCREMENTAL=0 cargo test --locked` | PASS, 212 tests |
| Python suites: root / thumbnail / fd05a / video_color / video_gpu | PASS, 64 / 59 / 9 / 13 / 62 = 207 tests |
| Included real-GDB controls / report controls | PASS, 7 / 10 tests, none skipped |
| X11/Xvfb, llvmpipe, GDB mixed lifecycle | PASS, 9/9, inferior exit 0 |
| Headless Sway/Wayland, llvmpipe, GDB mixed lifecycle | PASS, 9/9, inferior exit 0 |
| `cargo build --release --locked` | PASS |
| Physical AMD/GNOME rerun | NOT RUN; acceptance BLOCKED |

Controlled GUI commands, with the existing SDK and stock media plugin activated:

```sh
bash tests/run_video_context_lifecycle.sh --session x11 --software --api opengl3 --debug-mixed
bash tests/run_video_context_lifecycle.sh --session sway --software --debug-mixed
```

Python suites were run with `python3 -B -m unittest discover -s <directory>`
for `tests`, `tests/thumbnail`, `tests/fd05a`, `tests/video_color` and
`tests/video_gpu`, with GDB on PATH. Shader/range controls and all context,
frame and presentation drain assertions remain enabled.

Exact executed results are in [the evidence manifest](V026_GDB_STOP_CLASSIFICATION.json).
The native AMD nine-gate result for this revision is **NOT RUN / BLOCKED**.
No system GTK change, CPU fallback, stock-route replacement, timeout increase,
version bump, release/AUR action or unrelated optimization is included.

## Native rerun

```sh
bash tests/run_video_context_lifecycle.sh --debug-mixed
```

Keep `report.json`, `mixed-lifecycle.log` and `mixed-lifecycle.gdb.jsonl`. Normal
intentional observer breakpoints should continue to a verified inferior exit.
If they do not, the case must remain BLOCKED with application outcome UNKNOWN.
Only nine actual PASS results can satisfy the native gate. Native AMD/GNOME,
Intel/NVIDIA and physical visual acceptance are not inferred from llvmpipe.
