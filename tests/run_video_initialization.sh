#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
exec python3 -B tests/video_gpu/production_init.py "$@"
