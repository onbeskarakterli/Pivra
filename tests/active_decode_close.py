#!/usr/bin/env python3
"""Linux-only real secure decoder close gate. Never relaxes sandbox policy.

Pins its own descendant loader with pidfd, observes a partially written glycin-frame
memfd, briefly SIGSTOPs/rechecks it, then closes the real production view and resumes.
Exit 77 = BLOCKED/NOT RUN, including missing observation rights or missed timing.
"""
import argparse
from dataclasses import dataclass, field
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


@dataclass
class Baseline:
    processes: dict = field(default_factory=dict)  # PID -> starttime, never PID alone
    frame_ids: set = field(default_factory=set)  # (device, inode), all pre-target sizes
    frames: list = field(default_factory=list)

    def origin(self, pid, birth):
        return 'baseline' if self.processes.get(pid) == birth else 'new'

    def fresh(self, fd):
        info = os.fstat(fd)
        return info.st_size == SIZE and (info.st_dev, info.st_ino) not in self.frame_ids


def snapshot_baseline(parent, diagnostics):
    baseline = Baseline()
    for pid in sorted(descendants(parent) | {parent}):
        try:
            _, birth = stat(pid)
            baseline.processes[pid] = birth
            # Helpers are inventory identities, not frame producers. Avoid
            # unnecessary fd access to non-dumpable bwrap supervisors.
            if pid != parent:
                executable = os.readlink(f'/proc/{pid}/exe')
                if Path(executable.removesuffix(' (deleted)')).name != 'glycin-image-rs':
                    continue
            # Include the probe's retained old textures as well as loader buffers.
            for entry in Path(f'/proc/{pid}/fd').iterdir():
                try:
                    if os.readlink(entry) != '/memfd:glycin-frame (deleted)':
                        continue
                    info = os.stat(entry)
                    baseline.frame_ids.add((info.st_dev, info.st_ino))
                    baseline.frames.append({'pid': pid, 'starttime': birth, 'fd': entry.name,
                                            'device': info.st_dev, 'inode': info.st_ino, 'size': info.st_size})
                except FileNotFoundError:
                    diagnostics.mark('baseline_fd_disappeared', pid=pid, fd=entry.name)
        except FileNotFoundError:
            diagnostics.mark('baseline_process_disappeared', pid=pid)
        except PermissionError as error:
            # Without a complete readable inventory we cannot certify freshness.
            raise NotRun('cannot inventory pre-target processes/frames: ' + str(error)) from error
    diagnostics.mark('baseline_snapshot', processes=len(baseline.processes), frames=len(baseline.frame_ids))
    return baseline


def drain_target_events(events, evidence, diagnostics):
    if events is None:
        return
    while True:
        try:
            event = events.get_nowait()
        except queue.Empty:
            break
        evidence.setdefault('target_events', []).append(event)
        if event['message'] == 'TARGET_LOAD_ENTERED':
            diagnostics.mark('target_load_entered')
        elif event['message'].startswith('TARGET_LOAD_RETURNED'):
            diagnostics.mark('target_load_returned', message=event['message'])
            evidence['target_load_return'] = event['message']


def target_references(pid, identity):
    # A cached observer fd is closed before this check. Check both FDs and maps:
    # an mmap can keep a frame alive even after its descriptor has been closed.
    device, inode = identity
    references = []
    try:
        for entry in Path(f'/proc/{pid}/fd').iterdir():
            try:
                info = os.stat(entry)
                if (info.st_dev, info.st_ino) == identity:
                    references.append({'pid': pid, 'fd': entry.name})
            except FileNotFoundError:
                continue
        for line in Path(f'/proc/{pid}/maps').read_text().splitlines():
            fields = line.split(maxsplit=5)
            major, minor = (int(value, 16) for value in fields[3].split(':'))
            if int(fields[4]) == inode and (major, minor) == (os.major(device), os.minor(device)):
                references.append({'pid': pid, 'mapping': line})
    except FileNotFoundError:
        pass  # process gone, including its mappings
    return references


def surviving_identities(identities):
    remaining = {}
    for pid, birth in identities.items():
        try:
            if stat(pid)[1] == birth:
                remaining[pid] = birth  # includes unreaped zombies
        except FileNotFoundError:
            pass
    return remaining


def frame_sample(fd):
    began = time.monotonic()
    before = os.fstat(fd)
    prefix = os.pread(fd, 96, 0)
    tail = os.pread(fd, 96, SIZE - 96) if before.st_size == SIZE else b''
    after = os.fstat(fd)
    return {'sample_begin_monotonic': began, 'sample_end_monotonic': time.monotonic(),
            'size': before.st_size, 'inode': before.st_ino, 'device': before.st_dev,
            'size_correct': before.st_size == after.st_size == SIZE,
            'prefix_correct': prefix == COLOR * 32,
            'tail_zero': tail == bytes(96),
            'prefix_hex': prefix.hex(), 'tail_hex': tail.hex()}


def partial(fd):
    sample = frame_sample(fd)
    return all(sample[name] for name in ('size_correct', 'prefix_correct', 'tail_zero'))


class Diagnostics:
    def __init__(self, evidence):
        self.emitted = set()
        self.data = evidence.setdefault('diagnostics', {
            'counts': {}, 'first_events': {}, 'first_sample': None,
            'last_sample': None, 'last_partial_sample': None, 'errors': []})

    def mark(self, stage, **details):
        counts = self.data['counts']
        counts[stage] = counts.get(stage, 0) + 1
        if stage not in self.data['first_events']:
            event = {'monotonic': time.monotonic(), **details}
            self.data['first_events'][stage] = event
            # Flush outside the candidate-to-SIGSTOP path; stdout can block.

    def flush(self):
        for stage, event in self.data['first_events'].items():
            if stage not in self.emitted:
                print('DIAG', stage, json.dumps(event, sort_keys=True), flush=True)
                self.emitted.add(stage)

    def error(self, operation, error):
        self.mark('proc_error', operation=operation, error=str(error))
        if len(self.data['errors']) < 16:
            self.data['errors'].append({'operation': operation, 'error': str(error)})

    def sample(self, pid, entry, fd, phase="running"):
        return self.record_sample(pid, entry, frame_sample(fd), phase)

    def record_sample(self, pid, entry, raw, phase='running'):
        sample = {'pid': pid, 'fd': str(entry), 'phase': phase, **raw}
        previous = self.data['last_sample']
        if previous is not None:
            gap = max(0., raw['sample_begin_monotonic'] - previous['sample_end_monotonic'])
            self.data['max_sample_gap_seconds'] = max(self.data.get('max_sample_gap_seconds', 0.), gap)
        self.mark('frame_sampled')
        if self.data['first_sample'] is None:
            self.data['first_sample'] = sample
        self.data['last_sample'] = sample
        for key in ('size_correct', 'prefix_correct', 'tail_zero'):
            if sample[key]:
                self.mark(key, pid=pid, fd=str(entry))
        if all(sample[key] for key in ('size_correct', 'prefix_correct', 'tail_zero')):
            self.data['last_partial_sample'] = sample
            self.mark('partial_candidate', pid=pid, fd=str(entry))
            return True
        return False

    def missing(self):
        # Independent per-sample successes never certify a combined checkpoint.
        for stage in ('baseline_decoded', 'target_starting', 'observer_armed', 'target_load_entered', 'loader_found', 'pidfd_pinned', 'memfd_found', 'target_frame_fresh', 'size_correct',
                      'prefix_correct', 'tail_zero', 'partial_candidate', 'stop_complete',
                      'fd_owned', 'mapping_verified', 'sandbox_verified', 'checkpoint_certified',
                      'window_closed_worker_pending_verified', 'job_joined_widgets_released',
                      'target_resources_released', 'children_cleaned'):
            if not self.data['counts'].get(stage):
                return stage
        return 'none'


def owned_partial(pid, entry, frame, diagnostics=None):
    # A retained observer FD is not evidence of a live decoder mapping.
    try:
        owner = os.stat(entry)
        observed = os.fstat(frame)
        if (owner.st_dev, owner.st_ino) != (observed.st_dev, observed.st_ino):
            return False
        if os.readlink(entry) != '/memfd:glycin-frame (deleted)' or not partial(frame):
            return False
        if diagnostics:
            diagnostics.mark('fd_owned', pid=pid, inode=owner.st_ino)
        for line in Path(f'/proc/{pid}/maps').read_text().splitlines():
            fields = line.split(maxsplit=5)
            if len(fields) < 6 or fields[1] != 'rw-s' or int(fields[4]) != owner.st_ino:
                continue
            start, end = (int(value, 16) for value in fields[0].split('-'))
            if end - start >= SIZE and fields[5] == '/memfd:glycin-frame (deleted)':
                if diagnostics:
                    diagnostics.mark('mapping_verified', pid=pid, mapping=line)
                return True
    except FileNotFoundError:
        return False
    return False


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


def stop_and_certify(pid, birth, pidfd, entry, frame, process, evidence, diagnostics, candidate_sample):
    # PID was pinned while a verified same-UID descendant, BEFORE frame sampling.
    # No recursive process scan or namespace reads on the critical pre-STOP path.
    evidence['stop_requested_monotonic'] = time.monotonic()
    try:
        signal.pidfd_send_signal(pidfd, signal.SIGSTOP)
    except OSError as error:
        evidence['failed_stage'] = 'sigstop'
        diagnostics.error('SIGSTOP', error)
        raise
    finally:
        diagnostics.record_sample(pid, entry, candidate_sample, phase="candidate_before_stop")
    diagnostics.mark('stop_requested', pid=pid)
    deadline = time.monotonic() + 2
    while not stopped(pid) and time.monotonic() < deadline:
        time.sleep(0.0002)
    if not stopped(pid):
        raise NotRun('SIGSTOP did not stop every loader thread within 2 seconds')
    evidence['stop_confirmed_monotonic'] = time.monotonic()
    diagnostics.mark('stop_complete', pid=pid)
    if stat(pid)[1] != birth or select.select([pidfd], [], [], 0)[0]:
        raise NotRun('pinned identity changed or exited at stopped checkpoint')
    if not diagnostics.sample(pid, entry, frame, phase="stopped"):
        failed = [name for name in ('size_correct', 'prefix_correct', 'tail_zero')
                  if not diagnostics.data['last_sample'][name]]
        evidence['failed_stage'] = 'stopped_' + failed[0]
        raise NotRun('STOP race: stopped sample failed ' + ', '.join(failed))
    if not owned_partial(pid, entry, frame, diagnostics):
        evidence['failed_stage'] = 'stopped_ownership_or_mapping'
        raise NotRun('stopped partial frame no longer owned/mapped writable-shared by loader')
    # Security conditions are unchanged; checking AFTER STOP removes this I/O
    # from the candidate-to-signal race without weakening any acceptance gate.
    evidence['sandbox'] = sandbox_evidence(pid, process.pid)
    diagnostics.mark('sandbox_verified', pid=pid)
    evidence.update(loader_pid=pid, loader_starttime=birth,
                    loader_output_fd=entry.name, output_inode=os.fstat(frame).st_ino,
                    output_prefix_hex=os.pread(frame, 96, 0).hex(),
                    output_tail_hex=os.pread(frame, 96, SIZE - 96).hex(),
                    active_checkpoint='all threads stopped; owned writable shared partial RGB frame; Rust worker checkpoint still required')
    diagnostics.mark('checkpoint_certified', pid=pid)


def observe(process, baseline, evidence, diagnostics, arm, timeout=15, telemetry=None):
    # Cache each loader pidfd and frame descriptor. Only the known output FD is
    # sampled in the hot path; discovery has its own cadence and stable PID order.
    loaders, frames, observed = {}, {}, {}
    inventoried = set()
    held = None
    frame = None
    transferred = False
    deadline = time.monotonic() + timeout
    next_discovery = 0.
    try:
        diagnostics.mark('observer_armed')
        arm()  # TARGET cannot spawn until this barrier is released.
        while time.monotonic() < deadline:
            now = time.monotonic()
            if now >= next_discovery:
                diagnostics.mark('process_scan')
                for pid in sorted(descendants(process.pid)):
                    try:
                        state, birth = stat(pid)
                        observed[pid] = birth
                        if pid in loaders:
                            continue
                        executable = os.readlink(f'/proc/{pid}/exe')
                        if (pid, birth) not in inventoried:
                            inventoried.add((pid, birth))
                            evidence.setdefault('process_inventory', []).append({
                                'pid': pid, 'starttime': birth, 'state': state, 'executable': executable,
                                'origin': baseline.origin(pid, birth)})
                        if Path(executable.removesuffix(' (deleted)')).name != 'glycin-image-rs':
                            continue
                        if Path(f'/proc/{pid}').stat().st_uid != os.getuid():
                            diagnostics.mark('loader_uid_rejected', pid=pid, executable=executable)
                            continue
                        origin = baseline.origin(pid, birth)
                        diagnostics.mark('loader_found', pid=pid, starttime=birth, executable=executable, origin=origin)
                        diagnostics.mark('loader_found_' + origin, pid=pid, starttime=birth)
                        evidence.setdefault('loader_discoveries', []).append({
                            'pid': pid, 'starttime': birth, 'origin': origin, 'executable': executable})
                        pin = os.pidfd_open(pid)
                        try:
                            # Repeat identity/descendant verification ONCE when pinning.
                            if stat(pid)[1] != birth or pid not in descendants(process.pid):
                                continue
                            loaders[pid] = (birth, pin, executable)
                            pin = None
                            diagnostics.mark('pidfd_pinned', pid=pid, starttime=birth)
                        finally:
                            if pin is not None:
                                os.close(pin)
                    except (FileNotFoundError, ProcessLookupError):
                        diagnostics.mark('discovery_race', pid=pid)
                    except PermissionError as error:
                        diagnostics.error('loader discovery', error)
                # Scan FDs until a frame is found; handle disappearing FDs
                # individually so an unrelated transient FD cannot hide later ones.
                for pid, (_, pin, _) in sorted(loaders.items()):
                    if select.select([pin], [], [], 0)[0] or any(key[0] == pid for key in frames):
                        continue
                    try:
                        entries = sorted(Path(f'/proc/{pid}/fd').iterdir(), key=lambda p: int(p.name))
                    except FileNotFoundError:
                        continue
                    except PermissionError as error:
                        diagnostics.error('fd directory', error)
                        continue
                    for entry in entries:
                        try:
                            if os.readlink(entry) != '/memfd:glycin-frame (deleted)':
                                continue
                            fd = os.open(entry, os.O_RDONLY | os.O_CLOEXEC)
                            diagnostics.mark('memfd_found', pid=pid, fd=str(entry))
                            try:
                                if not baseline.fresh(fd):
                                    info = os.fstat(fd)
                                    diagnostics.mark('pretarget_or_wrong_size_frame_rejected', pid=pid,
                                                     device=info.st_dev, inode=info.st_ino, size=info.st_size)
                                    continue
                                diagnostics.mark('target_frame_fresh', pid=pid,
                                                 origin=baseline.origin(pid, loaders[pid][0]))
                                frames[(pid, entry)] = fd
                                fd = None
                                break
                            finally:
                                if fd is not None:
                                    os.close(fd)
                        except (FileNotFoundError, ProcessLookupError):
                            diagnostics.mark('fd_discovery_race', pid=pid, fd=str(entry))
                        except PermissionError as error:
                            diagnostics.error('frame descriptor', error)
                # The single target frame is now the only hot-path candidate.
                next_discovery = time.monotonic() + 0.002 if not frames else float('inf')
            drain_target_events(telemetry, evidence, diagnostics)
            if evidence.get('target_load_return'):
                raise NotRun('target load returned before active checkpoint: ' + evidence['target_load_return'])
            for (pid, entry), cached in list(frames.items()):
                birth, pin, executable = loaders[pid]
                if select.select([pin], [], [], 0)[0]:
                    diagnostics.mark('loader_exited', pid=pid)
                    continue
                sample = frame_sample(cached)
                if not all(sample[key] for key in ('size_correct', 'prefix_correct', 'tail_zero')):
                    diagnostics.record_sample(pid, entry, sample)
                    continue
                held = (pid, birth, os.dup(pin))
                frame = os.dup(cached)
                evidence['executable'] = executable
                evidence['candidate_loader_origin'] = baseline.origin(pid, birth)
                info = os.fstat(cached)
                evidence['target_frame_identity'] = {'device': info.st_dev, 'inode': info.st_ino}
                assert baseline.fresh(cached), 'pre-target frame cannot certify target decode'
                stop_and_certify(pid, birth, held[2], entry, frame, process, evidence, diagnostics, sample)
                transferred = True
                return held, frame, observed
            drain_target_events(telemetry, evidence, diagnostics)
            if evidence.get('target_load_return'):
                raise NotRun('target load returned before active checkpoint: ' + evidence['target_load_return'])
            # Keep stdout (including a potentially blocked pipe/terminal) out
            # of the entire live sampling interval. run() flushes on completion.
            if process.poll() is not None:
                raise AssertionError('probe exited while observing loader')
            if loaders and all(select.select([pin], [], [], 0)[0] for _, pin, _ in loaders.values()):
                raise NotRun('all pinned loaders exited before checkpoint; first uncompleted stage: ' + diagnostics.missing())
            time.sleep(0.0002)
        raise NotRun('observation deadline; first uncompleted stage: ' + diagnostics.missing())
    except (FileNotFoundError, ProcessLookupError, PermissionError) as error:
        diagnostics.error('observe/checkpoint', error)
        raise NotRun('loader observation interrupted at ' + diagnostics.missing() + ': ' + str(error)) from error
    finally:
        for fd in frames.values():
            os.close(fd)
        for _, pin, _ in loaders.values():
            os.close(pin)
        if not transferred:
            if held:
                try:
                    signal.pidfd_send_signal(held[2], signal.SIGCONT)
                except ProcessLookupError:
                    pass
                finally:
                    os.close(held[2])
            if frame is not None:
                os.close(frame)


def send(process, command):
    process.stdin.write(command + '\n')
    process.stdin.flush()


def read_events(stream, events, telemetry):
    for line in stream:
        print(line.rstrip(), flush=True)
        if line.startswith('PIV09 '):
            message = line.strip()[6:]
            if message.startswith('TARGET_LOAD_'):
                telemetry.put({'monotonic': time.monotonic(), 'message': message})
            else:
                events.put(message)
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
                'width': WIDTH, 'height': HEIGHT, 'output_bytes': SIZE,
                'candidate_loader_origin': 'not_observed', 'certified_loader_origin': 'not_certified',
                'reuse_observed': None, 'observer_version': 3}
    started = time.monotonic()
    diagnostics = Diagnostics(evidence)
    process = None
    held = None
    frame = None
    readers = []
    telemetry = queue.Queue()
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
            readers = [threading.Thread(target=read_events, args=(process.stdout, events, telemetry), daemon=True),
                       threading.Thread(target=stderr, daemon=True)]
            for reader in readers:
                reader.start()
            expect(events, 'READY survivor_loaded')
            diagnostics.mark('baseline_decoded')
            send(process, 'START')
            expect(events, 'TARGET_STARTING')
            diagnostics.mark('target_starting')
            baseline = snapshot_baseline(process.pid, diagnostics)
            evidence['baseline_processes'] = baseline.processes
            evidence['baseline_frames'] = baseline.frames
            evidence['baseline_snapshot_monotonic'] = time.monotonic()
            held, frame, observed = observe(process, baseline, evidence, diagnostics,
                                             arm=lambda: send(process, 'ARMED'), telemetry=telemetry)
            pause = time.monotonic()
            evidence['close_requested_monotonic'] = pause
            send(process, 'CLOSE')
            expect(events, 'WINDOW_CLOSED entered=true returned_at_checkpoint=false', timeout=5)
            diagnostics.mark('window_closed_worker_pending_verified')
            evidence['certified_loader_origin'] = evidence['candidate_loader_origin']
            evidence['reuse_observed'] = evidence['certified_loader_origin'] == 'baseline'
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
            diagnostics.mark('job_joined_widgets_released')
            evidence['worker_settled_monotonic'] = time.monotonic()
            deadline = pause + 10
            assert time.monotonic() < deadline, 'worker settlement exceeded 10 seconds from close'
            identity = evidence['target_frame_identity']
            frame_identity = (identity['device'], identity['inode'])
            # Pooled baseline loaders may remain alive. First prove the target
            # frame is released, then close the survivor/app normally. Final child
            # exit remains mandatory within the original 10s-from-close budget.
            while True:
                current = descendants(process.pid)
                for pid in current:
                    try:
                        observed[pid] = stat(pid)[1]
                    except FileNotFoundError:
                        pass
                refs = []
                pinned_live = set(surviving_identities({held[0]: held[1]}))
                for pid in current | {process.pid} | pinned_live:
                    refs.extend(target_references(pid, frame_identity))
                if not refs:
                    break
                assert time.monotonic() < deadline, ('target frame cleanup timed out', refs)
                time.sleep(0.01)
            diagnostics.mark('target_resources_released')
            live = surviving_identities(observed)
            evidence['pooled_processes_alive_before_normal_app_close'] = [
                {'pid': pid, 'starttime': birth, 'origin': baseline.origin(pid, birth)}
                for pid, birth in live.items()]
            send(process, 'FINISH')
            expect(events, 'PASS active_decode_close')
            assert process.wait(timeout=max(0.001, deadline - time.monotonic())) == 0
            while surviving_identities(observed):
                assert time.monotonic() < deadline, ('children remained after normal app exit', surviving_identities(observed))
                time.sleep(0.01)
            assert select.select([held[2]], [], [], 0)[0], 'pinned loader has not exited'
            evidence['cleanup_seconds'] = time.monotonic() - cleanup_started
            evidence['close_to_cleanup_seconds'] = time.monotonic() - pause
            assert evidence['close_to_cleanup_seconds'] < 10
            evidence['observed_child_identities'] = observed
            diagnostics.mark('children_cleaned')
            for reader in readers:
                reader.join(timeout=1)
            assert not errors, 'unexpected stderr: ' + ''.join(errors)
            evidence['result'] = 'PASS'
            return 0
    except (NotRun, PermissionError) as error:
        evidence['reason'] = str(error)
        evidence['first_uncompleted_stage'] = diagnostics.missing()
        print('BLOCKED/NOT RUN:', error, flush=True)
        return 77
    except Exception as error:
        evidence.update(result='FAIL', reason=str(error), first_uncompleted_stage=diagnostics.missing())
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
        drain_target_events(telemetry, evidence, diagnostics)
        if evidence['result'] != 'PASS':
            evidence['first_uncompleted_stage'] = diagnostics.missing()
        evidence['elapsed_seconds'] = time.monotonic() - started
        diagnostics.flush()
        print('DIAG summary', json.dumps(diagnostics.data['counts'], sort_keys=True), flush=True)
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
