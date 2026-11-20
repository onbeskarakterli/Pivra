"""Explicit private GTK experiment; never installs libraries or alters Pivra packaging."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import urllib.request
from lifecycle import ROOT
from gtk_egl_source_test import SOURCE_SHA256, PATCHED_SHA256, PATCH

URL = 'https://codeload.github.com/GNOME/gtk/tar.gz/refs/tags/4.24.1'
ARCHIVE_SHA256 = '6b03cc3eac17e6d7b252c7ac96559e58aa7c990874ca47da696d81d0add7521d'
OPTIONS = ['--wrap-mode=nofallback', '-Dwayland-backend=true', '-Dx11-backend=true',
           '-Dvulkan=disabled', '-Dmedia-gstreamer=auto', '-Daccesskit=disabled',
           '-Dprint-cups=disabled', '-Dintrospection=disabled', '-Ddocumentation=false',
           '-Dbuild-tests=false', '-Dbuild-testsuite=false', '-Dbuild-examples=false', '-Dbuild-demos=false']


def logged(command, path):
    print('Isolated build:', ' '.join(map(str, command)), '; log:', path, flush=True)
    with path.open('w') as log:
        result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT)
    if result.returncode:
        print('\n'.join(path.read_text(errors='replace').splitlines()[-20:]), flush=True)
        raise RuntimeError('isolated build failed, exit='+str(result.returncode))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--workspace', type=Path, required=True)
    parser.add_argument('--video', type=Path, required=True)
    parser.add_argument('--manual', action='store_true')
    parser.add_argument('--jobs', type=int, default=4)
    args = parser.parse_args()
    for key in ('LD_PRELOAD', 'GTK_MEDIA', 'PIVRA_VIDEO_BACKEND', 'PIVRA_VIDEO_TEST_MODULE_DIR'):
        if os.environ.get(key):
            parser.error('unset '+key+' before isolated GPU preparation')
    if not 1 <= args.jobs <= 16:
        parser.error('--jobs must be 1..16')
    if not args.video.is_file():
        parser.error('--video must be an existing file (read-only input)')
    work = args.workspace.resolve()
    if work.is_relative_to(ROOT) or work==Path('/') or any(work.is_relative_to(p) for p in map(Path, ['/usr', '/lib', '/lib64', '/etc', '/bin', '/sbin'])):
        parser.error('use a dedicated private workspace outside the application repository/system library directories')
    work.mkdir(parents=True, exist_ok=True)
    marker = work/'.pivra-gtk-egl-workspace'
    if not marker.exists() and any(work.iterdir()):
        parser.error('workspace is nonempty and unmanaged; choose a new directory')
    marker.write_text(json.dumps(dict(archive_sha256=ARCHIVE_SHA256, patch_sha256=hashlib.sha256(PATCH.read_bytes()).hexdigest()))+'\n')
    if not all(shutil.which(tool) for tool in ['meson', 'ninja', 'git', 'pkg-config', 'cc', 'cargo']):
        parser.exit(77, 'NOT RUN: meson >=1.8, ninja, git, pkg-config, C compiler and Cargo are required; no automatic system installation\n')
    archive = work/'gtk-4.24.1.tar.gz'
    if not archive.exists():
        with urllib.request.urlopen(URL, timeout=60) as response:
            archive.write_bytes(response.read())
    if hashlib.sha256(archive.read_bytes()).hexdigest() != ARCHIVE_SHA256:
        parser.error('archive SHA-256 mismatch; source not used')
    clean = work/'gtk-4.24.1'
    if not clean.exists():
        with tarfile.open(archive) as tar:
            tar.extractall(work, filter='data')
    if hashlib.sha256((clean/'gdk/gdkdisplay.c').read_bytes()).hexdigest() != SOURCE_SHA256:
        parser.error('baseline GTK source has been modified')
    patched = work/'gtk-patched'
    if not patched.exists():
        shutil.copytree(clean, patched)
        subprocess.run(['git', '-C', str(patched), 'apply', '--check', str(PATCH)], check=True)
        subprocess.run(['git', '-C', str(patched), 'apply', str(PATCH)], check=True)
    if hashlib.sha256((patched/'gdk/gdkdisplay.c').read_bytes()).hexdigest() != PATCHED_SHA256:
        parser.error('patched source does not match the reviewed EGL correction; use a new workspace')
    # The runtime verifier also rejects a mismatched/unreviewed patched source.
    builds = []
    for label, source in [('baseline', clean), ('patched', patched)]:
        build = work/(label+'-build')
        setup = ['meson', 'setup']
        if (build/'meson-info/meson-info.json').exists():
            setup.append('--reconfigure')
        logged([*setup, str(build), str(source), '--prefix='+str(work/(label+'-unused-prefix')), *OPTIONS], work/(label+'-setup.log'))
        targets = json.loads(subprocess.check_output(['meson', 'introspect', '--targets', str(build)], text=True))
        libs = [t for t in targets if t['name'] in ('gtk-4', 'media-gstreamer') and t['type'] in ('shared library', 'shared module')]
        if not any(t['name']=='gtk-4' for t in libs):
            raise RuntimeError('missing real GTK shared library target')
        files = [str(Path(t['filename'][0]).relative_to(build)) for t in libs]
        logged(['ninja', '-C', str(build), '-j'+str(args.jobs), *files], work/(label+'-compile.log'))
        builds.append(build)
    command = [sys.executable, '-B', str(ROOT/'tests/video_gpu/gtk_egl_isolated.py'),
               '--baseline-build', str(builds[0]), '--patched-build', str(builds[1]), '--video', str(args.video.resolve())]
    if args.manual:
        command.append('--manual')
    return subprocess.run(command, cwd=ROOT).returncode


if __name__ == '__main__':
    raise SystemExit(main())
