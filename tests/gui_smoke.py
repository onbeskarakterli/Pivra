#!/usr/bin/env python3
"""Real X11 window lifecycle smoke test; run inside xvfb-run + dbus-run-session.

Requires xdotool and libX11. It fails if no graphical display is available.
No application test flags, timers, or auto-close hooks are added to Pivra.
"""

import ctypes
import os
from pathlib import Path
import subprocess
import sys
import time


def xdo(*args):
    return subprocess.run(
        ["xdotool", *map(str, args)], capture_output=True, text=True, check=True
    ).stdout.strip()


def wait_window(process):
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        assert process.poll() is None, "Pivra exited before mapping a window"
        result = subprocess.run(
            ["xdotool", "search", "--all", "--onlyvisible", "--pid", str(process.pid),
             "--name", "^Pivra$"], capture_output=True, text=True
        )
        if result.returncode == 0 and result.stdout.strip():
            return int(result.stdout.splitlines()[0])
        time.sleep(0.1)
    raise AssertionError("Pivra window did not become visible")


def request_close(window):
    # Send the standard window-manager close protocol, not XDestroyWindow/SIGTERM.
    class Data(ctypes.Union):
        _fields_ = [("l", ctypes.c_long * 5)]

    class ClientMessage(ctypes.Structure):
        _fields_ = [("type", ctypes.c_int), ("serial", ctypes.c_ulong),
                    ("send_event", ctypes.c_int), ("display", ctypes.c_void_p),
                    ("window", ctypes.c_ulong), ("message_type", ctypes.c_ulong),
                    ("format", ctypes.c_int), ("data", Data)]

    class Event(ctypes.Union):
        _fields_ = [("client", ClientMessage), ("pad", ctypes.c_long * 24)]

    x11 = ctypes.CDLL("libX11.so.6")
    x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
    x11.XOpenDisplay.restype = ctypes.c_void_p
    x11.XInternAtom.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_int]
    x11.XInternAtom.restype = ctypes.c_ulong
    x11.XSendEvent.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int,
                              ctypes.c_long, ctypes.POINTER(Event)]
    x11.XFlush.argtypes = [ctypes.c_void_p]
    x11.XCloseDisplay.argtypes = [ctypes.c_void_p]
    display = x11.XOpenDisplay(None)
    assert display, "cannot connect to X11"
    try:
        event = Event()
        event.client.type = 33  # ClientMessage
        event.client.window = window
        event.client.message_type = x11.XInternAtom(display, b"WM_PROTOCOLS", 0)
        event.client.format = 32
        event.client.data.l[0] = x11.XInternAtom(display, b"WM_DELETE_WINDOW", 0)
        assert x11.XSendEvent(display, window, 0, 0, ctypes.byref(event))
        x11.XFlush(display)
    finally:
        x11.XCloseDisplay(display)


def main():
    assert os.environ.get("DISPLAY"), "run under an X11 display (e.g. xvfb-run)"
    binary = str(Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/pivra").resolve())
    processes = []
    try:
        windows = []
        for _ in range(2):
            process = subprocess.Popen([binary], stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE, text=True)
            processes.append(process)
            window = wait_window(process)
            assert xdo("getwindowname", window) == "Pivra"
            geometry = xdo("getwindowgeometry", "--shell", window)
            dimensions = dict(line.split("=", 1) for line in geometry.splitlines())
            assert (int(dimensions["WIDTH"]), int(dimensions["HEIGHT"])) == (960, 640)
            windows.append(window)
            print(f"PASS mapped window {window}: title=Pivra; {geometry!r}")

        assert windows[0] != windows[1], "launches must own separate windows"
        for index, (process, window) in enumerate(zip(processes, windows)):
            request_close(window)
            stdout, stderr = process.communicate(timeout=10)
            assert process.returncode == 0, (process.returncode, stdout, stderr)
            assert not stdout, stdout
            assert not stderr, stderr
            if index == 0:
                assert processes[1].poll() is None, "closing one launch killed the other"
                assert xdo("getwindowname", windows[1]) == "Pivra"
            print(f"PASS normal WM_DELETE_WINDOW close: exit={process.returncode}")
        print("PASS two independent launches; no stdout/stderr, crash or forced termination")
    finally:
        for process in processes:
            if process.poll() is None:
                process.terminate()
                process.communicate(timeout=10)


if __name__ == "__main__":
    main()
