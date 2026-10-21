#!/usr/bin/env python3
"""Real X11 arrow events against Pivra; window/title scope, never pixel PASS."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

from gui_smoke import xdo, request_close, wait_window
from gui_support import stop


def selected(process, window, name):
    expected = f'{name} — Pivra (3 candidates)'
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        assert process.poll() is None, 'app exited during navigation'
        if xdo('getwindowname', window) == expected:
            return
        time.sleep(0.02)
    raise AssertionError(('selection did not follow real arrow event', expected, xdo('getwindowname', window)))


def press(window, key):
    xdo('windowfocus', '--sync', window)
    xdo('key', '--window', window, key)


def main():
    assert os.environ.get('DISPLAY'), 'use Xvfb or an X11 session'
    binary = str(Path(sys.argv[1] if len(sys.argv) > 1 else 'target/debug/pivra').resolve())
    fixtures = Path(__file__).parent / 'fixtures'
    processes = []
    with tempfile.TemporaryDirectory(prefix='pivra-nav-x11-') as temp:
        root = Path(temp)
        original = {}
        for name, source in [('1.png', 'images/wide.png'), ('2.mp4', 'model-video.txt'), ('10.jpg', 'images/no-exif.jpg')]:
            original[name] = (fixtures / source).read_bytes()
            (root / name).write_bytes(original[name])
        with tempfile.TemporaryFile(mode='w+') as log:
            try:
                windows = []
                for name in ('1.png', '10.jpg'):
                    process = subprocess.Popen([binary, str(root / name)], stdout=subprocess.PIPE,
                                               stderr=log, text=True)
                    processes.append(process)
                    windows.append(wait_window(process, f'{name} — Pivra (3 candidates)'))
                first, second = windows
                press(first, 'Tab')  # focus traversal must not disable window capture
                press(first, 'Left'); selected(processes[0], first, '1.png')
                press(first, 'Right'); selected(processes[0], first, '2.mp4')
                press(first, 'Right'); selected(processes[0], first, '10.jpg')
                press(first, 'Right'); selected(processes[0], first, '10.jpg')
                press(first, 'ctrl+Left'); selected(processes[0], first, '10.jpg')
                for _ in range(6):
                    press(first, 'Left'); selected(processes[0], first, '2.mp4')
                    press(first, 'Left'); selected(processes[0], first, '1.png')
                    press(first, 'Right'); selected(processes[0], first, '2.mp4')
                    press(first, 'Right'); selected(processes[0], first, '10.jpg')
                selected(processes[1], second, '10.jpg')
                press(second, 'Left'); selected(processes[1], second, '2.mp4')
                selected(processes[0], first, '10.jpg')
                for index, (process, window) in enumerate(zip(processes, windows)):
                    request_close(window)
                    stdout, _ = process.communicate(timeout=10)
                    assert process.returncode == 0 and not stdout
                    if index == 0:
                        assert processes[1].poll() is None
                log.seek(0)
                diagnostics = log.read()
                assert not any(s in diagnostics for s in ('panicked', 'CRITICAL', 'Image worker failed')), diagnostics
                for name, payload in original.items():
                    assert (root / name).read_bytes() == payload
                print('PASS real X11 arrow events after Tab: 1/photo -> 2/video -> 10/photo, boundaries, Ctrl filtering, rapid alternation, independent windows, normal close')
                print('NOT TESTED real image pixels/Fit; controlled loader errors permitted in this scoped test')
            finally:
                for process in processes:
                    stop(process)


if __name__ == '__main__':
    main()
