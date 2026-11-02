#!/usr/bin/env python3
"""Build an isolated, opt-in GTK backend experiment; exercise real H.264.

No installation, product build integration, decoder replacement or system edits.
Original GTK source licenses remain in the downloaded source files. The patch
is an experimental CPU reference, NOT an upstream fix or shipping GTK module.
"""
import argparse
import ast
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import tempfile
import urllib.request

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
FILES = {
    "gtkgstmediafile.c": "329eff51643fd81d431b9c2fe5726655fbb7072cb74b5158fc7ad5195031d44f",
    "gtkgstmediafileprivate.h": "ddc377cffa1bef7ccac7709fdd0d0d0cb97f2eb0fdc3ac2926372d3698cff01f",
    "gtkgstpaintable.c": "c0b3ea2be4f5b4d1234f0bcfdef318451bd8da76bb77c275dcb22120ceb4c242",
    "gtkgstpaintableprivate.h": "2168a078d01264e95cc4a51dd7f6065b952949b006a7a02120393ef693482914",
    "gtkgstsink.c": "1a87621a35e9d11a05beed2f8798180e6e0f4eb1a29d8370972a366294dc55be",
    "gtkgstsinkprivate.h": "74c438f5a05147efce7f5590cda0c5d3358d1bfc3cea0a232c5b692d672129f6",
    "gdk/gdkdmabuffourccprivate.h": "5c5b4a65fd6ee4ef0bbb7918b6b75d7a83acd4667d76f8ecc2599927dfc13190",
}
MODULES = ["gtk4", "gstreamer-player-1.0", "gstreamer-gl-1.0", "gstreamer-allocators-1.0"]


def run(args, **kwargs):
    return subprocess.run(args, check=True, text=True, **kwargs)


def flags(option):
    result = subprocess.run(["pkg-config", option, *MODULES], capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError("NOT RUN native development packages: " + result.stderr)
    return shlex.split(result.stdout)


def cpu_reference_transport(source, variant):
    """Match RGB transport in color controls; capture installed GPU separately."""
    key = 'g_strcmp0 (g_getenv ("PIVRA_GTK_COLOR_CPU_TRANSPORT"), "1")'
    path = source / "gtkgstsink.c"
    text = path.read_text()
    start = text.index("static GstCaps *\ngtk_gst_sink_get_caps")
    end = text.index("static gboolean", start)
    caps = text[start:end]
    if variant == "stock":
        caps = caps.replace("if (self->gdk_display)", "if (self->gdk_display && " + key + " != 0)")
        caps = caps.replace("if (self->gdk_context)", "if (self->gdk_context && " + key + " != 0)")
        caps = caps.replace("gst_caps_from_string (MEMORY_TEXTURE_CAPS)", "gst_caps_from_string (" + key + ' == 0 ? GST_VIDEO_CAPS_MAKE ("RGB") : MEMORY_TEXTURE_CAPS)')
    else:
        flag = 'g_strcmp0 (g_getenv ("PIVRA_GTK_BT1886_CPU"), "1")'
        caps = caps.replace(flag + " != 0", "(" + flag + " != 0 && " + key + " != 0)")
        caps = caps.replace(flag + " == 0", "(" + flag + " == 0 || " + key + " == 0)")
    assert key in caps
    path.write_text(text[:start] + caps + text[end:])
    path = source / "gtkgstpaintable.c"
    text = path.read_text()
    if variant == "stock":
        text = text.replace("if (uses_gl)", "if (uses_gl && " + key + " != 0)")
    else:
        text = text.replace('if (uses_gl && g_strcmp0 (g_getenv ("PIVRA_GTK_BT1886_CPU"), "1") != 0)', 'if (uses_gl && g_strcmp0 (g_getenv ("PIVRA_GTK_BT1886_CPU"), "1") != 0 && ' + key + " != 0)")
    assert key in text
    path.write_text(text)


def isolate_symbols(source, variant):
    """Avoid both GType-name collisions and ELF symbol interposition with GTK."""
    title = variant.title()
    for path in source.glob("gtkgst*.[ch]"):
        text = path.read_text()
        text = text.replace("GTK_TYPE_GST", "PIVRA_TYPE_" + variant.upper())
        text = text.replace("GTK_GST", "PIVRA_" + variant.upper())
        text = text.replace("GtkGst", "Pivra" + title)
        for symbol in ["media_file", "paintable", "sink"]:
            text = text.replace("gtk_gst_" + symbol, "pivra_" + variant + "_" + symbol)
        text = text.replace("gtk_debug_gst_sink", "pivra_" + variant + "_debug_sink")
        for suffix in ["MEDIA_FILE", "PAINTABLE"]:
            text = text.replace("GTK, GST_" + suffix, "PIVRA, " + variant.upper() + "_" + suffix)
        if path.name == "gtkgstmediafile.c":
            text = text.replace('"gstreamer",', '"pivra-color-' + variant + '",')
            needle = "  self->paintable = pivra_" + variant + "_paintable_new ();"
            assert needle in text
            text = text.replace(needle, '  g_printerr ("PIVRA_COLOR_IMPL id=pivra-color-' + variant + ' source=4.18.6\\n");\n' + needle)
        path.write_text(text)


def build(root, cache):
    versions = run(["pkg-config", "--modversion", *MODULES], capture_output=True).stdout
    print("Build versions (GTK/player/GL/allocators):\n" + versions, flush=True)
    source = root / "stock-src" / "modules" / "media"
    source.mkdir(parents=True)
    for name, digest in FILES.items():
        if cache and (cache / name).is_file():
            data = (cache / name).read_bytes()
        else:
            suffix = name if name.startswith("gdk/") else "modules/media/" + name
            with urllib.request.urlopen("https://raw.githubusercontent.com/GNOME/gtk/4.18.6/" + suffix, timeout=30) as reply:
                data = reply.read(128 * 1024 + 1)
        assert hashlib.sha256(data).hexdigest() == digest, f"unverified GTK source: {name}"
        path = source / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    (source / "config.h").write_text('#define GETTEXT_PACKAGE "gtk40"\n')
    shutil.copytree(root / "stock-src", root / "patched-src")
    run(["patch", "--batch", "--fuzz=0", "-p1", "-i", str(HERE / "gtk-4.18.6-bt1886-cpu.patch")], cwd=root / "patched-src")
    cc = shlex.split(os.environ.get("CC", "cc"))
    cflags, libs = flags("--cflags"), flags("--libs")
    for variant, directory in [("stock", "stock-src"), ("candidate", "patched-src")]:
        source = root / directory / "modules" / "media"
        cpu_reference_transport(source, variant)
        isolate_symbols(source, variant)
        module = root / variant / "modules" / ("libpivra-color-" + variant + ".so")
        module.parent.mkdir(parents=True)
        # Upstream callback/sign comparisons are not part of this patch.
        # Keep sign-compare warnings visible; the new policy test is fully strict.
        run([*cc, "-shared", "-fPIC", "-O2", "-Wall", "-Wextra", "-Werror", "-Wno-unused-parameter", "-Wno-error=sign-compare",
             "-DGTK_COMPILATION", "-I" + str(source), *cflags,
             *[str(source / ("gtkgst" + name + ".c")) for name in ("mediafile", "paintable", "sink")],
             "-o", str(module), *libs, "-lm", "-Wl,-z,defs"])
        print("BUILT module=" + str(module) + " sha256=" + hashlib.sha256(module.read_bytes()).hexdigest(), flush=True)
    source = root / "patched-src" / "modules" / "media"
    unit = root / "policy-test"
    run([*cc, "-O2", "-Wall", "-Wextra", "-Werror", "-I" + str(source), *cflags,
         str(HERE / "bt1886_policy.c"), "-o", str(unit), *libs, "-lm"])
    run([str(unit)])


def fixtures(root):
    width, height = 192, 96
    raw = root / "bars.i420"
    raw.write_bytes(bytes([16] * 64 + [128] * 64 + [235] * 64) * height + bytes([128]) * (width * height // 2))
    for name, transfer, full in [("bt709", "bt709", False), ("srgb", "iec61966-2-1", False),
                                 ("unknown", "undef", False), ("full709", "bt709", True)]:
        path = root / (name + ".mp4")
        run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-f", "rawvideo", "-pixel_format", "yuv420p",
             "-video_size", f"{width}x{height}", "-framerate", "8", "-i", str(raw),
             "-vf", "loop=loop=15:size=1:start=0", "-frames:v", "16", "-c:v", "libx264", "-crf", "10",
             "-pix_fmt", "yuv420p", "-x264-params",
             f'colorprim={"undef" if name == "unknown" else "bt709"}:transfer={transfer}:colormatrix=bt709:fullrange={"on" if full else "off"}',
             str(path)])
        stream = json.loads(run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_streams", "-of", "json", str(path)], capture_output=True).stdout)["streams"][0]
        assert stream["codec_name"] == "h264" and stream["pix_fmt"] in (("yuv420p", "yuvj420p") if full else ("yuv420p",)), stream
        assert stream["color_range"] == ("pc" if full else "tv"), stream
        assert stream["color_space"] == "bt709", stream
        expected_transfer = {"bt709": "bt709", "srgb": "iec61966-2-1", "full709": "bt709"}.get(name)
        assert stream.get("color_transfer") == expected_transfer, stream
        print("Verified fixture metadata:", name, {key: stream.get(key) for key in ["color_range", "color_space", "color_transfer", "color_primaries"]}, flush=True)


def probe(root, binary, name, video, variant, enabled):
    env = os.environ.copy()
    for key in ["PIVRA_VIDEO_TEST_MODULE_DIR", "PIVRA_GTK_COLOR_CPU_TRANSPORT", "GTK_MEDIA"]:
        env.pop(key, None)
    if variant:
        env["PIVRA_VIDEO_TEST_MODULE_DIR"] = str(root / variant / "modules")
        env["GTK_MEDIA"] = "pivra-color-" + variant
        env["PIVRA_GTK_COLOR_CPU_TRANSPORT"] = "1"
    env.update(PIVRA_GTK_BT1886_CPU="1" if enabled else "0",
               GST_DEBUG="gtkgstsink:5", GST_DEBUG_NO_COLOR="1")
    result = subprocess.run([str(binary), str(video), "--check-controls"], env=env, capture_output=True, text=True, timeout=30)
    (root / (name + ".log")).write_text(result.stdout + result.stderr)
    print(name + ":\n" + result.stdout, flush=True)
    for line in result.stderr.splitlines():
        if "set caps with" in line:
            print(line, flush=True)  # Accepted sink caps, not advertised alternatives.
    if result.returncode == 77:
        raise RuntimeError("NOT RUN native backend: " + result.stderr)
    assert result.returncode == 0, (name, result.returncode, result.stderr)
    assert "PASS MediaStream" in result.stdout, name
    assert "max channel difference=0" in result.stdout, name
    if variant:
        expected_class = "Pivra" + variant.title() + "MediaFile"
        if ("BACKEND selected=" + expected_class) not in result.stdout or ("PIVRA_COLOR_IMPL id=pivra-color-" + variant) not in result.stderr:
            raise RuntimeError(f"NOT RUN backend provenance unverified: {name}")
        if "path=CPU-memory" not in result.stdout:
            raise RuntimeError(f"NOT RUN CPU candidate inactive: {name}; actual source path is not CPU memory; see {root / (name + '.log')}")
        if re.search(r"path=(?:DMA-BUF|GL|UNKNOWN)", result.stdout):
            raise RuntimeError(f"NOT RUN mixed/non-CPU candidate path: {name}")
    samples = [ast.literal_eval(pixel) for pixel in re.findall(r"sample x=\d+ y=\d+ sRGB=(\[[^\n]+\])", result.stdout)]
    assert len(samples) == 3, name
    return samples


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--source-cache", type=Path)
    parser.add_argument("--video", type=Path, help="also inspect the real installed backend and candidate using this read-only file")
    parser.add_argument("--build-only", action="store_true")
    parser.add_argument("--probe", type=Path, help="use an explicitly prebuilt video_color_probe")
    parser.add_argument("--visual", action="store_true", help="after all gates pass, open the real production window with the verified test module")
    args = parser.parse_args()
    if args.visual and (not args.video or args.build_only):
        parser.error("--visual requires --video and cannot be combined with --build-only")
    for tool in ["pkg-config", "patch", *([] if args.build_only or (args.probe and not args.visual) else ["cargo"]), *([] if args.build_only else ["ffmpeg", "ffprobe"])]:
        if not shutil.which(tool):
            raise RuntimeError("NOT RUN missing " + tool)
    root = args.output.resolve() if args.output else Path(tempfile.mkdtemp(prefix="pivra-bt1886-"))
    root.mkdir(mode=0o700, parents=True, exist_ok=True)
    assert not any(root.iterdir()), "output must be an empty private workspace; nothing was removed"
    print("Retained isolated workspace:", root, flush=True)
    build(root, args.source_cache)
    if args.build_only:
        print("PASS build/policy only; native decode and visual acceptance NOT RUN")
        return
    if args.probe:
        binary = args.probe.resolve(strict=True)
    else:
        run(["cargo", "build", "--locked", "--example", "video_color_probe"], cwd=REPO)
        binary = Path(os.environ.get("CARGO_TARGET_DIR", REPO / "target")) / "debug/examples/video_color_probe"
        if not binary.is_absolute():
            binary = REPO / binary
    if args.visual:
        run(["cargo", "build", "--locked", "--example", "video_color_window"], cwd=REPO)
    video = args.video.resolve(strict=True) if args.video else None
    if video:
        probe(root, binary, "user-installed-backend", video, None, False)
    fixtures(root)
    for fixture in ["bt709", "srgb", "unknown", "full709"]:
        clip = root / (fixture + ".mp4")
        stock = probe(root, binary, fixture + "-stock", clip, "stock", False)
        off = probe(root, binary, fixture + "-off", clip, "candidate", False)
        on = probe(root, binary, fixture + "-on", clip, "candidate", True)
        assert stock == off, ("opt-out changed pixels", fixture, stock, off)
        if fixture in ("srgb", "unknown"):
            assert stock == on, ("non-BT709 metadata changed", fixture, stock, on)
        else:
            expected = (0, 121, 253) if fixture == "bt709" else (4, 121, 234)
            for pixel, value in zip(on, expected):
                assert pixel[3] == 255 and all(abs(channel - value) <= 2 for channel in pixel[:3]), (fixture, on, expected)
            assert stock[1][0] > on[1][0] + 10, ("transfer effect not demonstrated", stock, on)
        print("PASS real H.264 backend regression:", fixture, flush=True)
    if video:
        probe(root, binary, "user-candidate-backend", video, "candidate", True)
    print("PASS automated CPU reference; Arch/MPV visual and GPU acceptance remain OPEN")
    print("Verified candidate module directory:", root / "candidate/modules")
    print("For a production-window comparison using test-only explicit loading:")
    print("GTK_MEDIA=pivra-color-candidate PIVRA_GTK_BT1886_CPU=1 PIVRA_VIDEO_TEST_MODULE_DIR=" + shlex.quote(str(root / "candidate/modules")) +
          " cargo run --locked --example video_color_window -- " + shlex.quote(str(video) if video else "/absolute/path/to/video.mp4"))
    if args.visual:
        env = os.environ.copy()
        env.update(GTK_MEDIA="pivra-color-candidate", PIVRA_GTK_BT1886_CPU="1",
                   PIVRA_VIDEO_TEST_MODULE_DIR=str(root / "candidate/modules"))
        target = Path(os.environ.get("CARGO_TARGET_DIR", REPO / "target"))
        if not target.is_absolute():
            target = REPO / target
        print("Opening production window through the verified test module; compare with MPV and close normally. Visual acceptance is manual.", flush=True)
        run([str(target / "debug/examples/video_color_window"), str(video)], env=env)
        print("Window closed; this is not a visual-color PASS. Report the comparison result explicitly.")



if __name__ == "__main__":
    try:
        main()
    except RuntimeError as error:
        print(error)
        raise SystemExit(77 if str(error).startswith("NOT RUN") else 1) from error
    except (AssertionError, OSError, subprocess.SubprocessError) as error:
        print("FAIL:", error)
        raise SystemExit(1) from error
