#!/usr/bin/env python3
"""X11/Xvfb real Pivra window close regression for the owned GPU backend."""
import argparse
import os
from pathlib import Path
import subprocess
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from gui_smoke import request_close


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, default=Path('target/debug/pivra'))
    p.add_argument('--module-dir', type=Path, required=True)
    p.add_argument('--video', type=Path, required=True)
    p.add_argument('--log', type=Path, required=True)
    a = p.parse_args()
    if a.log.exists():
        p.error('log must be new; existing evidence is not overwritten')
    env = os.environ.copy()
    for key in ('GTK_MEDIA', 'GTK_PATH', 'PIVRA_VIDEO_TEST_MODULE_DIR'):
        env.pop(key, None)
    env.update(PIVRA_GPU_VIDEO_TARGET=str(a.video.resolve(strict=True)),
               PIVRA_GPU_VIDEO_MODULE_DIR=str(a.module_dir.resolve(strict=True)))
    with a.log.open('w') as log:
        process = subprocess.Popen([str(a.binary.resolve()), str(a.video.resolve())],
                                   env=env, stdout=log, stderr=log)
        try:
            deadline = time.monotonic() + 20
            window = None
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError('product exited before rendering')
                windows = subprocess.run(['xdotool', 'search', '--onlyvisible', '--pid',
                    str(process.pid), '--name', '.*'], capture_output=True, text=True, timeout=5)
                if windows.returncode == 0 and 'PIVRA_GPU_DRAW' in a.log.read_text():
                    window = int(windows.stdout.splitlines()[-1])
                    break
                time.sleep(.05)
            assert window, 'real product window did not render a GPU frame'
            request_close(window)
            assert process.wait(timeout=15) == 0, 'nonzero product exit'
            assert 'PIVRA_GPU_STOP' in a.log.read_text(), 'window closed without filter retirement'
            print('PASS actual Pivra GPU window rendering, normal WM close and filter retirement')
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=5)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
