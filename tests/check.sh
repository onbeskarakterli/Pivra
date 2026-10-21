#!/usr/bin/env bash
# Core checks only. Graphical categories must be run and reported separately.
set -euo pipefail
if [[ -n "${PIVRA_ENV_ACTIVATE:-}" ]]; then
    source "$PIVRA_ENV_ACTIVATE"
fi
cd "$(dirname "${BASH_SOURCE[0]}")/.."
python3 tests/fixtures/generate.py --check
python3 tests/fixtures/images/generate.py --check
python3 -B tests/test_active_decode_observer.py
cargo +stable fmt --check
cargo +stable check --locked --offline -j 2
cargo +stable build --locked --offline -j 2
cargo +stable test --locked --offline -j 2
cargo +stable clippy --locked --offline --all-targets -j 2 -- -D warnings
