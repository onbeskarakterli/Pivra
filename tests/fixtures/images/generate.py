#!/usr/bin/env python3
"""Original synthetic JPEG/PNG assets; --check uses stdlib only.

--write requires Pillow 12.3.0 (encoding only). Never a production decoder.
"""
import argparse
import hashlib
import io
import json
from pathlib import Path
import struct


def write(root):
    from PIL import Image, __version__
    assert __version__ == "12.3.0", "fixture encoding is pinned to Pillow 12.3.0"
    colors = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 0)]
    def pattern(width, height):
        image = Image.new("RGB", (width, height))
        for index, color in enumerate(colors):
            left = (index % 2) * (width // 2)
            top = (index // 2) * (height // 2)
            image.paste(color, (left, top, left + width // 2, top + height // 2))
        return image
    files = {}
    for name, size in [("wide.png", (64, 32)), ("tall.png", (32, 64)),
                       ("small.png", (4, 2)), ("large.png", (1600, 1200))]:
        output = io.BytesIO()
        pattern(*size).save(output, format="PNG", compress_level=9, optimize=False)
        files[name] = output.getvalue()
    output = io.BytesIO()
    pattern(64, 32).save(output, format="JPEG", quality=100, subsampling=0, optimize=False, progressive=False)
    jpeg = output.getvalue()
    files["no-exif.jpg"] = jpeg
    corners = [[0, 1, 2, 3], [1, 0, 3, 2], [3, 2, 1, 0], [2, 3, 0, 1],
               [0, 2, 1, 3], [2, 0, 3, 1], [3, 1, 2, 0], [1, 3, 0, 2]]
    for orientation in range(1, 9):
        # One standard little-endian TIFF IFD entry, Orientation (SHORT).
        # Construct the entry explicitly: tag, type, count, inline SHORT + padding.
        tiff = b"II*\0" + struct.pack("<I", 8) + struct.pack("<H", 1)
        tiff += struct.pack("<HHIHHI", 0x112, 3, 1, orientation, 0, 0)
        payload = b"Exif\0\0" + tiff
        files[f"orientation-{orientation}.jpg"] = jpeg[:2] + b"\xff\xe1" + struct.pack(">H", len(payload) + 2) + payload + jpeg[2:]
    payload = b"Exif\0\0broken TIFF"
    files["malformed-exif.jpg"] = jpeg[:2] + b"\xff\xe1" + struct.pack(">H", len(payload) + 2) + payload + jpeg[2:]
    files["png-content.jpg"] = files["wide.png"]
    files["broken.jpg"] = b"\xff\xd8truncated synthetic JPEG"
    files["broken.png"] = b"\x89PNG\r\n\x1a\ntruncated synthetic PNG"
    files["text-content.jpg"] = b"Not an image. Original Pivra test payload.\n"
    for name, data in files.items():
        (root / name).write_bytes(data)
    manifest = {"source": "Original four-quadrant RGB patterns in generate.py; no downloaded assets",
                "encoder": "Pillow 12.3.0; JPEG quality=100 subsampling=0; PNG compress_level=9",
                "license_status": "Project license undecided; no license selection introduced",
                "reference_rgb": colors,
                "orientation_corner_indices_tl_tr_bl_br": corners,
                "jpeg_center_tolerance": 12,
                "files": {name: {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
                          for name, data in files.items()}}
    (root / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    assert args.write != args.check, "select exactly one of --write/--check"
    root = Path(__file__).resolve().parent
    if args.write:
        write(root)
    else:
        manifest = json.loads((root / "manifest.json").read_text())
        for name, expected in manifest["files"].items():
            data = (root / name).read_bytes()
            assert len(data) == expected["bytes"] and hashlib.sha256(data).hexdigest() == expected["sha256"], name
        print(f"PASS image fixture integrity: {len(manifest['files'])} original assets (no decode test)")


if __name__ == "__main__":
    main()
