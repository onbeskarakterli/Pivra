#!/usr/bin/env bash
# Native secure gates; the active-close observer remains the PIV-09 checkpoint.
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"
test "$(git branch --show-current)" = dev
git rev-parse HEAD
cargo build --locked --example image_probe --example async_cancel_probe --example active_decode_close_probe --target-dir "$repo/target"
export GDK_BACKEND=wayland
export G_DEBUG=fatal-criticals
"$repo/target/debug/examples/image_probe"
"$repo/target/debug/examples/async_cancel_probe"
exec python3 -B tests/active_decode_close.py --binary "$repo/target/debug/examples/active_decode_close_probe" --report /tmp/pivra-async-active-close.json
