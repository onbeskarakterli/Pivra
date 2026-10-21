#!/usr/bin/env python3
"""Linux-only real secure decoder close gate. Never relaxes sandbox policy.

Pins its own descendant loader with pidfd, observes a partially written glycin-frame
memfd, briefly SIGSTOPs/rechecks it, then closes the real production view and resumes.
Exit 77 = BLOCKED/NOT RUN, including missing observation rights or missed timing.
"""
import argparse
import contextlib
import hashlib
import json
import os
from pathlib import Path
import queue
import select
import signal
import struct
import subprocess
import tempfile
import threading
import time
import zlib

WIDTH, HEIGHT = 8000, 4000  # 32 MP: the existing production limit, RGB8 = 96 MB.
SIZE = WIDTH * HEIGHT * 3
COLOR = bytes((17, 83, 191))


class NotRun(Exception):
    pass


def png(path, width=WIDTH, height=HEIGHT):
    def chunk(stream, kind, data):
        stream.write(struct.pack('>I', len(data)) + kind + data
                     + struct.pack('>I', zlib.crc32(kind + data)))
    with path.open('wb') as stream:
        stream.write(b'\x89PNG\r\n\x1a\n')
        chunk(stream, b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
        compressor = zlib.compressobj(6)
        row = b'\0' + COLOR * width
        for _ in range(height):
            data = compressor.compress(row)
            if data:
                chunk(stream, b'IDAT', data)
        chunk(stream, b'IDAT', compressor.flush())
        chunk(stream, b'IEND', b'')
    assert path.stat().st_size < 64 * 1024 * 1024


def stat(pid):
    # comm can contain spaces/parentheses. starttime disambiguates PID reuse.
    data = Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()
    return data[0], int(data[19])


def descendants(pid):
    result, pending = set(), [pid]
    while pending:
        parent = pending.pop()
        try:
            tasks = list(Path(f'/proc/{parent}/task').iterdir())
        except FileNotFoundError:
            continue
        for task in tasks:
            try:
                children = (task / 'children').read_text().split()
            except FileNotFoundError:
                continue
            for child in map(int, children):
                if child not in result:
                    result.add(child)
                    pending.append(child)
    return result


def partial(fd):
    # All source RGB channels are nonzero. Prefix written + untouched zero tail
    # proves read_image output is incomplete, not a merely allocated idle frame.
    return (os.fstat(fd).st_size == SIZE
            and os.pread(fd, 96, 0) == COLOR * 32
            and os.pread(fd, 96, SIZE - 96) == bytes(96))


def stopped(pid):
    tasks = list(Path(f'/proc/{pid}/task').iterdir())
    return bool(tasks) and all(stat(int(t.name))[0] == 'T' for t in tasks)


def sandbox_evidence(pid, parent):
    values = {}
    for line in Path(f'/proc/{pid}/status').read_text().splitlines():
        key, _, value = line.partition(':')
        values[key] = value.strip()
    assert values['NoNewPrivs'] == '1', 'loader missing NoNewPrivs'
    assert values['Seccomp'] == '2', 'loader missing seccomp filter'
    assert int(values['CapEff'], 16) == 0, 'loader has effective capabilities'
    namespaces = {}
    for name in ('user', 'mnt', 'pid', 'net'):
        child = os.readlink(f'/proc/{pid}/ns/{name}')
        host = os.readlink(f'/proc/{parent}/ns/{name}')
        assert child != host, ('loader namespace not isolated', name)
        namespaces[name] = {'loader': child, 'probe': host}
    return {'NoNewPrivs': values['NoNewPrivs'], 'Seccomp': values['Seccomp'],
            'CapEff': values['CapEff'], 'namespaces': namespaces}


def send(process, command):
    process.stdin.write(command + '\n')
    process.stdin.flush()


def read_events(stream, events):
    for line in stream:
        print(line.rstrip(), flush=True)
        if line.startswith('PIV09 '):
            events.put(line.strip()[6:])
    events.put('EOF')


def expect(events, prefix, timeout=15):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            event = events.get(timeout=min(0.1, max(0.001, deadline - time.monotonic())))
        except queue.Empty:
            continue
        if event.startswith('BLOCKED'):
            raise NotRun(event)
        assert event != 'EOF', 'probe exited before expected event: ' + prefix
        assert event.startswith(prefix), (prefix, event)
        return
    raise AssertionError('probe event deadline: ' + prefix)


def run(binary, report):
    evidence = {'result': 'NOT RUN', 'decoder_time_overlap': 'NOT MEASURED',
                'method': 'owned descendant pidfd; partial RGB memfd; SIGSTOP/recheck/close/SIGCONT',
                'width': WIDTH, 'height': HEIGHT, 'output_bytes': SIZE}
    started = time.monotonic()
    process = None
    held = None
    frame = None
    readers = []
    try:
        evidence['driver_sha256'] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
        evidence['binary_sha256'] = hashlib.sha256(binary.read_bytes()).hexdigest()
        if not hasattr(os, 'pidfd_open') or not hasattr(signal, 'pidfd_send_signal'):
            raise NotRun('Linux pidfd APIs required; no PID-based signal fallback')
        with tempfile.TemporaryDirectory(prefix='pivra-active-decode-') as temporary:
            image = Path(temporary) / 'active-rgb.png'
            png(image)
            process = subprocess.Popen([str(binary), str(image)], stdin=subprocess.PIPE,
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                       text=True, bufsize=1)
            events = queue.Queue()
            errors = []
            def stderr():
                for line in process.stderr:
                    errors.append(line)
                    print(line.rstrip(), flush=True)
            readers = [threading.Thread(target=read_events, args=(process.stdout, events), daemon=True),
                       threading.Thread(target=stderr, daemon=True)]
            for reader in readers:
                reader.start()
            expect(events, 'READY survivor_loaded')
            baseline = descendants(process.pid)
            observed = {}
            send(process, 'START')
            expect(events, 'TARGET_STARTED')
            deadline = time.monotonic() + 15
            permission_errors = set()
            while time.monotonic() < deadline and held is None:
                for pid in descendants(process.pid) - baseline:
                    try:
                        state, birth = stat(pid)
                        observed[pid] = birth
                        executable = os.readlink(f'/proc/{pid}/exe')
                        if Path(executable.removesuffix(' (deleted)')).name != 'glycin-image-rs':
                            continue
                        if Path(f'/proc/{pid}').stat().st_uid != os.getuid():
                            continue
                        for entry in Path(f'/proc/{pid}/fd').iterdir():
                            if os.readlink(entry) != '/memfd:glycin-frame (deleted)':
                                continue
                            with contextlib.ExitStack() as handles:
                                candidate = os.open(entry, os.O_RDONLY | os.O_CLOEXEC)
                                handles.callback(os.close, candidate)
                                if not partial(candidate):
                                    continue
                                pidfd = os.pidfd_open(pid)
                                handles.callback(os.close, pidfd)
                                # Recheck identity and membership before any signal.
                                if stat(pid)[1] != birth or pid not in descendants(process.pid):
                                    continue
                                held = (pid, birth, pidfd)
                                frame = candidate
                                handles.pop_all()  # resources now owned by the outer finally
                                evidence['sandbox'] = sandbox_evidence(pid, process.pid)
                                evidence['stop_requested_monotonic'] = time.monotonic()
                                signal.pidfd_send_signal(pidfd, signal.SIGSTOP)
                                stop_deadline = time.monotonic() + 2
                                while not stopped(pid) and time.monotonic() < stop_deadline:
                                    time.sleep(0.001)
                                if not stopped(pid) or not partial(frame):
                                    raise NotRun('partial-frame race missed; cannot certify active read_image at close')
                                evidence['stop_confirmed_monotonic'] = time.monotonic()
                                evidence.update(loader_pid=pid, loader_starttime=birth,
                                                executable=executable, output_prefix_hex=os.pread(frame, 96, 0).hex(),
                                                output_tail_hex=os.pread(frame, 96, SIZE-96).hex(),
                                                active_checkpoint='all threads stopped; prefix exact RGB; tail zero; worker not yet returned')
                                print('EVIDENCE real loader mid-frame, incomplete output pinned:', pid, flush=True)
                                break
                    except (FileNotFoundError, ProcessLookupError):
                        continue
                    except PermissionError as error:
                        if held:
                            raise NotRun('cannot stop/inspect owned loader: ' + str(error)) from error
                        permission_errors.add(str(error))
                    if held:
                        break
                if held is None:
                    if process.poll() is not None:
                        raise AssertionError('probe exited while awaiting active loader')
                    time.sleep(0.001)
            if held is None or 'active_checkpoint' not in evidence:
                raise NotRun('no certifiable partial glycin-image-rs RGB frame; loader too fast, '
                             'different implementation/layout, or proc observation denied: '
                             + '; '.join(sorted(permission_errors)))
            pause = time.monotonic()
            evidence['close_requested_monotonic'] = pause
            send(process, 'CLOSE')
            expect(events, 'WINDOW_CLOSED entered=true returned_at_checkpoint=false', timeout=5)
            # Remove the observer's read-only reference before cleanup checks.
            os.close(frame)
            frame = None
            try:
                signal.pidfd_send_signal(held[2], signal.SIGCONT)
            except ProcessLookupError:
                pass  # cancellation may already have terminated the stopped loader
            evidence['close_request_to_resume_seconds'] = time.monotonic() - pause
            evidence['resumed_monotonic'] = time.monotonic()
            cleanup_started = time.monotonic()
            expect(events, 'SETTLED load_returned=true job_joined=true widgets_released=true stale_mutations=0 survivor_pixels_unchanged=true')
            evidence['worker_settled_monotonic'] = time.monotonic()
            deadline = pause + 10
            assert time.monotonic() < deadline, 'worker settlement exceeded 10 seconds from close'
            while True:
                remaining = []
                for pid, birth in observed.items():
                    try:
                        if stat(pid)[1] == birth:
                            remaining.append(pid)  # includes unreaped zombies
                    except FileNotFoundError:
                        pass
                new_children = descendants(process.pid) - baseline
                if not remaining and not new_children:
                    break
                assert time.monotonic() < deadline, ('loader/helper cleanup timed out', remaining, new_children)
                time.sleep(0.01)
            assert select.select([held[2]], [], [], 0)[0], 'pinned loader has not exited'
            evidence['cleanup_seconds'] = time.monotonic() - cleanup_started
            evidence['close_to_cleanup_seconds'] = time.monotonic() - pause
            assert evidence['close_to_cleanup_seconds'] < 10
            evidence['observed_child_identities'] = observed
            send(process, 'FINISH')
            expect(events, 'PASS active_decode_close')
            assert process.wait(timeout=5) == 0
            for reader in readers:
                reader.join(timeout=1)
            assert not errors, 'unexpected stderr: ' + ''.join(errors)
            evidence['result'] = 'PASS'
            return 0
    except (NotRun, PermissionError) as error:
        evidence['reason'] = str(error)
        print('BLOCKED/NOT RUN:', error, flush=True)
        return 77
    except Exception as error:
        evidence.update(result='FAIL', reason=str(error))
        print('FAIL:', error, flush=True)
        return 1
    finally:
        if frame is not None:
            os.close(frame)
        if held:
            try:
                signal.pidfd_send_signal(held[2], signal.SIGCONT)
            except OSError as error:
                print('Cleanup SIGCONT unavailable:', error, flush=True)
            os.close(held[2])
        if process is not None and process.poll() is None:
            process.terminate()  # failed probe cleanup, never a normal-close PASS
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
        evidence['elapsed_seconds'] = time.monotonic() - started
        report.write_text(json.dumps(evidence, indent=2) + '\n')
        print('Report:', report, flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/examples/active_decode_close_probe'))
    parser.add_argument('--report', type=Path, default=Path('/tmp/pivra-active-decode-close.json'))
    args = parser.parse_args()
    if not __debug__:
        parser.error('assertions are required: do not use -O or PYTHONOPTIMIZE')
    raise SystemExit(run(args.binary.resolve(), args.report.resolve()))


if __name__ == '__main__':
    main()
