#!/usr/bin/env python3
"""Validate a sandbox-produced binary PPM before any host-side image consumer."""
import os
import sys

MAX_PIXELS = 64_000_000
MAX_BYTES = 256 * 1024 * 1024


def check(path):
    with open(path, "rb") as image:
        magic = image.readline(16)
        if magic != b"P6\n":
            raise ValueError("invalid P6 magic")
        dimensions = image.readline(48)
        parts = dimensions.split()
        if len(parts) != 2 or any(not p.isascii() or not p.isdigit() for p in parts):
            raise ValueError("invalid dimensions")
        width, height = map(int, parts)
        if not 0 < width <= 16384 or not 0 < height <= 16384:
            raise ValueError("invalid image edges")
        if width * height > MAX_PIXELS:
            raise ValueError("image exceeds pixel ceiling")
        if image.readline(8) != b"255\n":
            raise ValueError("invalid maxval")
        offset = image.tell()
        expected = width * height * 3
        if expected > MAX_BYTES or os.fstat(image.fileno()).st_size != offset + expected:
            raise ValueError("incorrect raster length")
    print(f"PASS PPM transfer validation {width}x{height}, {expected} RGB bytes")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit("Usage: validate_raw_ppm.py FILE")
    try:
        check(sys.argv[1])
    except (OSError, ValueError) as error:
        sys.exit(f"FAIL PPM transfer: {error}")
