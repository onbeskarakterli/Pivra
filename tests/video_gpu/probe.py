#!/usr/bin/env python3
"""Prepare a pinned GTK GPU patch and execute its shader math via EGL/OpenGL.

This does not build/install GTK or establish native video/DMA-BUF acceptance.
Software GL may validate math, but never passes --require-hardware.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import stat
import subprocess
import tarfile
import tempfile
import urllib.request

HERE = Path(__file__).resolve().parent
ARCHIVE_URL = "https://codeload.github.com/GNOME/gtk/tar.gz/refs/tags/4.24.1"
ARCHIVE_SHA256 = "6b03cc3eac17e6d7b252c7ac96559e58aa7c990874ca47da696d81d0add7521d"
SOURCE_SHA256 = {
    "gdk/gdkcolorstateprivate.h": "bc4d3fcf4b7a1228bb40fd10d21f7274605ac3c40949abed47a5d0338ffa76a7",
    "gdk/gdkcolordefs.h": "eae9c9af935d49a329670543da516c8218861b8e5871a53ab7356c0bad2bcc78",
    "gdk/gdkcolorstate.c": "2efc7b706bd29b23ca31294198ac8de6a9112be47e8f4dec57e577fb5ab62d74",
    "gtk/media/gtkgstsink.c": "5a5591e2e72b2b93f49174cd25e3e975a9022d73e9e84990af93eb47ea6a9f04",
    "gsk/gpu/gskgpuutils.c": "45a5ed398c058f6684da295ae8db41950cd781e7e4b9da6709d85769e02847e1",
    "gsk/gpu/gskgpunodeprocessor.c": "6814566a558d602085aaeafd40977597b514e0b3bd3e50b840a8c59dade9c6ab",
    "gsk/gpu/gskglrenderer.c": "c10a64b5bc0b9874ff255991402796f409269921e8516961eab7f13dc7928d0c",
    "gsk/gpu/gskgpuuploadop.c": "19db87825b71090463be6a4964bf3366ce9c90b315ff99eca2ea5c8dc6e4918c",
    "gsk/gpu/shaders/gskgpuconvertcicp.glsl": "763870be96995c94dc0a229d7db66ac305827f5ab8280746ad742b53001feb8e",
    "gsk/gskrendernodeparser.c": "161231adaa6e8fd2603ff084a0244bc993c0eef78c39b9d69443f890eade6923",
    "testsuite/gdk/colorstate-internal.c": "aa92aa6d866b22f83be783ee613433c72826cdb848b07e818a3dd3bc78eeee1c",
    "testsuite/gsk/serialize.c": "afb7abfc5aaf38625c5da45feb89b2311ec7bc5b6bbc999feb7cc2459e899aa8",
    "gsk/gpu/shaders/color.glsl": "f8ddf9dc408acafce9fab81eefe23da135cd703658ad393c54b9863a75f9ad21",
    "gsk/gpu/shaders/generate-glsl-interface.py": "937f2594bac13644669393f3798e595bc7eb522dadb5f055edf008429d4ee007",
    "meson.build": "5b873274012a073ba189075862430a6407261368a4164cb992bafa9b37800a28"
}  # Filled from the pinned archive, not a moving source reference.


def run(args, **kwargs):
    return subprocess.run(args, text=True, check=True, **kwargs)


def fragment(source):
    """Reuse GTK's real CICP math; supply test uniforms instead of GSK instances.

    The generated GSK instance ABI is checked separately. This is not a test of
    the full GSK node processor, driver import or compositor.
    """
    shader = (source / "gsk/gpu/shaders/gskgpuconvertcicp.glsl").read_text()
    common = (source / "gsk/gpu/shaders/color.glsl").read_text()
    start = shader.index("#ifdef GSK_VERTEX_SHADER")
    vertex = shader[start:].split("void\nrun (", 1)[0].replace("#ifdef GSK_VERTEX_SHADER", "", 1)
    body = shader.split("#ifdef GSK_FRAGMENT_SHADER", 1)[1].split("void\nrun (", 1)[0]
    # This selector exists only in the patched shader; use its exact code.
    intent = ""
    if "uint\nvideo_bt1886_intent" in shader:
        a = shader.index("uint\nvideo_bt1886_intent")
        intent = shader[a:start]
    prefix = """#version 330 core
#define GDK_COLOR_STATE_ID_SRGB 0u
#define GDK_COLOR_STATE_ID_SRGB_LINEAR 1u
#define GDK_COLOR_STATE_ID_REC2100_PQ 2u
#define GDK_COLOR_STATE_ID_REC2100_LINEAR 3u
#define GSK_COLOR_STATES (1u << 16u)
#define VARIATION_PREMULTIPLY u_premul
uniform sampler2D u_tex;
uniform uint u_primaries, u_transfer, u_matrix, u_range, u_requested;
uniform bool u_reverse, u_premul;
"""
    varyings = """uint _transfer_function, _range, _video_bt1886;
mat3 _mat, _yuv;
vec3 _yuv_add;
"""
    selected = "video_bt1886_intent(u_primaries,u_transfer,u_matrix,u_range,u_requested)" if intent else "0u"
    vertex_main = """
void main() {
  _transfer_function=u_transfer; _range=u_range;
  _video_bt1886=SELECTED;
  if (u_reverse) {
    _mat=cicp_from_xyz(u_primaries)*srgb_to_xyz;
    _yuv=rgb_to_yuv(u_matrix,_yuv_add);
  } else {
    _mat=xyz_to_srgb*cicp_to_xyz(u_primaries);
    _yuv=yuv_to_rgb(u_matrix,_yuv_add);
  }
  MATRIX_FIX
  vec2 p=vec2((gl_VertexID<<1)&2,gl_VertexID&2);
  gl_Position=vec4(p*2.0-1.0,0,1);
}
""".replace("SELECTED", selected).replace("MATRIX_FIX", "_mat=video_bt1886_matrix(_mat,_video_bt1886);" if intent else "")
    fragment_main = """
out vec4 result;
void main() {
  vec4 pixel=texelFetch(u_tex,ivec2(gl_FragCoord.xy),0);
  result=u_reverse?convert_color_to_cicp(pixel):convert_color_from_cicp(pixel);
}
"""
    vertex_code = prefix + "\n".join("flat out " + line for line in varyings.splitlines()) + "\n" + common + intent + vertex + vertex_main
    fragment_code = prefix + "\n".join("flat in " + line for line in varyings.splitlines()) + "\n" + common + body + fragment_main
    return vertex_code, fragment_code


def prepare(root, source):
    if source is None:
        archive = root / "gtk.tar.gz"
        with urllib.request.urlopen(ARCHIVE_URL, timeout=40) as response:
            data = response.read(32 * 1024 * 1024 + 1)
        assert hashlib.sha256(data).hexdigest() == ARCHIVE_SHA256, "GTK archive checksum"
        archive.write_bytes(data)
        with tarfile.open(archive) as tar:
            members = tar.getmembers()
            assert len(members) <= 20000 and sum(m.size for m in members) <= 500 * 1024 * 1024
            tar.extractall(root / "source", filter="data")
        source = root / "source/gtk-4.24.1"
    for path, digest in SOURCE_SHA256.items():
        assert hashlib.sha256((source / path).read_bytes()).hexdigest() == digest, "modified/unverified upstream source: " + path
    patched = root / "gtk-4.24.1"
    shutil.copytree(source, patched, symlinks=True, ignore=shutil.ignore_patterns(".git", "__pycache__", "_build", "_build-*"))
    run(["patch", "--batch", "--fuzz=0", "-p1", "-i", str(HERE / "gtk-4.24.1-video-bt1886.patch")], cwd=patched)
    original_fragment = root / "original.frag"
    patched_fragment = root / "candidate.frag"
    old_vertex, old_fragment = fragment(source)
    new_vertex, new_fragment = fragment(patched)
    original_fragment.write_text(old_fragment)
    patched_fragment.write_text(new_fragment)
    (root / "original.vert").write_text(old_vertex)
    (root / "candidate.vert").write_text(new_vertex)
    generator = patched / "gsk/gpu/shaders/generate-glsl-interface.py"
    for mode in ["header", "source", "glsl"]:
        output = run([os.sys.executable, str(generator), "--generate-" + mode,
                      str(patched / "gsk/gpu/shaders/gskgpuconvertcicp.glsl")], capture_output=True).stdout
        assert "video_bt1886" in output, "shader parameter lost in code generation"
        (root / ("generated-" + mode + ".txt")).write_text(output)
    baseline = run([os.sys.executable, str(generator), "--generate-source",
                    str(source / "gsk/gpu/shaders/gskgpuconvertcicp.glsl")], capture_output=True).stdout
    layout = "#pragma once\n"
    for label, text in [("Original", baseline), ("Candidate", (root / "generated-source.txt").read_text())]:
        fields = re.search(r"struct _GskGpuConvertCicpInstance \{(.*?)\};", text, re.S)
        assert fields, "generated instance layout unavailable"
        layout += "struct Pivra" + label + "Instance {" + fields.group(1) + "};\n"
    (root / "gpu_layout.h").write_text(layout)
    print("PASS pinned patch application and GSK shader interface generation", flush=True)
    return patched, [root / "original.vert", original_fragment, root / "candidate.vert", patched_fragment]


def hardware(report):
    renderer = report["renderer"].lower()
    if any(word in renderer for word in ["llvmpipe", "softpipe", "swrast", "software", "lavapipe", "swiftshader"]):
        return False
    known = any(word in (report["vendor"] + " " + renderer).lower() for word in ["intel", "amd", "radeon", "nvidia", "geforce", "quadro"])
    node = report["drm_render_node"]
    if not known or not re.fullmatch(r"/dev/dri/renderD[0-9]+", node):
        return False
    try:
        info = Path(node).stat()
    except OSError:
        return False
    return stat.S_ISCHR(info.st_mode) and os.major(info.st_rdev) == 226


def gtk_prerequisites():
    result = {}
    for name, minimum in {"glib-2.0": "2.89.3", "pango": "1.58", "wayland-client": "1.24.0",
                          "wayland-protocols": "1.48", "gstreamer-1.0": "1.28.0"}.items():
        found = subprocess.run(["pkg-config", "--modversion", name], capture_output=True, text=True)
        ready = subprocess.run(["pkg-config", "--atleast-version=" + minimum, name], capture_output=True)
        result[name] = {"required": minimum, "available": found.stdout.strip() if found.returncode == 0 else None,
                        "satisfied": ready.returncode == 0}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, help="read-only unmodified GTK 4.24.1 source cache")
    parser.add_argument("--output", type=Path, help="new/empty private workspace to retain")
    parser.add_argument("--require-hardware", action="store_true")
    parser.add_argument("--prepare-only", action="store_true", help="prepare full source/patch and generated interfaces; do not build or run GTK")
    args = parser.parse_args()
    if args.prepare_only and args.require_hardware:
        parser.error("--prepare-only cannot satisfy --require-hardware")
    for tool in (["patch"] if args.prepare_only else ["cc", "pkg-config", "patch"]):
        if not shutil.which(tool):
            raise RuntimeError("NOT RUN missing tool: " + tool)
    root = args.output.resolve() if args.output else Path(tempfile.mkdtemp(prefix="pivra-video-gpu-"))
    root.mkdir(mode=0o700, parents=True, exist_ok=True)
    assert not any(root.iterdir()), "output must be empty; no files removed"
    print("Retained workspace:", root, flush=True)
    patched, shaders = prepare(root, args.source)
    if args.prepare_only:
        print("PREPARED", patched, "; full GTK build, shader execution, hardware and native acceptance NOT RUN")
        return
    packages = ["glib-2.0", "epoxy", "egl"]
    cflags = subprocess.run(["pkg-config", "--cflags", *packages], capture_output=True, text=True)
    libs = subprocess.run(["pkg-config", "--libs", *packages], capture_output=True, text=True)
    if cflags.returncode or libs.returncode:
        raise RuntimeError("NOT RUN development libraries: " + cflags.stderr + libs.stderr)
    binary = root / "gpu-probe"
    run(["cc", "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror", "-I" + str(root), "-I" + str(patched / "gdk"),
         *shlex.split(cflags.stdout), str(HERE / "gpu_probe.c"), "-o", str(binary), *shlex.split(libs.stdout), "-lm"])
    env = os.environ.copy()
    env.setdefault("EGL_PLATFORM", "surfaceless")
    result = subprocess.run([str(binary), *[str(p) for p in shaders]], env=env, capture_output=True, text=True, timeout=120)
    (root / "driver.log").write_text(result.stdout + result.stderr)
    if result.returncode == 77:
        raise RuntimeError("NOT RUN GL context: " + result.stderr)
    assert result.returncode == 0, result.stderr
    report = json.loads(result.stdout.splitlines()[-1])
    report["gtk_build_prerequisites"] = gtk_prerequisites()
    report.update(shader_math="PASS", hardware_context="PASS" if hardware(report) else "NOT RUN",
                  gtk_pipeline="NOT RUN", dmabuf_import="NOT RUN", hardware_decode="NOT RUN",
                  native_visual="NOT RUN", source_archive_sha256=ARCHIVE_SHA256)
    (root / "REPORT.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2), flush=True)
    print("Full GTK patch compilation/integration and native video acceptance remain NOT RUN.")
    if args.require_hardware and not hardware(report):
        raise RuntimeError("NOT RUN hardware gate: software or unverified GL device; shader math PASS is not hardware acceleration")


if __name__ == "__main__":
    try:
        main()
    except RuntimeError as error:
        print(error)
        raise SystemExit(77 if str(error).startswith("NOT RUN") else 1) from error
    except (AssertionError, OSError, subprocess.SubprocessError) as error:
        print("FAIL:", error)
        raise SystemExit(1) from error
