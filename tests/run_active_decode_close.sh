#!/usr/bin/env bash
# Native GNOME Wayland entry point: build the matching probe, then observe it.
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"
test "$(git branch --show-current)" = dev
git rev-parse HEAD
cargo build --locked --example active_decode_close_probe --target-dir "$repo/target"
export GDK_BACKEND=wayland
exec python3 -B tests/active_decode_close.py \
    --binary "$repo/target/debug/examples/active_decode_close_probe" \
    --report /tmp/pivra-active-decode-close.json "$@"
