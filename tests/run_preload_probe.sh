#!/usr/bin/env bash
# Native secure gates only: never substitute controlled data for decoder success.
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"
test "$(git branch --show-current)" = dev
git rev-parse HEAD
cargo build --locked --example image_probe --example preload_probe --example active_decode_close_probe --target-dir "$repo/target"
export GDK_BACKEND=wayland
export G_DEBUG=fatal-criticals
"$repo/target/debug/examples/image_probe"
"$repo/target/debug/examples/preload_probe"
# Independently certify real in-progress secure decode and child cleanup.
exec python3 -B tests/active_decode_close.py --binary "$repo/target/debug/examples/active_decode_close_probe" --report /tmp/pivra-preload-active-close.json
