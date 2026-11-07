#!/usr/bin/env python3
"""Collect locked third-party sources/notices and a rebuildable application snapshot.

Only explicit source directories are included, never .git, target or machine-local
packaging configuration. The snapshot is NOT an immutable approved release source.
"""
import argparse
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import tarfile
import tomllib
import sys
sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[2]
SOURCE_PATHS = ('Cargo.toml', 'Cargo.lock', 'build.rs', 'Makefile', 'LICENSE', 'README.md',
                'src', 'native', 'raw', 'data', 'tests', 'examples', 'scripts', 'docs',
                'packaging/linux', 'packaging/arch/collect-licenses.py')

def snapshot(root, destination):
    files = []
    for name in SOURCE_PATHS:
        path = root / name
        if not path.exists():
            raise ValueError('missing corresponding source: ' + name)
        for item in (path.rglob('*') if path.is_dir() else [path]):
            if '__pycache__' in item.parts or item.suffix == '.pyc':
                continue
            if item.is_symlink():
                raise ValueError('unexpected source symlink: ' + str(item))
            if item.is_file():
                files.append(item)
    inventory = {}
    with destination.open('wb') as output, gzip.GzipFile(filename='', fileobj=output, mode='wb', mtime=0) as zipped:
        with tarfile.open(mode='w', fileobj=zipped) as archive:
            for item in sorted(files):
                name = item.relative_to(root).as_posix()
                inventory[name] = hashlib.sha256(item.read_bytes()).hexdigest()
                info = archive.gettarinfo(str(item), arcname='Pivra-source/' + name)
                info.uid = info.gid = info.mtime = 0
                info.uname = info.gname = ''
                info.mode = 0o755 if item.stat().st_mode & 0o111 else 0o644
                with item.open('rb') as content:
                    archive.addfile(info, content)
    return inventory

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--metadata', type=Path, required=True)
    p.add_argument('--cargo-home', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=False)  # Never merge stale evidence.
    spec = importlib.util.spec_from_file_location('licenses', ROOT / 'packaging/arch/collect-licenses.py')
    collector = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(collector)
    count = collector.collect(json.loads(a.metadata.read_text()), tomllib.loads((ROOT / 'Cargo.lock').read_text()),
                              a.cargo_home, a.output / 'notices', a.output / 'crates')
    inventory = snapshot(ROOT, a.output / 'Pivra-source.tar.gz')
    (a.output / 'source-manifest.json').write_text(json.dumps(inventory, indent=2) + '\n')
    print(f'PASS distribution sources: {count} locked crates, {len(inventory)} application files; not release-source approval')

if __name__ == '__main__':
    main()
