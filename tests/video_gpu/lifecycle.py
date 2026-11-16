#!/usr/bin/env python3
"""Isolated mixed stock/GPU lifecycle gate. Never installs GTK or changes media."""
import argparse
import contextlib
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import select
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
MODULES = ['gtk4', 'gstreamer-gl-1.0', 'gstreamer-video-1.0', 'gstreamer-allocators-1.0']

def flags(option):
    return shlex.split(subprocess.check_output(['pkg-config', option, *MODULES], text=True))

@contextlib.contextmanager
def session(kind, env, out):
    if kind == 'native':
        yield env
        return
    env = dict(env, XDG_RUNTIME_DIR=str(out/'runtime'))
    Path(env['XDG_RUNTIME_DIR']).mkdir(mode=0o700)
    with (out/'compositor.log').open('w') as log:
        if kind == 'x11':
            read, write = os.pipe()
            proc = subprocess.Popen(['Xvfb', '-displayfd', str(write), '-screen', '0', '1280x1024x24', '-nolisten', 'tcp'], pass_fds=(write,), stdout=log, stderr=log)
            os.close(write)
        else:
            conf = out/'sway.conf'
            conf.write_text('xwayland disable\noutput HEADLESS-1 mode 1280x1024\ndefault_border none\n')
            env.update(WLR_BACKENDS='headless', WLR_RENDERER='pixman', WLR_LIBINPUT_NO_DEVICES='1')
            env.pop('DISPLAY', None)
            proc = subprocess.Popen(['sway', '-c', str(conf)], env=env, stdout=log, stderr=log)
        try:
            if kind == 'x11':
                try:
                    if not select.select([read], [], [], 10)[0]:
                        raise RuntimeError('Xvfb display deadline')
                    env.update(DISPLAY=':'+os.read(read, 32).decode().strip(), GDK_BACKEND='x11')
                finally:
                    os.close(read)
            else:
                deadline = time.monotonic()+10
                while True:
                    sockets = [s for s in Path(env['XDG_RUNTIME_DIR']).glob('wayland-*') if s.is_socket()]
                    if sockets:
                        env.update(WAYLAND_DISPLAY=sockets[0].name, GDK_BACKEND='wayland')
                        break
                    if proc.poll() is not None or time.monotonic() > deadline:
                        raise RuntimeError('Sway socket deadline; see compositor.log')
                    time.sleep(.05)
            yield env
        finally:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill(); proc.wait()

def run(name, command, env, out, timeout=90, expected_gpu_errors=0, expected_context_failures=0):
    with (out/(name+'.log')).open('w') as log:
        proc = subprocess.Popen([str(v) for v in command], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = proc.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGTERM)
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL); proc.wait()
            code = 124
    status = 'PASS' if code == 0 else 'NOT RUN' if code == 77 else 'FAIL'
    log = (out/(name+'.log')).read_text(errors='replace')
    diagnostics = sum(line.startswith('{"pivra_perf":1,') for line in log.splitlines())
    context_errors = log.count('gdk_gl_context_make_current() failed')
    gpu_errors = log.count('PIVRA_GPU_FAIL') + log.count('PIVRA_GPU_SHADER_FAIL')
    context_failures = log.count('PIVRA_GPU_CONTEXT_FAILURE')
    reasons = []
    if context_errors or gpu_errors != expected_gpu_errors or context_failures != expected_context_failures:
        reasons.append('native context/GPU failure logged')
    if diagnostics >= 4096:
        reasons.append('diagnostic cap reached')
    if code == 0 and reasons:
        status = 'FAIL'
    print(f'{status} {name}: exit={code}; {"; ".join(reasons)}; {out/(name+".log")}', flush=True)
    lines = log.splitlines()
    stages = [line for line in lines if line.startswith('LIFECYCLE_STAGE')]
    bindings = [line for line in lines if line.startswith('PIVRA_GPU_BINDING')]
    signals = [line.removeprefix('GDB_INFERIOR_SIGNAL ') for line in lines if line.startswith('GDB_INFERIOR_SIGNAL ')]
    libraries = {}
    for line in lines:
        if line.startswith('LIFECYCLE_LIBRARY '):
            path = Path(line.removeprefix('LIFECYCLE_LIBRARY '))
            try: libraries[str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
            except OSError: libraries[str(path)] = None
    if status == 'FAIL':
        print('FAIL last transition:', stages[-1] if stages else 'not reached', flush=True)
        print('\n'.join(lines[-12:]), flush=True)
    return dict(name=name, exit=code, status=status, diagnostic_events=diagnostics, context_errors=context_errors, context_failures=context_failures,gpu_errors=gpu_errors, reasons=reasons,last_stage=stages[-1] if stages else None,last_bindings=bindings[-8:],inferior_signals=signals,libraries_sha256=libraries)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--session', choices=['native','x11','sway'], default='native')
    parser.add_argument('--software', action='store_true', help='explicit llvmpipe control, never hardware acceptance')
    parser.add_argument('--api', choices=['gles2','opengl3'], help='optional controlled GL API')
    parser.add_argument('--ownership-only', action='store_true')
    parser.add_argument('--retirement-control', action='store_true', help='separate destructive EGL mechanism control, not native acceptance')
    parser.add_argument('--debug-mixed', action='store_true', help='run mixed-lifecycle under GDB, preserving fatal signal and EGL teardown stacks')
    args = parser.parse_args()
    if args.debug_mixed and not shutil.which('gdb'):
        print('NOT RUN debugger: gdb is not installed; ordinary nine-gate command is unchanged')
        return 77
    out = Path(tempfile.mkdtemp(prefix='pivra-video-context-'))
    print('Logs:', out, flush=True)
    env = os.environ.copy()
    if not args.software and env.get('PIVRA_GPU_ALLOW_SOFTWARE_TEST') == '1':
        parser.error('use --software to label inherited software permission explicitly')
    env.update(XDG_CONFIG_HOME=str(out/'config'), XDG_CACHE_HOME=str(out/'cache'), XDG_DATA_HOME=str(out/'data'), PIVRA_TRACE_VIDEO='1', PIVRA_PERF='1', GTK_A11Y='none')
    minor = int(subprocess.check_output(['pkg-config','--modversion','gtk4'], text=True).split('.')[1])
    env['GSK_RENDERER'] = 'ngl' if minor < 20 else 'gl'
    if args.software:
        env.update(LIBGL_ALWAYS_SOFTWARE='1', PIVRA_GPU_ALLOW_SOFTWARE_TEST='1')
    if args.api:
        env['GST_GL_API'] = args.api
    ctest = out/'context-lifecycle'
    if args.retirement_control:
        print('NEGATIVE EGL MECHANISM CONTROL: not a native lifecycle acceptance run',flush=True)
        ctest = out/'stock-retirement'
        extra_cflags = shlex.split(subprocess.check_output(['pkg-config','--cflags','epoxy'],text=True))
        extra_libs = shlex.split(subprocess.check_output(['pkg-config','--libs','epoxy'],text=True))
        subprocess.run([*shlex.split(env.get('CC','cc')),'-O2','-Wall','-Wextra','-Werror','-DGST_USE_UNSTABLE_API',*shlex.split(env.get('CPPFLAGS','')),*flags('--cflags'),*extra_cflags,'tests/video_gpu/stock_egl_retirement.c','-o',str(ctest),*shlex.split(env.get('LDFLAGS','')),*flags('--libs'),*extra_libs],cwd=ROOT,check=True)
        with session(args.session,env,out) as active:
            rows=[run('retirement-'+mode,[ctest,mode],active,out) for mode in ['legacy','borrowed']]
        for row in rows:
            if row['name']=='retirement-legacy' and row['status']=='PASS': row['status']='MECHANISM REPRODUCED'
        (out/'retirement-report.json').write_text(json.dumps(dict(native_acceptance=False,results=rows),indent=2)+'\n')
        print('EGL mechanism control only; NOT a native lifecycle acceptance result',flush=True)
        return 1 if any(r['status']=='FAIL' for r in rows) else 77 if any(r['status']=='NOT RUN' for r in rows) else 0
    policy = out/'current-api-policy'
    subprocess.run([*shlex.split(env.get('CC','cc')),'-O2','-Wall','-Wextra','-Werror','-Inative/video',*shlex.split(env.get('CPPFLAGS','')),*flags('--cflags'),'tests/video_gpu/current_api_policy.c','-o',str(policy),*shlex.split(env.get('LDFLAGS','')),*flags('--libs')],cwd=ROOT,check=True)
    subprocess.run([str(policy)],cwd=ROOT,check=True)
    subprocess.run([*shlex.split(env.get('CC','cc')), '-O2','-Wall','-Wextra','-Werror','-Wno-unused-parameter','-DGTK_COMPILATION','-Inative/video', *shlex.split(env.get('CPPFLAGS','')), *flags('--cflags'), 'tests/video_gpu/context_lifecycle.c','native/video/gtkgstpaintable.c','native/video/gtkgstsink.c','-o',str(ctest), *shlex.split(env.get('LDFLAGS','')), *flags('--libs'), '-lm'], cwd=ROOT, check=True)
    if not args.ownership_only:
        subprocess.run(['cargo','build','--locked','--example','integrated_video_probe','--example','video_color_probe'], cwd=ROOT, check=True)
        target = Path(json.loads(subprocess.check_output(['cargo','metadata','--locked','--no-deps','--format-version','1'], cwd=ROOT))['target_directory'])/'debug/examples'
        spec = importlib.util.spec_from_file_location('fixtures',ROOT/'tests/video_color/backend_probe.py')
        fixtures = importlib.util.module_from_spec(spec); spec.loader.exec_module(fixtures)
        media = out/'fixtures'; media.mkdir(); fixtures.fixtures(media)
        subprocess.run([sys.executable,'-B','tests/video_gpu/backend/fixtures.py',str(out/'ramps')],cwd=ROOT,check=True)
    results = []
    with session(args.session, env, out) as active:
        for check in ['foreign','release','current','presentation','failure']:
            results.append(run('ownership-'+check, [ctest,check], active, out, expected_gpu_errors=8 if check=='failure' else 0,expected_context_failures=8 if check=='failure' else 0))
        if not args.ownership_only:
            mixed = [target/'integrated_video_probe', *[media/(n+'.mp4') for n in ['bt709','srgb','unknown','full709']]]
            if args.debug_mixed:
                mixed = ['gdb','--batch','--return-child-result','-x',ROOT/'tests/video_gpu/context_trace.gdb','--args',*mixed]
            results.append(run('mixed-lifecycle',mixed,active,out))
            results.append(run('error-cleanup', [target/'integrated_video_probe','--error-path'], active, out))
            results.append(run('limited-presentation-controls', [target/'video_color_probe',media/'bt709.mp4','--check-controls'], active, out))
            results.append(run('full-ramp-shader-controls', [target/'video_color_probe',out/'ramps/ramp.mp4','--check-bt1886-ramp','--check-controls'], active, out))
    (out/'report.json').write_text(json.dumps(dict(source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),dirty=bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT)),session=args.session,software_control=args.software,api=args.api,results=results),indent=2)+'\n')
    return 1 if any(r['status']=='FAIL' for r in results) else 77 if any(r['status']=='NOT RUN' for r in results) else 0

if __name__ == '__main__':
    raise SystemExit(main())
