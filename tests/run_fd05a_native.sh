#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
cargo build --locked --example fd05a_memory_probe
bash tests/run_freedesktop_generator_probe.sh
bash tests/run_freedesktop_writer_probe.sh
python3 tests/fd05a/benchmark.py "$@"
