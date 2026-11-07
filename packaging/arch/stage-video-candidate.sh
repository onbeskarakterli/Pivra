#!/usr/bin/env bash
# Native Arch development-candidate staging, NOT makepkg/publication approval.
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$root"
[[ $(git branch --show-current) == dev && -z $(git status --porcelain) ]] ||
  { echo 'FAIL: use a clean reviewed dev checkout' >&2; exit 1; }
bash packaging/arch/verify-video-dependencies.sh
bash packaging/linux/dependencies.sh check
bash packaging/linux/build-install.sh build
stage=$(mktemp -d "${TMPDIR:-/tmp}/pivra-gpu-stage.XXXXXXXX")
PREFIX=/usr DESTDIR="$stage" bash packaging/linux/build-install.sh install
python3 packaging/linux/verify-video-binary.py "$stage/usr/bin/pivra"
test -x "$stage/usr/bin/pivra-thumbnail-factory"
test -s "$stage/usr/share/licenses/pivra/video-adapter/COPYING.LGPL-2.1"
test -s "$stage/usr/share/doc/pivra/corresponding-source/Pivra-source.tar.gz"
desktop-file-validate "$stage/usr/share/applications/io.github.onbeskarakterli.Pivra.desktop"
python3 scripts/release_preflight.py --version 0.2.5 --candidate --staging-root "$stage"
printf 'PASS native Arch DESTDIR staging: %s\n' "$stage"
echo 'NOT RUN: makepkg/installation. This is the current development version, not an approved 0.2.5 release.'
