/* Preserve the validated native Wayland display/window integration without
 * allowing GStreamer to terminate GTK's EGLDisplay. Public APIs only. */
#pragma once
#include <gst/gl/wayland/gstgldisplay_wayland.h>
#include <gst/gl/egl/gstgldisplay_egl.h>

/* Since 1.26. A weak ELF reference retains the >=1.24 runtime contract: old
 * runtimes keep the existing safe direct-EGL wrapper, never an owning wrapper.
 * libgstgl is already a mandatory strong dependency of this adapter. */
extern void gst_gl_display_egl_set_foreign(GstGLDisplayEGL *, gboolean)
    __attribute__((weak));

static GstGLDisplay *
pivra_wayland_display(gpointer wayland, gpointer owned_egl)
{
  if (!wayland || !owned_egl) return NULL;
  if (!gst_gl_display_egl_set_foreign)
    return GST_GL_DISPLAY(gst_gl_display_egl_new_with_egl_display(owned_egl));

  GstGLDisplay *display=GST_GL_DISPLAY(gst_gl_display_wayland_new_with_display(wayland));
  if (!display) return NULL;
  GstGLDisplayEGL *egl=gst_gl_display_egl_from_gl_display(display);
  if (!egl || gst_gl_display_get_handle(GST_GL_DISPLAY(egl)) != (guintptr)owned_egl) {
    gst_clear_object(&egl);
    gst_object_unref(display);
    return NULL;
  }
  /* from_gl_display caches this object; mark it before creating any worker
   * context and retain the native Wayland display type used by v0.2.5. */
  gst_gl_display_egl_set_foreign(egl,TRUE);
  gst_object_unref(egl);
  return display;
}
