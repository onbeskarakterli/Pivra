#!/usr/bin/env bash
# Bounded, repeatable DNG decode performance smoke test on the native Arch host.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
input="${1:-$HOME/Downloads/Ricoh-GR-III.DNG}"
[[ -f "$input" ]] || { printf 'Missing DNG: %s\n' "$input" >&2; exit 2; }
# GNU time is optional on minimal Arch installations. Do not require a package install.
if [[ -x /usr/bin/time ]]; then
    timing=gnu
else
    timing=python
    command -v python3 >/dev/null || { echo 'python3 required for elapsed-time fallback' >&2; exit 2; }
    echo 'NOTE: GNU time unavailable; Python wait4 child peak RSS fallback (not total Pivra memory)'
fi
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
for iteration in 1 2 3; do
    echo "===== RAW ITERATION $iteration/3 ====="
    # Probe compilation is included in wall-clock elapsed. Separate program
    # startup/compile costs from decoder time before drawing performance claims.
    if [[ "$timing" == gnu ]]; then
        /usr/bin/time -f 'ELAPSED_SECONDS=%e PEAK_RSS_KB=%M EXIT=%x' \
            bash tests/run_raw_sandbox_probe.sh "$input" "$tmp/frame-$iteration.ppm"
    else
        python3 -c 'import os, sys, time; start=time.monotonic(); pid=os.fork();
if pid == 0:
    os.execvp(sys.argv[1], sys.argv[1:]);
_, status, usage=os.wait4(pid, 0); code=os.waitstatus_to_exitcode(status); print(f"ELAPSED_SECONDS={time.monotonic()-start:.3f} PEAK_CHILD_RSS_KB={usage.ru_maxrss} EXIT={code}", flush=True); sys.exit(code)' \
            bash tests/run_raw_sandbox_probe.sh "$input" "$tmp/frame-$iteration.ppm"
    fi
    python3 tests/validate_raw_ppm.py "$tmp/frame-$iteration.ppm"
    echo "OUTPUT_BYTES=$(stat -c %s "$tmp/frame-$iteration.ppm")"
done
# Detect surviving helpers; grep excludes itself and this script.
if pgrep -af '[r]aw_decode_probe|[r]un_raw_sandbox_probe.sh|[b]wrap.*raw-probe'; then
    echo 'WARNING: matched RAW helpers after tests; inspect ownership before concluding leakage'
else
    echo 'PASS no matching RAW helper processes'
fi
echo 'OBSERVATION COMPLETE: elapsed time includes compilation, RSS refers to wrapper process tree peak and is not whole Pivra RSS.'
