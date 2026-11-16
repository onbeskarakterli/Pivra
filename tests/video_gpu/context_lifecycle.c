/* Real GDK/GStreamer ownership control, no decoded pixels or CPU fallback. */
#include "gtkgstsinkprivate.h"
#include <gtk/gtk.h>
#include <gst/gl/egl/gstgldisplay_egl.h>
#include "current_context.h"
int pivra_gpu_pending_presentations(void);
int pivra_gpu_live_contexts(void);

typedef struct { PivraGpuPaintable *paintable; GdkTexture *texture; } Delivery;
static gpointer enqueue(gpointer data) {
  Delivery *d = data;
  graphene_rect_t viewport = GRAPHENE_RECT_INIT(0, 0, 1, 1);
  for (int i = 0; i < 3; i++)
    pivra_gpu_paintable_queue_set_texture(d->paintable, d->texture, 1, &viewport, 0);
  return NULL;
}
static void presentation_check(void) {
  GMainContext *ctx = g_main_context_default();
  g_assert_true(g_main_context_acquire(ctx));
  GBytes *bytes = g_bytes_new_static("\0\0\0\xff", 4);
  Delivery d = { PIVRA_GPU_PAINTABLE(pivra_gpu_paintable_new()),
    gdk_memory_texture_new(1, 1, GDK_MEMORY_R8G8B8A8, bytes, 4) };
  g_bytes_unref(bytes);
  GThread *thread = g_thread_new("test-presentation", enqueue, &d);
  g_thread_join(thread);
  g_assert_cmpint(pivra_gpu_pending_presentations(), ==, 2);
  pivra_gpu_paintable_reset(d.paintable);
  while (g_main_context_pending(ctx)) g_main_context_iteration(ctx, FALSE);
  g_assert_cmpint(pivra_gpu_pending_presentations(), ==, 0);
  g_assert_cmpint(gdk_paintable_get_intrinsic_width(GDK_PAINTABLE(d.paintable)), ==, 0);
  /* With no main-loop owner, invoke_full may run inline on the producer.
   * Publication must remain queued until the GTK thread dispatches it. */
  g_main_context_release(ctx);
  thread = g_thread_new("test-unowned-context", enqueue, &d);
  g_thread_join(thread);
  g_assert_cmpint(pivra_gpu_pending_presentations(), ==, 2);
  g_assert_cmpint(gdk_paintable_get_intrinsic_width(GDK_PAINTABLE(d.paintable)), ==, 0);
  while (g_main_context_pending(ctx)) g_main_context_iteration(ctx, FALSE);
  /* New deliveries still work after reset; only the stale epoch is discarded. */
  g_assert_cmpint(gdk_paintable_get_intrinsic_width(GDK_PAINTABLE(d.paintable)), ==, 1);
  pivra_gpu_paintable_reset(d.paintable);
  g_object_unref(d.texture); g_object_unref(d.paintable);
  g_print("PASS stale queued frames discarded, queue remains bounded, new epoch works\n");
}

static void retired(gpointer data, GObject *object) {
  (void)object;
  g_atomic_int_set((gint *)data, 1);
}

int main(int argc, char **argv) {
  gtk_init();
  gst_init(&argc, &argv);
  const char *check = argc > 1 ? argv[1] : "all";
  if (!g_strcmp0(check, "presentation")) { presentation_check(); return 0; }
  gboolean failure = !g_strcmp0(check, "failure");
  if (failure) g_setenv("GST_GL_API", "invalid-test-api", TRUE);
  for (int i = 0; i < 8; i++) {
    GError *error = NULL;
    GdkGLContext *caller = gdk_display_create_gl_context(gdk_display_get_default(), &error);
    g_assert_no_error(error);
    g_assert_true(gdk_gl_context_realize(caller, &error));
    g_assert_no_error(error);
    gdk_gl_context_make_current(caller);
    PivraGpuPaintable *paintable = PIVRA_GPU_PAINTABLE(pivra_gpu_paintable_new());
    GstElement *bin = pivra_gpu_paintable_create_sink(paintable);
    if (failure) {
      g_assert_null(bin);
      g_object_unref(paintable);
      g_assert_cmpint(pivra_gpu_live_contexts(), ==, 0);
      g_assert_true(gdk_gl_context_get_current() == caller);
      gdk_gl_context_clear_current(); g_object_unref(caller);
      continue;
    }
    g_assert_nonnull(bin);
    gst_object_ref_sink(bin);
    GstIterator *iter = gst_bin_iterate_sinks(GST_BIN(bin));
    GValue item = G_VALUE_INIT;
    g_assert_cmpint(gst_iterator_next(iter, &item), ==, GST_ITERATOR_OK);
    PivraGpuSink *sink = PIVRA_GPU_SINK(g_value_get_object(&item));
    g_assert_nonnull(sink->gst_context);
    gint finalized = 0;
    g_object_weak_ref(G_OBJECT(sink->gst_context), retired, &finalized);
    if (gst_gl_context_get_gl_platform(sink->gst_context) == GST_GL_PLATFORM_EGL) {
      GstGLDisplayEGL *egl = gst_gl_display_egl_from_gl_display(sink->gst_display);
      g_assert_nonnull(egl);
      /* Borrowed GTK EGLDisplay must never be terminated by GStreamer. */
      if (!g_strcmp0(check, "all") || !g_strcmp0(check, "foreign"))
        g_assert_true(egl->foreign_display);
      gst_object_unref(egl);
    }
    g_value_unset(&item);
    gst_iterator_free(iter);
    gst_object_unref(bin);
    g_object_unref(paintable);
    if (!g_strcmp0(check, "all") || !g_strcmp0(check, "release"))
      g_assert_cmpint(g_atomic_int_get(&finalized), ==, 1);
    if (!g_strcmp0(check, "all") || !g_strcmp0(check, "current"))
      g_assert_true(gdk_gl_context_get_current() == caller);
    gdk_gl_context_clear_current();
    if (!g_strcmp0(check, "current")) {
      guintptr native = 1;
      GstGLPlatform platform = GST_GL_PLATFORM_EGL | GST_GL_PLATFORM_GLX;
      g_assert_cmpint(pivra_gpu_current_api(platform, &native), ==, GST_GL_API_NONE);
      g_assert_cmpuint(native, ==, 0);
      gdk_gl_context_make_current(caller);
      g_assert_cmpint(pivra_gpu_current_api(platform, &native), !=, GST_GL_API_NONE);
      g_assert_cmpuint(native, !=, 0);
      gdk_gl_context_clear_current();
    }
    g_object_unref(caller);
  }
  g_print("PASS context control=%s (8 cycles)\n", check);
  return 0;
}
