/* Bounded read-only EGL configuration control on a PRIVATE Wayland connection.
 * Does not initialize GTK, replace dispatch functions, or touch GTK's display. */
#include <stdio.h>
#include <epoxy/egl.h>
#include <wayland-client.h>

static void query(EGLDisplay display, const char *name, EGLint renderable,
                  EGLint alpha, EGLint size, int hdr) {
  EGLint attrs[20], i=0, count=-1;
  attrs[i++]=EGL_SURFACE_TYPE; attrs[i++]=EGL_WINDOW_BIT;
  attrs[i++]=EGL_COLOR_BUFFER_TYPE; attrs[i++]=EGL_RGB_BUFFER;
  attrs[i++]=EGL_RED_SIZE; attrs[i++]=hdr ? 9:8;
  attrs[i++]=EGL_GREEN_SIZE; attrs[i++]=hdr ? 9:8;
  attrs[i++]=EGL_BLUE_SIZE; attrs[i++]=hdr ? 9:8;
  attrs[i++]=EGL_ALPHA_SIZE; attrs[i++]=alpha;
  if (renderable != 0) { attrs[i++]=EGL_RENDERABLE_TYPE; attrs[i++]=renderable; }
  if (hdr && epoxy_has_egl_extension(display,"EGL_EXT_pixel_format_float")) {
    attrs[i++]=EGL_COLOR_COMPONENT_TYPE_EXT; attrs[i++]=EGL_DONT_CARE;
  }
  attrs[i++]=EGL_NONE;
  eglGetError();
  EGLBoolean ok=eglChooseConfig(display,attrs,NULL,size,&count);
  EGLint error=eglGetError();
  printf("%s {\"name\":\"%s\",\"ok\":%s,\"count\":%d,\"error\":%d,\"size\":%d,\"renderable_request\":%d,\"alpha\":%d,\"min_rgb\":%d}\n",
         hdr ? "EGL_HDR_QUERY":"EGL_CONFIG_QUERY",name,ok ? "true":"false",count,error,size,renderable,alpha,hdr ? 9:8);
  if (ok && count > 0) {
    EGLConfig config;
    EGLint n=0, id=0, mask=0, red=0, green=0, blue=0, a=0;
    if (eglChooseConfig(display,attrs,&config,1,&n) && n==1) {
      eglGetConfigAttrib(display,config,EGL_CONFIG_ID,&id);
      eglGetConfigAttrib(display,config,EGL_RENDERABLE_TYPE,&mask);
      eglGetConfigAttrib(display,config,EGL_RED_SIZE,&red);
      eglGetConfigAttrib(display,config,EGL_GREEN_SIZE,&green);
      eglGetConfigAttrib(display,config,EGL_BLUE_SIZE,&blue);
      eglGetConfigAttrib(display,config,EGL_ALPHA_SIZE,&a);
      printf("EGL_CONFIG_SAMPLE name=%s id=%d renderable=0x%x rgba=%d,%d,%d,%d\n",name,id,mask,red,green,blue,a);
    }
  }
}

int main(void) {
  struct wl_display *wayland=wl_display_connect(NULL);
  if (!wayland) { puts("NOT RUN: cannot connect to Wayland"); return 77; }
  EGLDisplay display=EGL_NO_DISPLAY;
  PFNEGLGETPLATFORMDISPLAYPROC get_platform=(void *)eglGetProcAddress("eglGetPlatformDisplay");
  PFNEGLGETPLATFORMDISPLAYEXTPROC get_platform_ext=(void *)eglGetProcAddress("eglGetPlatformDisplayEXT");
  if (get_platform) display=get_platform(EGL_PLATFORM_WAYLAND_KHR,wayland,NULL);
  if (display==EGL_NO_DISPLAY && get_platform_ext) display=get_platform_ext(EGL_PLATFORM_WAYLAND_EXT,wayland,NULL);
  if (display==EGL_NO_DISPLAY) display=eglGetDisplay((EGLNativeDisplayType)wayland);
  EGLint major=0, minor=0;
  if (display==EGL_NO_DISPLAY || !eglInitialize(display,&major,&minor)) {
    printf("NOT RUN: private Wayland EGL initialization failed error=0x%x\n",eglGetError());
    wl_display_disconnect(wayland); return 77;
  }
  printf("EGL_PRIVATE_DISPLAY version=%d.%d vendor=%s client_apis=%s\n",major,minor,
         eglQueryString(display,EGL_VENDOR),eglQueryString(display,EGL_CLIENT_APIS));
  printf("EGL_HDR_APPLICABLE no_config=%d float_format=%d\n",
         epoxy_has_egl_extension(display,"EGL_KHR_no_config_context"),epoxy_has_egl_extension(display,"EGL_EXT_pixel_format_float"));
  query(display,"gtk_sdr_implicit",0,8,-1,0);
  query(display,"gtk_sdr_size_zero",0,8,0,0);
  query(display,"explicit_es1",EGL_OPENGL_ES_BIT,8,0,0);
  query(display,"explicit_es2",EGL_OPENGL_ES2_BIT,8,0,0);
  query(display,"explicit_gl",EGL_OPENGL_BIT,8,0,0);
  query(display,"any_api",EGL_DONT_CARE,8,0,0);
  query(display,"any_api_no_alpha",EGL_DONT_CARE,0,0,0);
  query(display,"gtk_hdr_implicit",0,8,-1,1);
  query(display,"hdr_any_api",EGL_DONT_CARE,8,0,1);
  /* This process owns this distinct connection/display; never GTK's handle. */
  eglTerminate(display);
  wl_display_disconnect(wayland);
  return 0;
}
