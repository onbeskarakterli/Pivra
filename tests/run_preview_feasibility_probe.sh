#!/usr/bin/env bash
# Observation-only native experiment. Exit0 is not proof of early preview.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
git rev-parse HEAD
cargo build --locked --example preview_feasibility_probe
export GDK_BACKEND=wayland G_DEBUG=fatal-criticals
exec timeout 90s target/debug/examples/preview_feasibility_probe "$@"
