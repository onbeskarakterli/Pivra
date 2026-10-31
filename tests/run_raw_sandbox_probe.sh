#!/usr/bin/env bash
# Native LibRaw feasibility under mandatory Bubblewrap mount/network isolation.
# This is an experimental helper, NOT integrated into Pivra's rendering pipeline.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
input="${1:-$HOME/Downloads/Ricoh-GR-III.DNG}"
[[ -f "$input" ]] || { printf 'Missing RAW input: %s\n' "$input" >&2; exit 2; }
command -v bwrap >/dev/null || { echo 'FAIL: bubblewrap required' >&2; exit 2; }
command -v c++ >/dev/null || { echo 'FAIL: C++ compiler required' >&2; exit 2; }
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
c++ -std=c++17 -O2 -Wall -Wextra -Werror tests/raw_decode_probe.cpp -o "$tmp/raw_decode_probe" "${flags[@]}"
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
    --ro-bind "$tmp/raw_decode_probe" /raw-probe \
    --ro-bind "$input" /input.dng \
    --bind "$tmp" /output \
    /usr/bin/bash -c '
        test ! -e /home || { echo "FAIL: host home visible"; exit 1; }
        test ! -e /mnt || { echo "FAIL: host mounts visible"; exit 1; }
        echo "PASS mount isolation: host home/mnt unavailable"
        exec /raw-probe /input.dng /output/image.ppm
    '
printf 'PASS isolated native LibRaw decode (no Pivra integration, syscall audit, or memory budget yet).\n'
