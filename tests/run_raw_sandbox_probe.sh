#!/usr/bin/env bash
# Compatibility entrypoint for existing RAW probe tests.
set -euo pipefail
exec bash "$(dirname "${BASH_SOURCE[0]}")/../raw/run_decode.sh" "$@"
