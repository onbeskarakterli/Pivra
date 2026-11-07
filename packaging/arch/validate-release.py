#!/usr/bin/env python3
"""Native Arch validation of an explicitly approved immutable source; never install.

No moving-ref inputs, dependency installation, repository writes or publication.
Passing --approved-source is the caller's explicit source-approval attestation.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import shlex
import subprocess
import tarfile
import tempfile
import tomllib
import urllib.request

VERSION = '0.2.5'

def source_url(commit):
    if not re.fullmatch(r'[0-9a-f]{40}', commit):
        raise ValueError('full immutable 40-character source commit required')
    return f'https://github.com/onbeskarakterli/Pivra/archive/{commit}.tar.gz'

def digest(path):
    with path.open('rb') as file:
        return hashlib.file_digest(file, 'sha256').hexdigest()

def download(commit, target):
    # Inherited HTTPS proxy and CA trust, no credential/policy bypass.
    with urllib.request.urlopen(source_url(commit), timeout=30) as response, target.open('xb') as file:
        size = 0
        while data := response.read(1024 * 1024):
            size += len(data)
            if size > 64 * 1024 * 1024:
                raise ValueError('source archive exceeds 64 MiB')
            file.write(data)
        return response.url, size

def extract_source(archive, destination, commit):
    prefix = 'Pivra-' + commit
    with tarfile.open(archive) as tar:
        members = tar.getmembers()
        if len(members) > 20000 or sum(m.size for m in members) > 512 * 1024 * 1024:
            raise ValueError('source extraction bounds exceeded')
        for member in members:
            path = PurePosixPath(member.name)
            if path.is_absolute() or '..' in path.parts or not path.parts or path.parts[0] != prefix:
                raise ValueError('unsafe source archive path')
            if not (member.isdir() or member.isfile()):
                raise ValueError('source links/special files not permitted')
        for member in members:
            target = destination.joinpath(*PurePosixPath(member.name).parts)
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                with tar.extractfile(member) as data, target.open('xb') as output:
                    shutil.copyfileobj(data, output)
                target.chmod(0o755 if member.mode & 0o111 else 0o644)
    return destination / prefix

def fields(text):
    result = {}
    for line in text.splitlines():
        if ' = ' in line and not line.lstrip().startswith('#'):
            key, value = line.strip().split(' = ', 1)
            result.setdefault(key, []).append(value)
    return result

def verify_srcinfo(text, commit, sha256, collector_sha256):
    data = fields(text)
    expected = {'pkgbase': ['pivra'], 'pkgname': ['pivra'], 'pkgver': [VERSION],
                'pkgrel': ['1'], 'arch': ['x86_64'], 'license': ['GPL-3.0-only'],
                'source': [f'pivra-{VERSION}.tar.gz::{source_url(commit)}', 'collect-licenses.py'],
                'sha256sums': [sha256, collector_sha256]}
    for key, value in expected.items():
        if data.get(key) != value:
            raise ValueError(f'.SRCINFO {key} mismatch: {data.get(key)!r}')
    if '!debug' not in data.get('options', []) or 'debug' in data.get('options', []):
        raise ValueError('.SRCINFO must disable the debug split package')
    for dependency in ('gtk4>=4.16', 'gnome-desktop-4', 'glycin', 'gstreamer>=1.24',
                       'gst-plugins-base-libs>=1.24', 'gst-plugins-good', 'gst-plugins-bad-libs', 'gst-libav'):
        if dependency not in data.get('depends', []):
            raise ValueError('.SRCINFO missing dependency: ' + dependency)

def run(args, cwd, env, log):
    print('RUN:', shlex.join(args), '; log:', log, flush=True)
    if '--cleanbuild' in args:
        # Keep long native builds observable without buffering their whole output.
        with log.open('w') as output:
            result = subprocess.run(args, cwd=cwd, env=env, text=True, stdout=output, stderr=subprocess.STDOUT)
        stdout = ''
    else:
        result = subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True)
        stdout = result.stdout
        log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f'FAIL {args[0]} {args[1:]} exit={result.returncode}; log={log}')
    return stdout


def verify_contents(root, source, source_sha256):
    metadata = fields((root / '.PKGINFO').read_text())
    for key, value in {'pkgname': ['pivra'], 'pkgver': [VERSION + '-1'], 'arch': ['x86_64'], 'license': ['GPL-3.0-only']}.items():
        if metadata.get(key) != value:
            raise ValueError('package metadata mismatch: ' + key)
    binary = root / 'usr/bin/pivra'
    subprocess.run(['python3', str(source / 'packaging/linux/verify-video-binary.py'), str(binary)], check=True)
    if subprocess.check_output([str(binary), '--version'], text=True).strip() != f'Pivra {VERSION}':
        raise ValueError('packaged binary version mismatch')
    helper = root / 'usr/bin/pivra-thumbnail-factory'
    if not helper.is_file() or not os.access(helper, os.X_OK):
        raise ValueError('packaged GNOME helper missing/not executable')
    dynamic = subprocess.check_output(['readelf', '-d', str(helper)], text=True)
    if 'libgnome-desktop-4.so.' not in dynamic or re.search(r'\((?:RPATH|RUNPATH)\)', dynamic):
        raise ValueError('GNOME helper library/loader-path contract mismatch')
    for relative in ('applications/io.github.onbeskarakterli.Pivra.desktop',
                     'icons/hicolor/scalable/apps/io.github.onbeskarakterli.Pivra.svg',
                     'icons/hicolor/symbolic/apps/io.github.onbeskarakterli.Pivra-symbolic.svg'):
        original = source / 'data' / relative.removeprefix('applications/') if relative.startswith('applications/') else source / 'data' / relative
        if digest(root / 'usr/share' / relative) != digest(original):
            raise ValueError('desktop/icon differs from approved source: ' + relative)
    docs = root / 'usr/share/doc/pivra'
    corresponding = docs / 'corresponding-source'
    if digest(corresponding / f'pivra-{VERSION}.tar.gz') != source_sha256:
        raise ValueError('packaged application source differs from approved archive')
    for original in (source / 'native/video').iterdir():
        if original.is_file() and digest(corresponding / 'video-adapter' / original.name) != digest(original):
            raise ValueError('modified adapter source/notice missing or changed: ' + original.name)
    for name in ('COPYING.LGPL-2', 'COPYING.LGPL-2.1'):
        if digest(root / 'usr/share/licenses/pivra/video-adapter' / name) != digest(source / 'native/video' / name):
            raise ValueError('packaged LGPL text differs')
    if digest(root / 'usr/share/licenses/pivra/LICENSE') != digest(source / 'LICENSE'):
        raise ValueError('packaged GPL text differs')
    if digest(docs / f'PIVRA_V{VERSION}_RELEASE_NOTES.md') != digest(source / f'docs/releases/PIVRA_V{VERSION}_RELEASE_NOTES.md'):
        raise ValueError('packaged release notes differ')
    inventory = json.loads((root / 'usr/share/licenses/pivra/third-party/inventory.json').read_text())
    lock = tomllib.loads((source / 'Cargo.lock').read_text())
    expected = {(p['name'], p['version']) for p in lock['package'] if p.get('source', '').startswith('registry+')}
    if {(p['name'], p['version']) for p in inventory} != expected:
        raise ValueError('locked corresponding-source inventory incomplete')
    for entry in inventory:
        name = entry['source_archive']
        if Path(name).name != name or digest(corresponding / 'crates' / name) != entry['sha256']:
            raise ValueError('packaged crate source checksum mismatch')
        stem = entry['name'] + '-' + entry['version']
        with tarfile.open(corresponding / 'crates' / name) as crate:
            for notice in entry['notice_files']:
                path = PurePosixPath(notice)
                if path.is_absolute() or '..' in path.parts:
                    raise ValueError('unsafe notice path')
                packaged = root / 'usr/share/licenses/pivra/third-party' / stem / notice
                original = crate.extractfile(stem + '/' + notice)
                if original is None or packaged.read_bytes() != original.read(4 * 1024 * 1024 + 1):
                    raise ValueError('packaged crate notice missing/changed: ' + notice)
    print(f'PASS package ELF/version/helper/LGPL/modified sources and {len(inventory)} locked crate sources')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--approved-source', required=True, help='Owner-approved immutable SHA; do not use before approval')
    parser.add_argument('--sha256', required=True, help='Owner-approved real archive SHA-256')
    args = parser.parse_args()
    source_url(args.approved_source)
    if not re.fullmatch(r'[0-9a-f]{64}', args.sha256):
        parser.error('64-character approved SHA-256 required')
    if os.geteuid() == 0:
        parser.error('makepkg must run as an ordinary user')
    missing = [p for p in ('makepkg', 'pacman', 'bsdtar', 'readelf', 'desktop-file-validate') if not shutil.which(p)]
    if missing:
        parser.exit(77, 'NOT RUN: native Arch tools missing: ' + ', '.join(missing) + '\n')
    work = Path(tempfile.mkdtemp(prefix='pivra-025-package-'))
    print('Retained validation workspace:', work, flush=True)
    archive = work / f'pivra-{VERSION}.tar.gz'
    download(args.approved_source, archive)
    if digest(archive) != args.sha256:
        raise ValueError('downloaded archive checksum differs; no build attempted')
    source = extract_source(archive, work / 'source', args.approved_source)
    manifest = tomllib.loads((source / 'Cargo.toml').read_text())['package']
    if manifest['version'] != VERSION or manifest['license'] != 'GPL-3.0-only':
        raise ValueError('approved source version/license mismatch')
    recipe = work / 'recipe'; recipe.mkdir()
    for name in ('PKGBUILD', 'collect-licenses.py'):
        shutil.copyfile(source / 'packaging/arch' / name, recipe / name)
    shutil.copyfile(archive, recipe / archive.name)
    (recipe / 'release-source.conf').write_text(
        '# Explicit caller source approval; not publication authorization.\n'
        f"release_commit='{args.approved_source}'\nrelease_url='{source_url(args.approved_source)}'\nrelease_sha256='{args.sha256}'\n")
    env = dict(os.environ, PKGDEST=str(work / 'packages'), SRCDEST=str(recipe),
               BUILDDIR=str(work / 'build'), LOGDEST=str(work / 'logs'))
    for name in ('packages', 'build', 'logs'): (work / name).mkdir()
    # Preserve system compiler policy, but isolate output paths and user config.
    config = work / 'makepkg.conf'
    config.write_text('source /etc/makepkg.conf\n' + ''.join(
        key + '=' + shlex.quote(env[key]) + '\n' for key in ('PKGDEST', 'SRCDEST', 'BUILDDIR', 'LOGDEST')))
    env.update(XDG_CONFIG_HOME=str(work / 'config'), XDG_CACHE_HOME=str(work / 'cache'), XDG_DATA_HOME=str(work / 'data'))
    for name in ('makepkg.conf', 'pacman/makepkg.conf'):
        path = work / 'config' / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_text('# isolated validation\n')
    makepkg = ['makepkg', '--config', str(config)]
    run(makepkg + ['--verifysource', '--force'], recipe, env, work / 'verifysource.log')
    info = run(makepkg + ['--printsrcinfo'], recipe, env, work / 'srcinfo.log')
    verify_srcinfo(info, args.approved_source, args.sha256, digest(recipe / 'collect-licenses.py'))
    (recipe / '.SRCINFO').write_text(info)
    planned = run(makepkg + ['--packagelist'], recipe, env, work / 'packagelist.log').splitlines()
    packages = [Path(p).resolve() for p in planned if p.strip()]
    if len(packages) != 1 or not packages[0].is_relative_to(work) or not packages[0].name.startswith(f'pivra-{VERSION}-1-x86_64.pkg.tar.'):
        raise ValueError('expected one private pivra package, never pivra-debug or external output')
    run(makepkg + ['--cleanbuild', '--force', '--check'], recipe, env, work / 'build.log')
    package = packages[0]
    listing = run(['bsdtar', '-tf', str(package)], recipe, env, work / 'contents.log')
    if any(PurePosixPath(p).is_absolute() or '..' in PurePosixPath(p).parts for p in listing.splitlines()):
        raise ValueError('unsafe package member path')
    unpacked = work / 'unpacked'; unpacked.mkdir()
    run(['bsdtar', '-xf', str(package), '--no-same-owner', '--no-same-permissions', '-C', str(unpacked)], recipe, env, work / 'extract.log')
    verify_contents(unpacked, source, args.sha256)
    subprocess.run(['desktop-file-validate', str(unpacked / 'usr/share/applications/io.github.onbeskarakterli.Pivra.desktop')], check=True)
    print('PASS native makepkg source/.SRCINFO/clean build/contents; package SHA256:', digest(package))
    print('NOT RUN: installation, main/tag/GitHub/AUR publication. Outputs:', work)

if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, RuntimeError, subprocess.SubprocessError) as error:
        raise SystemExit('FAIL native release validation: ' + str(error))
