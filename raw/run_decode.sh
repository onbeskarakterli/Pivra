#!/usr/bin/env bash
# Experimental LibRaw runtime launcher with mandatory Bubblewrap isolation.
set -euo pipefail
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# Resolve relative media paths before changing to the source directory.
# The development runner should interpret paths relative to the caller.
input="${1:-$HOME/Downloads/Ricoh-GR-III.DNG}"
output="${2:-}"
if [[ "$input" != /* ]]; then
    input="$PWD/$input"
fi
if [[ -n "$output" && "$output" != /* ]]; then
    output="$PWD/$output"
fi
if [[ "${PIVRA_RAW_INSTALLED:-0}" == 1 ]]; then
    # Installed runtime must never use sample inputs or development defaults.
    [[ "$#" == 2 ]] || { echo "Usage: pivra-raw-runner INPUT OUTPUT" >&2; exit 2; }
else
    cd "$script_dir/.."
    [[ "$#" -le 2 ]] || { echo "Usage: run_decode.sh [INPUT] [OUTPUT]" >&2; exit 2; }
fi
[[ -f "$input" && ! -L "$input" ]] || { printf 'Missing or unsafe RAW input: %s\n' "$input" >&2; exit 2; }
# Match the native decoder's 64 MiB bound before entering the sandbox.
[[ "$(stat -c %s -- "$input")" -gt 0 && "$(stat -c %s -- "$input")" -le 67108864 ]] ||
    { echo 'RAW input must be between 1 byte and 64 MiB' >&2; exit 2; }
command -v bwrap >/dev/null || { echo 'FAIL: bubblewrap required' >&2; exit 2; }
tmp="$(mktemp -d)"
# Install cleanup immediately: output validation can fail before decoding.
trap 'rm -rf -- "$tmp"' EXIT
if [[ -n "$output" ]]; then
    [[ ! -e "$output" && ! -L "$output" ]] ||
        { echo "Output already exists; refusing overwrite" >&2; exit 2; }
fi
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
        trap 'rm -f -- "$staging"; rm -rf -- "$tmp"' EXIT
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
    # The completed decoder frame already lives in a private scratch
    # directory. Publish it with one exclusive hard-link operation instead
    # of copying a second 200+ MiB temporary file beside the destination.
    # This is atomic, never overwrites a destination, and leaves no staging
    # file behind even if the runner is killed with SIGKILL.
    # Pivra places the decoder scratch and output on the same filesystem.
    python3 - "$tmp/image.ppm" "$output" <<'PY'
import errno
import os
import sys

source, destination = sys.argv[1:]
try:
    os.link(source, destination, follow_symlinks=False)
except OSError as error:
    if error.errno == errno.EXDEV:
        sys.exit("RAW output must be on the same filesystem as its scratch directory")
    raise
PY
    printf 'PPM output: %s\n' "$output"
fi
