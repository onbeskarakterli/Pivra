#!/usr/bin/env bash
# Native TH-02 manual UI harness: synthetic main image + labelled TEST cells only.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
cargo build --locked --example thumbnail_overlay_probe
export GDK_BACKEND=wayland G_DEBUG=fatal-criticals
exec target/debug/examples/thumbnail_overlay_probe --manual
