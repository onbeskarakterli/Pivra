/* Tests the policy shipped inside the experimental GTK patch. */
#include "gtkgstbt1886private.h"
#include <assert.h>
#include <stdio.h>

int main (int argc, char **argv)
{
  GstVideoInfo info;
  gst_init (&argc, &argv);
  gst_video_info_init (&info);
  gst_video_info_set_format (&info, GST_VIDEO_FORMAT_RGB, 192, 96);
  info.colorimetry = (GstVideoColorimetry) {
    GST_VIDEO_COLOR_RANGE_0_255, GST_VIDEO_COLOR_MATRIX_RGB,
    GST_VIDEO_TRANSFER_BT709, GST_VIDEO_COLOR_PRIMARIES_BT709
  };
  assert (gtk_gst_bt1886_eligible (&info));
  for (guint i = 0; i < 256; i++)
    {
      float linear = gtk_gst_bt1886_linear (i);
      assert (isfinite (linear) && linear >= 0 && linear <= 1);
      assert (fabs (linear - pow ((double) i / 255., 2.4)) < 0.000001);
      if (i > 0) assert (linear > gtk_gst_bt1886_linear (i - 1));
    }
  assert (gtk_gst_bt1886_linear (0) == 0);
  assert (gtk_gst_bt1886_linear (255) == 1);
  /* Do not reinterpret PQ, HLG, sRGB, BT.2020, unknown or limited RGB. */
  for (guint i = 0; i <= GST_VIDEO_TRANSFER_BT601; i++)
    {
      info.colorimetry.transfer = i;
      assert (gtk_gst_bt1886_eligible (&info) == (i == GST_VIDEO_TRANSFER_BT709));
    }
  info.colorimetry.transfer = GST_VIDEO_TRANSFER_BT709;
  for (guint i = 0; i <= GST_VIDEO_COLOR_PRIMARIES_EBU3213; i++)
    {
      info.colorimetry.primaries = i;
      assert (gtk_gst_bt1886_eligible (&info) == (i == GST_VIDEO_COLOR_PRIMARIES_BT709));
    }
  info.colorimetry.primaries = GST_VIDEO_COLOR_PRIMARIES_BT709;
  for (guint i = 0; i <= GST_VIDEO_COLOR_MATRIX_BT2020; i++)
    {
      info.colorimetry.matrix = i;
      assert (gtk_gst_bt1886_eligible (&info) == (i == GST_VIDEO_COLOR_MATRIX_RGB));
    }
  info.colorimetry.matrix = GST_VIDEO_COLOR_MATRIX_RGB;
  info.colorimetry.range = GST_VIDEO_COLOR_RANGE_16_235;
  assert (!gtk_gst_bt1886_eligible (&info));
  info.colorimetry.range = GST_VIDEO_COLOR_RANGE_UNKNOWN;
  assert (!gtk_gst_bt1886_eligible (&info));
  info.colorimetry.range = GST_VIDEO_COLOR_RANGE_0_255;
  info.finfo = gst_video_format_get_info (GST_VIDEO_FORMAT_RGBA);
  assert (!gtk_gst_bt1886_eligible (&info));
  info.finfo = gst_video_format_get_info (GST_VIDEO_FORMAT_I420);
  assert (!gtk_gst_bt1886_eligible (&info));
  gsize size;
  assert (gtk_gst_bt1886_frame_size (720, 1142, &size) && size == 13155840);
  assert (gtk_gst_bt1886_frame_size (4096, 4096, &size) && size == 268435456);
  assert (!gtk_gst_bt1886_frame_size (4097, 4096, &size));
  assert (!gtk_gst_bt1886_frame_size (0, 1, &size));
  assert (!gtk_gst_bt1886_frame_size (1, 0, &size));
  assert (!gtk_gst_bt1886_frame_size (G_MAXSIZE, 2, &size));
  assert (!gtk_gst_bt1886_frame_size (G_MAXSIZE / 8, 1, &size));
  puts ("PASS BT.1886 policy: 256 values, monotonic/endpoints, metadata/range/format exclusions, allocation budget/overflow");
  return 0;
}
