#!/usr/bin/env bash
# Build the isolated RAW helper ahead of runtime.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
command -v c++ >/dev/null || { echo 'C++ compiler required at build time' >&2; exit 2; }
out="${1:-$PWD/target/pivra-raw-decoder}"
mkdir -p "$(dirname "$out")"
if pkg-config --exists libraw 2>/dev/null; then
    read -r -a flags <<< "$(pkg-config --cflags --libs libraw)"
else
    flags=(-lraw)
fi
staging="$(mktemp "$(dirname "$out")/.pivra-raw-build.XXXXXX")"
trap 'rm -f "$staging"' EXIT
c++ -std=c++17 -O2 -Wall -Wextra -Werror raw/decoder.cpp -o "$staging" "${flags[@]}"
chmod 755 "$staging"
mv -f "$staging" "$out"
printf 'PREBUILT_RAW_HELPER=%s\n' "$(realpath -- "$out")"
