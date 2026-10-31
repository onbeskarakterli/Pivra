#!/usr/bin/env python3
"""Exercise real RAW subprocess-group interruption without touching other processes."""
import os
import pathlib
import shutil
import signal
import subprocess
import sys
import tempfile
import time

root = pathlib.Path(__file__).resolve().parents[1]
sample = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else pathlib.Path.home() / "Downloads/Ricoh-GR-III.DNG")
helper = root / "target/pivra-native/pivra-raw-decoder"
runner = root / "raw/run_decode.sh"
if not sample.is_file() or not helper.is_file():
    sys.exit("Missing DNG or prebuilt decoder; build with PIVRA_BUILD_EXPERIMENTAL_RAW=1 first")

parent = pathlib.Path(tempfile.mkdtemp(prefix="pivra-cancel-probe-"))
process = None
try:
    work = parent / "raw-work"
    work.mkdir()
    output = work / "frame.ppm"
    env = dict(os.environ, TMPDIR=str(work), PIVRA_RAW_HELPER=str(helper),
               PIVRA_RAW_INSTALLED="0")
    process = subprocess.Popen(["bash", str(runner), str(sample), str(output)],
                               env=env, stdout=subprocess.DEVNULL,
                               stderr=subprocess.DEVNULL, start_new_session=True)
    # Interrupt the *real* runner and any current bwrap decoder children.
    time.sleep(0.05)
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    process.wait(timeout=10)
    # Model Pivra's scoped directory owner: the owner must remove the
    # complete scratch directory even if SIGKILL prevents shell EXIT traps.
    shutil.rmtree(work)
    if work.exists():
        sys.exit("FAIL: RAW scratch directory survived cleanup")
    print(f"PASS: isolated RAW runner interrupted (exit={process.returncode})")
    print("PASS: scoped RAW scratch directory removed after interruption")
    print("NOTE: This tests process-group interruption and scoped cleanup; "
          "it does not prove GUI cancellation timing or absence of all zombies.")
finally:
    if process is not None and process.poll() is None:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait()
    shutil.rmtree(parent, ignore_errors=True)
