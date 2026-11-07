#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
cargo build --release --locked --example performance_probe --example performance_gui_probe --example thumbnail_provider_probe --example integrated_video_probe --example video_color_probe
benchmark_target=$(cargo metadata --locked --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
PIVRA_PERF_BIN_DIR="$benchmark_target/release/examples" python3 -B tests/performance/collect.py "$@"
