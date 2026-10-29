#!/usr/bin/env bash
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"
test "$(git branch --show-current)" = dev
git rev-parse HEAD
cargo build --locked --example view_modes_probe --example image_probe --target-dir "$repo/target"
export GDK_BACKEND=wayland
export G_DEBUG=fatal-criticals
"$repo/target/debug/examples/image_probe"
exec "$repo/target/debug/examples/view_modes_probe" --zoom-pan "$@"
