#!/usr/bin/env python3
"""Build application-owned public-API media adapter, not GTK itself."""
import argparse
import hashlib
import importlib.util
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import urllib.request

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location('reference', ROOT / 'tests/video_color/backend_probe.py')
ref = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ref)


def build(root, cache=None, prepare_only=False):
    root.mkdir(parents=True, exist_ok=False)
    source = root / 'source'
    source.mkdir()
    for name, digest in ref.FILES.items():
        if cache and (cache / name).is_file():
            data = (cache / name).read_bytes()
        else:
            suffix = name if name.startswith('gdk/') else 'modules/media/' + name
            with urllib.request.urlopen('https://raw.githubusercontent.com/GNOME/gtk/4.18.6/' + suffix, timeout=30) as reply:
                data = reply.read(128 * 1024 + 1)
        if hashlib.sha256(data).hexdigest() != digest:
            raise RuntimeError('upstream source hash: ' + name)
        path = source / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    (source / 'config.h').write_text('#define GETTEXT_PACKAGE "pivra-gpu-prototype"\n')
    # Public MediaFile subclass, distinct ELF symbols/GTypes. No CPU reference patch.
    ref.isolate_symbols(source, 'gpu')
    path = source / 'gtkgstmediafile.c'
    text = path.read_text().replace('                                  10);', '                                  -1000);')
    text = text.replace('clock_time == -1', 'clock_time == GST_CLOCK_TIME_NONE')
    path.write_text(text)
    path = source / 'gtkgstpaintable.c'
    text = path.read_text().replace('#include "gtkgstsinkprivate.h"', '#include "gtkgstsinkprivate.h"\n#include "filter.c"')
    start = text.index('  if (uses_gl)\n')
    end = text.index('  g_clear_object (&context);', start)
    text = text[:start] + '''  if (!uses_gl) {
      g_printerr ("PIVRA_GPU_FAIL no shared GL context\\n");
      g_object_unref(sink); g_clear_object(&context); return NULL;
    }
  {
    GstElement *bin = gst_bin_new(NULL);
    GstElement *upload = gst_element_factory_make("glupload", NULL);
    GstElement *convert = gst_element_factory_make("glcolorconvert", NULL);
    GstElement *normalize = gst_element_factory_make("capsfilter", NULL);
    GstCaps *normalized = gst_caps_from_string("video/x-raw(memory:GLMemory),format=RGBA,texture-target=2D,colorimetry=1:1:5:1");
    g_object_set(normalize, "caps", normalized, NULL); gst_caps_unref(normalized);
    GstElement *transfer = g_object_new(pivra_transfer_get_type(), NULL);
    GstPad *pad;
    if (!upload || !convert) {
      g_printerr ("PIVRA_GPU_FAIL missing glupload/glcolorconvert\\n");
      if(upload) gst_object_unref(upload);
      if(convert) gst_object_unref(convert);
      gst_object_unref(normalize); gst_object_unref(transfer); gst_object_unref(sink); gst_object_unref(bin);
      g_clear_object(&context); return NULL;
    }
    gst_bin_add_many(GST_BIN(bin), upload, convert, normalize, transfer, sink, NULL);
    if (!gst_element_link_many(upload, convert, normalize, transfer, sink, NULL)) {
      gst_object_unref(bin); g_clear_object(&context); return NULL;
    }
    pad = gst_element_get_static_pad(upload, "sink");
    gst_pad_add_probe(pad, GST_PAD_PROBE_TYPE_EVENT_DOWNSTREAM, source_metadata, bin, NULL);
    gst_element_add_pad(bin, gst_ghost_pad_new("sink", pad));
    gst_object_unref(pad);
    sink = bin;
  }
''' + text[end:]
    text = text.replace('  GdkSurface *surface;', '  GdkSurface *surface;\n  gint pending;')
    text = text.replace('  g_object_unref (invoke->paintable);', '  g_atomic_int_add (&invoke->paintable->pending, -1);\n  g_object_unref (invoke->paintable);')
    text = text.replace('  invoke = g_new0 (SetTextureInvocation, 1);', '  if (g_atomic_int_add (&self->pending, 1) >= 2) { g_atomic_int_add (&self->pending, -1); return; }\n  invoke = g_new0 (SetTextureInvocation, 1);')
    path.write_text(text)
    path = source / 'gtkgstsink.c'
    text = path.read_text()
    start = text.index('  unfiltered = gst_caps_new_empty ();', text.index('pivra_gpu_sink_get_caps'))
    end = text.index('  GST_DEBUG_OBJECT', start)
    text = text[:start] + '''  unfiltered = self->gdk_context
    ? gst_caps_from_string("video/x-raw(memory:GLMemory),format=RGBA,texture-target=2D,width=[1,2147483647],height=[1,2147483647],framerate=[0/1,2147483647/1]")
    : gst_caps_new_empty();
''' + text[end:]
    text = text.replace('GstCaps *unfiltered, *tmp, *result;', 'GstCaps *unfiltered, *result;')
    # Keep only the GLMemory map/presentation branch. A failed GL map must
    # never fall through into a CPU pixel map or MemoryTexture fallback.
    a = text.index('  if (gst_is_dmabuf_memory (mem))')
    b = text.index('  else if (gst_is_gl_memory (mem)', a)
    text = text[:a] + text[b:].replace('  else if (gst_is_gl_memory (mem)', '  if (gst_is_gl_memory (mem)', 1)
    a = text.index('  else if (gst_video_frame_map (frame, &self->v_info, buffer, GST_MAP_READ))')
    b = text.index('  else\n', a)
    text = text[:a] + text[b:]
    assert 'gdk_memory_texture_builder_build' not in text
    text = text.replace('      if (sync_meta)\n        gst_gl_sync_meta_set_sync_point',
      '      if (!sync_meta) { video_frame_free(frame); return NULL; }\n      { static gint samples; if(g_atomic_int_add(&samples, 1)<16) g_printerr("PIVRA_GPU_PRESENT GLMemory sync-meta retained\\n"); }\n      if (sync_meta)\n        gst_gl_sync_meta_set_sync_point')
    # Filter output retains negotiated BT709 caps for local pipeline bookkeeping;
    # samples are already sRGB encoded. Only this sink consumes the private stage.
    text = text.replace('self->color_state = gtk_gst_color_state_from_colorimetry (self, &self->v_info.colorimetry);',
                        'self->color_state = gdk_color_state_ref(gdk_color_state_get_srgb());')
    text = text.replace('return GST_FLOW_OK;', 'return texture ? GST_FLOW_OK : GST_FLOW_ERROR;')
    text = text.replace('size, 0, 0);', 'size, 2, 8);').replace('pool, size, 2, 0);', 'pool, size, 2, 8);')
    a = text.index('static GdkColorState *\n')
    b = text.index('static GstCaps *\n', a)
    text = text[:a] + text[b:]
    a = text.index('static void\nadd_drm_formats_and_modifiers')
    b = text.index('static GstCaps *\n', a)
    text = text[:a] + text[b:]
    text = text.replace('#include <gdk/gdkdmabuffourccprivate.h>', '')
    text = text.replace('i < vmeta->n_planes', 'i < (int) vmeta->n_planes')
    path.write_text(text)
    shutil.copy(ROOT / 'native/video/filter.c', source / 'filter.c')
    if prepare_only:
        print('PREPARED pinned adapter sources; compilation NOT RUN')
        return source
    module = root / 'modules/libpivra-color-gpu.so'
    module.parent.mkdir()
    subprocess.run([*shlex.split(os.environ.get('CC', 'cc')), '-shared', '-fPIC', '-O2', '-Wall', '-Wextra', '-Werror',
                    '-Wno-unused-parameter', *shlex.split(os.environ.get('CPPFLAGS', '')), '-DGTK_COMPILATION', '-I' + str(source), *ref.flags('--cflags'),
                    *[str(source / ('gtkgst' + n + '.c')) for n in ('mediafile', 'paintable', 'sink')],
                    '-o', str(module), *shlex.split(os.environ.get('LDFLAGS', '')), *ref.flags('--libs'), '-lm', '-Wl,-z,defs'], check=True)
    unit = root / 'policy-test'
    subprocess.run([*shlex.split(os.environ.get('CC', 'cc')), '-O2', '-Wall', '-Wextra', '-Werror',
        *shlex.split(os.environ.get('CPPFLAGS', '')), '-I' + str(Path(__file__).parent),
        *ref.flags('--cflags'), str(Path(__file__).with_name('policy.c')), '-o', str(unit),
        *shlex.split(os.environ.get('LDFLAGS', '')), *ref.flags('--libs'), '-lm'], check=True)
    subprocess.run([str(unit)], check=True)
    return module


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--source-cache', type=Path)
    p.add_argument('--prepare-only', action='store_true')
    a = p.parse_args()
    print(build(a.output, a.source_cache, a.prepare_only))
