#!/usr/bin/env bash
# Native FD-04: one secure gate, then real pixels; output stays in memory.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
cargo build --locked --example fd04_generator_probe
probe_root=$(mktemp -d /tmp/pivra-fd04-native.XXXXXX)
trap 'rm -rf -- "$probe_root"' EXIT
export XDG_CONFIG_HOME="$probe_root/config" XDG_CACHE_HOME="$probe_root/cache"
mkdir -p "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME"
export G_DEBUG=fatal-criticals
# No headless fallback, insecure decoder, shared thumbnail writes or automatic retry.
target/debug/examples/fd04_generator_probe
