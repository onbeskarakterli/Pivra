"""Identify GTK's EGL request boundary; no production rendering overrides."""
import argparse
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
from lifecycle import ROOT, session

NAMES = ('gtk_sdr_implicit', 'gtk_sdr_size_zero', 'explicit_es1', 'explicit_es2', 'explicit_gl', 'any_api', 'any_api_no_alpha')
SUCCESS = 0x3000


def classify(rows):
    by_name = {r['name']: r for r in rows}
    if len(rows) != len(NAMES) or set(by_name) != set(NAMES):
        return 'INCOMPLETE'
    if any(not r['ok'] or r['error'] != SUCCESS or r['count'] < 0 for r in rows):
        return 'EGL_QUERY_ERROR'
    if by_name['gtk_sdr_implicit']['count'] != by_name['gtk_sdr_size_zero']['count']:
        return 'QUERY_SIZE_DIVERGENCE'
    if by_name['gtk_sdr_implicit']['count'] != by_name['explicit_es1']['count']:
        return 'IMPLICIT_ES1_CONTROL_MISMATCH'
    if by_name['gtk_sdr_implicit']['count'] > 0:
        return 'GTK_SDR_CONFIG_FAILURE_NOT_REPRODUCED'
    if by_name['explicit_es2']['count'] > 0 or by_name['explicit_gl']['count'] > 0:
        return 'IMPLICIT_ES1_FILTER_MISMATCH'
    if by_name['any_api']['count'] > 0:
        return 'NO_EXPLICIT_MODERN_API_MATCH'
    if by_name['any_api_no_alpha']['count'] > 0:
        return 'RGBA8_REQUIREMENT_MISMATCH'
    return 'NO_MATCHING_WINDOW_RGB_CONFIG'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--session', choices=['native', 'sway'], default='native')
    parser.add_argument('--software', action='store_true')
    args = parser.parse_args()
    if args.session != 'native' and not args.software:
        parser.error('headless control requires --software')
    if not args.software and os.environ.get('LIBGL_ALWAYS_SOFTWARE') == '1':
        parser.error('software environment requires --software')
    out = Path(tempfile.mkdtemp(prefix='pivra-wayland-config-'))
    print('Report:', out, flush=True)
    binary = out/'egl-config'
    packages = ['epoxy', 'wayland-client']
    flags = lambda arg: shlex.split(subprocess.check_output(['pkg-config', arg, *packages], text=True))
    subprocess.run([*shlex.split(os.environ.get('CC', 'cc')), '-O2', '-Wall', '-Wextra', '-Werror',
                    *shlex.split(os.environ.get('CPPFLAGS', '')), *flags('--cflags'),
                    str(ROOT/'tests/video_gpu/wayland_egl_config.c'), '-o', str(binary),
                    *shlex.split(os.environ.get('LDFLAGS', '')), *flags('--libs')], check=True)
    env = dict(os.environ)
    if args.software:
        env['LIBGL_ALWAYS_SOFTWARE'] = '1'
    with session(args.session, env, out) as active:
        try:
            result = subprocess.run([binary], env=active, capture_output=True, text=True, timeout=15)
            log = result.stdout + result.stderr
            (out/'egl-config.log').write_text(log)
            print(log, end='', flush=True)
            rows = [json.loads(line.removeprefix('EGL_CONFIG_QUERY ')) for line in result.stdout.splitlines() if line.startswith('EGL_CONFIG_QUERY ')]
            hdr = [json.loads(line.removeprefix('EGL_HDR_QUERY ')) for line in result.stdout.splitlines() if line.startswith('EGL_HDR_QUERY ')]
            classification = classify(rows) if result.returncode == 0 else 'NOT RUN' if result.returncode == 77 else 'FAIL'
            code = result.returncode
        except subprocess.TimeoutExpired:
            classification, code, rows, hdr = 'FAIL deadline', 1, [], []
    report = dict(source=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  software_control=args.software, classification=classification, exit=code, queries=rows, optional_hdr_queries=hdr,
                  scope='private Wayland connection: configuration query only, not GPU playback or a native fix',
                  native_wayland_acceptance='PENDING; GTK cached-display state is not reproduced by a private connection')
    (out/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print('CONFIG_CLASSIFICATION', classification, flush=True)
    if classification in ('INCOMPLETE', 'EGL_QUERY_ERROR') and code == 0:
        return 1
    return code


if __name__ == '__main__':
    raise SystemExit(main())
