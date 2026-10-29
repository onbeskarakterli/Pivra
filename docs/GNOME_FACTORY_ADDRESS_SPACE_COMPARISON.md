# GNOME factory RLIMIT_AS investigation

> Current policy: 2 GiB soft/hard AS, following the user-reported native A/B/A
> result below. The original investigation and experiment history is retained.

## Confirmed origin; remaining causal test

The 1 GiB limit is **Pivra policy**, not a GNOME factory default:
`native/thumbnail_factory.c::main()` initializes both `rlim_cur` and `rlim_max`
to `1024UL*1024*1024`, then calls `setrlimit(RLIMIT_AS, &memory)` before creating
the factory. Limits are inherited by spawned thumbnailer/bwrap descendants.
Official GNOME 51.0 `gnome-desktop-thumbnail.c` and
`gnome-desktop-thumbnail-script.c` reviewed in the preceding investigation do
not set this address-space ceiling. Other inherited OS/session/cgroup limits
can still exist; this is not a statement that GNOME has no sandbox limits.

The user observed `Glycin Thumbnailer: Failed to allocate memory` and
`Failed to map fd: mmap() failed: Cannot allocate memory` on the 3294×3440 JPEG
under the helper. Direct system thumbnailer success is user-provided native
evidence. ENOMEM alone does not distinguish RLIMIT_AS, memory pressure,
process/container limits, or other mapping failures. The actual JPEG is not
available in Codex. Its specific RLIMIT_AS causality remains **NOT PROVEN here**.

## Controlled native A/B/A experiment

```bash
bash tests/run_gnome_factory_as_comparison.sh "/absolute/path/2023-08-12_15petq5_1.jpg" >factory-as-comparison.log 2>&1
```

Prerequisites: Arch GNOME desktop-4 development API, pkgconf, C compiler, Python,
GIO and a namespace-capable native session. No install or root privilege needed.

The driver:

1. Builds a baseline helper from the current exact C source and a diagnostic
   copy changing **only** the address-space initializer to 2 GiB. The source
   transformation refuses any missing/duplicated/unexpected initializer and
   accepts only 1024 or 2048 MiB. No runtime override or compile-time switch is
   added to the installed helper.
2. Runs 1 GiB → 2 GiB → 1 GiB, sequentially, using the same source URI and GIO
   MIME. Every trial gets fresh private cache and temporary directories; data,
   config and session thumbnailer selection remain inherited. No shared cache
   or installed application/helper is replaced. No direct unsandboxed decoder
   call or system-security change is made.
3. Retains AS soft=hard, CPU=15s, FSIZE=256MiB, source bound=64MiB, source
   identity/output guards and GNOME's existing bwrap/seccomp flow. Each trial
   has a 20-second deadline and process-group kill/reap on timeout/interruption.
   The experiment never uses unlimited AS. Only one trial runs at a time.
4. Logs actual configured AS, command and child error via the existing native
   diagnostics. The native log now prints the configured resource structs
   rather than hardcoded numbers, so the experimental 2 GiB run cannot falsely
   report 1 GiB. Production values remain unchanged.
5. Checks source dev/inode/size/mtime/ctime and SHA-256 before/after trials.
   Records exit, allocation-error evidence, monotonic elapsed time and Linux
   wait4 resource usage. `helper_tree_maxrss_kib` is the helper/reaped-descendant
   high-water metric, **not** a simultaneous process-tree RSS sum or an
   individual Glycin loader measurement. Virtual address allowance is not RSS.

The driver reports file/environment-specific RLIMIT_AS-dependent **evidence**
only if both 1 GiB trials exit 4 with allocation errors, the 2 GiB trial exits 0
with `generated`, all actual limits match, sandbox command evidence exists,
no trial times out, and source identity stays unchanged. Inspect the expanded
commands as well: the same thumbnailer, MIME and sandbox must have been used.
Any other pattern is `NOT PROVEN` (exit 1), not a hidden PASS. Missing native
GNOME prerequisites yield `NOT RUN` (exit 77). This experiment does not claim a
universal 2 GiB fix or authorize a production limit change automatically.

A/B/A reduces the risk that ordinary warm-up or transient load explains the
result, but cannot eliminate changing background memory/session configuration.
The fresh caches keep an earlier generated thumbnail from bypassing decode.

## Permanent solution and fallback assessment

No production limit increase, retry increase or fallback is applied without
native comparison evidence. If this file demonstrates AS sensitivity, evaluate
a bounded policy using additional representative files and process memory
measurements; 2 GiB is an experimental alternative, not a measured universal
minimum. Retain CPU/time/output/source/worker limits and GNOME sandboxing.

The existing `freedesktop_generator::generate_admitted()` is technically a
possible secure generation adapter: it reuses mandatory Glycin Bwrap and the
shared admitted permit, cancellation/deadline/source identity, thumbnail memory
reservation and single-large-job guard. This is full secure decode then
resampling, not an early or reduced-cost decode. It has independent limits and
is not guaranteed to succeed merely because the system thumbnailer does.

A future file-specific recovery could reuse that adapter under the existing
GNOME session selection, without changing GNOME→Freedesktop session priority.
However current helper exit 4/GError does not provide a trustworthy typed
memory-exhaustion reason; matching arbitrary child stderr is insufficient for
a permanent routing policy. Such recovery would need an explicit bounded
classification/admission design, at most one generation path at a time, no
permanent-provider failure on a file error, source/cancellation checks and
native GNOME/non-GNOME regressions. It is **researched, not implemented** here.

## Codex validation — 2026-10-28

- Native comparison: **NOT RUN**, exit 77; GNOME desktop-4 development API
  absent. No attempts to bypass Bwrap; the actual problem JPEG is unavailable.
- New controlled kernel test uses real `mmap(PROT_NONE)` to reserve 1200 MiB
  of virtual address space, without allocating that amount of physical RAM.
  The real native helper with a stub GNOME API fails under 1 GiB, succeeds
  under 2 GiB, then fails under 1 GiB. CPU/FSIZE checks stay enforced. **PASS**
  validates the experimental mechanism, not the JPEG decoder or native cause.
- Classification negative tests reject missing repeat, absent allocation
  evidence, wrong effective limits, timeout, absent sandbox evidence, failed
  higher-limit generation and changed source. Source transformation guards
  reject arbitrary limits or changed helper source. **PASS**.
- Full thumbnail Python suite: 48 PASS. Rust regression: 197 PASS.
- fmt/check/Clippy: PASS; existing Clippy warnings unchanged.
- Bash syntax: PASS. Production AS remains 1 GiB. GNOME/fallback selection,
  sandbox, installed helper, packaging, main, release, tag, AUR and CI unchanged.

Native causality and permanent remediation remain OPEN until the comparison
report is available. Do not interpret these controlled tests as Arch acceptance.


## Production correction — 2026-10-28

**User-reported real Arch/GNOME Wayland acceptance of the causal experiment**:
1024 MiB fails with `mmap() failed: Cannot allocate memory`, 2048 MiB generates
the thumbnail, then 1024 MiB fails again. The user confirmed identical JPEG,
MIME, thumbnailer and sandbox command; CPU=15s, FSIZE=256MiB and deadline=20s
were unchanged, and source integrity was preserved. Reported helper-tree max
RSS was approximately 27 MiB. These are user-supplied results, not a Codex native
run. That high-water metric is not evidence of a simultaneous RSS sum or a
universal memory bound for other images.

The production initializer now sets **AS soft=hard=2147483648 bytes** instead
of 1073741824. This is a bounded increase in virtual mapping allowance, not a
2 GiB physical allocation or RSS limit. It addresses the demonstrated mapping
compatibility issue on Arch x86_64 while allowing a greater per-process address
space and, potentially, greater memory use. The cap remains finite; this is
not a new aggregate process-tree memory guarantee. Larger inputs still depend
on existing admission/source/output/time controls. No automatic retry increase
or decoder/provider fallback is introduced.

Unchanged: CPU soft/hard 15s, FSIZE soft/hard 256MiB, parent 20s deadline,
process-group cancellation/reap, admission/dedup/concurrency, source identity,
output checks, GNOME-first selection and Freedesktop selection rules, native
GNOME Bubblewrap/seccomp sandbox. `setrlimit` still fails closed: any failure
returns exit 2 before creating a GNOME factory. No privileged attempt to raise
an inherited hard limit or fallback to unbounded operation is made.

The diagnostic A/B/A driver was adjusted to the new production source: 2048
MiB is now the unmodified case and 1024 MiB the isolated lower-limit copy.
It still refuses arbitrary limits and still runs 1024→2048→1024 for historical
reproduction without editing the installed helper.

Controlled Codex verification:

- C helper built with real GLib/GIO/GdkPixbuf, stub GNOME API, `-Wall -Wextra
  -Werror`: verifies actual soft **and** hard AS=2GiB, CPU=15s, FSIZE=256MiB.
- Three real-kernel negative tests lower inherited hard AS/CPU/FSIZE separately;
  helper exits 2, creates no cache and never enters the stub GNOME API.
- Existing debug protocol, source-change, output-size and controlled VM A/B/A
  tests retained. These are not native JPEG decode acceptance.
- Python thumbnail suite: 52 PASS; Rust: 197 PASS. fmt/check/Clippy: PASS with
  pre-existing warnings. No GUI/render/decoder implementation is changed.
- Updated installed helper on real Arch and the 665-photo cold-cache/stress
  acceptance: **NOT RUN in Codex / pending user verification**. Rebuild and
  reinstall the native package so `/usr/bin/pivra-thumbnail-factory` is updated;
  pulling Rust sources alone does not replace that installed binary.

For a source-built native verification of the now-production 2GiB helper:

```bash
bash tests/run_gnome_factory_diagnostic.sh "/absolute/path/2023-08-12_15petq5_1.jpg" >factory-2gib.log 2>&1
```

Confirm `limits_as=2147483648`, retained CPU/FSIZE, and `generated` / exit 0.
Then validate the rebuilt installed application with the same JPEG and the
large mixed-size photo folder: visible thumbnail completion, scroll to new
items, cancellation/close and unchanged source files. Do not delete the user's
shared thumbnail cache to arrange a test; use a private test cache if needed.
