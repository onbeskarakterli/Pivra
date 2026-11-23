#!/usr/bin/env bash
# Explicit test-only private GTK build; no meson install/system replacements.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
exec python3 -B tests/video_gpu/gtk_egl_build.py "$@"
