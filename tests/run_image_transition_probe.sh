#!/usr/bin/env bash
# Native TH-06.2: secure JPEG/PNG prerequisite, fresh completion state, retained
# paintables, foreground cancellation, cache hit/miss/prefetch promotion. No bypass.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
cargo build --locked --example image_probe --example async_cancel_probe --example preload_probe
probe_config=$(mktemp -d /tmp/pivra-th06-config.XXXXXX)
trap 'rm -rf -- "$probe_config"' EXIT
export XDG_CONFIG_HOME="$probe_config" GDK_BACKEND=wayland G_DEBUG=fatal-criticals
# First secure failure exits77 and stops this script, never a successful skip.
target/debug/examples/image_probe
target/debug/examples/async_cancel_probe
target/debug/examples/preload_probe
printf '%s\n' 'PASS native transition/cache/cancellation probes. Physical visual acceptance remains separate.'
