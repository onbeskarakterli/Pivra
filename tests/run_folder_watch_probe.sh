#!/usr/bin/env bash
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"
test "$(git branch --show-current)" = dev
git rev-parse HEAD
cargo build --locked --example folder_watch_probe --target-dir "$repo/target"
export GDK_BACKEND=wayland
export G_DEBUG=fatal-criticals
exec "$repo/target/debug/examples/folder_watch_probe" "$@"
