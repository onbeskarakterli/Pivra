/* Real GTK context/window gate. Never queries GL without a current context. */
#include <gtk/gtk.h>
#include <epoxy/gl.h>
#include <stdio.h>
#include <string.h>

int main(void) {
  gtk_init();
  g_print("GTK_RUNTIME %u.%u.%u\n",gtk_get_major_version(),gtk_get_minor_version(),gtk_get_micro_version());
  FILE *maps=fopen("/proc/self/maps","r");
  if (maps) {
    char *line=NULL; size_t n=0;
    while (getline(&line,&n,maps)>=0) {
      char *path=strchr(line,'/');
      if (path && strstr(path,"/libgtk-4.so")) {
        path[strcspn(path,"\n")]=0; g_print("GTK_LOADED %s\n",path); break;
      }
    }
    free(line); fclose(maps);
  }
  GError *error=NULL;
  GdkDisplay *display=gdk_display_get_default();
  if (!gdk_display_prepare_gl(display,&error)) {
    g_printerr("FAIL GTK display GL configuration: %s\n",error->message);
    g_error_free(error); return 1;
  }
  GtkWidget *window=gtk_window_new();
  gtk_window_present(GTK_WINDOW(window));
  GdkSurface *surface=gtk_native_get_surface(GTK_NATIVE(window));
  GdkGLContext *context=gdk_surface_create_gl_context(surface,&error);
  if (!context || !gdk_gl_context_realize(context,&error)) {
    g_printerr("FAIL GTK surface GL context: %s\n",error ? error->message:"unavailable");
    g_clear_error(&error);g_clear_object(&context);gtk_window_destroy(GTK_WINDOW(window));return 1;
  }
  gdk_gl_context_make_current(context);
  gboolean current=gdk_gl_context_get_current()==context;
  if (current) {
    g_print("GL_RUNTIME renderer=%s vendor=%s version=%s\n",glGetString(GL_RENDERER),glGetString(GL_VENDOR),glGetString(GL_VERSION));
  }
  GskRenderer *renderer=gtk_native_get_renderer(GTK_NATIVE(window));
  gboolean gpu=renderer && g_str_equal(G_OBJECT_TYPE_NAME(renderer),"GskGLRenderer");
  g_print("GTK_RENDERER %s\n",renderer ? G_OBJECT_TYPE_NAME(renderer):"none");
  gdk_gl_context_clear_current();g_object_unref(context);gtk_window_destroy(GTK_WINDOW(window));
  if (!current || !gpu) { g_printerr("FAIL GTK current context / GL renderer\n");return 1; }
  g_print("PASS real GTK EGL selection, window and current GL context\n");
  return 0;
}
