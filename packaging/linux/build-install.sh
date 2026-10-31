#!/usr/bin/env bash
# Portable source build/install for Pivra on Linux.
# Build: ./packaging/linux/build-install.sh build
# Install: ./packaging/linux/build-install.sh install
# Defaults: PREFIX=/usr/local, DESTDIR= (staging is supported).
set -euo pipefail

action="${1:-}"
if [[ "$action" != build && "$action" != install ]]; then
  printf 'Usage: %s {build|install}\n' "$0" >&2
  exit 2
fi

root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

prefix="${PREFIX:-/usr/local}"
destdir="${DESTDIR:-}"
if [[ "$prefix" != /* || "$prefix" == / || "$prefix" == *..* ]]; then
  printf 'ERROR: PREFIX must be an absolute, non-root path without ..\n' >&2
  exit 2
fi
if [[ -n "$destdir" && "$destdir" != /* ]]; then
  printf 'ERROR: DESTDIR must be an absolute path\n' >&2
  exit 2
fi

for cmd in cargo rustc cc pkg-config install python3; do
  command -v "$cmd" >/dev/null || { printf 'ERROR: missing command: %s\n' "$cmd" >&2; exit 1; }
done
pkg-config --atleast-version=4.16 gtk4 || { echo 'ERROR: GTK >= 4.16 required' >&2; exit 1; }
pkg-config --atleast-version=1.2 libadwaita-1 || { echo 'ERROR: libadwaita >= 1.2 required' >&2; exit 1; }
pkg-config --exists gnome-desktop-4 gio-2.0 gobject-2.0 gdk-pixbuf-2.0 lcms2 libseccomp ||
  { echo 'ERROR: missing native development libraries (GNOME Desktop 4, GLib/GDK, LCMS2 or libseccomp)' >&2; exit 1; }

build_dir="$root/target/pivra-native"
helper="$build_dir/pivra-thumbnail-factory"
# Honor Cargo's effective target directory (including .cargo/config.toml and
# CARGO_TARGET_DIR). Otherwise a stale target/release/pivra can be installed
# or scanned even when Cargo built into a different directory.
cargo_target_dir="$(cargo metadata --locked --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
viewer="$cargo_target_dir/release/pivra"
raw_helper="$build_dir/pivra-raw-decoder"

build_all() {
  [[ -z "${LCMS2_STATIC+x}" && -z "${LCMS2_LIB_DIR+x}" ]] ||
    { echo 'ERROR: LCMS2 overrides are not supported for distribution builds' >&2; return 1; }
  mkdir -p "$build_dir"
  # The helper must be installed alongside the viewer; never embed Cargo OUT_DIR.
  read -r -a cc_args <<< "${CC:-cc}"
  read -r -a cpp_args <<< "${CPPFLAGS:-}"
  read -r -a c_args <<< "${CFLAGS:-}"
  read -r -a ld_args <<< "${LDFLAGS:-}"
  read -r -a pkg_cflags <<< "$(pkg-config --cflags gnome-desktop-4 gio-2.0 gobject-2.0 gdk-pixbuf-2.0)"
  read -r -a pkg_libs <<< "$(pkg-config --libs gnome-desktop-4 gio-2.0 gobject-2.0 gdk-pixbuf-2.0)"
  "${cc_args[@]}" "${cpp_args[@]}" "${c_args[@]}" "${pkg_cflags[@]}" -Wall -Wextra -Werror \
    native/thumbnail_factory.c -o "$helper" "${ld_args[@]}" "${pkg_libs[@]}"
  if [[ "${PIVRA_BUILD_EXPERIMENTAL_RAW:-0}" == 1 ]]; then
    command -v c++ >/dev/null || { echo "ERROR: C++ required for experimental RAW build" >&2; return 1; }
    pkg-config --exists libraw || { echo "ERROR: libraw development files required for experimental RAW build" >&2; return 1; }
    bash raw/build_helper.sh "$raw_helper"
  fi
  PIVRA_DISTRIBUTION_BUILD=1 cargo build --release --locked --bin pivra
  [[ -x "$helper" && -x "$viewer" ]] || { echo 'ERROR: missing build output' >&2; return 1; }
  if grep -aFq '/out/pivra-thumbnail-factory' "$viewer"; then
    echo 'ERROR: viewer embeds temporary thumbnail helper path' >&2
    return 1
  fi
  echo 'PASS: Pivra and thumbnail helper built'
  [[ "${PIVRA_BUILD_EXPERIMENTAL_RAW:-0}" != 1 ]] || echo 'PASS: experimental RAW helper built (not installed by default)'
}

case "$action" in
  build) build_all ;;
  install)
    [[ -x "$viewer" && -x "$helper" ]] ||
      { echo 'ERROR: build outputs missing; run build first' >&2; exit 1; }
    bindir="$destdir$prefix/bin"
    sharedir="$destdir$prefix/share"
    install -Dm755 "$viewer" "$bindir/pivra"
    install -Dm755 "$helper" "$bindir/pivra-thumbnail-factory"
    # Experimental RAW is installed only when explicitly requested.
    if [[ "${PIVRA_INSTALL_EXPERIMENTAL_RAW:-0}" == 1 ]]; then
      [[ -x "$raw_helper" ]] || { echo "ERROR: experimental RAW helper missing; rebuild with PIVRA_BUILD_EXPERIMENTAL_RAW=1" >&2; exit 1; }
      install -Dm755 "$raw_helper" "$bindir/pivra-raw-decoder"
      install -Dm755 raw/run_decode.sh "$bindir/pivra-raw-runner"
    fi
    install -Dm644 data/io.github.onbeskarakterli.Pivra.desktop \
      "$sharedir/applications/io.github.onbeskarakterli.Pivra.desktop"
    for variant in scalable symbolic; do
      name=io.github.onbeskarakterli.Pivra
      [[ "$variant" == symbolic ]] && name+=-symbolic
      install -Dm644 "data/icons/hicolor/$variant/apps/$name.svg" \
        "$sharedir/icons/hicolor/$variant/apps/$name.svg"
    done
    install -Dm644 LICENSE "$sharedir/licenses/pivra/LICENSE"
    echo "PASS: Pivra installed to $destdir$prefix"
    echo 'NOTE: If distributing binaries, review third-party license and corresponding-source obligations.'
    ;;
esac
