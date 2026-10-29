#!/usr/bin/env bash
# Requires an existing display. Build a disposable probe, never edit app Cargo files.
set -euo pipefail
repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
probe_dir=$(mktemp -d "${TMPDIR:-/tmp}/pivra-gtk-api-XXXXXXXX")
trap 'rm -rf "$probe_dir"' EXIT
mkdir "$probe_dir/src"
cp "$repo_root/tests/probes/gtk_api.rs" "$probe_dir/src/main.rs"
cat > "$probe_dir/Cargo.toml" <<'MANIFEST'
[package]
name = "pivra-gtk-api-probe"
version = "0.0.0"
edition = "2024"
rust-version = "1.85"
[dependencies]
gtk = { package = "gtk4", version = "=0.10.3", features = ["v4_6"] }
adw = { package = "libadwaita", version = "=0.8.1", features = ["v1_2"] }
MANIFEST
cargo +stable generate-lockfile --manifest-path "$probe_dir/Cargo.toml" --offline
cargo +stable run --manifest-path "$probe_dir/Cargo.toml" --locked --offline -j 2 \
    --target-dir "${PIVRA_PROBE_TARGET_DIR:-$repo_root/target}"
