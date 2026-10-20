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
```

The development environment used to prepare PIV-01 did not have `cargo` or `rustc`, so these commands have **not** yet been executed. Do not mark PIV-01 as completed before verification.

See [development roadmap](docs/PIVRA_DEVELOPMENT_ROADMAP.md) and [commit tracker](docs/PIVRA_COMMIT_TRACKER.md). License selection is pending and must not be assumed.
