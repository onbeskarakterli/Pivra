#!/usr/bin/env python3
"""Diagnostic ABA experiment; never changes installed helper or shared cache."""
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import signal
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
LIMIT_LINE = 'struct rlimit memory={1024UL*1024*1024,1024UL*1024*1024};'


def diagnostic_source(source, mib):
    if mib not in (1024, 2048) or source.count(LIMIT_LINE) != 1:
        raise ValueError('unsupported experiment or changed helper source; review required')
    return source.replace(LIMIT_LINE, f'struct rlimit memory={{{mib}UL*1024*1024,{mib}UL*1024*1024}};') if mib != 1024 else source


def classify(trials, stable):
    if not stable or len(trials) != 3:
        return 'NOT PROVEN'
    for trial, mib in zip(trials, (1024, 2048, 1024)):
        if (trial['limit_mib'] != mib or trial['observed_as'] != mib * 1024**2
                or trial['timeout'] or not trial['sandbox_command']):
            return 'NOT PROVEN'
    first, higher, repeat = trials
    if (first['exit'] == repeat['exit'] == 4 and first['allocation_error']
            and repeat['allocation_error'] and higher['exit'] == 0 and higher['generated']):
        return 'EVIDENCE: RLIMIT_AS dependent failure for this file/environment'
    return 'NOT PROVEN'


def identity(path):
    stat = path.stat()
    if not path.is_file() or stat.st_size > 64 * 1024**2:
        raise ValueError('source must be a regular file <=64 MiB')
    digest = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(65536), b''):
            digest.update(chunk)
    return (stat.st_dev, stat.st_ino, stat.st_size, stat.st_mtime_ns,
            stat.st_ctime_ns, digest.hexdigest())


def excerpt(path):
    with path.open('rb') as stream:
        head = stream.read(32768)
        size = stream.seek(0, 2)
        if size > 65536:
            stream.seek(-32768, 2)
            head += b'\n[middle omitted]\n' + stream.read(32768)
        elif size > len(head):
            stream.seek(len(head))
            head += stream.read(32768)
    return head.decode('utf-8', errors='replace')


def run_trial(helper, source, mime, mtime, mib, root):
    root.mkdir(mode=0o700)
    cache = root / 'cache'
    cache.mkdir(mode=0o700)
    # Preserve data/config/session selection and all sandbox choices. Only cache
    # and temporary artifact paths differ; every run starts with an empty cache.
    env = {**os.environ, 'XDG_CACHE_HOME': str(cache), 'TMPDIR': str(root),
           'PIVRA_TRACE_GNOME_THUMB': '1'}
    started = time.monotonic()
    timed_out = False
    with (root/'stdout').open('wb') as out, (root/'stderr').open('wb') as err:
        process = subprocess.Popen([str(helper), source.as_uri(), mime, str(mtime), 'missing'],
                                   env=env, stdin=subprocess.DEVNULL, stdout=out, stderr=err,
                                   start_new_session=True)
        try:
            while True:
                pid, status, usage = os.wait4(process.pid, os.WNOHANG)
                if pid:
                    break
                if time.monotonic() - started >= 20:
                    timed_out = True
                    os.killpg(process.pid, signal.SIGKILL)
                    _, status, usage = os.wait4(process.pid, 0)
                    break
                time.sleep(0.01)
        except BaseException:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait()
            raise
        process.returncode = os.waitstatus_to_exitcode(status)
    stdout, stderr = excerpt(root/'stdout'), excerpt(root/'stderr')
    print(f'=== {mib} MiB trial ===\n{stdout}\n{stderr}', flush=True)
    observed = re.search(r'limits_as=(\d+)', stderr)
    return dict(limit_mib=mib, observed_as=int(observed[1]) if observed else None,
                exit=process.returncode, timeout=timed_out,
                allocation_error=('Failed to allocate memory' in stderr
                                  or 'Cannot allocate memory' in stderr),
                sandbox_command=('About to launch script:' in stderr and 'bwrap' in stderr),
                generated=(stdout == 'generated\n'), elapsed_seconds=time.monotonic()-started,
                # Linux wait4 for the helper, including reaped descendants;
                # not a per-loader RSS nor a simultaneous sum of all processes.
                helper_tree_maxrss_kib=usage.ru_maxrss,
                helper_tree_cpu_seconds=usage.ru_utime+usage.ru_stime)


def main():
    if len(sys.argv) != 2:
        raise SystemExit('usage: factory_as_comparison.py /path/to/photo.jpg')
    packages = ['gnome-desktop-4', 'gio-2.0', 'gobject-2.0', 'gdk-pixbuf-2.0']
    if subprocess.run(['pkg-config', '--exists', *packages]).returncode:
        print('NOT RUN: GNOME desktop-4 development API unavailable')
        return 77
    source = Path(sys.argv[1]).absolute()
    original = identity(source)
    query = subprocess.check_output(['gio', 'info', '-a', 'standard::content-type', str(source)],
                                    env={**os.environ, 'LC_ALL': 'C'}, text=True, timeout=5)
    mime = next((line.split('standard::content-type:', 1)[1].strip()
                 for line in query.splitlines() if 'standard::content-type:' in line), None)
    if not mime or '/' not in mime:
        raise ValueError('GIO did not return a MIME type')
    cflags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', *packages], text=True))
    libs = shlex.split(subprocess.check_output(['pkg-config', '--libs', *packages], text=True))
    helper_source = (ROOT/'native/thumbnail_factory.c').read_text()
    trials = []
    stable = True
    with tempfile.TemporaryDirectory(prefix='pivra-factory-as-') as folder:
        root = Path(folder)
        binaries = {}
        for mib in (1024, 2048):
            c_source = root/f'factory-{mib}.c'
            c_source.write_text(diagnostic_source(helper_source, mib))
            binary = root/f'helper-{mib}'
            subprocess.run(['cc', '-O2', '-Wall', '-Wextra', '-Werror', *cflags,
                            str(c_source), '-o', str(binary), *libs], check=True, timeout=30)
            binaries[mib] = binary
        print(f'INPUT uri={source.as_uri()!r} mime={mime!r} identity={original!r}', flush=True)
        print('Only RLIMIT_AS differs: CPU=15s, FSIZE=256MiB, deadline=20s, sequential jobs=1.', flush=True)
        for index, mib in enumerate((1024, 2048, 1024)):
            if identity(source) != original:
                stable = False
                break
            trials.append(run_trial(binaries[mib], source, mime, original[3]//10**9,
                                    mib, root/f'trial-{index}'))
            stable = stable and identity(source) == original
    result = classify(trials, stable)
    print(json.dumps(dict(result=result, source_stable=stable, trials=trials), indent=2))
    print('Review expanded commands: same thumbnailer/MIME and sandbox required. '
          'This is file-specific evidence, not approval for a production limit change.')
    return 0 if result.startswith('EVIDENCE:') else 1


if __name__ == '__main__':
    raise SystemExit(main())
