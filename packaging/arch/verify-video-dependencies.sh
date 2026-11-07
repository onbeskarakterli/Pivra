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
# Filename + owner are both strict: do not infer another plugin's ownership
# from a package split or accept the former OpenGL owner as a fallback.
for spec in 'playbin3:gst-plugins-base-libs:libgstplayback.so' \
  'glupload:gst-plugins-base-libs:libgstopengl.so' \
  'glcolorconvert:gst-plugins-base-libs:libgstopengl.so' \
  'qtdemux:gst-plugins-good:libgstisomp4.so' \
  'h264parse:gst-plugins-bad-libs:libgstvideoparsersbad.so' \
  'avdec_h264:gst-libav:libgstlibav.so' \
  'autoaudiosink:gst-plugins-good:libgstautodetect.so'; do
  IFS=: read -r element package expected_filename <<< "$spec"
  filename=$(gst-inspect-1.0 "$element" | awk '$1 == "Filename" {print $2}')
  [[ "$filename" == /* && "${filename##*/}" == "$expected_filename" ]] || {
    printf 'FAIL plugin file: %s expected=%s actual=%s\n' "$element" "$expected_filename" "$filename" >&2
    exit 1
  }
  owner "$filename" "$package"
done
