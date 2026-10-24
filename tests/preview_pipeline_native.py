"""Native full-only fallback gates. Each subprocess retains its existing assertions."""
import json
from pathlib import Path
import subprocess
import sys


def main():
    report = {"early_preview": "UNSUPPORTED current provider",
              "preview_full_continuity": "NOT RUN: no early preview provider",
              "stages": [], "result": "NOT RUN"}
    commands = [
        ["target/debug/examples/image_probe"],
        ["target/debug/examples/application_open_probe"],
        ["target/debug/examples/preload_probe"],
        ["target/debug/examples/view_modes_probe", "--zoom-pan"],
        ["target/debug/examples/view_modes_probe", "--resampling"],
        ["target/debug/examples/async_cancel_probe"],
        [sys.executable, "-B", "tests/active_decode_close.py", "--binary",
         "target/debug/examples/active_decode_close_probe", "--report",
         "/tmp/pivra-preview-active-close.json"],
    ]
    code = 0
    for command in commands:
        try:
            code = subprocess.run(command, check=False, timeout=180).returncode
        except subprocess.TimeoutExpired:
            code = 124
        report["stages"].append({"command": command, "exit": code,
                                 "result": "PASS" if code == 0 else
                                 "NOT RUN" if code == 77 else "FAIL"})
        if code:
            break
    report["result"] = "PASS full-only fallback" if code == 0 else (
        "NOT RUN" if code == 77 else "FAIL")
    Path("/tmp/pivra-preview-pipeline.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return code


if __name__ == "__main__":
    sys.exit(main())
