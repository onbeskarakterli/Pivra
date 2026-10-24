#!/usr/bin/env bash
# TH-01 capability/cache metadata only. Does not generate or display thumbnails.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
build=$(mktemp -d /tmp/pivra-thumbnail-factory.XXXXXX)
trap 'rm -rf "$build"' EXIT
unset PIVRA_THUMBNAIL_FACTORY_PROBE
module=$(pkg-config --list-all 2>/dev/null | awk '$1 ~ /^gnome-desktop(-[0-9.]+)?$/ {print $1; exit}')
if [[ -n "$module" ]]; then
    # The actual installed module supplies the header/library paths; no guessed ABI.
    cc -O2 -Wall -Wextra -Werror tests/thumbnail/factory_probe.c -o "$build/factory-probe" $(pkg-config --cflags --libs "$module")
    export PIVRA_THUMBNAIL_FACTORY_PROBE="$build/factory-probe"
    printf 'GNOME factory module: %s\n' "$module"
else
    printf 'GNOME factory unavailable: no generation adapter; cache-only when GNOME; no package installation\n'
fi
cargo build --locked --example thumbnail_provider_probe
exec_binary=target/debug/examples/thumbnail_provider_probe
"$exec_binary" "${1:-tests/fixtures/images/wide.png}"
