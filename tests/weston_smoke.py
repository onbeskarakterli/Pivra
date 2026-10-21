#!/usr/bin/env python3
"""Weston headless surface check, separate from the full Sway lifecycle check.

Run inside dbus-run-session. PIVRA_WESTON_PREFIX selects an extracted /usr tree.
Exit 0 means mapped-surface/title/geometry PASS, NOT normal-close validation.
Normal close is NOT RUN: Weston headless exposes no standard window-control IPC.
"""

import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

from gui_support import stop, file_launches, check_file_copies, wait_image_error, check_image_diagnostics


def main():
    prefix = Path(os.environ.get("PIVRA_WESTON_PREFIX", "/usr"))
    weston = prefix / "bin/weston"
    debug = prefix / "bin/weston-debug"
    lib = prefix / "lib/x86_64-linux-gnu"
    backend = lib / "libweston-14/headless-backend.so"
    shell = lib / "weston/desktop-shell.so"
    desktop = prefix / "libexec/weston-desktop-shell"
    keyboard = prefix / "libexec/weston-keyboard"
    for required in [weston, debug, backend, shell, desktop, keyboard]:
        if not required.exists():
            print(f"BLOCKED Weston 14 surface check: missing {required}", file=sys.stderr)
            return 77
    assert os.environ.get("DBUS_SESSION_BUS_ADDRESS"), "run inside dbus-run-session"
    blocked = "--expect-image-error" in sys.argv[2:]
    binary = str(Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/pivra").resolve())
    with tempfile.TemporaryDirectory(prefix="pivra-weston-") as temp:
        root = Path(temp)
        args, title = file_launches(root)[1] if "--files" in sys.argv[2:] else ([], "Pivra")
        env = os.environ.copy()
        for name in ["DISPLAY", "WAYLAND_DISPLAY", "WAYLAND_DEBUG", "SWAYSOCK", "I3SOCK"]:
            env.pop(name, None)
        for name, folder in [("XDG_RUNTIME_DIR", "runtime"), ("XDG_CACHE_HOME", "cache"),
                             ("XDG_CONFIG_HOME", "config"), ("XDG_DATA_HOME", "data")]:
            path = root / folder
            path.mkdir(mode=0o700)
            env[name] = str(path)
        env.update(GDK_BACKEND="wayland", GSK_RENDERER="cairo", GTK_A11Y="none",
                   WAYLAND_DISPLAY="pivra-weston", WESTON_DATA_DIR=str(prefix / "share/weston"),
                   WESTON_MODULE_MAP=f"headless-backend.so={backend};desktop-shell.so={shell}")
        env["LD_LIBRARY_PATH"] = f"{lib}/weston:{lib}:" + env.get("LD_LIBRARY_PATH", "")
        config = root / "weston.ini"
        config.write_text(
            "[core]\nxwayland=false\nidle-time=0\n"
            f"[shell]\nclient={desktop}\npanel-position=none\nlocking=false\n"
            f"[input-method]\npath={keyboard}\n", encoding="utf-8"
        )
        socket = Path(env["XDG_RUNTIME_DIR"]) / env["WAYLAND_DISPLAY"]
        app = trace = compositor = None
        with (root / "weston.log").open("w+") as log, (root / "proto.log").open("w+") as proto:
            try:
                compositor = subprocess.Popen(
                    [str(weston), "--backend=headless", "--renderer=pixman", "--shell=desktop",
                     f"--config={config}", "--socket=pivra-weston", "--width=1280",
                     "--height=1024", "--debug"], env=env, stdout=log, stderr=log
                )
                deadline = time.monotonic() + 15
                while not socket.is_socket():
                    if compositor.poll() is not None or time.monotonic() > deadline:
                        log.seek(0)
                        raise AssertionError(f"Weston headless startup failed: {log.read()}")
                    time.sleep(0.1)
                # Private 0700 runtime only; debug extension is stopped with the test.
                trace = subprocess.Popen([str(debug), "proto"], env=env, stdout=proto, stderr=proto)
                # Prove the debug subscription is bound before starting the application.
                deadline = time.monotonic() + 5
                while True:
                    probe = subprocess.run([str(debug), "scene-graph"], env=env,
                                           capture_output=True, text=True, check=True, timeout=5)
                    proto.seek(0)
                    if "weston_debug" in proto.read():
                        break
                    if time.monotonic() > deadline:
                        raise AssertionError("Weston protocol trace did not subscribe")
                    time.sleep(0.1)
                app = subprocess.Popen([binary, *args], env=env, stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE, text=True)
                deadline = time.monotonic() + 15
                while True:
                    assert app.poll() is None, "Pivra exited before mapping a surface"
                    scene = subprocess.run([str(debug), "scene-graph"], env=env,
                                           capture_output=True, text=True, check=True, timeout=5).stdout
                    views = re.split(r"(?=\tView \d+ |Layer \d+ )", scene)
                    mapped = any(
                        f"role xdg_toplevel, PID {app.pid}," in view
                        and f"top-level window '{title}'" in view
                        and "outputs: 0 (headless)" in view and "SHM buffer" in view
                        for view in views
                    )
                    proto.seek(0)
                    messages = "\n".join(line for line in proto.read().splitlines()
                                         if f"(PID {app.pid})" in line)
                    geometry = re.search(r"set_window_geometry\(-?\d+, -?\d+, 960, 640\)", messages)
                    if mapped and geometry and f'.set_title("{title}")' in messages:
                        assert re.search(r"\.attach\(wl_buffer@\d+,", messages)
                        assert ".ack_configure(" in messages
                        assert ".commit()" in messages
                        break
                    if time.monotonic() > deadline:
                        raise AssertionError(f"no configured/rendered Pivra surface:\n{scene}\n{messages}")
                    time.sleep(0.1)
                print(f"PASS Weston 14 headless/Pixman: mapped xdg_toplevel {title} on headless output")
                check_file_copies(root)
                print("PASS server protocol: title, 960x640 window geometry, ack_configure, buffer attach/commit")
                print("PASS native Wayland: GDK_BACKEND=wayland, DISPLAY absent, XWayland disabled")
                print("NOT RUN Weston normal close: no standard close-control IPC; cleanup uses SIGTERM")
                image_error = wait_image_error(app) if blocked else ""
                # Cleanup termination is not counted as a normal window close or exit-0 check.
                app.terminate()
                stdout, stderr = app.communicate(timeout=5)
                assert stdout == "", stdout
                check_image_diagnostics(image_error + stderr, blocked)
                print("PASS Weston application has no unexpected diagnostics (termination exit status not validated)")
            finally:
                stop(app)
                stop(trace)
                stop(compositor)
    return 0


if __name__ == "__main__":
    sys.exit(main())
