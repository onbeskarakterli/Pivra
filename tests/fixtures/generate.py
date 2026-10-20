#!/usr/bin/env python3
"""Generate tiny original fixtures, or check exact bytes and metadata.

No external media, encoders or network. No project license is assigned here.
Use --write intentionally; normal validation is read-only --check.
"""

import argparse
import binascii
import hashlib
import json
from pathlib import Path
import struct
import zlib


def chunk(kind, data):
    return (struct.pack(">I", len(data)) + kind + data
            + struct.pack(">I", binascii.crc32(kind + data) & 0xFFFFFFFF))


def fixtures():
    # Two rows, filter 0; RGBA red/green/blue/transparent-white, no metadata.
    raw = bytes([0, 255, 0, 0, 255, 0, 255, 0, 255,
                 0, 0, 0, 255, 255, 255, 255, 255, 0])
    # Fixed uncompressed DEFLATE block: stable across zlib encoder versions.
    stream = (b"\x78\x01\x01" + struct.pack("<HH", len(raw), len(raw) ^ 0xFFFF)
              + raw + struct.pack(">I", zlib.adler32(raw) & 0xFFFFFFFF))
    png = (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 2, 2, 8, 6, 0, 0, 0))
           + chunk(b"IDAT", stream) + chunk(b"IEND", b""))
    return {
        "pixels-2x2.png": png,
        "model-video.txt": b"PIVRA model-only video placeholder. NOT playable media; no codec or decoder claim.\n",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--check", action="store_true")
    action.add_argument("--write", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    files = fixtures()
    metadata = {
        "source": "Original hard-coded values in generate.py; no downloaded third-party assets",
        "license_status": "Repository license undecided; no license grant or selection introduced",
        "files": {name: {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
                  for name, data in files.items()},
    }
    files["manifest.json"] = (json.dumps(metadata, ensure_ascii=False, indent=2) + "\n").encode()
    for name, expected in files.items():
        path = root / name
        if args.write:
            path.write_bytes(expected)
        else:
            if path.read_bytes() != expected:
                raise SystemExit(f"FAIL fixture differs: {name}; inspect before regenerating")
            print(f"PASS deterministic fixture: {name} ({len(expected)} bytes)")


if __name__ == "__main__":
    main()
