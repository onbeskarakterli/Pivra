"""Production surface initialization and one-shot stock recovery, not AMD certification."""
import argparse
import json
import os
import shlex
from pathlib import Path
import subprocess
import tempfile
from lifecycle import ROOT, run, session
from normal_playback import build_binding


def verify(row, log, markers):
    missing = [m for m in markers if m not in log]
    if row['status'] == 'PASS' and missing:
        row = dict(row, status='FAIL', reasons=[*row['reasons'], 'missing production evidence: ' + ', '.join(missing)])
    return row


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--session', choices=['native', 'x11', 'sway'], default='native')
    parser.add_argument('--software', action='store_true')
    args = parser.parse_args()
    if args.session != 'native' and not args.software:
        parser.error('headless controls require --software')
    if not args.software and os.environ.get('PIVRA_GPU_ALLOW_SOFTWARE_TEST') == '1':
        parser.error('software permission requires --software')
    for key in ('PIVRA_VIDEO_BACKEND', 'PIVRA_VIDEO_TEST_MODULE_DIR', 'GTK_MEDIA', 'GST_GL_API'):
        if os.environ.get(key):
            parser.error('unset ' + key + ' for automatic production selection')
    out = Path(tempfile.mkdtemp(prefix='pivra-production-init-'))
    print('Reports:', out, flush=True)
    subprocess.run(['cargo', 'build', '--locked', '--example', 'production_video_init_probe', '--example', 'integrated_video_probe'], cwd=ROOT, check=True)
    target = Path(json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--no-deps', '--format-version', '1'], cwd=ROOT))['target_directory'])/'debug/examples'
    subprocess.run(['python3', '-B', 'tests/video_gpu/backend/fixtures.py', str(out/'fixtures')], cwd=ROOT, check=True)
    binding = out/'surface-binding'
    build_binding(ROOT, binding)
    injection = out/'fail-pivra-surface.so'
    subprocess.run([*shlex.split(os.environ.get('CC', 'cc')), '-shared', '-fPIC', '-O2', '-Wall', '-Wextra', '-Werror',
                    *shlex.split(os.environ.get('CPPFLAGS', '')),
                    *shlex.split(subprocess.check_output(['pkg-config', '--cflags', 'gtk4'], text=True)),
                    str(ROOT/'tests/video_gpu/fail_surface_context.c'), '-o', str(injection),
                    *shlex.split(os.environ.get('LDFLAGS', '')),
                    *shlex.split(subprocess.check_output(['pkg-config', '--libs', 'gtk4'], text=True)), '-ldl'], check=True)
    env = dict(os.environ, GTK_A11Y='none', PIVRA_TRACE_VIDEO='1',
               XDG_CONFIG_HOME=str(out/'config'), XDG_CACHE_HOME=str(out/'cache'), XDG_DATA_HOME=str(out/'data'))
    env['GSK_RENDERER'] = 'ngl' if int(subprocess.check_output(['pkg-config', '--modversion', 'gtk4'], text=True).split('.')[1]) < 20 else 'gl'
    if args.software:
        env.update(LIBGL_ALWAYS_SOFTWARE='1', PIVRA_GPU_ALLOW_SOFTWARE_TEST='1')
    rows = []
    with session(args.session, env, out, floating=True) as active:
        rows.append(run('native-surface-binding', [binding, '--surface', '--require-borrowed'], active, out))
        for mode in ['normal', 'init-failure', 'async-recovery', 'async-playing', 'stale-recovery', 'close-pending', 'deleted-source']:
            child = dict(active, XDG_CONFIG_HOME=str(out/mode/'config'), XDG_CACHE_HOME=str(out/mode/'cache'), XDG_DATA_HOME=str(out/mode/'data'))
            if mode == 'init-failure':
                child['LD_PRELOAD'] = str(injection)
            name = 'production-' + mode
            result = run(name, [target/'production_video_init_probe', mode, out/'fixtures/ramp.mp4', out/'fixtures/bars.mp4'], child, out,
                         expected_gpu_errors=2 if mode == 'init-failure' else 0,
                         expected_context_failures=2 if mode == 'init-failure' else 0)
            markers = [f'PASS production-init {mode}', 'PIVRA_GPU_INITIALIZE source=window-surface']
            if mode == 'init-failure':
                markers += ['CONTROLLED_PIVRA_CONTEXT_FAILURE stock-and-renderer-unchanged', 'PIVRA_VIDEO_FALLBACK stage=initialization backend=stock', 'PIVRA_GPU_STOCK']
            else:
                markers += ['PIVRA_GPU_DRAW BT1886 GLMemory -> GLMemory', 'PIVRA_GPU_PRESENT GLMemory sync-meta retained']
            if mode in ('async-recovery', 'async-playing', 'stale-recovery', 'close-pending'):
                markers.append('PIVRA_VIDEO_FALLBACK stage=pipeline backend=stock')
            rows.append(verify(result, (out/(name+'.log')).read_text(errors='replace'), markers))
        rows.append(run('full-production-window', [target/'integrated_video_probe', '--normal-only', out/'fixtures/ramp.mp4'], active, out))
    (out/'report.json').write_text(json.dumps(dict(source=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        dirty=bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT)), session=args.session,
        software_control=args.software, results=rows, native_amd_acceptance='PENDING unless actually run and visually confirmed on AMD',
        mixed_lifecycle='Separate previously blocked stock EGL ownership gate; not closed by these tests'), indent=2)+'\n')
    return 1 if any(r['status'] == 'FAIL' for r in rows) else 77 if any(r['status'] in ('BLOCKED', 'NOT RUN') for r in rows) else 0


if __name__ == '__main__':
    raise SystemExit(main())
