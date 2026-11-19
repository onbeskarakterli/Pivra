#!/usr/bin/env bash
# Fresh-process normal playback comparison; mixed stress is a separate gate.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
exec python3 -B tests/video_gpu/normal_playback.py "$@"
