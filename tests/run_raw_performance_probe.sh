#!/usr/bin/env bash
# Bounded, repeatable DNG decode performance smoke test on the native Arch host.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
input="${1:-$HOME/Downloads/Ricoh-GR-III.DNG}"
[[ -f "$input" ]] || { printf 'Missing DNG: %s\n' "$input" >&2; exit 2; }
command -v /usr/bin/time >/dev/null || { echo '/usr/bin/time required for peak RSS metrics' >&2; exit 2; }
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
for iteration in 1 2 3; do
    echo "===== RAW ITERATION $iteration/3 ====="
    # Probe compilation is included in wall-clock elapsed. Separate program
    # startup/compile costs from decoder time before drawing performance claims.
    /usr/bin/time -f 'ELAPSED_SECONDS=%e PEAK_RSS_KB=%M EXIT=%x' \
        bash tests/run_raw_sandbox_probe.sh "$input" "$tmp/frame-$iteration.ppm"
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
