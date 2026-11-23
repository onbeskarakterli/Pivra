"""Test the exact clean/patched GTK 4.24.1 selection function, not GPU rendering."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
from lifecycle import ROOT

SOURCE_SHA256 = '1dfc7c6da4cff4883e7835e430501426eae196229eceed6e6a3de0e950f72f67'
PATCHED_SHA256 = 'bad9c80634ab2112c9d361708a78747f01058c6e7c6cf96a6ad966353b0fba10'
PATCH = ROOT/'docs/performance/gtk-4.24.1-modern-egl-config-review.patch'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    args = parser.parse_args()
    source = (args.source/'gdk/gdkdisplay.c').read_bytes()
    if hashlib.sha256(source).hexdigest() != SOURCE_SHA256:
        parser.error('source is not the pinned clean GTK 4.24.1 gdkdisplay.c')
    out = Path(tempfile.mkdtemp(prefix='pivra-gtk-egl-source-'))
    print('Source unit report:', out, flush=True)
    patched = out/'patched/gdk/gdkdisplay.c'
    patched.parent.mkdir(parents=True)
    patched.write_bytes(source)
    subprocess.run(['git', '-C', str(out/'patched'), 'apply', '--check', str(PATCH)], check=True)
    subprocess.run(['git', '-C', str(out/'patched'), 'apply', str(PATCH)], check=True)
    modules = ['glib-2.0', 'epoxy']
    flags = lambda key: shlex.split(subprocess.check_output(['pkg-config', key, *modules], text=True))
    rows = []
    for label, data in [('baseline', source), ('patched', patched.read_bytes())]:
        folder = out/label
        folder.mkdir(exist_ok=True)
        text = data.decode()
        selection = text[text.index('#define MAX_EGL_ATTRS 30'):text.index('#undef MAX_EGL_ATTRS')]
        (folder/'gtk-selection.h').write_text(selection)
        binary = folder/'source-test'
        subprocess.run([*shlex.split(os.environ.get('CC', 'cc')), '-O2', '-Wall', '-Wextra', '-Werror',
                        f'-DPATCHED_VARIANT={int(label=="patched")}', '-I'+str(folder),
                        *shlex.split(os.environ.get('CPPFLAGS', '')), *flags('--cflags'),
                        str(ROOT/'tests/video_gpu/gtk_egl_selection.c'), '-o', str(binary),
                        *shlex.split(os.environ.get('LDFLAGS', '')), *flags('--libs')], check=True)
        result = subprocess.run([binary], capture_output=True, text=True, timeout=10)
        (out/(label+'.log')).write_text(result.stdout+result.stderr)
        print(result.stdout+result.stderr, end='', flush=True)
        if result.returncode != 0:
            raise RuntimeError(label+' source selection failed: '+str(result.returncode))
        rows.append(dict(variant=label, exit=result.returncode, cases=8,
                         executable_sha256=hashlib.sha256(binary.read_bytes()).hexdigest()))
    (out/'report.json').write_text(json.dumps(dict(scope='GTK source unit, NOT a driver/GPU/native test',
        source_sha256=SOURCE_SHA256, patch_sha256=hashlib.sha256(PATCH.read_bytes()).hexdigest(), results=rows), indent=2)+'\n')


if __name__ == '__main__':
    main()
