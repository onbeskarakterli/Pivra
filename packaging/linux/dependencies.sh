#!/usr/bin/env bash
# Native prerequisites for the portable Pivra source build.
# Installs only system packages; verifies capabilities after installation.
set -euo pipefail
mode="${1:-check}"
[[ "$mode" == check || "$mode" == install ]] ||
  { echo "Usage: $0 [check|install]" >&2; exit 2; }

check() {
  local failed=0 bin module
  for bin in cargo rustc cc pkg-config python3 bwrap; do
    if ! command -v "$bin" >/dev/null 2>&1; then
      echo "MISSING COMMAND: $bin"; failed=1
    fi
  done
  if command -v rustc >/dev/null 2>&1; then
    local version
    version="$(rustc --version | awk '{print $2}')"
    if [[ "$(printf '%s\n%s\n' '1.85.0' "$version" | sort -V | head -n1)" != 1.85.0 ]]; then
      echo "RUST TOO OLD: $version (requires >= 1.85)"; failed=1
    fi
  fi
  if command -v pkg-config >/dev/null 2>&1; then
    pkg-config --atleast-version=4.16 gtk4 ||
      { echo 'REQUIRES GTK4 >= 4.16 development package'; failed=1; }
    pkg-config --atleast-version=1.2 libadwaita-1 ||
      { echo 'REQUIRES libadwaita >= 1.2 development package'; failed=1; }
    for module in gnome-desktop-4 gio-2.0 gobject-2.0 gdk-pixbuf-2.0 lcms2 libseccomp; do
      pkg-config --exists "$module" ||
        { echo "MISSING NATIVE MODULE: $module"; failed=1; }
    done
  fi
  if [[ "${PIVRA_BUILD_EXPERIMENTAL_RAW:-0}" == 1 ]]; then
    command -v c++ >/dev/null 2>&1 ||
      { echo 'MISSING COMMAND: c++ (experimental RAW build)'; failed=1; }
    command -v timeout >/dev/null 2>&1 ||
      { echo 'MISSING COMMAND: timeout (experimental RAW runtime)'; failed=1; }
    pkg-config --exists libraw 2>/dev/null ||
      { echo 'MISSING NATIVE MODULE: libraw (experimental RAW build)'; failed=1; }
  fi
  if [[ "$failed" == 0 ]]; then echo 'PASS: required build tools and native development APIs available'; fi
  return "$failed"
}

if [[ "$mode" == check ]]; then check; exit $?; fi
[[ -r /etc/os-release ]] || { echo 'ERROR: cannot identify distribution' >&2; exit 1; }
# shellcheck disable=SC1091
. /etc/os-release
ids=" ${ID:-} ${ID_LIKE:-} "
if [[ "$ids" == *" arch "* ]]; then
  sudo pacman -S --needed -- base-devel rust cargo pkgconf python gtk4 libadwaita \
    gnome-desktop-4 gdk-pixbuf2 glycin bubblewrap libseccomp lcms2 \
    fontconfig cairo graphene hicolor-icon-theme
elif [[ "$ids" == *" fedora "* || "$ids" == *" rhel "* ]]; then
  sudo dnf install -- gcc make rust cargo pkgconf-pkg-config python3 \
    gtk4-devel libadwaita-devel gnome-desktop4-devel glib2-devel \
    gdk-pixbuf2-devel lcms2-devel libseccomp-devel glycin-loaders \
    bubblewrap fontconfig-devel cairo-devel graphene-devel hicolor-icon-theme
elif [[ "$ids" == *" debian "* || "$ids" == *" ubuntu "* ]]; then
  sudo apt-get update
  sudo apt-get install -- build-essential rustc cargo pkg-config python3 \
    libgtk-4-dev libadwaita-1-dev libgnome-desktop-4-dev libglib2.0-dev \
    libgdk-pixbuf-2.0-dev liblcms2-dev libseccomp-dev glycin-loaders \
    bubblewrap libfontconfig-dev libcairo2-dev libgraphene-1.0-dev \
    hicolor-icon-theme
elif [[ "$ids" == *" opensuse "* || "$ids" == *" suse "* ]]; then
  echo 'openSUSE package names vary by repository; install prerequisites manually.'
  echo 'Check required APIs with: make check'
  exit 2
else
  echo "ERROR: unsupported distribution: ${ID:-unknown}; use make check and install prerequisites manually" >&2
  exit 2
fi
check
