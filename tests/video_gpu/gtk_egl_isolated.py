"""Explicit private clean/patched GTK comparison; no installation or production integration."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
from lifecycle import ROOT, run, session
from gtk_egl_source_test import SOURCE_SHA256, PATCHED_SHA256, PATCH
from normal_playback import GPU_MARKERS, PRODUCT_MARKERS, check_normal


def provenance(build, expected):
    build = build.resolve(strict=True)
    info = json.loads((build/'meson-info/meson-info.json').read_text())
    source = Path(info['directories']['source'])
    if hashlib.sha256((source/'gdk/gdkdisplay.c').read_bytes()).hexdigest() != expected:
        raise RuntimeError('GTK source mismatch: '+str(source))
    library = (build/'gtk/libgtk-4.so.1').resolve(strict=True)
    if not library.is_relative_to(build) or b'video-bt1886 GPU CICP op' in library.read_bytes():
        raise RuntimeError('not a private EGL-only GTK build: '+str(library))
    return dict(source=str(source), build=str(build), library=str(library),
                sha256=hashlib.sha256(library.read_bytes()).hexdigest())


def require_loader(binary, library, env):
    output = subprocess.check_output(['ldd', str(binary)], env=env, text=True)
    paths = [Path(line.split('=>', 1)[1].split()[0]).resolve() for line in output.splitlines() if 'libgtk-4.so' in line and '=>' in line]
    if paths != [Path(library)]:
        raise RuntimeError('wrong runtime GTK resolution: '+str(paths))
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline-build', type=Path, required=True)
    parser.add_argument('--patched-build', type=Path, required=True)
    parser.add_argument('--video', type=Path)
    parser.add_argument('--manual', action='store_true')
    parser.add_argument('--session', choices=['native', 'sway'], default='native')
    parser.add_argument('--software', action='store_true')
    args = parser.parse_args()
    if args.session != 'native' and not args.software:
        parser.error('headless tests require --software')
    if args.manual and (args.software or not args.video):
        parser.error('manual acceptance requires a native session and actual --video')
    for key in ('LD_PRELOAD', 'GTK_MEDIA', 'PIVRA_VIDEO_BACKEND', 'PIVRA_VIDEO_TEST_MODULE_DIR'):
        if os.environ.get(key):
            parser.error('unset '+key+' for genuine isolated GPU testing')
    if not args.software and os.environ.get('PIVRA_GPU_ALLOW_SOFTWARE_TEST') == '1':
        parser.error('software permission requires --software')
    out = Path(tempfile.mkdtemp(prefix='pivra-gtk-egl-isolated-'))
    print('Isolated runtime report:', out, flush=True)
    variants = {'baseline': provenance(args.baseline_build, SOURCE_SHA256),
                'patched': provenance(args.patched_build, PATCHED_SHA256)}
    clean, patched = (Path(variants[k]['source']) for k in ['baseline', 'patched'])
    for directory in ['gdk', 'gsk', 'gtk', 'modules']:
        for file in (clean/directory).rglob('*'):
            if file.suffix in ('.c', '.h', '.glsl', '.frag', '.vert') and file.is_file():
                relative = file.relative_to(clean)
                if str(relative) != 'gdk/gdkdisplay.c' and file.read_bytes() != (patched/relative).read_bytes():
                    raise RuntimeError('unreviewed GTK color/renderer change: '+str(relative))
    subprocess.run(['cargo', 'build', '--locked', '--bin', 'pivra', '--example', 'video_color_probe', '--example', 'integrated_video_probe'], cwd=ROOT, check=True)
    target = Path(json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--no-deps', '--format-version', '1'], cwd=ROOT))['target_directory'])/'debug'
    ctest = out/'gtk-context'
    flags = lambda key: shlex.split(subprocess.check_output(['pkg-config', key, 'gtk4', 'epoxy'], text=True))
    subprocess.run([*shlex.split(os.environ.get('CC', 'cc')), '-O2', '-Wall', '-Wextra', '-Werror',
                    *shlex.split(os.environ.get('CPPFLAGS', '')), *flags('--cflags'), str(ROOT/'tests/video_gpu/gtk_egl_window.c'),
                    '-o', str(ctest), *shlex.split(os.environ.get('LDFLAGS', '')), *flags('--libs')], check=True)
    if args.video:
        video = args.video.resolve(strict=True); extra = []
    else:
        subprocess.run(['python3', '-B', 'tests/video_gpu/backend/fixtures.py', str(out/'fixtures')], cwd=ROOT, check=True)
        video = out/'fixtures/ramp.mp4'; extra = ['--check-bt1886-ramp']
    rows = execute(args, variants, out, target, ctest, video, extra)
    report = dict(source=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  software_control=args.software, variants=variants, results=rows,
                  patch_sha256=hashlib.sha256(PATCH.read_bytes()).hexdigest(),
                  native_visual_audio_acceptance='PENDING explicit user report; software GL/process success cannot pass physical acceptance',
                  stock_fallback='Existing stock GTK policy untouched; not exercised in EGL-only normal-video comparison',
                  publication='BLOCKED pending native Wayland acceptance; this private GTK is never bundled or installed')
    (out/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    gates = [r for r in rows if r['variant']=='patched']
    return 1 if any(r['status']=='FAIL' for r in gates) else 77 if any(r['status'] in ('NOT RUN', 'BLOCKED') for r in gates) else 0


def execute(args, variants, out, target, ctest, video, extra):
    rows = []
    for backend in ['wayland', 'x11']:
        folder = out/backend; folder.mkdir()
        env = dict(os.environ, GDK_BACKEND=backend, GSK_RENDERER='gl', GTK_A11Y='none', PIVRA_TRACE_VIDEO='1')
        env.pop('GTK_PATH', None); env.pop('GIO_EXTRA_MODULES', None)
        if args.software:
            env.update(LIBGL_ALWAYS_SOFTWARE='1', PIVRA_GPU_ALLOW_SOFTWARE_TEST='1')
        kind = ('sway' if backend=='wayland' else 'x11') if args.software else 'native'
        with session(kind, env, folder, floating=True) as active:
            for label, meta in variants.items():
                child = dict(active, LD_LIBRARY_PATH=str(Path(meta['library']).parent)+':'+os.environ.get('LD_LIBRARY_PATH', ''),
                             XDG_CONFIG_HOME=str(folder/label/'config'), XDG_CACHE_HOME=str(folder/label/'cache'), XDG_DATA_HOME=str(folder/label/'data'))
                # Never load an incompatible system GTK media plugin.
                media = Path(meta['build'])/'modules/media/libmedia-gstreamer.so'
                meta['stock_module_built'] = media.exists()
                if media.exists():
                    module = folder/label/'gtk-modules/4.0.0/media'; module.mkdir(parents=True)
                    (module/media.name).symlink_to(media)
                    child['GTK_PATH'] = str(folder/label/'gtk-modules')
                for name, command in [('context', [ctest]), ('gpu-color-controls', [target/'examples/video_color_probe', video, '--check-controls', *extra]),
                                      ('production-window', [target/'examples/integrated_video_probe', '--normal-only', video])]:
                    test = label+'-'+name
                    (folder/(test+'-ldd.log')).write_text(require_loader(command[0], meta['library'], child))
                    row = run(test, command, child, folder)
                    log = (folder/(test+'.log')).read_text(errors='replace')
                    if name=='context' and row['status']=='PASS':
                        if 'GTK_RUNTIME 4.24.1' not in log or 'GTK_LOADED '+meta['library'] not in log:
                            row = dict(row, status='FAIL', reasons=['runtime GTK identity/version not confirmed'])
                    if name=='gpu-color-controls': row = check_normal(row, log, GPU_MARKERS)
                    row.update(variant=label, backend=backend)
                    rows.append(row)
                if args.manual and backend=='wayland' and label=='patched' and all(r['status']=='PASS' for r in rows if r['variant']=='patched'):
                    print('MANUAL patched GTK Pivra: compare original video/colors with mpv; test audio, seek, controls. Close to finish.', flush=True)
                    row = run('patched-manual', [target/'pivra', video], child, folder, timeout=600)
                    row = check_normal(row, (folder/'patched-manual.log').read_text(errors='replace'), PRODUCT_MARKERS)
                    row.update(variant=label, backend=backend); rows.append(row)
    return rows


if __name__ == '__main__':
    raise SystemExit(main())
