"""Bounded slideshow gate evidence; no signal/debugger exit is a PASS."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys

sys.path.insert(0, str(Path(__file__).parent / 'video_gpu'))
from gdb_report import outcome, records_from


def result(code, log, *, cycles=False, debugger_records=None):
    if debugger_records is not None:
        status = outcome(code, debugger_records)
        if status['status'] != 'PASS':
            return status
    elif code != 0:
        return dict(status='NOT RUN' if code == 77 else 'FAIL', reason=f'application exit={code}')
    for marker in ('gdk_gl_context_make_current() failed', 'No EGL configuration available',
                   'PIVRA_GPU_FAIL', 'PIVRA_GPU_SHADER_FAIL', 'PIVRA_GPU_CONTEXT_FAILURE'):
        if marker in log:
            return dict(status='FAIL', reason=f'context failure: {marker}')
    eos = [i for i, line in enumerate(log.splitlines()) if line.startswith('PIVRA_SLIDESHOW event=video-ended ')]
    if not eos:
        return dict(status='FAIL', reason='error-free video EOS was not observed')
    if cycles:
        stages = []
        for i, line in enumerate(log.splitlines()):
            match = re.match(r'SLIDESHOW_STAGE cycle=([123]) (corrupt-select|video-selected|photo-presented)(?: |$)', line)
            if match:
                stages.append((i, int(match[1]), match[2]))
        expected = [(n, event) for n in (1, 2, 3) for event in ('corrupt-select', 'video-selected', 'photo-presented')]
        if [(n, event) for _, n, event in stages] != expected:
            return dict(status='FAIL', reason='three complete ordered EOS/photo cycles not observed')
        for index in range(0, len(stages), 3):
            begin, _, _ = stages[index + 1]
            end, _, _ = stages[index + 2]
            if sum(begin < event < end for event in eos) != 1:
                return dict(status='FAIL', reason='each cycle requires one genuine EOS before photo presentation')
        if 'PASS slideshow EOS regression: 3 corrupt/video/photo cycles and independent close' not in log:
            return dict(status='FAIL', reason='EOS close/cleanup gate did not complete')
    return dict(status='PASS', reason='application completed; required EOS/presentation evidence present')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('log', type=Path)
    parser.add_argument('exit_code', type=int)
    parser.add_argument('--cycles', action='store_true')
    parser.add_argument('--gdb-events', type=Path)
    args = parser.parse_args()
    log = args.log.read_text(errors='replace')
    report = result(args.exit_code, log, cycles=args.cycles,
                    debugger_records=records_from(log, args.gdb_events) if args.gdb_events else None)
    libraries = {}
    for line in log.splitlines():
        if line.startswith('SLIDESHOW_LIBRARY '):
            path = Path(line.removeprefix('SLIDESHOW_LIBRARY '))
            try:
                libraries[str(path)] = dict(sha256=hashlib.sha256(path.read_bytes()).hexdigest())
            except OSError as error:
                libraries[str(path)] = dict(error=str(error))
    report['loaded_libraries'] = libraries
    args.log.with_suffix('.result.json').write_text(json.dumps(report, indent=2) + '\n')
    signal = report.get('inferior_result', {}).get('signal') if report.get('inferior_result') else None
    print(f'{report["status"]} slideshow: {report["reason"]}' + (f' ({signal})' if signal else ''), flush=True)
    return 0 if report['status'] == 'PASS' else 77 if report['status'] == 'NOT RUN' else 1


if __name__ == '__main__':
    raise SystemExit(main())
