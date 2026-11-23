#!/usr/bin/env bash
# Native GNOME smoke: private fixtures/config only; no package/system changes.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
[[ $(git branch --show-current) == feature/v0.2.7-slideshow ]]
work=$(mktemp -d "${TMPDIR:-/tmp}/pivra-slideshow.XXXXXXXX")
cleanup() {
  local status=$?
  if [[ $status == 0 ]]; then rm -rf -- "$work"; else printf 'Probe files/log: %s\n' "$work" >&2; fi
}
trap cleanup EXIT
export PIVRA_TRACE_SLIDESHOW=1
export XDG_CONFIG_HOME="$work/config" XDG_CACHE_HOME="$work/cache" XDG_DATA_HOME="$work/data"
mode=--native
cycles=()
debug=()
color=()
extra=()
duration=5
for arg in "$@"; do
  case "$arg" in
    --native|--controlled) mode=$arg ;;
    --eos-regression) cycles=(--eos-regression) ;;
    --video-interval-regression) cycles=(--video-interval-regression); duration=8 ;;
    --bt709) color=(-x264-params colorprim=bt709:transfer=bt709:colormatrix=bt709:fullrange=off); extra=(--bt709) ;;
    --gdb) debug=(--gdb-events "$work/gdb.jsonl") ;;
    *) echo 'Usage: tests/run_slideshow_probe.sh [--native|--controlled] [--eos-regression|--video-interval-regression] [--bt709] [--gdb]' >&2; exit 2 ;;
  esac
done
if [[ ${#debug[@]} != 0 ]] && ! command -v gdb >/dev/null; then
  echo 'NOT RUN: gdb required for --gdb' >&2; exit 77
fi
if [[ ${#extra[@]} != 0 ]]; then
  [[ ${cycles[0]:-} == --video-interval-regression ]] || { echo 'FAIL: --bt709 requires --video-interval-regression' >&2; exit 2; }
  gtk_minor=$(pkg-config --modversion gtk4 | cut -d. -f2)
  if [[ -z ${GSK_RENDERER:-} ]]; then
    if ((gtk_minor < 20)); then export GSK_RENDERER=ngl; else export GSK_RENDERER=gl; fi
  fi
fi
ffmpeg -hide_banner -loglevel error -f lavfi -i testsrc2=size=192x96:rate=12 -t "$duration" -c:v libx264 -pix_fmt yuv420p "${color[@]}" "$work/video.mp4"
cargo build --locked --example slideshow_probe
# Honor Cargo's target directory; never guess which binary GDB launches.
target=$(cargo metadata --locked --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
command=("$target/debug/examples/slideshow_probe" "$mode" "${cycles[@]}" "${extra[@]}" "$work/video.mp4")
sha256sum "$target/debug/examples/slideshow_probe" > "$work/binary.sha256"
if [[ ${#cycles[@]} != 0 ]]; then export PIVRA_TRACE_VIDEO=1; fi
if [[ ${#debug[@]} != 0 ]]; then
  export PIVRA_GDB_EVENTS="$work/gdb.jsonl" PIVRA_TRACE_VIDEO=1
  command=(gdb -nx --quiet --batch --return-child-result -x tests/video_gpu/context_trace.gdb --args "${command[@]}")
fi
set +e
"${command[@]}" 2>&1 | tee "$work/probe.log"
pipeline=("${PIPESTATUS[@]}")
set -e
[[ ${pipeline[1]} == 0 ]] || { echo 'FAIL: diagnostic log write failed' >&2; exit 1; }
checks=()
case ${cycles[0]:-} in
  --eos-regression) checks=(--cycles) ;;
  --video-interval-regression) checks=(--video-interval) ;;
esac
python3 -B tests/slideshow_result.py "$work/probe.log" "${pipeline[0]}" "${checks[@]}" "${debug[@]}"
# Keep debugger provenance even on success, for clean-versus-native comparison.
if [[ ${#debug[@]} != 0 ]]; then echo "Debugger artifacts: $work"; trap - EXIT; fi
