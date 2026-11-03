/* Conservative encoded-stream eligibility; no defaults for missing metadata. */
#pragma once
#include <gst/video/video.h>
static inline gboolean pivra_gpu_caps_eligible(const GstCaps *caps) {
  if(!caps || gst_caps_get_size(caps)!=1) return FALSE;
  const GstStructure *s=gst_caps_get_structure(caps,0);
  const char *tag=gst_structure_get_string(s,"colorimetry");
  GstVideoColorimetry c;guint depth;gint signed_depth,width,height;
  if(!gst_structure_get_uint(s,"bit-depth-luma",&depth)) {
    if(!gst_structure_get_int(s,"bit-depth-luma",&signed_depth) || signed_depth<0) return FALSE;
    depth=(guint)signed_depth;
  }
  return gst_structure_has_name(s,"video/x-h264") && tag && gst_video_colorimetry_from_string(&c,tag) &&
    c.primaries==GST_VIDEO_COLOR_PRIMARIES_BT709 && c.transfer==GST_VIDEO_TRANSFER_BT709 &&
    c.matrix==GST_VIDEO_COLOR_MATRIX_BT709 &&
    (c.range==GST_VIDEO_COLOR_RANGE_16_235 || c.range==GST_VIDEO_COLOR_RANGE_0_255) &&
    depth==8 && gst_structure_get_int(s,"width",&width) && gst_structure_get_int(s,"height",&height) &&
    width>0 && height>0 && (gint64)width*height<=8388608;
}
