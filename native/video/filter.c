/* Pivra experimental GL-only BT.1886 sink stage. No pixel download/map. */
#include <string.h>
#include <gst/gl/gl.h>
#include <gst/gl/gstglfuncs.h>
#include <gst/video/video.h>

typedef struct { GstGLFilter parent; GstGLShader *shader; guint frames; } PivraTransfer;
typedef GstGLFilterClass PivraTransferClass;
G_DEFINE_TYPE (PivraTransfer, pivra_transfer, GST_TYPE_GL_FILTER)
static gint live_filters;
int pivra_gpu_live_filters(void) { return g_atomic_int_get(&live_filters); }

/* Input has already undergone YUV range/matrix conversion on the GPU. */
static gboolean eligible (const GstVideoInfo *info) {
  const GstVideoColorimetry *c = &info->colorimetry;
  return GST_VIDEO_INFO_FORMAT(info) == GST_VIDEO_FORMAT_RGBA &&
    c->primaries == GST_VIDEO_COLOR_PRIMARIES_BT709 &&
    c->transfer == GST_VIDEO_TRANSFER_BT709 &&
    c->matrix == GST_VIDEO_COLOR_MATRIX_RGB && c->range == GST_VIDEO_COLOR_RANGE_0_255;
}
static gboolean caps (GstGLFilter *filter, GstCaps *in, GstCaps *out) {
  GstVideoInfo info;
  (void)filter; (void)out;
  if (!gst_video_info_from_caps (&info, in) || !eligible (&info) || info.size > 32 * 1024 * 1024) {
    gchar *text = gst_caps_to_string(in);
    g_printerr ("PIVRA_GPU_REJECT %s\n", text); g_free(text);
    return FALSE;
  }
  gchar *text = gst_caps_to_string(in);
  g_printerr ("PIVRA_GPU_CAPS %s\n", text); g_free(text);
  return TRUE;
}
static GstPadProbeReturn source_metadata (GstPad *pad, GstPadProbeInfo *probe, gpointer data) {
  GstEvent *event = GST_PAD_PROBE_INFO_EVENT(probe);
  (void)pad;
  if (GST_EVENT_TYPE(event) == GST_EVENT_CAPS) {
    GstCaps *caps; GstVideoColorimetry color;
    gst_event_parse_caps(event, &caps);
    const char *tag = gst_structure_get_string(gst_caps_get_structure(caps, 0), "colorimetry");
    if (!tag || !gst_video_colorimetry_from_string(&color, tag) ||
        color.primaries != GST_VIDEO_COLOR_PRIMARIES_BT709 ||
        color.transfer != GST_VIDEO_TRANSFER_BT709 ||
        !((color.matrix == GST_VIDEO_COLOR_MATRIX_BT709 &&
           (color.range == GST_VIDEO_COLOR_RANGE_16_235 || color.range == GST_VIDEO_COLOR_RANGE_0_255)) ||
          (color.matrix == GST_VIDEO_COLOR_MATRIX_RGB && color.range == GST_VIDEO_COLOR_RANGE_0_255))) {
      GST_ELEMENT_ERROR(GST_ELEMENT(data), STREAM, FORMAT,
        ("GPU prototype requires explicit BT709 SDR source metadata"), (NULL));
      return GST_PAD_PROBE_DROP;
    }
    gchar *text = gst_caps_to_string(caps);
    g_printerr("PIVRA_GPU_SOURCE %s\n", text); g_free(text);
  }
  return GST_PAD_PROBE_OK;
}
static const char fragment[] =
  "#ifdef GL_ES\nprecision highp float;\n#endif\n"
  "varying vec2 v_texcoord; uniform sampler2D tex;\n"
  "void main(){vec4 p=texture2D(tex,v_texcoord);"
  "vec3 l=pow(max(p.rgb,vec3(0.0)),vec3(2.4));"
  "vec3 s=mix(12.92*l,1.055*pow(l,vec3(1.0/2.4))-0.055,step(vec3(0.0031308),l));"
  "gl_FragColor=vec4(s,p.a);}";
static gboolean start (GstGLBaseFilter *base) {
  PivraTransfer *self = (PivraTransfer *)base;
  GError *error = NULL;
  GstGLContext *context = base->context;
  if (!context->gl_vtable->FenceSync || !context->gl_vtable->WaitSync ||
      !context->gl_vtable->DeleteSync || !context->gl_vtable->Flush) {
    GST_ELEMENT_ERROR(base, RESOURCE, FAILED, ("GPU synchronization is unavailable"), (NULL));
    return FALSE;
  }
  const char *renderer = (const char *)context->gl_vtable->GetString(GL_RENDERER);
  gchar *lower = renderer ? g_ascii_strdown(renderer, -1) : NULL;
  gboolean software = !lower || strstr(lower,"llvmpipe") || strstr(lower,"softpipe") ||
    strstr(lower,"swrast") || strstr(lower,"swiftshader") || strstr(lower,"software");
  g_free(lower);
  if (software && g_strcmp0(g_getenv("PIVRA_GPU_ALLOW_SOFTWARE_TEST"),"1") != 0) {
    GST_ELEMENT_ERROR(base, RESOURCE, FAILED,
      ("GPU correction requires hardware GL; software driver rejected"),
      ("Stock playback override: PIVRA_VIDEO_BACKEND=stock"));
    return FALSE;
  }
  gboolean core = (gst_gl_context_get_gl_api(context) & GST_GL_API_OPENGL3) != 0;
  const char *vertex_core = "#version 150\nin vec4 a_position; in vec2 a_texcoord; out vec2 v_texcoord;"
    "void main(){gl_Position=a_position;v_texcoord=a_texcoord;}";
  gchar *frag = core ? g_strconcat("#version 150\n#define varying in\n#define texture2D texture\n"
    "out vec4 result;\n#define gl_FragColor result\n", fragment, NULL) : g_strdup(fragment);
  self->shader = gst_gl_shader_new_link_with_stages (context, &error,
    core ? gst_glsl_stage_new_with_string(context, GL_VERTEX_SHADER, GST_GLSL_VERSION_150,
      GST_GLSL_PROFILE_CORE, vertex_core) : gst_glsl_stage_new_default_vertex(context),
    gst_glsl_stage_new_with_string(context, GL_FRAGMENT_SHADER,
      core ? GST_GLSL_VERSION_150 : GST_GLSL_VERSION_NONE,
      core ? GST_GLSL_PROFILE_CORE : GST_GLSL_PROFILE_ES | GST_GLSL_PROFILE_COMPATIBILITY, frag), NULL);
  g_free(frag);
  if (!self->shader) {
    g_printerr ("PIVRA_GPU_SHADER_FAIL %s\n", error ? error->message : "unknown");
    g_clear_error (&error); return FALSE;
  }
  g_atomic_int_inc(&live_filters);
  g_printerr ("PIVRA_GPU_CONTEXT renderer=%s version=%s\n",
    context->gl_vtable->GetString(GL_RENDERER), context->gl_vtable->GetString(GL_VERSION));
  return TRUE;
}
static void stop (GstGLBaseFilter *base) {
  PivraTransfer *self = (PivraTransfer *)base;
  g_printerr ("PIVRA_GPU_STOP frames=%u\n", self->frames);
  if (self->shader) g_atomic_int_add(&live_filters,-1);
  gst_clear_object (&self->shader);
}
static gboolean draw (GstGLFilter *filter, GstGLMemory *in, GstGLMemory *out) {
  PivraTransfer *self = (PivraTransfer *)filter;
  if (!self->shader) return FALSE;
  gst_gl_filter_render_to_target_with_shader (filter, in, out, self->shader);
  if (self->frames++ < 16 && g_getenv("PIVRA_TRACE_VIDEO")) g_printerr ("PIVRA_GPU_DRAW BT1886 GLMemory -> GLMemory\n");
  return TRUE;
}
static void pivra_transfer_init (PivraTransfer *self) { (void)self; }
static void pivra_transfer_class_init (PivraTransferClass *klass) {
  GstGLBaseFilterClass *base = GST_GL_BASE_FILTER_CLASS(klass);
  base->supported_gl_api = GST_GL_API_OPENGL | GST_GL_API_OPENGL3 | GST_GL_API_GLES2;
  base->gl_start = start; base->gl_stop = stop;
  klass->set_caps = caps; klass->filter_texture = draw;
  gst_gl_filter_add_rgba_pad_templates(klass);
  gst_element_class_set_static_metadata(GST_ELEMENT_CLASS(klass),
    "Pivra BT1886 GPU prototype", "Filter/Video", "GPU SDR display transfer", "Pivra contributors");
}
