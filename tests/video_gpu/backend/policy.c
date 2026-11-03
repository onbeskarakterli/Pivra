#include "filter.c"
int main(int argc, char **argv) {
  GstVideoInfo info;
  gst_init(&argc, &argv);
  gst_video_info_set_format(&info, GST_VIDEO_FORMAT_RGBA, 192, 96);
  info.colorimetry = (GstVideoColorimetry) {GST_VIDEO_COLOR_RANGE_0_255,
    GST_VIDEO_COLOR_MATRIX_RGB, GST_VIDEO_TRANSFER_BT709, GST_VIDEO_COLOR_PRIMARIES_BT709};
  g_assert_true(eligible(&info));
  info.colorimetry.transfer = GST_VIDEO_TRANSFER_SRGB; g_assert_false(eligible(&info));
  info.colorimetry.transfer = GST_VIDEO_TRANSFER_SMPTE2084; g_assert_false(eligible(&info));
  info.colorimetry.transfer = GST_VIDEO_TRANSFER_ARIB_STD_B67; g_assert_false(eligible(&info));
  info.colorimetry.transfer = GST_VIDEO_TRANSFER_UNKNOWN; g_assert_false(eligible(&info));
  info.colorimetry.transfer = GST_VIDEO_TRANSFER_BT709;
  info.colorimetry.primaries = GST_VIDEO_COLOR_PRIMARIES_BT2020; g_assert_false(eligible(&info));
  info.colorimetry.primaries = GST_VIDEO_COLOR_PRIMARIES_BT709;
  info.colorimetry.range = GST_VIDEO_COLOR_RANGE_16_235; g_assert_false(eligible(&info));
  GstElement *bin = gst_bin_new(NULL);
  GstCaps *caps = gst_caps_from_string("video/x-raw,format=I420,width=192,height=96,colorimetry=bt709");
  GstEvent *event = gst_event_new_caps(caps);
  GstPadProbeInfo probe = { .type=GST_PAD_PROBE_TYPE_EVENT_DOWNSTREAM, .data=event };
  g_assert_cmpint(source_metadata(NULL, &probe, bin), ==, GST_PAD_PROBE_OK);
  gst_event_unref(event); gst_caps_unref(caps);
  caps = gst_caps_from_string("video/x-raw,format=I420,width=192,height=96,colorimetry=bt2020");
  event = gst_event_new_caps(caps); probe.data = event;
  g_assert_cmpint(source_metadata(NULL, &probe, bin), ==, GST_PAD_PROBE_DROP);
  gst_event_unref(event); gst_caps_unref(caps);
  caps = gst_caps_from_string("video/x-raw,format=I420,width=192,height=96");
  event = gst_event_new_caps(caps); probe.data = event;
  g_assert_cmpint(source_metadata(NULL, &probe, bin), ==, GST_PAD_PROBE_DROP);
  gst_event_unref(event); gst_caps_unref(caps); gst_object_unref(bin);
  return 0;
}
