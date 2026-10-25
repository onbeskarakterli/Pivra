#!/usr/bin/env python3
"""Real GTK Wayland surface lifecycle test using isolated headless Sway IPC.

Run after building: dbus-run-session -- python3 tests/wayland_smoke.py
Requires sway/swaymsg (wlroots headless + pixman). Missing tools are BLOCKED,
never a passing skip. No application source hooks or XWayland fallback.
"""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

from gui_support import stop, file_launches, check_file_copies, wait_image_error, check_image_diagnostics

APP_ID = "io.github.onbeskarakterli.Pivra"


def nodes(tree):
    yield tree
    for child in tree.get("nodes", []) + tree.get("floating_nodes", []):
        yield from nodes(child)


def ipc(env, kind, command=None):
    args = ["swaymsg", "-r", "-s", env["SWAYSOCK"], "-t", kind]
    if command is not None:
        args.append(command)
    result = subprocess.run(args, env=env, capture_output=True, text=True,
                            check=True, timeout=5)
    data = json.loads(result.stdout)
    if kind == "command":
        assert data and all(reply.get("success") for reply in data), data
    return data


def wait_surface(process, env, title="Pivra"):
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        assert process.poll() is None, "Pivra exited before mapping a Wayland surface"
        for node in nodes(ipc(env, "get_tree")):
            if node.get("pid") != process.pid or node.get("app_id") != APP_ID:
                continue
            assert node.get("shell") == "xdg_shell", node
            assert node.get("visible") is True, node
            assert node.get("name") == title, node
            rect = node["window_rect"]
            assert (rect["width"], rect["height"]) == (960, 640), node
            return node["id"]
        time.sleep(0.1)
    raise AssertionError("no mapped Pivra xdg_shell surface within 15s")


def main():
    missing = [name for name in ["sway", "swaymsg"] if not shutil.which(name)]
    if missing:
        print(f"BLOCKED headless Wayland: missing {', '.join(missing)}", file=sys.stderr)
        return 77
    assert os.environ.get("DBUS_SESSION_BUS_ADDRESS"), "run inside dbus-run-session"
    binary = str(Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/pivra").resolve())
    processes = []
    diagnostics = []
    blocked = "--expect-image-error" in sys.argv[2:]
    with tempfile.TemporaryDirectory(prefix="pivra-wayland-") as temp:
        root = Path(temp)
        env = os.environ.copy()
        for name in ["DISPLAY", "WAYLAND_DISPLAY", "SWAYSOCK", "I3SOCK", "XAUTHORITY", "WAYLAND_DEBUG"]:
            env.pop(name, None)
        for name, folder in [("XDG_RUNTIME_DIR", "runtime"), ("XDG_CACHE_HOME", "cache"),
                             ("XDG_CONFIG_HOME", "config"), ("XDG_DATA_HOME", "data")]:
            path = root / folder
            path.mkdir(mode=0o700)
            env[name] = str(path)
        env.update(WLR_BACKENDS="headless", WLR_HEADLESS_OUTPUTS="1",
                   WLR_RENDERER="pixman", WLR_LIBINPUT_NO_DEVICES="1",
                   GDK_BACKEND="wayland", GSK_RENDERER="cairo", GTK_A11Y="none")
        runtime = Path(env["XDG_RUNTIME_DIR"])
        config = root / "sway.conf"
        config.write_text(
            "xwayland disable\n"
            "output HEADLESS-1 mode 1280x1024\n"
            "default_border none\n"
            "default_floating_border none\n"
            f'for_window [app_id="{APP_ID}"] floating enable\n', encoding="utf-8"
        )
        with (root / "sway.log").open("w+") as log:
            compositor = subprocess.Popen(["sway", "-c", str(config)], env=env,
                                          stdout=log, stderr=log)
            try:
                deadline = time.monotonic() + 15
                while time.monotonic() < deadline:
                    if compositor.poll() is not None:
                        log.seek(0)
                        raise AssertionError(f"headless Sway failed: {log.read()}")
                    ipc_sockets = list(runtime.glob("sway-ipc.*.sock"))
                    displays = [p for p in runtime.glob("wayland-*") if p.is_socket()]
                    if ipc_sockets and displays:
                        env["SWAYSOCK"] = str(ipc_sockets[0])
                        env["WAYLAND_DISPLAY"] = displays[0].name
                        break
                    time.sleep(0.1)
                else:
                    raise AssertionError("headless compositor sockets did not become ready")

                version = ipc(env, "get_version")
                print(f"Headless Wayland compositor: {version['human_readable']}; "
                      "GDK_BACKEND=wayland, XWayland disabled, DISPLAY absent")
                if "--navigation-probe" in sys.argv[2:]:
                    args = [binary] + (["--expect-image-error"] if blocked else [])
                    if "--media-failures" in sys.argv[2:]:
                        args.append("--media-failures")
                    if "--rename-pending" in sys.argv[2:]:
                        args.append("--rename-pending")
                    if "--queue-timeout" in sys.argv[2:]:
                        args.append("--queue-timeout")
                    if "--controlled" in sys.argv[2:]:
                        args.append("--controlled")
                    if "--resampling" in sys.argv[2:]:
                        args.append("--resampling")
                    if "--zoom-pan" in sys.argv[2:]:
                        args.append("--zoom-pan")
                    if "--synthetic" in sys.argv[2:]:
                        args.append("--synthetic")
                    if "--thumbnail-catalog" in sys.argv[2:]:
                        args.append("--catalog-regression")
                    if "--thumbnail-retention" in sys.argv[2:]:
                        args.append("--retention-regression")
                    if "--dark" in sys.argv[2:]:
                        args.append("--dark")
                    result = subprocess.run(args, env={**env, "G_DEBUG": "fatal-criticals"},
                                            capture_output=True, text=True, timeout=60)
                    print(result.stdout, end="")
                    if result.returncode == 77:
                        print(result.stderr, file=sys.stderr, end="")
                        return 77
                    assert result.returncode == 0, (result.returncode, result.stdout, result.stderr)
                    assert "panicked" not in result.stderr and "CRITICAL" not in result.stderr, result.stderr
                    print("PASS headless native Wayland probe; action/signals are not physical input")
                    return 0
                surfaces = []
                cases = file_launches(root) if "--files" in sys.argv[2:] else [([], "Pivra")] * 2
                for args, title in cases:
                    process = subprocess.Popen([binary, *args], env=env, stdout=subprocess.PIPE,
                                               stderr=subprocess.PIPE, text=True)
                    processes.append(process)
                    surface = wait_surface(process, env, title)
                    diagnostics.append(wait_image_error(process) if blocked else "")
                    surfaces.append(surface)
                    print(f"PASS mapped xdg_shell surface {surface}: {title}, 960x640")
                assert surfaces[0] != surfaces[1]

                for index, (process, surface) in enumerate(zip(processes, surfaces)):
                    # Sway sends xdg_toplevel.close; it does not kill the client process.
                    ipc(env, "command", f"[con_id={surface}] kill")
                    stdout, stderr = process.communicate(timeout=10)
                    check_image_diagnostics(diagnostics[index] + stderr, blocked)
                    assert (process.returncode, stdout) == (0, ""), (
                        process.returncode, stdout, stderr
                    )
                    tree = list(nodes(ipc(env, "get_tree")))
                    assert not any(node.get("id") == surface for node in tree)
                    if index == 0:
                        assert processes[1].poll() is None
                        assert wait_surface(processes[1], env, cases[1][1]) == surfaces[1]
                    print(f"PASS Wayland normal close of {surface}: exit 0, no unexpected diagnostics")
                check_file_copies(root)
                print("PASS headless Wayland: two independent native Wayland surfaces")
            finally:
                for process in processes:
                    stop(process)
                stop(compositor)
    return 0


if __name__ == "__main__":
    sys.exit(main())
