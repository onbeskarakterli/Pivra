/* Opt-in, bounded lifecycle events. Never called for each playback frame. */
#pragma once
#include <glib.h>
static inline void pivra_video_lifecycle(const char *stage, gpointer owner) {
  static gint events;
  if (g_getenv("PIVRA_TRACE_VIDEO") && g_atomic_int_add(&events, 1) < 128)
    g_printerr("PIVRA_GPU_LIFECYCLE time_us=%" G_GINT64_FORMAT " stage=%s owner=%p thread=%p\n",
               g_get_monotonic_time(), stage, owner, (void *)g_thread_self());
}
