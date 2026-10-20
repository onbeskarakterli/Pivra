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


def file_launches(root):
    """Private copies only; second launch exercises percent-encoded URI input."""
    from pathlib import Path
    payload = (Path(__file__).parent / "fixtures/pixels-2x2.png").read_bytes()
    paths = [root / "fotoğraf # % bir.png", root / "Türkçe 🦀 iki.png"]
    for path in paths:
        path.write_bytes(payload)
    (root / "neighbor.MP4").write_bytes((Path(__file__).parent / "fixtures/model-video.txt").read_bytes())
    (root / "unsupported.txt").write_text("not a candidate")
    (root / "nested").mkdir()
    (root / "nested/hidden.png").write_bytes(payload)
    return [([str(paths[0])], f"{paths[0].name} — Pivra (3 candidates)"),
            ([paths[1].as_uri()], f"{paths[1].name} — Pivra (3 candidates)")]


def check_file_copies(root):
    from pathlib import Path
    payload = (Path(__file__).parent / "fixtures/pixels-2x2.png").read_bytes()
    for path in root.glob("*.png"):
        assert path.read_bytes() == payload, "opening modified a fixture copy"

    if (root / "neighbor.MP4").exists():
        assert (root / "neighbor.MP4").read_bytes() == (Path(__file__).parent / "fixtures/model-video.txt").read_bytes()
        assert (root / "unsupported.txt").read_text() == "not a candidate"
        assert (root / "nested/hidden.png").read_bytes() == payload
