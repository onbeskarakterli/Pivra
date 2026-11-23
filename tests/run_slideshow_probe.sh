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
ffmpeg -hide_banner -loglevel error -f lavfi -i testsrc2=size=192x96:rate=12 -t 5 -c:v libx264 -pix_fmt yuv420p "$work/video.mp4"
cargo build --locked --example slideshow_probe
# Default is real secure decode. --controlled labels photo pixels explicitly.
case "${1:-}" in
  ''|--native) mode=--native ;;
  --controlled) mode=--controlled ;;
  *) echo 'Usage: tests/run_slideshow_probe.sh [--native|--controlled]' >&2; exit 2 ;;
esac
cargo run --locked --example slideshow_probe -- "$mode" "$work/video.mp4" 2>&1 | tee "$work/probe.log"
grep -q 'PIVRA_SLIDESHOW event=video-ended' "$work/probe.log" || { echo 'FAIL: video EOS was not observed' >&2; exit 1; }
