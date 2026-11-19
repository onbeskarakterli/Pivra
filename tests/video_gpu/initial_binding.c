/* Real EGL binding control, not a pixel/decoder or hardware acceptance test.
 * Another public EGL client unbinds; GDK must normalize its cached pointer.
 * No normal initializer may require an incoming current context. */
#include <gtk/gtk.h>
#include <gst/gl/gl.h>
#include <epoxy/egl.h>
#include "gtkgstpaintableprivate.h"
#include "gtkgstsinkprivate.h"
#include <gst/gl/egl/gstgldisplay_egl.h>
#include <gst/gl/wayland/gstgldisplay_wayland.h>

int main(int argc, char **argv) {
  gtk_init(); gst_init(&argc,&argv);
  GtkWidget *window=NULL;
  GdkSurface *surface=NULL;
  for (int i=1; i<argc; i++) if (g_str_equal(argv[i],"--surface")) {
    window=gtk_window_new();
    gtk_widget_realize(window);
    surface=gtk_native_get_surface(GTK_NATIVE(window));
    g_assert_nonnull(surface);
  }
  GError *error=NULL;
  GdkGLContext *caller=gdk_display_create_gl_context(gdk_display_get_default(),&error);
  if (!caller || !gdk_gl_context_realize(caller,&error)) {
    g_printerr("NOT RUN initial EGL context: %s\n",error ? error->message : "unavailable");
    g_clear_error(&error);g_clear_object(&caller);return 77;
  }
  gdk_gl_context_make_current(caller);
  EGLDisplay display=eglGetCurrentDisplay();
  EGLContext native=eglGetCurrentContext();
  if (display==EGL_NO_DISPLAY || native==EGL_NO_CONTEXT) {
    g_printerr("NOT RUN initial-binding control requires EGL, not GLX\n");
    gdk_gl_context_clear_current();g_object_unref(caller);return 77;
  }
  g_assert_true(eglMakeCurrent(display,EGL_NO_SURFACE,EGL_NO_SURFACE,EGL_NO_CONTEXT));
  g_assert_null(gdk_gl_context_get_current());
  g_assert_true(eglGetCurrentContext()==EGL_NO_CONTEXT);
  g_print("INITIAL_BINDING gdk-normalized=true native-binding=none\n");
  PivraGpuPaintable *paintable=PIVRA_GPU_PAINTABLE(pivra_gpu_paintable_new());
  if (surface) pivra_gpu_paintable_realize(paintable,surface);
  GstElement *bin=pivra_gpu_paintable_create_sink(paintable);
  gboolean succeeded=bin!=NULL;
  if (bin) {
    gst_object_ref_sink(bin);
    GstIterator *iter=gst_bin_iterate_sinks(GST_BIN(bin));
    GValue value=G_VALUE_INIT;
    g_assert_cmpint(gst_iterator_next(iter,&value),==,GST_ITERATOR_OK);
    PivraGpuSink *sink=PIVRA_GPU_SINK(g_value_get_object(&value));
    if (surface) g_assert_true(gdk_gl_context_get_surface(sink->gdk_context)==surface);
    GstGLDisplayEGL *egl=gst_gl_display_egl_from_gl_display(sink->gst_display);
    g_assert_nonnull(egl);
    g_assert_cmpuint(gst_gl_display_get_handle(GST_GL_DISPLAY(egl)),==,(guintptr)display);
    g_print("INITIAL_BINDING display_type=%s borrowed=%s\n",G_OBJECT_TYPE_NAME(sink->gst_display),egl->foreign_display ? "true":"false");
    for (int i=1; i<argc; i++) {
      if (g_str_equal(argv[i],"--require-borrowed")) g_assert_true(egl->foreign_display);
      if (g_str_has_prefix(G_OBJECT_TYPE_NAME(gdk_display_get_default()),"GdkWayland") && g_str_equal(argv[i],"--expect-wayland"))
        g_assert_true(GST_IS_GL_DISPLAY_WAYLAND(sink->gst_display));
      if (g_str_has_prefix(G_OBJECT_TYPE_NAME(gdk_display_get_default()),"GdkWayland") && g_str_equal(argv[i],"--expect-egl"))
        g_assert_true(GST_IS_GL_DISPLAY_EGL(sink->gst_display));
    }
    gst_object_unref(egl);g_value_unset(&value);gst_iterator_free(iter);
    gst_object_unref(bin);
  }
  if (surface) pivra_gpu_paintable_unrealize(paintable,surface);
  g_object_unref(paintable);
  /* The actual incoming binding was NONE, so leaving GDK TLS and EGL unbound
   * is the correct restoration. A valid caller context can be rebound later. */
  gboolean unbound=gdk_gl_context_get_current()==NULL && eglGetCurrentContext()==EGL_NO_CONTEXT;
  gdk_gl_context_clear_current();
  gdk_gl_context_make_current(caller);
  gboolean reusable=eglGetCurrentContext()==native;
  gdk_gl_context_clear_current();g_object_unref(caller);
  if (window) gtk_window_destroy(GTK_WINDOW(window));
  g_print("INITIAL_BINDING gpu_sink=%s restored_unbound=%s caller_reusable=%s\n",
      succeeded ? "ready":"failed",unbound ? "true":"false",reusable ? "true":"false");
  return succeeded && unbound && reusable ? 0:1;
}
