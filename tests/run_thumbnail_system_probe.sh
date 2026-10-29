#!/usr/bin/env bash
# Real GNOME generation + secure Glycin cache/main pixels. Private TEST media only.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
cargo build --locked --example thumbnail_overlay_probe
probe=target/debug/examples/thumbnail_overlay_probe
"$probe" --require-generation-helper
probe_root=$(mktemp -d /tmp/pivra-th05-native.XXXXXX)
trap 'rm -rf -- "$probe_root"' EXIT
export XDG_CONFIG_HOME="$probe_root/config" XDG_CACHE_HOME="$probe_root/cache"
mkdir -p "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME"
export GDK_BACKEND=wayland G_DEBUG=fatal-criticals
# Explicit synthetic large-cache retention regression, then real secure acceptance.
"$probe" --retention-regression
"$probe" --catalog-regression --native-thumbnails --manual
