/* Query native binding before asking GL for its version. No CPU pixel path. */
#pragma once
#ifndef GST_USE_UNSTABLE_API
#define GST_USE_UNSTABLE_API
#endif
#include <gst/gl/gl.h>
static inline GstGLAPI pivra_gpu_current_api(GstGLPlatform platform, guintptr *handle) {
  *handle = platform == GST_GL_PLATFORM_NONE ? 0 : gst_gl_context_get_current_gl_context(platform);
  if (!*handle) return GST_GL_API_NONE;
  return gst_gl_context_get_current_gl_api(platform, NULL, NULL);
}
