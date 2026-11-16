#include <gtk/gtk.h>
#include <gst/pbutils/pbutils.h>
#include "gtkgstmediafileprivate.h"
#include "policy.h"
#include "current_context.h"
#include "lifecycle.h"

void pivra_gpu_trace_binding(const char *stage) {
  static gint samples;
  if (!g_getenv("PIVRA_TRACE_VIDEO") || g_atomic_int_add(&samples,1) >= 128) return;
  /* Native getters do not make a context current or issue GL queries. */
  guintptr egl = gst_gl_context_get_current_gl_context(GST_GL_PLATFORM_EGL);
  guintptr glx = gst_gl_context_get_current_gl_context(GST_GL_PLATFORM_GLX);
  g_printerr("PIVRA_GPU_BINDING time_us=%" G_GINT64_FORMAT " stage=%s thread=%p gdk=%p egl=%" G_GUINTPTR_FORMAT " glx=%" G_GUINTPTR_FORMAT "\n",
             g_get_monotonic_time(),stage,(void *)g_thread_self(),(void *)gdk_gl_context_get_current(),egl,glx);
}
static void stock_retired(gpointer unused, GObject *object) {
  (void)unused;
  /* Finalization can run on a player thread: no GTK/GL calls here. */
  pivra_video_lifecycle("stock-stream-retired",object);
}
void pivra_gpu_watch_stock(GObject *stream) {
  if (g_getenv("PIVRA_TRACE_VIDEO")) g_object_weak_ref(stream,stock_retired,NULL);
}
GType pivra_gpu_backend_type(void) {
  if(!gst_init_check(NULL,NULL,NULL)) return G_TYPE_INVALID;
  return pivra_gpu_media_file_get_type();
}
int pivra_gpu_eligible(const char *uri) {
  GError *error=NULL;int result=0;
  if(!gst_init_check(NULL,NULL,&error)) { g_clear_error(&error);return 0; }
  GstDiscoverer *d=gst_discoverer_new(2*GST_SECOND,&error);
  if(!d) { g_clear_error(&error);return 0; }
  GstDiscovererInfo *info=gst_discoverer_discover_uri(d,uri,&error);
  if(info && !error && gst_discoverer_info_get_result(info)==GST_DISCOVERER_OK) {
    GList *videos=gst_discoverer_info_get_video_streams(info);
    GList *subtitles=gst_discoverer_info_get_subtitle_streams(info);
    if(!subtitles && g_list_length(videos)==1 && !gst_discoverer_video_info_is_image(videos->data)) {
      GstCaps *caps=gst_discoverer_stream_info_get_caps(videos->data);
      if(caps && g_getenv("PIVRA_TRACE_VIDEO")) {
        gchar *text=gst_caps_to_string(caps);g_printerr("PIVRA_GPU_DISCOVERY %s\n",text);g_free(text);
      }
      result=pivra_gpu_caps_eligible(caps);
      if(caps) gst_caps_unref(caps);
    }
    gst_discoverer_stream_info_list_free(videos);gst_discoverer_stream_info_list_free(subtitles);
  }
  if(info) gst_discoverer_info_unref(info);
  g_clear_error(&error);g_object_unref(d);return result;
}
