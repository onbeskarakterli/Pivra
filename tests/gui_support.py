"""Cleanup for processes owned by a smoke test, including failed assertions.

Forced cleanup is never evidence of a normal application window close.
"""

import subprocess


def stop(process):
    if process is not None and process.poll() is None:
        process.terminate()
        try:
            process.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.communicate(timeout=5)
