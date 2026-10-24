#!/usr/bin/env bash
# TH-01 capability/cache metadata only. Does not generate or display thumbnails.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
build=$(mktemp -d /tmp/pivra-thumbnail-factory.XXXXXX)
trap 'rm -rf "$build"' EXIT
unset PIVRA_THUMBNAIL_FACTORY_PROBE
# Capture the complete listing: no early pipe consumer can SIGPIPE pkg-config.
if ! listing=$(pkg-config --list-all 2>/dev/null); then
    printf 'Factory module listing unavailable; using cache-only fallback when GNOME\n' >&2
    listing=''
fi
mapfile -t modules < <(awk '$1 ~ /^gnome-desktop(-[0-9.]+)?$/ {print $1}' <<<"$listing" | sort -Vru)
module=''
for candidate in "${modules[@]}"; do
    if ! flags_text=$(pkg-config --cflags --libs "$candidate"); then
        printf 'Factory module flags unavailable: %s\n' "$candidate" >&2
        continue
    fi
    read -r -a flags <<<"$flags_text"
    if cc -O2 -Wall -Wextra -Werror tests/thumbnail/factory_probe.c -o "$build/factory-probe" "${flags[@]}" >"$build/compile.log" 2>&1; then
        module=$candidate
        export PIVRA_THUMBNAIL_FACTORY_PROBE="$build/factory-probe"
        printf 'GNOME factory module: %s\n' "$module"
        break
    fi
    printf 'Factory module not buildable: %s; trying next candidate\n' "$candidate" >&2
    cat "$build/compile.log" >&2
done
if [[ -z "$module" ]]; then
    printf 'GNOME factory unavailable/unbuildable: cache-only when GNOME; no package installation\n'
fi
cargo build --locked --example thumbnail_provider_probe
exec_binary=target/debug/examples/thumbnail_provider_probe
"$exec_binary" "${1:-tests/fixtures/images/wide.png}"
