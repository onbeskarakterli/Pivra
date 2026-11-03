#!/usr/bin/env python3
"""Run one exact GTK colorstate case in clean and patched private build trees."""
import argparse
import hashlib
import json
import secrets
from pathlib import Path
import signal
import subprocess

from native import command, environment
from probe import SOURCE_SHA256


def classify(baseline, patched):
    # Never describe matching crashes as proof the patch is harmless.
    if baseline == -signal.SIGBUS and patched == -signal.SIGBUS:
        return 'BASELINE REPRODUCIBLE SIGBUS; patch-specific effects not excluded'
    if baseline == 0 and patched == -signal.SIGBUS:
        return 'PATCHED-ONLY SIGBUS: integration blocked'
    if baseline == -signal.SIGBUS and patched == 0:
        return 'BASELINE-ONLY SIGBUS: investigate before attributing improvement'
    if baseline == 0 and patched == 0:
        return 'NOT REPRODUCED in this exact case/environment'
    return 'INCONCLUSIVE: skip, timeout or another failure'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline-build', required=True, type=Path)
    parser.add_argument('--patched-build', required=True, type=Path)
    parser.add_argument('--case', required=True, help='Exact /colorstate/... case, not a prefix')
    parser.add_argument('--seed', help='Original GLib R02S seed; otherwise one shared seed is generated')
    parser.add_argument('--renderer', choices=('gl', 'vulkan'), default='gl')
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    seed = args.seed or 'R02S' + secrets.token_hex(16)
    report = {'seed': seed, 'case': args.case, 'renderer_requested': args.renderer, 'status': 'INCOMPLETE'}
    try:
        for label, build in [('baseline', args.baseline_build), ('patched', args.patched_build)]:
            build = build.resolve(strict=True)
            info = json.loads((build / 'meson-info/meson-info.json').read_text())
            source = Path(info['directories']['source'])
            if label == 'baseline':
                for path, digest in SOURCE_SHA256.items():
                    if hashlib.sha256((source / path).read_bytes()).hexdigest() != digest:
                        raise RuntimeError('baseline differs from pinned upstream: ' + path)
            binary = build / 'testsuite/gdk/colorstate'
            lib = (build / 'gtk/libgtk-4.so.1').resolve(strict=True)
            marker = b'video-bt1886 GPU CICP op' in lib.read_bytes()
            if marker != (label == 'patched'):
                raise RuntimeError(label + ': unexpected patch instrumentation')
            env = environment(lib.parent, args.renderer)
            listed = subprocess.run([str(binary), '-l'], env=env, capture_output=True,
                                    text=True, timeout=30, check=True)
            cases = [line.strip() for line in listed.stdout.splitlines() if line.startswith('/colorstate/')]
            if args.case not in cases or any(c.startswith(args.case + '/') for c in cases):
                raise RuntimeError('not an exact registered test leaf: ' + args.case)
            loader = subprocess.run(['ldd', str(binary)], env=env, capture_output=True,
                                    text=True, check=True)
            (args.output / f'{label}-ldd.log').write_text(loader.stdout)
            if not any('libgtk-4.so' in line and str(lib.parent) + '/' in line
                       for line in loader.stdout.splitlines()):
                raise RuntimeError(label + ': wrong GTK resolution')
            (args.output / f'{label}-buildoptions.json').write_bytes(
                (build / 'meson-info/intro-buildoptions.json').read_bytes())
            # Loader trace retained even if the test crashes too quickly for /proc sampling.
            env['LD_DEBUG'] = 'libs'
            env['LD_DEBUG_OUTPUT'] = str(args.output.resolve() / f'{label}-loader')
            result = command([str(binary), '-p', args.case, '--seed=' + seed], env,
                             args.output / f'{label}.log')
            log = (args.output / f'{label}.log').read_text(errors='replace')
            result.update(library=str(lib), sha256=hashlib.sha256(lib.read_bytes()).hexdigest(),
                          source=str(source), skipped='# SKIP' in log.upper())
            report[label] = result
        b, p = report['baseline'], report['patched']
        report['status'] = classify(None if b['skipped'] else b.get('exit'),
                                    None if p['skipped'] else p.get('exit'))
        print(report['status'])
        return 0 if report['status'].startswith('NOT REPRODUCED') else 1
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        report['error'] = str(error)
        print('FAIL control:', error)
        return 1
    finally:
        (args.output / 'REPORT.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    raise SystemExit(main())
