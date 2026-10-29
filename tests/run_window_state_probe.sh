#!/usr/bin/env bash
# Uses the current compositor, fresh config, and real production windows.
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"
cargo build --locked --example window_state_probe --target-dir "$repo/target"
state_root=$(mktemp -d -t pivra-window-state-XXXXXXXX)
trap 'rm -rf -- "$state_root"' EXIT
export XDG_CONFIG_HOME="$state_root/config"
export XDG_CACHE_HOME="$state_root/cache"
export XDG_DATA_HOME="$state_root/data"
mkdir -m 700 "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME" "$XDG_DATA_HOME"
export G_DEBUG=fatal-criticals
"$repo/target/debug/examples/window_state_probe" "$@"
