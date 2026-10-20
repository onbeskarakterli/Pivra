# Pivra
A fast, lightweight GNOME image viewer with seamless photo and video navigation, smart zoom, and basic editing.

## Development status

**PIV-01: DONE — verified on 2026-10-21, on `dev`.** The dependency-free Rust 2024 CLI scaffold builds and passes its existing tests. PIV-02 has not started: no GTK window, image/video functionality, media engine or CI is included.

## Development and validation

Use a Rust stable toolchain supporting the Rust 2024 edition, with rustfmt and Clippy. The prepared cloud environment can be activated in each shell with:

```sh
source /workspace/pivra-environment/activate.sh
cd /workspace/Pivra
```

That activation path is specific to the prepared cloud environment; other machines can use their own Rust stable installation. Work on `dev`; do not change `main`.

```sh
cargo +stable fmt --check
cargo +stable check --offline --locked
cargo +stable build --offline --locked
cargo +stable test --offline --locked
cargo +stable clippy --offline --locked --all-targets -- -D warnings
cargo +stable run --offline --locked --quiet -- --help
cargo +stable run --offline --locked --quiet -- --version
cargo +stable run --offline --locked --quiet --
cargo +stable run --offline --locked --quiet -- --unknown
```

The last command intentionally exits **2** with a controlled error; do not treat it as an unexpected test failure. The other commands exit 0.

### Verified results — 2026-10-21

- Toolchain: rustc 1.99.1, Cargo 1.99.1, rustfmt 1.10.0-stable, Clippy 0.1.99.
- Format, check, build and Clippy: PASS, exit 0; Clippy treats warnings as errors.
- Unit tests: **3 passed, 0 failed, 0 ignored**.
- Eight CLI scenarios: PASS, exact stdout/stderr and exit codes checked. `--help`/`-h` print usage (exit 0); `--version`/`-V` print `Pivra 0.1.0` (exit 0); no arguments print `Pivra: graphical application is not yet implemented` (exit 0). `--unknown`, `one two`, and `--help --version` print `Pivra: invalid arguments; use --help` to stderr, leave stdout empty, and exit 2.
- Compilation and CLI validation used a separate initially empty target directory, `/tmp/pivra-piv01-validation`, with the checked-in dependency-free Cargo.lock and offline/locked commands.

The validated source is commit `86a54f7094639d6a75a0c8798d8b48e7a65e8f12`; this closure only adds Cargo.lock and updates documentation. Actual closure commit SHAs are recorded in the [commit tracker](docs/PIVRA_COMMIT_TRACKER.md), with the final documentation commit available from its Git history.

Earlier audits on 2026-10-20 were BLOCKED by the unavailable execution environment; those historical results are preserved in the tracker and are superseded by the successful run above. The feature specification and roadmap also contain historical pre-code wording; their accepted product decisions remain unchanged.

See the [development roadmap](docs/PIVRA_DEVELOPMENT_ROADMAP.md), [feature specification](docs/PIVRA_FEATURE_SPEC_TR.md), and [commit tracker](docs/PIVRA_COMMIT_TRACKER.md). License selection remains open; no license is assumed. GUI/GNOME validation belongs to PIV-02 and later tasks, not this CLI bootstrap closure.
