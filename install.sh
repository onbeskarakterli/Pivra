#!/usr/bin/env bash
# Pivra source installer. Requires supported distro packages and sudo for system installation.
# Usage: bash install.sh [--check|--deps-only|--build-only|--install] [--prefix=/usr/local]
set -euo pipefail
root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$root"

mode=install
prefix=/usr/local
for arg in "$@"; do
  case "$arg" in
    --check) mode=check ;;
    --deps-only) mode=deps ;;
    --build-only) mode=build ;;
    --install) mode=install ;;
    --prefix=*) prefix="${arg#--prefix=}" ;;
    -h|--help)
      echo 'Usage: bash install.sh [--check|--deps-only|--build-only|--install] [--prefix=/usr/local]'
      exit 0 ;;
    *) echo "Unknown option: $arg" >&2; exit 2 ;;
  esac
done

[[ "$prefix" == /* && "$prefix" != / && "$prefix" != *..* ]] ||
  { echo 'ERROR: --prefix must be an absolute non-root path without ..' >&2; exit 2; }
[[ -f /etc/os-release ]] || { echo 'ERROR: /etc/os-release unavailable' >&2; exit 1; }
# OS identifiers are distro metadata; do not execute arbitrary code from them.
os_id="$(sed -n 's/^ID=//p' /etc/os-release | head -n1 | tr -d '"')"
os_like="$(sed -n 's/^ID_LIKE=//p' /etc/os-release | head -n1 | tr -d '"')"
distro="$os_id"
case " $os_id $os_like " in
  *" arch "*) distro=arch ;;
  *" fedora "*|*" rhel "*) distro=fedora ;;
  *" ubuntu "*|*" debian "*) distro=debian ;;
  *" opensuse "*|*" suse "*) distro=opensuse ;;
esac

check_deps() {
  local bad=0 command_name
  for command_name in cargo rustc cc pkg-config bash install python3; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
      echo "MISSING COMMAND: $command_name"
      bad=1
    fi
  done
  if command -v rustc >/dev/null 2>&1; then
    local version
    version="$(rustc --version | awk '{print $2}')"
    if [[ "$(printf '%s\n%s\n' '1.85.0' "$version" | sort -V | head -n1)" != '1.85.0' ]]; then
      echo "ERROR: Rust >= 1.85 required; found $version"
      bad=1
    fi
  fi
  if command -v pkg-config >/dev/null 2>&1; then
    pkg-config --atleast-version=4.16 gtk4 || { echo 'MISSING: GTK4 >= 4.16 development files'; bad=1; }
    pkg-config --atleast-version=1.2 libadwaita-1 || { echo 'MISSING: libadwaita >= 1.2 development files'; bad=1; }
    local module
    for module in gnome-desktop-4 gio-2.0 gobject-2.0 gdk-pixbuf-2.0 lcms2 libseccomp; do
      pkg-config --exists "$module" || { echo "MISSING PKG-CONFIG MODULE: $module"; bad=1; }
    done
  fi
  command -v bwrap >/dev/null 2>&1 || { echo 'MISSING: Bubblewrap (bwrap)'; bad=1; }
  if command -v ldconfig >/dev/null 2>&1; then
    ldconfig -p 2>/dev/null | grep -q 'liblcms2.so.2' || echo 'NOTE: liblcms2 runtime linker entry not detected'
  fi
  return "$bad"
}

install_dependencies() {
  echo "Installing build and runtime prerequisites for: $distro"
  case "$distro" in
    arch)
      sudo pacman -S --needed -- \
        base-devel rust cargo pkgconf python gtk4 libadwaita gnome-desktop-4 \
        gdk-pixbuf2 glycin bubblewrap libseccomp lcms2 fontconfig cairo graphene \
        hicolor-icon-theme ;;
    fedora)
      sudo dnf install -- \
        gcc make rust cargo pkgconf-pkg-config python3 gtk4-devel libadwaita-devel \
        gnome-desktop4-devel glib2-devel gdk-pixbuf2-devel lcms2-devel libseccomp-devel \
        glycin-loaders bubblewrap fontconfig-devel cairo-devel graphene-devel \
        hicolor-icon-theme ;;
    debian)
      sudo apt-get update
      sudo apt-get install -- \
        build-essential rustc cargo pkg-config python3 libgtk-4-dev libadwaita-1-dev \
        libgnome-desktop-4-dev libglib2.0-dev libgdk-pixbuf-2.0-dev liblcms2-dev \
        libseccomp-dev bubblewrap glycin-loaders libfontconfig-dev libcairo2-dev \
        libgraphene-1.0-dev hicolor-icon-theme ;;
    opensuse)
      sudo zypper install -- \
        gcc make rust cargo pkgconf-pkg-config python3 gtk4-devel libadwaita-devel \
        gnome-desktop-devel glib2-devel gdk-pixbuf-devel liblcms2-devel \
        libseccomp-devel bubblewrap fontconfig-devel cairo-devel \
        graphene-devel hicolor-icon-theme ;;
    *)
      echo "ERROR: Unsupported distro ID=$os_id. Install prerequisites manually; use --check." >&2
      return 1 ;;
  esac
}

if [[ "$mode" == check ]]; then
  check_deps
  exit $?
fi

if [[ "$mode" == deps || "$mode" == install ]]; then
  echo 'System dependencies may be installed using sudo and the distribution package manager.'
  echo 'The package manager will display its transaction and request confirmation.'
  install_dependencies
fi
if [[ "$mode" == deps ]]; then exit 0; fi
check_deps || { echo 'ERROR: prerequisite check failed; build stopped.' >&2; exit 1; }
bash "$root/packaging/linux/build-install.sh" build
if [[ "$mode" == build ]]; then exit 0; fi

# Build as the normal user, install only the finished files with privilege.
if [[ -w "$prefix" || "$prefix" == "$HOME/"* ]]; then
  PREFIX="$prefix" bash "$root/packaging/linux/build-install.sh" install
else
  sudo env PREFIX="$prefix" bash "$root/packaging/linux/build-install.sh" install
fi
echo 'Installation complete. Run: pivra --version'
echo 'NOTE: Manual installations are not tracked by a distribution package manager.'
