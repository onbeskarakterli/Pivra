#!/usr/bin/env bash
# Current provider is full-only. Never label full decode + scaling as early preview.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
cargo build --locked --example application_open_probe --example image_probe --example preload_probe --example view_modes_probe --example async_cancel_probe --example active_decode_close_probe
export GDK_BACKEND=wayland G_DEBUG=fatal-criticals
python3 -B tests/preview_pipeline_native.py
