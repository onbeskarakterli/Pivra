#!/usr/bin/env bash
# Dependency metadata only: no compilation, pipeline, image load or app edits.
set -euo pipefail
probe_dir=$(mktemp -d "${TMPDIR:-/tmp}/pivra-media-resolution-XXXXXXXX")
trap 'rm -rf "$probe_dir"' EXIT
cat > "$probe_dir/Cargo.toml" <<'MANIFEST'
[package]
name = "pivra-media-resolution-probe"
version = "0.0.0"
edition = "2024"
rust-version = "1.85"
[lib]
path = "lib.rs"
[dependencies]
gtk = { package = "gtk4", version = "=0.10.3", features = ["v4_6"] }
adw = { package = "libadwaita", version = "=0.8.1", features = ["v1_2"] }
glycin = { version = "=3.0.8", features = ["gdk4"] }
gstreamer = "=0.24.5"
MANIFEST
printf '%s\n' '// Metadata-only probe; no media implementation.' > "$probe_dir/lib.rs"
cargo +stable generate-lockfile --manifest-path "$probe_dir/Cargo.toml"
python3 - "$probe_dir/Cargo.lock" <<'PY'
import json
import sys
import tomllib

lock = tomllib.load(open(sys.argv[1], "rb"))
names = ["gtk4", "libadwaita", "gdk4", "glib", "gio", "glycin",
         "glycin-common", "glycin-utils", "gstreamer"]
versions = {name: [p["version"] for p in lock["package"] if p["name"] == name]
            for name in names}
assert versions["gdk4"] == ["0.10.3"], versions
assert all(v.startswith("0.21.") for v in versions["glib"] + versions["gio"]), versions
print(json.dumps({"scope": "RESOLVED ONLY; not compiled or runtime-tested", "versions": versions}, indent=2))
PY
