#!/usr/bin/env bash
# TH-03: isolated automated state/persistence checks, then two manual TEST windows.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
cargo build --locked --example thumbnail_overlay_probe
export GDK_BACKEND=wayland G_DEBUG=fatal-criticals
probe_config=$(mktemp -d)
trap 'rm -rf -- "$probe_config"' EXIT
probe=target/debug/examples/thumbnail_overlay_probe
XDG_CONFIG_HOME="$probe_config" "$probe" --expect-enabled=false
XDG_CONFIG_HOME="$probe_config" "$probe" --persist-enabled=true
XDG_CONFIG_HOME="$probe_config" "$probe" --expect-enabled=true
XDG_CONFIG_HOME="$probe_config" "$probe" --persist-enabled=false
XDG_CONFIG_HOME="$probe_config" "$probe" --expect-enabled=false
XDG_CONFIG_HOME="$probe_config" "$probe"
printf '%s\n' 'Manual TEST windows: move to bottom, click/T, leave, resize, compare two windows.' \
    'The manual session uses your Pivra thumbnail preference; re-run to check saved Enabled.'
"$probe" --manual
