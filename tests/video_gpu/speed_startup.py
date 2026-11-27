"""Run the real Pivra executable on X11. Software GL is a test control."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

from lifecycle import ROOT, session

sys.path.insert(0, str(ROOT / "tests"))
from slideshow_smoke import window_manager


def fixture(path, *, bt709):
    color = "bt709" if bt709 else "undef"
    subprocess.run([
        "ffmpeg", "-hide_banner", "-loglevel", "error", "-f", "lavfi",
        "-i", "testsrc2=size=192x96:rate=12", "-t", "12", "-c:v", "libx264",
        "-pix_fmt", "yuv420p", "-x264-params",
        f"colorprim={color}:transfer={color}:colormatrix=bt709:fullrange=off",
        str(path),
    ], check=True)


def check(binary, path, env, log_path, expected, forbidden):
    with log_path.open("w") as log:
        proc = subprocess.Popen([str(binary), str(path)], cwd=ROOT, env=env,
                                stdout=log, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic() + 15
            until = ()
            while proc.poll() is None and time.monotonic() < deadline:
                text = log_path.read_text()
                until = tuple(marker for marker in expected if marker not in text)
                if not until:
                    break
                time.sleep(0.03)
            assert proc.poll() is None and not until, (log_path, until, log_path.read_text())
            subprocess.run(["xdotool", "key", "Escape"], env=env, check=True)
            assert proc.wait(timeout=5) == 0, log_path.read_text()
            text = log_path.read_text()
            assert not any(marker in text for marker in forbidden), text
        finally:
            if proc.poll() is None:
                proc.terminate()
                proc.wait(timeout=5)
    print(f"PASS {log_path.stem}: {log_path}", flush=True)


def main():
    out = Path(tempfile.mkdtemp(prefix="pivra-speed-startup-"))
    print(f"Logs: {out} (X11 + software GL control)", flush=True)
    bt709 = out / "bt709.mp4"
    unknown = out / "unknown.mp4"
    fixture(bt709, bt709=True)
    fixture(unknown, bt709=False)
    subprocess.run(["cargo", "build", "--locked", "--bin", "pivra"], cwd=ROOT, check=True)
    binary = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/pivra"
    version = subprocess.check_output(["pkg-config", "--modversion", "gtk4"], text=True)
    alias = "NGL" if int(version.split(".")[1]) < 18 else "opengl"
    env = dict(os.environ, XDG_CACHE_HOME=str(out / "cache"), XDG_CONFIG_HOME=str(out / "config"),
               XDG_DATA_HOME=str(out / "data"), PIVRA_TRACE_VIDEO="1",
               LIBGL_ALWAYS_SOFTWARE="1", PIVRA_GPU_ALLOW_SOFTWARE_TEST="1")
    for key in ("GDK_BACKEND", "GSK_RENDERER", "PIVRA_VIDEO_BACKEND"):
        env.pop(key, None)
    ready = "PIVRA_VIDEO_SPEED backend=PivraGpuMediaFile enabled=true reason=ready"
    presented = "PIVRA_GPU_PRESENT GLMemory sync-meta retained"
    stock = "enabled=false reason=stock-backend-no-rate-api"
    with session("x11", env, out) as active:
        active.pop("GDK_BACKEND")
        with window_manager("x11", active, out):
            for name, renderer in (("automatic", {}), ("opengl-alias", {"GSK_RENDERER": alias})):
                check(binary, bt709, dict(active, **renderer), out / f"{name}.log",
                      ("reason=realized-opengl-renderer", ready, presented), (stock,))
            check(binary, bt709, dict(active, GSK_RENDERER="cairo"), out / "cairo.log",
                  ("reason=incompatible-realized-renderer", stock), (ready, presented))
            check(binary, unknown, active, out / "untagged-transfer.log",
                  ("reason=video-metadata-outside-gpu-policy", stock), (ready, presented))


if __name__ == "__main__":
    main()
