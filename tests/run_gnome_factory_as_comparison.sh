#!/usr/bin/env bash
# Diagnostic only: production helper and user cache are never replaced.
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
exec python3 "$repo/tests/thumbnail/factory_as_comparison.py" "$@"
