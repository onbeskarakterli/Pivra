#!/usr/bin/env bash
# Native LibRaw feasibility under mandatory Bubblewrap mount/network isolation.
# This is an experimental helper, NOT integrated into Pivra's rendering pipeline.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
input="${1:-$HOME/Downloads/Ricoh-GR-III.DNG}"
[[ -f "$input" ]] || { printf 'Missing RAW input: %s\n' "$input" >&2; exit 2; }
command -v bwrap >/dev/null || { echo 'FAIL: bubblewrap required' >&2; exit 2; }
command -v c++ >/dev/null || { echo 'FAIL: C++ compiler required' >&2; exit 2; }
cache="${XDG_CACHE_HOME:-$HOME/.cache}/pivra/raw-helper"
mkdir -p "$cache"
chmod 700 "$cache"
tmp="$(mktemp -d)"
output="${2:-}"
if [[ -n "$output" ]]; then
    [[ ! -e "$output" ]] || { echo "Output already exists; refusing overwrite" >&2; exit 2; }
fi
trap 'rm -rf "$tmp"' EXIT
if pkg-config --exists libraw 2>/dev/null; then
    read -r -a flags <<< "$(pkg-config --cflags --libs libraw)"
else
    flags=(-lraw)
fi
# Compile once per source/compiler/LibRaw version rather than every DNG open.
key="$( { sha256sum tests/raw_decode_probe.cpp; c++ --version | head -n 1; pkg-config --modversion libraw 2>/dev/null || pacman -Q libraw; } | sha256sum | cut -d' ' -f1)"
helper="$cache/raw-probe-$key"
if [[ ! -x "$helper" ]]; then
    staging="$(mktemp "$cache/.building.XXXXXX")"
    trap 'rm -f "$staging"; rm -rf "$tmp"' EXIT
    c++ -std=c++17 -O2 -Wall -Wextra -Werror tests/raw_decode_probe.cpp -o "$staging" "${flags[@]}"
    chmod 700 "$staging"
    mv -f "$staging" "$helper"
    echo "RAW_HELPER_BUILD=compiled"
else
    echo "RAW_HELPER_BUILD=cached"
fi
# The input is the only host media visible inside the sandbox.
# No --bind of /home or working directory and no network namespace sharing.
printf 'SANDBOX: mandatory=bwrap, unshare-all, input-only read-only mount\n'
timeout 90s bwrap \
    --die-with-parent --new-session --unshare-all --clearenv \
    --setenv PATH /usr/bin --setenv HOME /tmp \
    --ro-bind /usr /usr \
    --ro-bind /etc /etc \
    --ro-bind-try /lib /lib --ro-bind-try /lib64 /lib64 \
    --proc /proc --dev /dev --tmpfs /tmp \
    --ro-bind "$helper" /raw-probe \
    --ro-bind "$input" /input.dng \
    --bind "$tmp" /output \
    /usr/bin/bash -c '
        test ! -e /home || { echo "FAIL: host home visible"; exit 1; }
        test ! -e /mnt || { echo "FAIL: host mounts visible"; exit 1; }
        echo "PASS mount isolation: host home/mnt unavailable"
        ulimit -v 1572864
        exec /raw-probe /input.dng /output/image.ppm
    '
printf 'PASS isolated native LibRaw decode (no Pivra integration or syscall audit yet).\n'
if [[ -n "$output" ]]; then
    test -s "$tmp/image.ppm"
    python3 tests/validate_raw_ppm.py "$tmp/image.ppm"
    cp --no-clobber -- "$tmp/image.ppm" "$output"
    printf 'PPM output: %s\n' "$output"
fi
