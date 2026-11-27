#!/usr/bin/env bash
# Interactive physical GNOME capture; no portal/backend overrides or cache deletion.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
if (( $# != 1 )); then
    echo 'Usage: bash tests/trace_open_latency.sh /absolute/path/to/photo.jpg' >&2
    exit 2
fi
media=$(realpath -e -- "$1")
if [[ ! -f "$media" ]]; then
    echo 'Expected an existing regular media file.' >&2
    exit 2
fi
if ! command -v pgrep >/dev/null; then
    echo 'pgrep is required to exclude an already running Pivra instance.' >&2
    exit 2
fi
if pgrep -u "$(id -u)" -x pivra >/dev/null; then
    echo 'Close existing Pivra instances before capturing a fresh-process comparison.' >&2
    exit 2
fi
out=$(mktemp -d /tmp/pivra-open-latency.XXXXXXXX)
export PIVRA_TRACE_OPEN=1 PIVRA_PERF=1
cargo build --locked
printf 'Logs: %s\nA: Direct open. Press Right, Left, Right; close the window.\n' "$out"
# pipefail preserves the actual application status; no background process survives.
target/debug/pivra "$media" 2>&1 | tee "$out/direct.log"
printf 'B: Empty start. Use Open File to select the SAME file: %s\n' "$media"
printf 'Then press Right, Left, Right. Reopen with Ctrl+O and context Open..., cancel both, then close.\n'
target/debug/pivra 2>&1 | tee "$out/chooser.log"
printf 'Capture complete: %s\nGTK map/after-paint is not physical scanout; portal windows are external and visibility is unobservable here.\n' "$out"
