#!/usr/bin/env python3
"""Read-only PIV-05 inventory; initializes Gst registry, never a media pipeline.

Factory availability is not a decoding/playback or hardware capability test.
The registry is isolated in a private temporary directory, then removed.
"""

import ctypes
import ctypes.util
import json
import platform
import subprocess
import tempfile
import os


def command(args):
    result = subprocess.run(args, capture_output=True, text=True, timeout=20)
    return {"exit": result.returncode, "stdout": result.stdout.strip(),
            "stderr": result.stderr.strip()}


def gst_inventory():
    library = ctypes.util.find_library("gstreamer-1.0")
    if not library:
        return {"status": "NOT RUN", "reason": "GStreamer runtime not found"}
    gst = ctypes.CDLL(library)
    pointer = ctypes.c_void_p
    gst.gst_init_check.argtypes = [pointer, pointer, ctypes.POINTER(pointer)]
    gst.gst_init_check.restype = ctypes.c_int
    gst.gst_version.argtypes = [ctypes.POINTER(ctypes.c_uint)] * 4
    gst.gst_element_factory_find.argtypes = [ctypes.c_char_p]
    gst.gst_element_factory_find.restype = pointer
    gst.gst_plugin_feature_get_plugin.argtypes = [pointer]
    gst.gst_plugin_feature_get_plugin.restype = pointer
    gst.gst_plugin_get_version.argtypes = [pointer]
    gst.gst_plugin_get_version.restype = ctypes.c_char_p
    gst.gst_plugin_get_filename.argtypes = [pointer]
    gst.gst_plugin_get_filename.restype = ctypes.c_char_p
    gst.gst_object_unref.argtypes = [pointer]
    error = pointer()
    if not gst.gst_init_check(None, None, ctypes.byref(error)):
        raise RuntimeError("gst_init_check failed (no decoding attempted)")
    versions = [ctypes.c_uint() for _ in range(4)]
    gst.gst_version(*(ctypes.byref(value) for value in versions))
    factories = {}
    names = ["playbin3", "decodebin3", "uridecodebin3", "appsink", "videoconvert",
             "gtk4paintablesink", "waylandsink", "glimagesink", "ximagesink",
             "autoaudiosink", "pulsesink", "pipewiresink", "volume", "scaletempo",
             "subparse", "textoverlay", "assrender", "avdec_h264", "openh264dec",
             "avdec_h265", "libde265dec", "vp9dec", "av1dec", "dav1ddec",
             "vah264dec", "v4l2h264dec", "nvh264dec", "jpegdec", "pngdec", "webpdec"]
    for name in names:
        factory = gst.gst_element_factory_find(name.encode())
        if not factory:
            factories[name] = {"present": False}
            continue
        plugin = gst.gst_plugin_feature_get_plugin(factory)
        try:
            version = gst.gst_plugin_get_version(plugin)
            filename = gst.gst_plugin_get_filename(plugin)
            factories[name] = {"present": True,
                               "plugin_version": version.decode() if version else None,
                               "plugin_file": filename.decode() if filename else None}
        finally:
            if plugin:
                gst.gst_object_unref(plugin)
            gst.gst_object_unref(factory)
    return {"status": "REGISTRY INSPECTED ONLY", "version": [v.value for v in versions],
            "factories": factories, "pipelines_created": 0, "media_decoded": False}


def main():
    pc = command(["pkg-config", "--list-all"])
    assert pc["exit"] == 0, pc
    modules = sorted(line.split()[0] for line in pc["stdout"].splitlines() if line.strip())
    selected = ["gtk4", "gtk4-wayland", "gtk4-x11", "libadwaita-1", "glib-2.0",
                "gstreamer-1.0", "gstreamer-video-1.0", "glycin-1", "glycin-2",
                "libwebp", "libtiff-4", "lcms2", "libheif", "libjxl", "vips",
                "fontconfig", "libseccomp", "gdk-pixbuf-2.0"]
    results = {"scope": "PIV-05 inventory, not codec or rendering certification",
               "architecture": platform.machine(),
               "rustc": command(["rustc", "+stable", "--version"]),
               "cargo": command(["cargo", "+stable", "--version"]),
               "pkg_config_modules": modules,
               "pkg_config_versions": {name: command(["pkg-config", "--modversion", name])
                                       for name in selected}}
    with tempfile.TemporaryDirectory(prefix="pivra-gst-registry-") as root:
        os.environ["GST_REGISTRY_1_0"] = os.path.join(root, "registry.bin")
        results["gstreamer"] = gst_inventory()
    print(json.dumps(results, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
