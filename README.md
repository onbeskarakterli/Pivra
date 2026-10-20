# Pivra
A fast, lightweight GNOME image viewer with seamless photo and video navigation, smart zoom, and basic editing.

## Development status

PIV-01 bootstrap is in progress. The current Rust binary is a CLI scaffold only; the GTK4/libadwaita window will be introduced in PIV-02. No image or video functionality is implemented yet.

Requires Rust toolchain with Rust 2024 edition support. Once available, validate with:

```sh
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help
cargo run -- --version
cargo run --
cargo run -- --unknown
```

The initial PIV-01 environment did not have `cargo` or `rustc`. During the subsequent Work audit on 2026-10-20, the execution service returned `No usable execution environment is available` on both attempts, before any shell command could start. Rust/Cargo versions and installation availability could not be checked; the commands above remain **BLOCKED, not executed**. No build, unit-test, formatting, lint, or CLI runtime result is claimed. PIV-01 remains incomplete.

Source review found a dependency-free Rust 2024 binary and three existing unit tests. The CLI source handles help/version aliases, prints a scaffold message with no arguments, and exits with code 2 for invalid arguments; these are source observations, not runtime verification. Once execution is available, verify exit codes and stdout/stderr as well as successful compilation. `cargo run -- --unknown` must return a nonzero exit code and a controlled error.

The existing feature specification and roadmap contain historical pre-code status wording even though the PIV-01 scaffold now exists. Their accepted product decisions are unchanged; current implementation and verification status is recorded in the commit tracker.

See [development roadmap](docs/PIVRA_DEVELOPMENT_ROADMAP.md) and [commit tracker](docs/PIVRA_COMMIT_TRACKER.md). License selection is pending and must not be assumed.
