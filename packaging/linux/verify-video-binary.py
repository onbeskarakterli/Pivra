#!/usr/bin/env python3
"""Check the actual ELF linkage and embedded GPU adapter; no execution or ldd."""
import argparse
from pathlib import Path
import re
import subprocess

REQUIRED = {'libgtk-4.so.1', 'libgstreamer-1.0.so.0', 'libgstgl-1.0.so.0',
            'libgstvideo-1.0.so.0', 'libgstpbutils-1.0.so.0'}

def verify(binary):
    data = binary.read_bytes()
    for marker in (b'PIVRA_GPU_BACKEND automatic type=', b'PIVRA_GPU_CONTEXT renderer='):
        if marker not in data:
            raise ValueError('GPU backend missing from actual executable: ' + marker.decode())
    if b'/out/pivra-thumbnail-factory' in data:
        raise ValueError('temporary OUT_DIR thumbnail helper embedded')
    dynamic = subprocess.run(['readelf', '-d', str(binary)], check=True, text=True, capture_output=True).stdout
    needed = set(re.findall(r'\(NEEDED\).*?\[(.*?)\]', dynamic))
    if REQUIRED - needed:
        raise ValueError('missing dynamic native libraries: ' + ', '.join(sorted(REQUIRED - needed)))
    # Runtime libraries must be normal loader-resolved SONAMEs, not local SDK paths.
    if any('/' in name for name in needed) or re.search(r'\((?:RUNPATH|RPATH)\)', dynamic):
        raise ValueError('absolute native library dependency or embedded RPATH/RUNPATH')
    return sorted(needed)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    args = parser.parse_args()
    try:
        print('PASS GPU ELF contract:', ', '.join(verify(args.binary)))
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'FAIL GPU ELF contract: {error}\n')

if __name__ == '__main__':
    main()
