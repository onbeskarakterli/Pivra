#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
cargo build --locked --example thumbnail_overlay_probe
probe_root=$(mktemp -d /tmp/pivra-fd06-native.XXXXXX)
trap 'rm -rf -- "$probe_root"' EXIT
export XDG_CONFIG_HOME="$probe_root/config" XDG_CACHE_HOME="$probe_root/cache"
export XDG_CURRENT_DESKTOP=Pivra-FD06-Test GDK_BACKEND=wayland G_DEBUG=fatal-criticals
mkdir -p "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME"
# Process-only desktop snapshot; system GNOME/factory packages untouched.
target/debug/examples/thumbnail_overlay_probe --require-freedesktop --catalog-regression --native-thumbnails --manual
python3 tests/fd06/active_close.py
