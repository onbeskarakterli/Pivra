# GNOME factory JPEG failure investigation

> Follow-up: the user has confirmed RLIMIT_AS causality by native A/B/A.
> Production now uses a bounded 2 GiB AS limit; see the production correction
> in [the address-space record](GNOME_FACTORY_ADDRESS_SPACE_COMPARISON.md).
> The original evidence and diagnostic history below is preserved.

## Status and evidence

User-reported Arch/GNOME failure: `2023-08-12_15petq5_1.jpg`, baseline 8-bit
JFIF, 3294×3440, 675923 bytes. Direct system `glycin-thumbnailer` succeeds;
Pivra's installed GNOME adapter exits 4 with
`failed:Child process exited with code 1`. This repeats without folder load.
The failing source image and user's thumbnailer registration files were not
available in Codex. The exact native decoder failure is **NOT YET PROVEN**.
A scheduling-only explanation is insufficient for this evidence.

## Confirmed loss of diagnostic information

Examined official GNOME **51.0**, not an assumed Rust Glycin API:

- [thumbnail factory source](https://github.com/GNOME/gnome-desktop/blob/51.0/libgnome-desktop/gnome-desktop-thumbnail.c)
- [script/sandbox source](https://github.com/GNOME/gnome-desktop/blob/51.0/libgnome-desktop/gnome-desktop-thumbnail-script.c)
- [log domain](https://github.com/GNOME/gnome-desktop/blob/51.0/libgnome-desktop/meson.build)

`gnome_desktop_thumbnail_script_exec()` calls `g_spawn_sync()` with an
`error_out` capture. `g_spawn_check_wait_status()` supplies the generic child
exit GError. The actual child stderr is emitted only by
`g_debug("Failed to launch script: ...")` in domain `GnomeDesktop`.
The expanded command is likewise a DEBUG message, `About to launch script:`.
Capturing the helper's ordinary stderr alone therefore misses both messages.
`RUST_LOG=debug` does not enable these GLib debug records.

Pivra's native helper also limits its GError stdout message to 160 characters.
Rust previously read only the first 4096 bytes of helper stderr; the lifecycle
logger further caps each event at 512 characters. A long expanded command
could hide the later child error. These are confirmed observation problems,
not proof of the image's decoding failure cause.

## Actual call path and differences from a direct thumbnailer call

1. Production requests GIO `standard::content-type`, converts it to MIME,
   captures source identity, then invokes the adapter with URI, MIME, modified
   time in seconds, and `missing` or `stale` mode. The new `helper-input` trace
   includes the actual MIME, source byte size and modified time.
2. Native adapter creates GNOME NORMAL-size factory, checks current lookup and
   eligibility, calls `generate_thumbnail(uri, mime, NULL, &error)`, verifies
   output/source identity, then saves via the GNOME API. Exit 4 is a generation
   failure, before Pivra's output-size/source-after-decode/save guards.
3. GNOME may first use a GIO preview icon. Otherwise its MIME map chooses a
   registered thumbnailer. Registration is from user XDG data thumbnailers,
   then system XDG data thumbnailers, subject to TryExec/disabled settings.
   The first accepted registration for a MIME wins; multiple candidate files
   do not establish which executable actually ran.
4. The chosen Exec template is expanded with the actual URI/path, output,
   MIME and requested size. On ordinary native Linux GNOME uses its own bwrap
   sandbox: read-only runtime/input, private output, namespaces, seccomp and
   die-with-parent. This is an additional execution context compared with a
   direct command. The exact selected command must come from native logs.
5. The adapter's inherited limits are unchanged: AS 1 GiB per process, CPU
   15 seconds, file/memfd size 256 MiB; Rust retains its 20-second deadline,
   process-group termination/reaping, source bounds and identity checks.
   Direct manual runs need not inherit these same limits/TMPDIR/cache context.
6. System Glycin 2.2.4/thumbnailer and the application's Rust Glycin 3.0.8 are
   separate components. Having a working Rust loader or direct thumbnailer is
   not proof that GNOME selected that same thumbnailer with the same arguments.

The example has 11,331,360 pixels (45,325,440 bytes for one RGBA buffer), below
256 MiB. That arithmetic does **not** establish peak memory or address-space
usage. AS/CPU exhaustion, a different MIME registration, unavailable sandbox
resources, or a thumbnailer-specific failure remain hypotheses. Limits,
provider selection, retry classification and sandbox policy are not changed.

## Implemented diagnostic-only change

Opt-in via either `PIVRA_TRACE_GNOME_THUMB` or `PIVRA_TRACE_THUMBNAILS=1`:

- Native handler captures the two GNOME DEBUG messages to stderr, never stdout.
  Up to 16 records, each at most 2048 input bytes, escaped to prevent line
  injection; noisy unrelated DEBUG messages are ignored.
- Adds effective-limit/MIME context and generate/save GError domain/code.
- Rust captures diagnostics for both switches, preserves bounded head/tail
  (8 KiB each when long), and emits source-associated 384-character chunks
  within the existing 4096-event budget. Middle truncation is explicit.
- Normal stdout success/failure protocol and terminal failure behavior remain
  unchanged. Unknown exit 4 is not reclassified as a transient retry.

This does not fix the unobserved native failure by guessing or raising limits.

## Native diagnostic command

From updated dev on Arch (C compiler, pkgconf, Python, GIO and GNOME desktop-4
headers installed):

```bash
bash tests/run_gnome_factory_diagnostic.sh "/absolute/path/2023-08-12_15petq5_1.jpg" >factory-diagnostic.log 2>&1
```

This compiles **the updated real native helper**, no installation needed. It
keeps session config/data registration and source URI, queries GIO MIME, lists
MIME registration candidates, uses private cache/temp directories and the same
resource limits/20-second timeout. It never invokes a decoder unsandboxed or
changes shared cache/settings/source files. It prints the actual GNOME command,
underlying child error when GNOME exposes it, GError and final helper status.
A private cache also avoids an existing cached thumbnail masking generation.
Candidates alone are not asserted as actual selection. Exit 0 means this
GNOME helper generation succeeded; 1 means failure, 77 means prerequisites
unavailable. A private-cache result can differ from normal cache/session
behavior and must be compared with application traces, not treated as universal.

For the installed application, rebuild/reinstall the updated package first,
then capture `PIVRA_TRACE_GNOME_THUMB=1 /usr/bin/pivra /path/to/photo.jpg` stderr.
Do not remove shared failed markers or modify GNOME registration to force PASS.
If native output still lacks child detail, retain that as an evidence gap.

## Codex validation (2026-10-28)

- Real native GNOME diagnostic attempted once: **NOT RUN**, exit 77, GNOME
  desktop-4 development API unavailable. No repeated sandbox attempts.
- Python native-adapter regression suite uses real C/GLib/GIO/GdkPixbuf with a
  **stub GNOME API**, not actual source decode. Verifies unchanged resource
  limits/identity checks, opt-in log capture, unchanged success/error protocol,
  error domain/code, escaped messages and bounded verbose output.
- Rust regression covers bounded/non-UTF-8 stderr, preserving final failure
  after verbose output, existing helper protocol and process-group cancellation.
- `cargo fmt --check`, `cargo check --locked`: PASS.
- `cargo test --locked`: PASS, 197 tests.
- Python thumbnail tests: PASS, 45 tests, including 13 native adapter cases.
- `cargo clippy --locked`: PASS (exit 0); existing `load_stats` dead-code and
  identical pin-icon branch warnings remain.
- Shell and embedded Python syntax checks: PASS.
- Real Arch failure cause and native correction acceptance remain OPEN pending
  this diagnostic output.

## Follow-up: resource-limit origin and causal experiment

The user subsequently supplied real native allocation/mmap ENOMEM errors.
The 1 GiB ceiling is explicitly installed by Pivra's helper; the file-specific
causal comparison is tracked in
[GNOME_FACTORY_ADDRESS_SPACE_COMPARISON.md](GNOME_FACTORY_ADDRESS_SPACE_COMPARISON.md).
The original trace and diagnostic history above remains applicable.
