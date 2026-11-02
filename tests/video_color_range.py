#!/usr/bin/env python3
"""Controlled GStreamer CPU range regression; not a GTK/H.264 or visual test.

Uses the installed C runtime via ctypes, without adding product dependencies.
Missing runtime/plugins: NOT RUN (77). Pipeline errors/deadlines: FAIL (1).
"""
import ctypes as C
import tempfile
from pathlib import Path


class GError(C.Structure):
    _fields_ = [("domain", C.c_uint), ("code", C.c_int), ("message", C.c_char_p)]


def main():
    try:
        gst = C.CDLL("libgstreamer-1.0.so.0")
        glib = C.CDLL("libglib-2.0.so.0")
    except OSError as error:
        print(f"NOT RUN: {error}")
        return 77
    gst.gst_init.argtypes = [C.c_void_p, C.c_void_p]
    gst.gst_element_factory_find.argtypes = [C.c_char_p]
    gst.gst_element_factory_find.restype = C.c_void_p
    gst.gst_parse_launch.argtypes = [C.c_char_p, C.POINTER(C.POINTER(GError))]
    gst.gst_parse_launch.restype = C.c_void_p
    gst.gst_element_set_state.argtypes = [C.c_void_p, C.c_int]
    gst.gst_element_get_bus.argtypes = [C.c_void_p]
    gst.gst_element_get_bus.restype = C.c_void_p
    gst.gst_bus_timed_pop_filtered.argtypes = [C.c_void_p, C.c_uint64, C.c_int]
    gst.gst_bus_timed_pop_filtered.restype = C.c_void_p
    gst.gst_message_get_structure.argtypes = [C.c_void_p]
    gst.gst_message_get_structure.restype = C.c_void_p
    gst.gst_structure_has_name.argtypes = [C.c_void_p, C.c_char_p]
    gst.gst_structure_has_name.restype = C.c_int
    gst.gst_message_parse_error.argtypes = [C.c_void_p, C.POINTER(C.POINTER(GError)), C.POINTER(C.c_char_p)]
    gst.gst_message_unref.argtypes = [C.c_void_p]
    gst.gst_object_unref.argtypes = [C.c_void_p]
    glib.g_error_free.argtypes = [C.POINTER(GError)]
    glib.g_free.argtypes = [C.c_void_p]
    gst.gst_init(None, None)
    for name in (b"filesrc", b"rawvideoparse", b"videoconvert", b"filesink"):
        factory = gst.gst_element_factory_find(name)
        if not factory:
            print(f"NOT RUN: missing {name.decode()}")
            return 77
        gst.gst_object_unref(factory)
    with tempfile.TemporaryDirectory(prefix="pivra-color-range-") as temporary:
        root = Path(temporary)
        # Three neutral I420 patches, no decoder, display or source photographs.
        width, height = 96, 16
        (root / "input").write_bytes(bytes([16] * 32 + [128] * 32 + [235] * 32) * height + bytes([128]) * (width * height // 2))
        for colorimetry, expected in (("bt709", (0, 128, 255)), ("1:3:5:1", (16, 128, 235))):
            pipeline = (f'filesrc location="{root}/input" ! rawvideoparse format=i420 '
                        f'width={width} height={height} framerate=1/1 colorimetry={colorimetry} ! '
                        f'videoconvert ! video/x-raw,format=RGB ! filesink location="{root}/output"')
            error = C.POINTER(GError)()
            element = gst.gst_parse_launch(pipeline.encode(), C.byref(error))
            if error:
                message = error.contents.message.decode(errors="replace")
                glib.g_error_free(error)
                if element:
                    gst.gst_object_unref(element)
                raise AssertionError(f"pipeline parse: {message}")
            assert element, "no pipeline"
            bus = gst.gst_element_get_bus(element)
            message = None
            try:
                assert gst.gst_element_set_state(element, 4) != 0, "PLAYING failed"
                message = gst.gst_bus_timed_pop_filtered(bus, 5_000_000_000, 3)
                assert message, "pipeline deadline (5s)"
                structure = gst.gst_message_get_structure(message)
                if structure and gst.gst_structure_has_name(structure, b"GstMessageError"):
                    error = C.POINTER(GError)()
                    debug = C.c_char_p()
                    gst.gst_message_parse_error(message, C.byref(error), C.byref(debug))
                    detail = error.contents.message.decode(errors="replace")
                    glib.g_error_free(error)
                    glib.g_free(debug)
                    raise AssertionError(f"pipeline error: {detail}")
            finally:
                gst.gst_element_set_state(element, 1)
                if message:
                    gst.gst_message_unref(message)
                gst.gst_object_unref(bus)
                gst.gst_object_unref(element)
            output = (root / "output").read_bytes()
            assert len(output) == width * height * 3, f"{colorimetry}: incomplete RGB frame ({len(output)} bytes); rerun GST_DEBUG=2 for pipeline error"
            samples = [output[x * 3:x * 3 + 3] for x in (16, 48, 80)]
            for pixel, value in zip(samples, expected):
                assert all(abs(channel - value) <= 2 for channel in pixel), (colorimetry, samples, expected)
            print(f"PASS CPU {colorimetry} I420 -> RGB: {[list(p) for p in samples]}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (AssertionError, OSError) as error:
        print(f"FAIL: {error}")
        raise SystemExit(1) from error
