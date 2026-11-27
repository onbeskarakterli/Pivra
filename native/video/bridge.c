#include <gtk/gtk.h>
#include <gst/pbutils/pbutils.h>
#include "gtkgstmediafileprivate.h"
#include "policy.h"
#include "current_context.h"
#include "lifecycle.h"

/* Metadata from the existing eligibility discoverer, not another pipeline. */
static GPrivate last_information = G_PRIVATE_INIT(g_free);
static void trace_video_route(gboolean gpu, const char *reason)
{
  if (g_getenv("PIVRA_TRACE_VIDEO"))
    g_printerr("PIVRA_VIDEO_ROUTE backend=%s stage=discovery reason=%s\n",
               gpu ? "gpu" : "stock", reason);
}

char *pivra_gpu_take_information(void) {
  char *text = g_private_get(&last_information);
  g_private_set(&last_information, NULL);
  return text;
}

void pivra_gpu_trace_binding(const char *stage) {
  static gint samples;
  if (!g_getenv("PIVRA_TRACE_VIDEO") || g_atomic_int_add(&samples,1) >= 128) return;
  /* Native getters do not make a context current or issue GL queries. */
  guintptr egl = gst_gl_context_get_current_gl_context(GST_GL_PLATFORM_EGL);
  guintptr glx = gst_gl_context_get_current_gl_context(GST_GL_PLATFORM_GLX);
  g_printerr("PIVRA_GPU_BINDING time_us=%" G_GINT64_FORMAT " stage=%s thread=%p gdk=%p egl=%" G_GUINTPTR_FORMAT " glx=%" G_GUINTPTR_FORMAT " pid=%ld tid=%ld\n",
             g_get_monotonic_time(),stage,(void *)g_thread_self(),(void *)gdk_gl_context_get_current(),egl,glx,(long)getpid(),pivra_video_tid());
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
  g_private_replace(&last_information, NULL);
  GError *error=NULL;int result=0;
  if(!gst_init_check(NULL,NULL,&error)) {
    trace_video_route(FALSE, "gstreamer-initialization-failed");
    g_clear_error(&error);return 0;
  }
  GstDiscoverer *d=gst_discoverer_new(2*GST_SECOND,&error);
  if(!d) {
    trace_video_route(FALSE, "discoverer-unavailable");
    g_clear_error(&error);return 0;
  }
  GstDiscovererInfo *info=gst_discoverer_discover_uri(d,uri,&error);
  const char *reason = "video-discovery-failed";
  if(info && !error && gst_discoverer_info_get_result(info)==GST_DISCOVERER_OK) {
    GList *videos=gst_discoverer_info_get_video_streams(info);
    GList *subtitles=gst_discoverer_info_get_subtitle_streams(info);
    guint video_count = g_list_length(videos);
    if(videos) {
      GstDiscovererVideoInfo *v=videos->data;
      GstCaps *caps=gst_discoverer_stream_info_get_caps(videos->data);
      gchar *codec=caps ? gst_pb_utils_get_codec_description(caps) : NULL;
      GString *text=g_string_new(NULL);
      if(codec) g_string_append_printf(text,"Codec\t%s\n",codec);
      guint n=gst_discoverer_video_info_get_framerate_num(v), d=gst_discoverer_video_info_get_framerate_denom(v);
      if(n && d) g_string_append_printf(text,"Frame rate\t%.3f fps\n",(double)n/d);
      guint bitrate=gst_discoverer_video_info_get_bitrate(v);
      if(bitrate) g_string_append_printf(text,"Bitrate\t%u bit/s\n",bitrate);
      g_private_replace(&last_information,g_string_free(text,FALSE));
      g_free(codec);if(caps)gst_caps_unref(caps);
    }
    if(!subtitles && video_count==1 && !gst_discoverer_video_info_is_image(videos->data)) {
      GstCaps *caps=gst_discoverer_stream_info_get_caps(videos->data);
      if(caps && g_getenv("PIVRA_TRACE_VIDEO")) {
        gchar *text=gst_caps_to_string(caps);g_printerr("PIVRA_GPU_DISCOVERY %s\n",text);g_free(text);
      }
      result=pivra_gpu_caps_eligible(caps);
      reason = result ? "bt709-h264-eligible" : "video-metadata-outside-gpu-policy";
      if(caps) gst_caps_unref(caps);
    } else if (subtitles) reason = "subtitles";
    else if (video_count != 1) reason = "video-stream-count";
    else reason = "still-image-stream";
    gst_discoverer_stream_info_list_free(videos);gst_discoverer_stream_info_list_free(subtitles);
  } else if (g_getenv("PIVRA_TRACE_VIDEO")) {
    g_printerr("PIVRA_VIDEO_DISCOVERER status=%d detail=%s\n",
               info ? (int)gst_discoverer_info_get_result(info) : -1,
               error ? error->message : "none");
  }
  trace_video_route(result, reason);
  if(info) gst_discoverer_info_unref(info);
  g_clear_error(&error);g_object_unref(d);return result;
}
