/* Source-level unit control, NOT a GL driver or video test. The generated
 * header is the verbatim upstream selection function, clean or patched. */
#include <glib.h>
#include <epoxy/egl.h>
#include <stdint.h>

typedef struct { EGLDisplay egl_display; } GdkDisplayPrivate;
typedef struct { gboolean have_egl_pixel_format_float; GdkDisplayPrivate priv; } GdkDisplay;
static GdkDisplayPrivate *gdk_display_get_instance_private(GdkDisplay *d) { return &d->priv; }
static guint rate(GdkDisplay *d, gpointer display, gpointer config) {
  (void)d; (void)display; (void)config; return 0;
}
static struct { guint (*rate_egl_config)(GdkDisplay *,gpointer,gpointer); } display_class={rate};
#define GDK_DISPLAY_GET_CLASS(d) (&display_class)
#define GDK_GL_ERROR g_quark_from_static_string("test-gdk-gl-error")
#define GDK_GL_ERROR_NOT_AVAILABLE 0
#define GDK_PROFILER_CURRENT_TIME 0
#define gdk_profiler_end_mark(a,b,c) ((void)0)
#define _(text) (text)

static EGLint masks[4];
static int length;
static gboolean fail_query;
static EGLBoolean fake_choose(EGLDisplay display, const EGLint *attrs,
                              EGLConfig *configs, EGLint size, EGLint *count) {
  (void)display;
  EGLint wanted=EGL_OPENGL_ES_BIT, surface=0, color=0, alpha=0, r=0, g=0, b=0;
  int i=0;
  for (; i<30 && attrs[i]!=EGL_NONE; i+=2) {
    switch (attrs[i]) {
      case EGL_RENDERABLE_TYPE: wanted=attrs[i+1]; break;
      case EGL_SURFACE_TYPE: surface=attrs[i+1]; break;
      case EGL_COLOR_BUFFER_TYPE: color=attrs[i+1]; break;
      case EGL_ALPHA_SIZE: alpha=attrs[i+1]; break;
      case EGL_RED_SIZE: r=attrs[i+1]; break;
      case EGL_GREEN_SIZE: g=attrs[i+1]; break;
      case EGL_BLUE_SIZE: b=attrs[i+1]; break;
      default: g_assert_not_reached();
    }
  }
  g_assert_cmpint(i,<,30);
  g_assert_cmpint(surface,==,EGL_WINDOW_BIT);
  g_assert_cmpint(color,==,EGL_RGB_BUFFER);
  g_assert_cmpint(alpha,==,8);
  g_assert_cmpint(r,==,8); g_assert_cmpint(g,==,8); g_assert_cmpint(b,==,8);
  if (fail_query) return EGL_FALSE;
  *count=0;
  for (int n=0; n<length; n++) {
    if (wanted!=EGL_DONT_CARE && (masks[n]&wanted)!=wanted) continue;
    if (configs && *count<size) configs[*count]=(EGLConfig)(uintptr_t)(n+1);
    (*count)++;
  }
  if (configs) *count=MIN(*count,size);
  return EGL_TRUE;
}
static G_GNUC_UNUSED EGLBoolean fake_attrib(EGLDisplay display, EGLConfig config,
                                           EGLint key, EGLint *value) {
  (void)display;
  g_assert_cmpint(key,==,EGL_RENDERABLE_TYPE);
  *value=masks[(uintptr_t)config-1]; return EGL_TRUE;
}
#undef eglChooseConfig
#undef eglGetConfigAttrib
#define eglChooseConfig fake_choose
#define eglGetConfigAttrib fake_attrib
#include "gtk-selection.h"

int main(void) {
  const struct { const char *name; EGLint masks[4]; int n, baseline, patched; gboolean error; } cases[]={
    {"modern-only, skip ES2/OpenVG-only candidates",{EGL_OPENGL_ES2_BIT,EGL_OPENVG_BIT,EGL_OPENGL_BIT,EGL_OPENGL_ES3_BIT_KHR},4,0,3,FALSE},
    {"desktop GL alone is sufficient",{EGL_OPENGL_BIT},1,0,1,FALSE},
    {"GLES3 alone is sufficient",{EGL_OPENGL_ES3_BIT_KHR},1,0,1,FALSE},
    {"ES2 only is below GTK requirements",{EGL_OPENGL_ES2_BIT},1,0,0,FALSE},
    {"OpenVG only is not a GL config",{EGL_OPENVG_BIT},1,0,0,FALSE},
    {"existing ES1-compatible modern config",{EGL_OPENGL_ES_BIT|EGL_OPENGL_BIT},1,1,1,FALSE},
    {"ES1 alone is not a modern config",{EGL_OPENGL_ES_BIT},1,1,0,FALSE},
    {"actual EGL query error remains failure",{EGL_OPENGL_BIT},1,0,0,TRUE},
  };
  for (guint i=0; i<G_N_ELEMENTS(cases); i++) {
    memcpy(masks,cases[i].masks,sizeof(masks)); length=cases[i].n; fail_query=cases[i].error;
    GdkDisplay display={0}; GError *error=NULL;
    EGLConfig result=gdk_display_create_egl_config(&display,0,&error);
    int expected=PATCHED_VARIANT ? cases[i].patched:cases[i].baseline;
    g_assert_cmpuint((uintptr_t)result,==,(uintptr_t)expected);
    if (expected) g_assert_no_error(error);
    else g_assert_error(error,GDK_GL_ERROR,GDK_GL_ERROR_NOT_AVAILABLE);
    g_clear_error(&error);
    g_print("PASS source-selection %s %s\n",PATCHED_VARIANT ? "patched":"baseline",cases[i].name);
  }
  return 0;
}
