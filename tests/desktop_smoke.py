#!/usr/bin/env python3
"""X11 GIO launch of the source desktop entry. No install or MIME default writes.

Normal window close and process disappearance are observed; the detached app's
exit code is unavailable here (direct CLI GUI tests check exit 0 separately).
"""
import os
import re
from pathlib import Path
import select
import signal
import subprocess
import sys
import tempfile
import time

from gui_smoke import xdo, request_close
from gui_support import check_image_diagnostics


def main():
    assert os.environ.get("DISPLAY"), "run under Xvfb + dbus-run-session"
    blocked = "--expect-image-error" in sys.argv[2:]
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/pivra").resolve()
    repo = Path(__file__).resolve().parent.parent
    entry = repo / "data/io.github.onbeskarakterli.Pivra.desktop"
    with tempfile.TemporaryDirectory(prefix="pivra-desktop-") as temporary:
        root = Path(temporary)
        env = os.environ.copy()
        for variable in ["XDG_RUNTIME_DIR", "XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_CACHE_HOME"]:
            directory = root / variable
            directory.mkdir(mode=0o700)
            env[variable] = str(directory)
        (root / "bin").mkdir()
        (root / "bin/pivra").symlink_to(binary)
        env["PATH"] = str(root / "bin") + os.pathsep + env["PATH"]
        path = root / f"{root.name} fotoğraf # % 🦀.png"
        payload = (repo / "tests/fixtures/pixels-2x2.png").read_bytes()
        path.write_bytes(payload)
        title = f"{path.name} — Pivra (1 candidate)"
        descriptor = None
        with (root / "launch.log").open("w+") as log:
            try:
                subprocess.run(["gio", "launch", str(entry), path.as_uri()], env=env,
                               stdout=log, stderr=log, check=True, timeout=10)
                deadline = time.monotonic() + 15
                while True:
                    result = subprocess.run(["xdotool", "search", "--onlyvisible", "--name", "^" + re.escape(title) + "$"],
                                            capture_output=True, text=True, timeout=5)
                    if result.returncode == 0 and result.stdout.strip():
                        window = int(result.stdout.splitlines()[0])
                        if xdo("getwindowname", window) == title:
                            break
                    if time.monotonic() > deadline:
                        log.seek(0)
                        raise AssertionError(f"desktop launch mapped no associated window: {log.read()}")
                    time.sleep(0.1)
                pid = int(xdo("getwindowpid", window))
                descriptor = os.pidfd_open(pid)
                if blocked:
                    deadline = time.monotonic() + 15
                    while True:
                        log.seek(0)
                        if "Pivra: image load failed:" in log.read(): break
                        assert time.monotonic() < deadline, "no controlled loader failure"
                        time.sleep(0.1)
                request_close(window)
                assert select.select([descriptor], [], [], 10)[0], "desktop-launched app did not terminate"
                log.seek(0)
                check_image_diagnostics(log.read(), blocked)
                assert path.read_bytes() == payload
                print("PASS GIO desktop Exec=pivra %u: percent-encoded Unicode URI -> associated X11 window")
                print("PASS normal WM close + process termination; detached exit code NOT OBSERVED")
                print("PASS no installation/default MIME writes; fixture bytes unchanged")
            finally:
                if descriptor is not None:
                    if not select.select([descriptor], [], [], 0)[0]:
                        signal.pidfd_send_signal(descriptor, signal.SIGTERM)
                    os.close(descriptor)
    return 0


if __name__ == "__main__":
    sys.exit(main())
