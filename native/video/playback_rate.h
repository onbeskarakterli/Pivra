/* SPDX-License-Identifier: LGPL-2.1-or-later */
#pragma once

#include <gst/gst.h>

/* Quarter-speed steps are exact in both GStreamer and the window action. */
enum { PIVRA_RATE_NORMAL = 4, PIVRA_RATE_MAX = 8 };

static gboolean
pivra_rate_seek(GstElement *pipeline, guint quarters, gint64 position)
{
  return gst_element_seek(pipeline, quarters / 4.0, GST_FORMAT_TIME,
                          GST_SEEK_FLAG_FLUSH | GST_SEEK_FLAG_ACCURATE,
                          GST_SEEK_TYPE_SET, position, GST_SEEK_TYPE_NONE, GST_CLOCK_TIME_NONE);
}

static gboolean
pivra_rate_in_segment(GstPad *video_pad, guint quarters)
{
  GstEvent *event = gst_pad_get_sticky_event(video_pad, GST_EVENT_SEGMENT, 0);
  if (!event)
    return FALSE;
  const GstSegment *segment;
  gst_event_parse_segment(event, &segment);
  gboolean accepted = segment->format == GST_FORMAT_TIME && segment->rate == quarters / 4.0;
  gst_event_unref(event);
  return accepted;
}
