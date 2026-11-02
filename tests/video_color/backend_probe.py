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
        module = root / variant / "gtk-4.0" / "4.0.0" / "media" / "libmedia-gstreamer.so"
        module.parent.mkdir(parents=True)
        # Upstream callback/sign comparisons are not part of this patch.
        # Keep sign-compare warnings visible; the new policy test is fully strict.
        run([*cc, "-shared", "-fPIC", "-O2", "-Wall", "-Wextra", "-Werror", "-Wno-unused-parameter", "-Wno-error=sign-compare",
             "-DGTK_COMPILATION", "-I" + str(source), *cflags,
             *[str(source / ("gtkgst" + name + ".c")) for name in ("mediafile", "paintable", "sink")],
             "-o", str(module), *libs, "-lm", "-Wl,-z,defs"])
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
    if variant:
        env["GTK_PATH"] = str(root / variant / "gtk-4.0")
    env.update(GTK_MEDIA="gstreamer", PIVRA_GTK_BT1886_CPU="1" if enabled else "0",
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
    args = parser.parse_args()
    for tool in ["pkg-config", "patch", *([] if args.build_only or args.probe else ["cargo"]), *([] if args.build_only else ["ffmpeg", "ffprobe"])]:
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
    if args.video:
        video = args.video.resolve(strict=True)
        probe(root, binary, "user-installed-backend", video, None, False)
        probe(root, binary, "user-candidate-backend", video, "candidate", True)
    print("PASS automated CPU reference; Arch/MPV visual and GPU acceptance remain OPEN")
    print("For an explicit temporary Pivra visual comparison (no installation):")
    print("GTK_MEDIA=gstreamer PIVRA_GTK_BT1886_CPU=1 GTK_PATH=" + shlex.quote(str(root / "candidate/gtk-4.0")) +
          " cargo run --locked -- " + shlex.quote(str(args.video.resolve()) if args.video else "/absolute/path/to/video.mp4"))


if __name__ == "__main__":
    try:
        main()
    except RuntimeError as error:
        print(error)
        raise SystemExit(77 if str(error).startswith("NOT RUN") else 1) from error
    except (AssertionError, OSError, subprocess.SubprocessError) as error:
        print("FAIL:", error)
        raise SystemExit(1) from error
