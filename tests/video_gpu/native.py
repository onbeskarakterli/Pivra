#!/usr/bin/env python3
"""Isolated, evidence-only native integration session; never installs GTK."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time

CPU_ENV = ('GTK_MEDIA', 'GTK_PATH', 'GIO_EXTRA_MODULES',
           'PIVRA_VIDEO_TEST_MODULE_DIR', 'PIVRA_GTK_BT1886_CPU',
           'PIVRA_GTK_COLOR_CPU_TRANSPORT')


def environment(libdir):
    env = os.environ.copy()
    for key in CPU_ENV:
        env.pop(key, None)
    # No inherited private SDK or module directories in this comparison.
    env.update(LD_LIBRARY_PATH=str(libdir) if libdir else '',
               GDK_BACKEND='wayland', GSK_RENDERER='gl',
               GDK_DEBUG='misc,opengl,dmabuf', GSK_DEBUG='renderer,fallback',
               GST_DEBUG='gtkgstsink:5', GST_DEBUG_NO_COLOR='1')
    env.pop('LD_PRELOAD', None)
    return env


def mapped_gtk(pid):
    try:
        lines = Path(f'/proc/{pid}/maps').read_text().splitlines()
    except (OSError, UnicodeError):
        return []
    return sorted({line.split(None, 5)[5] for line in lines
                   if len(line.split(None, 5)) == 6 and '/libgtk-4.so' in line})


def command(argv, env, output, timeout=120):
    with output.open('w') as log:
        try:
            result = subprocess.run(argv, env=env, stdout=log,
                                    stderr=subprocess.STDOUT, timeout=timeout,
                                    check=False)
            return {'command': argv, 'exit': result.returncode}
        except subprocess.TimeoutExpired:
            return {'command': argv, 'status': 'FAIL timeout'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--prefix', type=Path, required=True)
    parser.add_argument('--video', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--control-video', type=Path,
                        help='Known non-target sRGB/PQ/HLG clip; not a synthetic substitute')
    parser.add_argument('--baseline-build', type=Path)
    parser.add_argument('--patched-build', type=Path)
    parser.add_argument('--colorstate-path',
                        help='Exact failing GLib test path from the native crash log')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    lib = (args.prefix / 'lib/libgtk-4.so.1').resolve(strict=True)
    if not lib.is_relative_to(args.prefix.resolve()) or args.prefix.resolve() == Path('/usr'):
        parser.error('GTK library must resolve inside a private prefix, not /usr')
    if b'video-bt1886 GPU CICP op' not in lib.read_bytes():
        parser.error('private GTK lacks patch instrumentation; debug build required')
    video = args.video.resolve(strict=True)
    args.output.mkdir(parents=True, exist_ok=False)
    out = args.output.resolve()
    report = {'patched_library': str(lib), 'gtk_sha256': hashlib.sha256(lib.read_bytes()).hexdigest(),
              'visual_acceptance': 'NOT RUN: requires owner comparison with mpv',
              'gpu_and_no_readback': 'NOT PROVEN: inspect traces and native GPU profiler',
              'sigbus_classification': 'OPEN: no clean baseline comparison yet'}
    stock, patched = environment(None), environment(lib.parent)
    try:
        subprocess.run(['cargo', 'build', '--locked'], cwd=root, check=True)
        app = str(root / 'target/debug/pivra')
        report['ffprobe'] = command(['ffprobe', '-v', 'error', '-show_streams',
                                     '-of', 'json', str(video)], stock, out / 'metadata.json')
        if args.colorstate_path:
            if not args.baseline_build or not args.patched_build:
                parser.error('targeted comparison requires both build directories')
            # Exact one-test control only, never the whole GTK suite.
            for name, build in [('baseline', args.baseline_build), ('patched', args.patched_build)]:
                report[name + '_colorstate'] = command(
                    [str(build.resolve() / 'testsuite/gdk/colorstate'),
                     '-p', args.colorstate_path],
                    environment((build.resolve() / 'gtk')), out / f'{name}-colorstate.log')
        processes = []
        try:
            sessions = [('stock', stock, video), ('patched', patched, video)]
            if args.control_video:
                sessions.append(('non-target', patched, args.control_video.resolve(strict=True)))
            for name, env, clip in sessions:
                log = (out / f'{name}.log').open('w')
                process = subprocess.Popen([app, str(clip)], env=env, stdout=log,
                                           stderr=subprocess.STDOUT, start_new_session=True)
                processes.append((name, process, log))
                deadline = time.monotonic() + 15
                paths = []
                while process.poll() is None and time.monotonic() < deadline:
                    paths = mapped_gtk(process.pid)
                    if paths:
                        break
                    time.sleep(0.02)
                report[name + '_loaded_gtk'] = paths
                expected = name != 'stock'
                if not paths or any((Path(p).resolve() == lib) != expected for p in paths):
                    raise RuntimeError(f'{name}: wrong or unobservable loaded GTK: {paths}')
                print(f'{name}: runtime GTK {paths}', flush=True)
            mpvlog = (out / 'mpv.log').open('w')
            mpv = subprocess.Popen(['mpv', '--no-config', str(video)], env=stock,
                                   stdout=mpvlog, stderr=subprocess.STDOUT, start_new_session=True)
            processes.append(('mpv', mpv, mpvlog))
            print('Compare stock / patched / mpv; test controls. Close all windows to finish. Logs:', out,
                  flush=True)
            while any(p.poll() is None for _, p, _ in processes):
                time.sleep(0.2)
            report['exits'] = {n: p.returncode for n, p, _ in processes}
        finally:
            for _, process, log in processes:
                if process.poll() is None:
                    os.killpg(process.pid, signal.SIGTERM)
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        os.killpg(process.pid, signal.SIGKILL)
                        process.wait()
                log.close()
            for name in ('patched', 'non-target'):
                path = out / f'{name}.log'
                if path.exists():
                    text = path.read_text(errors='replace')
                    report[name + '_traces'] = {key: text.count(key) for key in
                        ('video-bt1886 GPU CICP op', 'video-bt1886 CPU transfer',
                         'video-bt1886 GPU readback/upload fallback')}
    finally:
        (out / 'REPORT.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Session recorded, NOT automatic visual/GPU acceptance:', out / 'REPORT.json')
    failed = any(code != 0 for code in report.get('exits', {}).values())
    failed |= any(report[key].get('exit') != 0 for key in
                  ('baseline_colorstate', 'patched_colorstate') if key in report)
    if failed:
        print('FAIL: a playback process or targeted control failed; inspect REPORT.json and logs')
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
