#!/usr/bin/env bash
# Standalone host LibRaw test; does not change Pivra's secure Glycin pipeline.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
input="${1:-$HOME/Downloads/Ricoh-GR-III.DNG}"
if [[ ! -f "$input" ]]; then
    printf 'Missing RAW input: %s\n' "$input" >&2
    exit 2
fi
command -v c++ >/dev/null || { echo "C++ compiler required" >&2; exit 2; }
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
if pkg-config --exists libraw 2>/dev/null; then
    read -r -a flags <<< "$(pkg-config --cflags --libs libraw)"
else
    flags=(-lraw)
fi
c++ -std=c++17 -O2 -Wall -Wextra -Werror tests/raw_decode_probe.cpp -o "$tmp/raw_decode_probe" "${flags[@]}"
timeout 90s "$tmp/raw_decode_probe" "$input"
