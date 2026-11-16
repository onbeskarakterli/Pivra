/* Isolated negative control for GTK 4.24.1's Wayland display wrapping policy.
 * No GTK replacement, pixel decoder or production workaround. */
#include <gtk/gtk.h>
#include <gdk/wayland/gdkwayland.h>
#include <gst/gl/gl.h>
#include <gst/gl/wayland/gstgldisplay_wayland.h>
#include <gst/gl/egl/gstgldisplay_egl.h>
#include <epoxy/egl.h>
#include <unistd.h>
#include <stdio.h>

int main(int argc, char **argv) {
  gtk_init(); gst_init(&argc, &argv);
  GdkDisplay *display = gdk_display_get_default();
  if (!GDK_IS_WAYLAND_DISPLAY(display)) {
    g_printerr("NOT RUN Wayland-specific retirement control\n"); return 77;
  }
  gboolean borrowed = argc > 1 && g_str_equal(argv[1], "borrowed");
  GError *error = NULL;
  GdkGLContext *context = gdk_display_create_gl_context(display, &error);
  g_assert_no_error(error);
  g_assert_true(gdk_gl_context_realize(context, &error));
  g_assert_no_error(error);
  gdk_gl_context_make_current(context);
  EGLDisplay native = gdk_wayland_display_get_egl_display(display);
  EGLContext handle = (EGLContext)gst_gl_context_get_current_gl_context(GST_GL_PLATFORM_EGL);
  g_assert_true(handle != EGL_NO_CONTEXT);
  EGLint version;
  g_assert_true(eglQueryContext(native, handle, EGL_CONTEXT_CLIENT_VERSION, &version));
  gdk_gl_context_clear_current();

  GstGLDisplay *wrapped = borrowed
    ? GST_GL_DISPLAY(gst_gl_display_egl_new_with_egl_display(native))
    : GST_GL_DISPLAY(gst_gl_display_wayland_new_with_display(gdk_wayland_display_get_wl_display(display)));
  GstGLDisplayEGL *egl = gst_gl_display_egl_from_gl_display(wrapped);
  g_assert_true(egl->display == native);
  g_assert_cmpint(egl->foreign_display, ==, borrowed);
  gst_object_unref(egl); gst_object_unref(wrapped);
  gboolean valid = eglQueryContext(native, handle, EGL_CONTEXT_CLIENT_VERSION, &version);
  EGLint result = eglGetError();
  g_print("RETIREMENT_CONTROL policy=%s gtk_context_valid=%s egl_error=0x%x\n",
          borrowed ? "borrowed" : "gtk424-wl-wrapper", valid ? "true" : "false", result);
  g_assert_cmpint(valid, ==, borrowed);
  if (!borrowed) {
    /* The negative control deliberately terminated this process's EGLDisplay.
     * Do not make GL calls or assert successful GTK cleanup on that display. */
    g_print("MECHANISM_REPRODUCED external wrapper invalidated GTK context; native AMD acceptance NOT established\n");
    fflush(stdout);
    _exit(0);
  }
  gdk_gl_context_make_current(context);
  g_assert_true((EGLContext)gst_gl_context_get_current_gl_context(GST_GL_PLATFORM_EGL) == handle);
  gdk_gl_context_clear_current(); g_object_unref(context);
  g_print("PASS borrowed wrapper preserves GTK context\n");
  return 0;
}
