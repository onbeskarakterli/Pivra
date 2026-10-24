#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
test "$(git branch --show-current)" = dev
build=$(mktemp -d /tmp/pivra-jpeg-adapter.XXXXXX)
trap 'rm -rf "$build"' EXIT
cc -O2 -Wall -Wextra -Werror ${PIVRA_JPEG_CFLAGS:-} tests/jpeg_preview/decoder.c -o "$build/decoder" ${PIVRA_JPEG_LIBS:--ljpeg}
python3 -B tests/jpeg_preview/probe.py --decoder "$build/decoder" "${1:-tests/fixtures/images/no-exif.jpg}"
