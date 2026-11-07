#!/usr/bin/env bash
# Capability check, not visual/hardware acceptance. Never changes plugin ranks.
set -euo pipefail
command -v gst-inspect-1.0 >/dev/null || { echo 'FAIL: gst-inspect-1.0 is required' >&2; exit 1; }
for element in playbin3 glupload glcolorconvert qtdemux h264parse avdec_h264 audioconvert audioresample autoaudiosink; do
  gst-inspect-1.0 "$element" >/dev/null || { echo "FAIL: missing GStreamer element $element" >&2; exit 1; }
done
echo 'PASS: GPU video runtime elements available (hardware/visual acceptance separate)'
