#!/usr/bin/env bash
# Negative tests for the opt-in, sandboxed LibRaw decoder.
# Does not mutate source images or install any system files.
set -euo pipefail
root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
runner="${PIVRA_RAW_RUNNER:-$root/raw/run_decode.sh}"
helper="${PIVRA_RAW_HELPER:-$root/target/pivra-native/pivra-raw-decoder}"
[[ -x "$helper" && -f "$runner" ]] || {
    echo 'Build the RAW helper first: PIVRA_BUILD_EXPERIMENTAL_RAW=1 bash packaging/linux/build-install.sh build' >&2
    exit 2
}
tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT
: > "$tmp/empty file.dng"
printf 'This is not a DNG file\n' > "$tmp/invalid file.dng"
truncate -s 68157441 "$tmp/oversized file.dng"
ln -s "$tmp/does-not-exist.dng" "$tmp/missing link.dng"
fail_case() {
    local label="$1" input="$2" output="$tmp/$1.ppm"
    local status=0
    PIVRA_RAW_INSTALLED=0 PIVRA_RAW_HELPER="$helper" \
        bash "$runner" "$input" "$output" >"$tmp/log-$label" 2>&1 || status=$?
    if [[ "$status" == 0 || -e "$output" ]]; then
        printf 'FAIL %s: status=%s output=%s\n' "$label" "$status" "$output" >&2
        cat "$tmp/log-$label" >&2
        return 1
    fi
    printf 'PASS %s: rejected (exit=%s), no output\n' "$label" "$status"
}
fail_case empty "$tmp/empty file.dng"
fail_case invalid "$tmp/invalid file.dng"
fail_case oversized "$tmp/oversized file.dng"
fail_case missing "$tmp/missing link.dng"
echo 'PASS negative RAW input tests (no successful decode expected)'
