/* Opt-in, bounded lifecycle events. Never called for each playback frame. */
#pragma once
#include <glib.h>
#include <unistd.h>
#include <sys/syscall.h>
static inline long pivra_video_tid(void) { return syscall(SYS_gettid); }
static inline void pivra_video_lifecycle(const char *stage, gpointer owner) {
  static gint events;
  if (g_getenv("PIVRA_TRACE_VIDEO") && g_atomic_int_add(&events, 1) < 128)
    g_printerr("PIVRA_GPU_LIFECYCLE time_us=%" G_GINT64_FORMAT " stage=%s owner=%p thread=%p pid=%ld tid=%ld\n",
               g_get_monotonic_time(), stage, owner, (void *)g_thread_self(), (long)getpid(), pivra_video_tid());
}
