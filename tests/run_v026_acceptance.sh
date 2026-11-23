#!/usr/bin/env bash
# Development acceptance only: private fixtures, no install/tag/source approval.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
exec python3 -B tests/performance/v026_acceptance.py "$@"
