#!/usr/bin/env bash
# Verify actual installed Arch ownership; no package database refresh or installation.
set -euo pipefail
export LC_ALL=C
command -v pacman >/dev/null || { echo 'NOT RUN: Arch pacman ownership required' >&2; exit 77; }
owner() {
  local actual
  actual=$(pacman -Qoq -- "$1") || return 1
  [[ "$actual" == "$2" ]] || { printf 'FAIL owner: %s expected=%s actual=%s\n' "$1" "$2" "$actual" >&2; return 1; }
  printf 'PASS owner: %s -> %s\n' "$1" "$actual"
}
owner "$(command -v gst-inspect-1.0)" gstreamer
for module in gstreamer-gl-1.0 gstreamer-video-1.0 gstreamer-allocators-1.0 gstreamer-pbutils-1.0; do
  owner "$(pkg-config --variable=pcfiledir "$module")/$module.pc" gst-plugins-base-libs
done
for pair in 'playbin3:gst-plugins-base-libs' 'glupload:gst-plugins-base' \
  'glcolorconvert:gst-plugins-base' 'qtdemux:gst-plugins-good' \
  'h264parse:gst-plugins-bad-libs' 'avdec_h264:gst-libav' 'autoaudiosink:gst-plugins-good'; do
  element=${pair%%:*}; package=${pair#*:}
  filename=$(gst-inspect-1.0 "$element" | awk '$1 == "Filename" {print $2}')
  [[ -n "$filename" ]] || { echo "FAIL: $element has no plugin filename" >&2; exit 1; }
  owner "$filename" "$package"
done
