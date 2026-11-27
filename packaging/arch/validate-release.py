#!/usr/bin/env python3
"""Build and inspect an Arch package from an exact commit; never install.

--local-candidate archives a local Git commit for testing only.
--approved-source requires the owner's approved GitHub archive checksum.
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

ROOT = Path(__file__).resolve().parents[2]

def require_commit(commit):
    if not re.fullmatch(r'[0-9a-f]{40}', commit):
        raise ValueError('full immutable 40-character source commit required')

def source_url(commit):
    require_commit(commit)
    return f'https://github.com/onbeskarakterli/Pivra/archive/{commit}.tar.gz'

def archive_local(repo, commit, target):
    require_commit(commit)
    git = ['git', '--no-replace-objects', '-C', str(repo)]
    kind = subprocess.check_output(git + ['cat-file', '-t', commit], text=True).strip()
    if kind != 'commit':
        raise ValueError('local candidate must identify a commit object')
    with target.open('xb') as output:
        subprocess.run(git + ['archive', '--format=tar.gz', f'--prefix=Pivra-{commit}/', commit],
                       stdout=output, check=True)

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

def verify_srcinfo(text, version, archive_source, sha256, collector_sha256):
    data = fields(text)
    expected = {'pkgbase': ['pivra'], 'pkgname': ['pivra'], 'pkgver': [version],
                'pkgrel': ['1'], 'arch': ['x86_64'], 'license': ['GPL-3.0-only'],
                'source': [archive_source, 'collect-licenses.py'],
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


def verify_contents(root, source, version, source_sha256, recipe, config_name):
    metadata = fields((root / '.PKGINFO').read_text())
    for key, value in {'pkgname': ['pivra'], 'pkgver': [version + '-1'], 'arch': ['x86_64'], 'license': ['GPL-3.0-only']}.items():
        if metadata.get(key) != value:
            raise ValueError('package metadata mismatch: ' + key)
    if {p.name for p in (root / 'usr/bin').iterdir()} != {'pivra', 'pivra-thumbnail-factory'}:
        raise ValueError('unexpected package executable inventory')
    if any(p.exists() or p.is_symlink() for p in (root / 'usr/lib/debug', root / 'usr/src/debug')):
        raise ValueError('unexpected packaged debug files')
    binary = root / 'usr/bin/pivra'
    subprocess.run(['python3', str(source / 'packaging/linux/verify-video-binary.py'), str(binary)], check=True)
    if subprocess.check_output([str(binary), '--version'], text=True).strip() != f'Pivra {version}':
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
    if digest(corresponding / f'pivra-{version}.tar.gz') != source_sha256:
        raise ValueError('packaged application source differs from verified archive')
    for name in ('PKGBUILD', config_name, 'collect-licenses.py'):
        if digest(corresponding / name) != digest(recipe / name):
            raise ValueError('packaged build instructions differ: ' + name)
    for original in (source / 'native/video').iterdir():
        if original.is_file() and digest(corresponding / 'video-adapter' / original.name) != digest(original):
            raise ValueError('modified adapter source/notice missing or changed: ' + original.name)
    for name in ('COPYING.LGPL-2', 'COPYING.LGPL-2.1'):
        if digest(root / 'usr/share/licenses/pivra/video-adapter' / name) != digest(source / 'native/video' / name):
            raise ValueError('packaged LGPL text differs')
    if digest(root / 'usr/share/licenses/pivra/LICENSE') != digest(source / 'LICENSE'):
        raise ValueError('packaged GPL text differs')
    for name in (f'PIVRA_V{version}_RELEASE_NOTES.md', f'PIVRA_V{version}_RC_VALIDATION.md'):
        if digest(docs / name) != digest(source / 'docs/releases' / name):
            raise ValueError('packaged release document differs: ' + name)
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
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--local-candidate', metavar='COMMIT', help='Archive this exact local Git commit for testing')
    mode.add_argument('--approved-source', metavar='COMMIT', help='Owner-approved GitHub archive commit')
    parser.add_argument('--version', help='Expected X.Y.Z; defaults to the checkout Cargo.toml version')
    parser.add_argument('--sha256', help='Required only for the owner-approved GitHub archive')
    args = parser.parse_args()
    version = args.version or tomllib.loads((ROOT / 'Cargo.toml').read_text())['package']['version']
    if not re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+', version):
        parser.error('expected a numeric X.Y.Z version')
    commit = args.local_candidate or args.approved_source
    require_commit(commit)
    if args.approved_source and not re.fullmatch(r'[0-9a-f]{64}', args.sha256 or ''):
        parser.error('--approved-source requires a 64-character --sha256')
    if args.local_candidate and args.sha256 is not None:
        parser.error('--local-candidate computes its own archive checksum')
    if os.geteuid() == 0:
        parser.error('makepkg must run as an ordinary user')
    required = ['makepkg', 'pacman', 'bsdtar', 'readelf', 'desktop-file-validate']
    if args.local_candidate:
        required.append('git')
    missing = [p for p in required if not shutil.which(p)]
    if missing:
        parser.exit(77, 'NOT RUN: native Arch tools missing: ' + ', '.join(missing) + '\n')
    work = Path(tempfile.mkdtemp(prefix=f'pivra-{version}-package-'))
    print('Retained validation workspace:', work, flush=True)
    archive = work / f'pivra-{version}.tar.gz'
    if args.local_candidate:
        archive_local(ROOT, commit, archive)
        config_name = 'candidate-source.conf'
        url = ''
    else:
        download(commit, archive)
        config_name = 'release-source.conf'
        url = source_url(commit)
    sha256 = digest(archive)
    if args.approved_source and sha256 != args.sha256:
        raise ValueError('downloaded archive checksum differs; no build attempted')
    print(f'SOURCE {config_name} commit={commit} sha256={sha256}', flush=True)
    source = extract_source(archive, work / 'source', commit)
    manifest = tomllib.loads((source / 'Cargo.toml').read_text())['package']
    lock = tomllib.loads((source / 'Cargo.lock').read_text())
    locked_versions = [p['version'] for p in lock['package'] if p['name'] == 'pivra' and 'source' not in p]
    if manifest['version'] != version or locked_versions != [version] or manifest['license'] != 'GPL-3.0-only':
        raise ValueError('source Cargo.toml/Cargo.lock version or license mismatch')
    recipe = work / 'recipe'; recipe.mkdir()
    for name in ('PKGBUILD', 'collect-licenses.py'):
        shutil.copyfile(source / 'packaging/arch' / name, recipe / name)
    shutil.copyfile(archive, recipe / archive.name)
    description = ('Local candidate only. Git archive checksum is not a GitHub release checksum.'
                   if args.local_candidate else 'Explicit caller source approval; not publication authorization.')
    (recipe / config_name).write_text(
        f"# {description}\nrelease_commit='{commit}'\nrelease_url='{url}'\nrelease_sha256='{sha256}'\n")
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
    archive_source = f'{archive.name}::{url}' if url else archive.name
    verify_srcinfo(info, version, archive_source, sha256, digest(recipe / 'collect-licenses.py'))
    (recipe / '.SRCINFO').write_text(info)
    planned = run(makepkg + ['--packagelist'], recipe, env, work / 'packagelist.log').splitlines()
    packages = [Path(p).resolve() for p in planned if p.strip()]
    if len(packages) != 1 or not packages[0].is_relative_to(work) or not packages[0].name.startswith(f'pivra-{version}-1-x86_64.pkg.tar.'):
        raise ValueError('expected one private pivra package, never pivra-debug or external output')
    run(makepkg + ['--cleanbuild', '--force', '--check'], recipe, env, work / 'build.log')
    package = packages[0]
    listing = run(['bsdtar', '-tf', str(package)], recipe, env, work / 'contents.log')
    if any(PurePosixPath(p).is_absolute() or '..' in PurePosixPath(p).parts for p in listing.splitlines()):
        raise ValueError('unsafe package member path')
    unpacked = work / 'unpacked'; unpacked.mkdir()
    run(['bsdtar', '-xf', str(package), '--no-same-owner', '--no-same-permissions', '-C', str(unpacked)], recipe, env, work / 'extract.log')
    verify_contents(unpacked, source, version, sha256, recipe, config_name)
    subprocess.run(['desktop-file-validate', str(unpacked / 'usr/share/applications/io.github.onbeskarakterli.Pivra.desktop')], check=True)
    print('PASS native makepkg source/.SRCINFO/clean build/contents; package SHA256:', digest(package))
    print('Package:', package)
    print('NOT RUN: installation, main/tag/GitHub/AUR publication. Outputs:', work)

if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, RuntimeError, subprocess.SubprocessError) as error:
        raise SystemExit('FAIL native release validation: ' + str(error))
