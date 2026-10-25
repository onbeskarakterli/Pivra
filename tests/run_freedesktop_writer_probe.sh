#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
cargo build --locked --example fd05_writer_probe
probe_root=$(mktemp -d /tmp/pivra-fd05-native.XXXXXX)
trap 'rm -rf -- "$probe_root"' EXIT
export XDG_CONFIG_HOME="$probe_root/config" XDG_CACHE_HOME="$probe_root/cache" G_DEBUG=fatal-criticals
mkdir -p "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME"
# First secure gate fails closed with77; no retries, production wiring or real user cache.
target/debug/examples/fd05_writer_probe
