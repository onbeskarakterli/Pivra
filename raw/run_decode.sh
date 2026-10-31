#!/usr/bin/env bash
# Native LibRaw feasibility under mandatory Bubblewrap mount/network isolation.
# This is an experimental helper, NOT integrated into Pivra's rendering pipeline.
set -euo pipefail
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
source_root="$(cd -- "$script_dir/.." && pwd)"
if [[ "${PIVRA_RAW_INSTALLED:-0}" != 1 ]]; then
    cd "$source_root"
fi
input="${1:-$HOME/Downloads/Ricoh-GR-III.DNG}"
[[ -f "$input" ]] || { printf 'Missing RAW input: %s\n' "$input" >&2; exit 2; }
command -v bwrap >/dev/null || { echo 'FAIL: bubblewrap required' >&2; exit 2; }
tmp="$(mktemp -d)"
output="${2:-}"
if [[ -n "$output" ]]; then
    [[ ! -e "$output" ]] || { echo "Output already exists; refusing overwrite" >&2; exit 2; }
fi
trap 'rm -rf "$tmp"' EXIT
# Package builders can provide a precompiled helper. Never silently compile
# when a specifically configured helper is missing or invalid.
if [[ -n "${PIVRA_RAW_HELPER:-}" || "${PIVRA_RAW_INSTALLED:-0}" == 1 ]]; then
    helper="${PIVRA_RAW_HELPER:-$script_dir/pivra-raw-decoder}"
    [[ -f "$helper" && -x "$helper" && ! -L "$helper" ]] || {
        echo "Configured RAW helper is unavailable or unsafe" >&2; exit 2;
    }
    helper="$(realpath -- "$helper")"
    echo "RAW_HELPER_BUILD=prebuilt"
else
    # Legacy development-tree fallback; this path still requires the compiler.
    command -v c++ >/dev/null || { echo 'FAIL: C++ compiler required; set PIVRA_RAW_HELPER to a built executable' >&2; exit 2; }
    cache="${XDG_CACHE_HOME:-$HOME/.cache}/pivra/raw-helper"
    mkdir -p "$cache"
    chmod 700 "$cache"
    if pkg-config --exists libraw 2>/dev/null; then
        read -r -a flags <<< "$(pkg-config --cflags --libs libraw)"
    else
        flags=(-lraw)
    fi
    key="$( { sha256sum raw/decoder.cpp; c++ --version | head -n 1; pkg-config --modversion libraw 2>/dev/null || pacman -Q libraw; } | sha256sum | cut -d' ' -f1)"
    helper="$cache/raw-probe-$key"
    if [[ ! -x "$helper" ]]; then
        staging="$(mktemp "$cache/.building.XXXXXX")"
        trap 'rm -f "$staging"; rm -rf "$tmp"' EXIT
        c++ -std=c++17 -O2 -Wall -Wextra -Werror raw/decoder.cpp -o "$staging" "${flags[@]}"
        chmod 700 "$staging"
        mv -f "$staging" "$helper"
        echo "RAW_HELPER_BUILD=compiled"
    else
        echo "RAW_HELPER_BUILD=cached"
    fi
fi
# The input is the only host media visible inside the sandbox.
# No --bind of /home or working directory and no network namespace sharing.
printf 'SANDBOX: mandatory=bwrap, unshare-all, input-only read-only mount\n'
timeout 90s bwrap \
    --die-with-parent --new-session --unshare-all --clearenv \
    --setenv PATH /usr/bin --setenv HOME /tmp \
    --setenv PIVRA_RAW_METRICS "${PIVRA_RAW_METRICS:-0}" \
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
        if test "$PIVRA_RAW_METRICS" = 1; then
            # wait4 measures the actual LibRaw executable, not the bash wrapper.
            exec /usr/bin/python3 -c "import os,sys
pid=os.fork()
if pid==0:
    os.execv(\"/raw-probe\", [\"/raw-probe\",\"/input.dng\",\"/output/image.ppm\"])
_,status,usage=os.wait4(pid,0)
code=os.waitstatus_to_exitcode(status)
print(f\"RAW_DECODER_PEAK_RSS_KB={usage.ru_maxrss} RAW_DECODER_EXIT={code}\", flush=True)
sys.exit(code)"
        fi
        exec /raw-probe /input.dng /output/image.ppm
    '
printf 'PASS isolated native LibRaw decode (no Pivra integration or syscall audit yet).\n'
if [[ -n "$output" ]]; then
    test -s "$tmp/image.ppm"
    if [[ "${PIVRA_RAW_INSTALLED:-0}" != 1 ]]; then
        python3 tests/validate_raw_ppm.py "$tmp/image.ppm"
    fi
    cp --no-clobber -- "$tmp/image.ppm" "$output"
    printf 'PPM output: %s\n' "$output"
fi
