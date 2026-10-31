#!/usr/bin/env bash
# One-shot, read-only-to-host RAW/DNG acceptance for the development branch.
# Builds into the configured Cargo target and stages packaging under /tmp.
# Does not install to the system, change branches or touch the AUR package.
set -euo pipefail

root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
sample="${1:-$HOME/Downloads/Ricoh-GR-III.DNG}"
[[ -f "$sample" ]] || {
    printf 'ERROR: DNG sample not found: %s\n' "$sample" >&2
    exit 2
}
sample="$(realpath -- "$sample")"

echo '=== 1/5 Static checks ==='
bash -n raw/run_decode.sh raw/build_helper.sh packaging/linux/build-install.sh \
    tests/run_raw_negative_probe.sh tests/run_raw_performance_probe.sh
python3 -m py_compile tests/run_raw_cancel_probe.py tests/validate_raw_ppm.py
cargo fmt --all -- --check

echo '=== 2/5 Rust RAW unit tests ==='
cargo test --locked --bin pivra raw_loader::tests

echo '=== 3/5 Native experimental RAW build ==='
PIVRA_BUILD_EXPERIMENTAL_RAW=1 bash packaging/linux/build-install.sh build

echo '=== 4/5 Negative inputs and cancellation ==='
bash tests/run_raw_negative_probe.sh
python3 tests/run_raw_cancel_probe.py "$sample"

echo '=== 5/5 Native staged installation ==='
stage="$(mktemp -d -t pivra-raw-acceptance.XXXXXXXX)"
trap 'rm -rf -- "$stage"' EXIT
PIVRA_INSTALL_EXPERIMENTAL_RAW=1 PREFIX=/usr DESTDIR="$stage" \
    bash packaging/linux/build-install.sh install
for name in pivra pivra-thumbnail-factory pivra-raw-decoder pivra-raw-runner; do
    [[ -x "$stage/usr/bin/$name" ]] || {
        printf 'FAIL: staged executable missing: %s\n' "$name" >&2
        exit 1
    }
done

echo 'PASS: static checks, Rust RAW tests, negative inputs, cancellation and staged packaging'
echo 'NOTE: GUI DNG rendering and orphan-free cancellation still require separate observation.'
