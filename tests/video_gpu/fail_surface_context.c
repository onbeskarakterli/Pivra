/* Test-only interposition: reject Pivra's executable-owned context request,
 * never GTK's renderer or stock sink. No product hook/environment switch. */
#define _GNU_SOURCE
#include <gtk/gtk.h>
#include <dlfcn.h>
#include <unistd.h>
#include <sys/stat.h>

GdkGLContext *gdk_surface_create_gl_context(GdkSurface *surface, GError **error) {
  Dl_info caller={0};
  struct stat executable, origin;
  if (dladdr(__builtin_return_address(0),&caller) && caller.dli_fname &&
      stat("/proc/self/exe",&executable)==0 && stat(caller.dli_fname,&origin)==0 &&
      executable.st_dev==origin.st_dev && executable.st_ino==origin.st_ino) {
    g_printerr("CONTROLLED_PIVRA_CONTEXT_FAILURE stock-and-renderer-unchanged\n");
    g_set_error_literal(error,GDK_GL_ERROR,GDK_GL_ERROR_NOT_AVAILABLE,"controlled Pivra surface-context failure");
    return NULL;
  }
  GdkGLContext *(*real_create)(GdkSurface *,GError **)=dlsym(RTLD_NEXT,"gdk_surface_create_gl_context");
  if (!real_create) {
    g_set_error_literal(error,GDK_GL_ERROR,GDK_GL_ERROR_NOT_AVAILABLE,"test interposer missing real GTK function");
    return NULL;
  }
  return real_create(surface,error);
}
