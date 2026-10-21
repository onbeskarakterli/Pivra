#!/usr/bin/env python3
"""Real X11 discovery warning/failure UI checks, using only private fixtures."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from gui_smoke import wait_window, request_close
from gui_support import stop, wait_image_error


def main():
    assert os.environ.get("DISPLAY"), "run under Xvfb + dbus-run-session"
    blocked = "--expect-image-error" in sys.argv[2:]
    binary = str(Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/pivra").resolve())
    payload = (Path(__file__).parent / "fixtures/pixels-2x2.png").read_bytes()
    processes = []
    with tempfile.TemporaryDirectory(prefix="pivra-discovery-errors-") as temporary:
        root = Path(temporary)
        partial = root / "partial"
        denied = root / "denied"
        partial.mkdir()
        denied.mkdir()
        for folder in [partial, denied]:
            (folder / "open.png").write_bytes(payload)
        (partial / "broken.jpg").symlink_to(partial / "missing.jpg")
        denied.chmod(0o111)  # known photo is accessible; listing the folder is not
        try:
            cases = [(partial, "1 candidate, 1 warning", "folder scan incomplete:"),
                     (denied, "folder scan failed", "folder scan failed:")]
            for folder, status, error in cases:
                process = subprocess.Popen([binary, str(folder / "open.png")],
                                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                processes.append(process)
                window = wait_window(process, f"open.png — Pivra ({status})")
                image_error = wait_image_error(process) if blocked else ""
                request_close(window)
                stdout, stderr = process.communicate(timeout=10)
                assert process.returncode == 0 and stdout == "", (process.returncode, stdout, stderr)
                stderr = image_error + stderr
                assert f"Pivra: {error}" in stderr, stderr
                if blocked: assert "Pivra: image load failed:" in stderr, stderr
                else: assert "image load failed:" not in stderr, stderr
                assert "panicked" not in stderr
                assert (folder / "open.png").read_bytes() == payload
                print(f"PASS X11 visible discovery state: {status}; controlled diagnostic; normal window close")
        finally:
            for process in processes:
                stop(process)
            denied.chmod(0o700)
    return 0


if __name__ == "__main__":
    sys.exit(main())
