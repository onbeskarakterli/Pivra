#!/usr/bin/env bash
# Real GNOME Wayland acceptance. Private TEST media/config/cache; no sandbox bypass.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
cargo build --locked --example image_probe --example thumbnail_overlay_probe
probe_root=$(mktemp -d /tmp/pivra-th07-native.XXXXXX)
trap 'rm -rf -- "$probe_root"' EXIT
export XDG_CONFIG_HOME="$probe_root/config" XDG_CACHE_HOME="$probe_root/cache"
export GDK_BACKEND=wayland G_DEBUG=fatal-criticals
mkdir -p "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME"
# A failed secure prerequisite stops here; exit77 never becomes PASS.
target/debug/examples/image_probe
target/debug/examples/thumbnail_overlay_probe --fullscreen-regression --native-thumbnails --manual
