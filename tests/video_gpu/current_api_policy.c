/* Call-order unit control only: not a GPU renderer or decoder test. */
#define GST_USE_UNSTABLE_API
#include <gst/gl/gl.h>
static guintptr native_handle;
static int handle_calls, api_calls;
static guintptr test_handle(GstGLPlatform platform) { (void)platform; handle_calls++; return native_handle; }
static GstGLAPI test_api(GstGLPlatform platform, guint *major, guint *minor) {
  (void)platform; (void)major; (void)minor;
  g_assert_cmpuint(native_handle, !=, 0);
  api_calls++; return GST_GL_API_GLES2;
}
#define gst_gl_context_get_current_gl_context test_handle
#define gst_gl_context_get_current_gl_api test_api
#include "current_context.h"
int main(void) {
  guintptr output = 123;
  g_assert_cmpint(pivra_gpu_current_api(GST_GL_PLATFORM_EGL, &output), ==, GST_GL_API_NONE);
  g_assert_cmpuint(output, ==, 0); g_assert_cmpint(api_calls, ==, 0);
  native_handle = 456;
  g_assert_cmpint(pivra_gpu_current_api(GST_GL_PLATFORM_EGL, &output), ==, GST_GL_API_GLES2);
  g_assert_cmpuint(output, ==, 456); g_assert_cmpint(api_calls, ==, 1);
  int calls = handle_calls;
  g_assert_cmpint(pivra_gpu_current_api(GST_GL_PLATFORM_NONE, &output), ==, GST_GL_API_NONE);
  g_assert_cmpuint(output, ==, 0); g_assert_cmpint(api_calls, ==, 1);
  g_assert_cmpint(handle_calls, ==, calls);
  g_print("PASS current API policy: no GL version query without native binding\n");
  return 0;
}
