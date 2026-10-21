#!/usr/bin/env python3
"""Observer guard tests only; these never certify a real decoder or GUI PASS."""
import os
import io
import queue
import mmap
from pathlib import Path
import struct
import tempfile
import unittest
from unittest import mock
from types import SimpleNamespace
from contextlib import ExitStack
import itertools
import zlib

import active_decode_close as probe


class ObserverGuards(unittest.TestCase):
    def test_only_exact_prefix_with_unwritten_tail_is_a_candidate(self):
        with tempfile.TemporaryFile() as output:
            output.truncate(probe.SIZE)
            self.assertFalse(probe.partial(output.fileno()), 'zero allocation is not decode')
            os.pwrite(output.fileno(), probe.COLOR * 32, 0)
            self.assertTrue(probe.partial(output.fileno()))
            os.pwrite(output.fileno(), probe.COLOR * 32, probe.SIZE - 96)
            self.assertFalse(probe.partial(output.fileno()), 'completed output is not active')

    def test_wrong_pixel_format_or_source_is_rejected(self):
        with tempfile.TemporaryFile() as output:
            output.truncate(probe.SIZE)
            os.pwrite(output.fileno(), bytes([255]) * 96, 0)
            self.assertFalse(probe.partial(output.fileno()))

    def test_wrong_size_is_rejected(self):
        with tempfile.TemporaryFile() as output:
            output.truncate(probe.SIZE - 1)
            os.pwrite(output.fileno(), probe.COLOR * 32, 0)
            self.assertFalse(probe.partial(output.fileno()))

    def test_generated_rgb_png_crc_dimensions_and_scanlines(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / 'small.png'
            probe.png(path, width=8, height=3)
            data = path.read_bytes()
        self.assertEqual(data[:8], b'\x89PNG\r\n\x1a\n')
        chunks = []
        offset = 8
        compressed = b''
        while offset < len(data):
            size = struct.unpack('>I', data[offset:offset+4])[0]
            kind = data[offset+4:offset+8]
            value = data[offset+8:offset+8+size]
            crc = struct.unpack('>I', data[offset+8+size:offset+12+size])[0]
            self.assertEqual(crc, zlib.crc32(kind + value))
            chunks.append(kind)
            if kind == b'IHDR':
                self.assertEqual(struct.unpack('>IIBBBBB', value), (8, 3, 8, 2, 0, 0, 0))
            if kind == b'IDAT':
                compressed += value
            offset += size + 12
        self.assertEqual(chunks[0], b'IHDR')
        self.assertEqual(chunks[-1], b'IEND')
        self.assertEqual(zlib.decompress(compressed), (b'\0' + probe.COLOR * 8) * 3)

    def test_retained_observer_fd_is_not_enough_after_decoder_unmaps_or_closes(self):
        fd = os.memfd_create('glycin-frame', os.MFD_CLOEXEC)
        retained = os.dup(fd)
        entry = Path(f'/proc/{os.getpid()}/fd/{fd}')
        try:
            os.ftruncate(fd, probe.SIZE)
            os.pwrite(fd, probe.COLOR * 32, 0)
            with mmap.mmap(fd, probe.SIZE) as mapping:
                self.assertEqual(mapping[:3], probe.COLOR)
                self.assertTrue(probe.owned_partial(os.getpid(), entry, retained))
            self.assertTrue(probe.partial(retained))
            self.assertFalse(probe.owned_partial(os.getpid(), entry, retained), 'unmapped output is not active')
        finally:
            os.close(fd)
        try:
            self.assertTrue(probe.partial(retained))
            self.assertFalse(probe.owned_partial(os.getpid(), entry, retained), 'closed output is not active')
        finally:
            os.close(retained)

    def test_linux_process_identity_is_not_just_pid(self):
        state, birth = probe.stat(os.getpid())
        self.assertIn(state, 'RSDIT')
        self.assertGreater(birth, 0)
        self.assertNotIn(os.getpid(), probe.descendants(os.getpid()))


class DiagnosisAndRaceGuards(unittest.TestCase):
    def output(self):
        output = tempfile.TemporaryFile()
        self.addCleanup(output.close)
        output.truncate(probe.SIZE)
        os.pwrite(output.fileno(), probe.COLOR * 32, 0)
        return output.fileno()

    def checkpoint(self, frame, overrides=None):
        order = []
        defaults = {
            'signal.pidfd_send_signal': lambda *_: order.append('STOP'),
            'stopped': lambda _: True,
            'stat': lambda _: ('T', 123),
            'select.select': lambda *_: ([], [], []),
            'owned_partial': lambda *_: order.append('mapping') or True,
            'sandbox_evidence': lambda *_: order.append('sandbox') or {},
        }
        defaults.update(overrides or {})
        evidence = {}
        diagnostics = probe.Diagnostics(evidence)
        with ExitStack() as patches:
            for name, value in defaults.items():
                patches.enter_context(mock.patch('active_decode_close.' + name, side_effect=value))
            probe.stop_and_certify(99, 123, 88, Path('/proc/99/fd/7'), frame,
                                  SimpleNamespace(pid=os.getpid()), evidence, diagnostics,
                                  probe.frame_sample(frame))
        return order, evidence

    def test_twelve_good_prefix_bytes_are_insufficient(self):
        frame = self.output()
        os.pwrite(frame, bytes(96), 0)
        os.pwrite(frame, probe.COLOR * 4, 0)
        self.assertFalse(probe.partial(frame))

    def test_twelve_zero_tail_bytes_are_insufficient(self):
        frame = self.output()
        os.pwrite(frame, bytes([1]) * 84 + bytes(12), probe.SIZE - 96)
        self.assertFalse(probe.partial(frame))

    def test_conditions_from_different_samples_do_not_certify_partial(self):
        frame = self.output()
        evidence = {}
        diagnostics = probe.Diagnostics(evidence)
        for stage in ('baseline_decoded', 'target_starting', 'observer_armed', 'target_load_entered', 'loader_found',
                      'pidfd_pinned', 'memfd_found', 'target_frame_fresh'):
            diagnostics.mark(stage)
        os.pwrite(frame, probe.COLOR * 32, probe.SIZE - 96)
        self.assertFalse(diagnostics.sample(1, Path('fd'), frame))
        os.pwrite(frame, bytes(96), 0)
        os.pwrite(frame, bytes(96), probe.SIZE - 96)
        self.assertFalse(diagnostics.sample(1, Path('fd'), frame))
        self.assertEqual(diagnostics.missing(), 'partial_candidate')
        self.assertIn('prefix_correct', diagnostics.data['counts'])
        self.assertIn('tail_zero', diagnostics.data['counts'])
        self.assertIsNone(diagnostics.data['last_partial_sample'])

    def test_expensive_security_and_mapping_checks_are_after_stop(self):
        order, evidence = self.checkpoint(self.output())
        self.assertEqual(order, ['STOP', 'mapping', 'sandbox'])
        self.assertEqual(evidence['diagnostics']['last_sample']['phase'], 'stopped')
        self.assertIn('checkpoint_certified', evidence['diagnostics']['counts'])
        self.assertNotIn('result', evidence, 'unit checkpoint simulation is not a native gate PASS')

    def test_completion_between_candidate_and_stop_is_not_run(self):
        frame = self.output()
        def complete(*_):
            os.pwrite(frame, probe.COLOR * 32, probe.SIZE - 96)
        with self.assertRaisesRegex(probe.NotRun, 'STOP race'):
            self.checkpoint(frame, {'signal.pidfd_send_signal': complete})

    def test_reused_pid_is_not_run_even_with_partial_output(self):
        with self.assertRaisesRegex(probe.NotRun, 'identity changed'):
            self.checkpoint(self.output(), {'stat': lambda _: ('T', 999)})

    def test_missing_mapping_is_not_run(self):
        with self.assertRaisesRegex(probe.NotRun, 'owned/mapped'):
            self.checkpoint(self.output(), {'owned_partial': lambda *_: False})

    def test_missing_sandbox_still_fails_after_stop(self):
        def unsafe(*_):
            raise AssertionError('missing isolation')
        with self.assertRaisesRegex(AssertionError, 'missing isolation'):
            self.checkpoint(self.output(), {'sandbox_evidence': unsafe})

    def test_stop_permission_denial_is_not_a_certificate(self):
        def denied(*_):
            raise PermissionError('signal denied')
        with self.assertRaises(PermissionError):
            self.checkpoint(self.output(), {'signal.pidfd_send_signal': denied})

    def test_stop_timeout_does_not_accept_a_running_loader(self):
        clock = itertools.count()
        with mock.patch('active_decode_close.time.monotonic', side_effect=lambda: next(clock)), \
             mock.patch('active_decode_close.time.sleep'):
            with self.assertRaisesRegex(probe.NotRun, 'every loader thread'):
                self.checkpoint(self.output(), {'stopped': lambda _: False})

    def discovery_case(self, baseline=None, old_frame=False, returned=False):
        frame = self.output()
        baseline = baseline or probe.Baseline()
        if old_frame:
            info = os.fstat(frame)
            baseline.frame_ids.add((info.st_dev, info.st_ino))
        telemetry = queue.Queue()
        if returned:
            telemetry.put({'message': 'TARGET_LOAD_ENTERED', 'monotonic': 1.})
            telemetry.put({'message': 'TARGET_LOAD_RETURNED ok', 'monotonic': 2.})
        if old_frame:
            self.assertTrue(probe.partial(frame), 'old frame fixture must really match partial RGB')
        else:
            os.pwrite(frame, bytes(96), 0)
        reads = []
        real_sample = probe.frame_sample
        def progress(fd):
            reads.append(fd)
            if len(reads) == 4:
                os.pwrite(frame, probe.COLOR * 32, 0)
            return real_sample(fd)
        process = SimpleNamespace(pid=90000, poll=lambda: None)
        first, good = Path('/proc/99/fd/1'), Path('/proc/99/fd/7')
        armed = []
        evidence = {}
        diagnostics = probe.Diagnostics(evidence)
        def children(_):
            self.assertTrue(armed, 'observe must arm before process discovery')
            return {99}
        def link(path):
            if str(path) == '/proc/99/exe':
                return '/usr/lib/glycin-image-rs'
            if path == first:
                raise FileNotFoundError('transient descriptor')
            self.assertEqual(path, good)
            return '/memfd:glycin-frame (deleted)'
        with mock.patch('active_decode_close.descendants', side_effect=children) as discover, \
             mock.patch('active_decode_close.stat', return_value=('R', 123)), \
             mock.patch('active_decode_close.os.readlink', side_effect=link), \
             mock.patch('active_decode_close.Path.stat', return_value=SimpleNamespace(st_uid=os.getuid())), \
             mock.patch('active_decode_close.Path.iterdir', return_value=[first, good]), \
             mock.patch('active_decode_close.os.pidfd_open', side_effect=lambda _: os.dup(frame)), \
             mock.patch('active_decode_close.select.select', return_value=([], [], [])), \
             mock.patch('active_decode_close.os.open', side_effect=lambda *_: os.dup(frame)), \
             mock.patch('active_decode_close.signal.pidfd_send_signal'), \
             mock.patch('active_decode_close.frame_sample', side_effect=progress), \
             mock.patch.object(probe.Diagnostics, 'flush') as flush, \
             mock.patch('active_decode_close.stop_and_certify', side_effect=probe.NotRun('candidate reached')):
            reason = 'target load returned' if returned else 'candidate reached'
            with self.assertRaisesRegex(probe.NotRun, reason):
                probe.observe(process, baseline, evidence, diagnostics, arm=lambda: armed.append(True), telemetry=telemetry)
        flush.assert_not_called()
        self.assertEqual(len(reads), 0 if returned else 4)
        self.assertEqual(discover.call_count, 2, 'cached polling must not repeat tree scans')
        self.assertEqual(diagnostics.data['counts']['fd_discovery_race'], 1)
        self.assertEqual(diagnostics.data['counts']['memfd_found'], 1)
        self.assertEqual(diagnostics.data['counts']['pidfd_pinned'], 1)
        self.assertNotIn('checkpoint_certified', diagnostics.data['counts'])
        self.assertEqual(evidence['loader_discoveries'][0]['origin'], baseline.origin(99, 123))
        return evidence

    def test_disappearing_early_fd_does_not_hide_later_frame(self):
        self.discovery_case()

    def test_existing_loader_is_discovered_and_its_fresh_target_frame_is_examined(self):
        evidence = self.discovery_case(probe.Baseline(processes={99: 123}))
        self.assertEqual(evidence['candidate_loader_origin'], 'baseline')
        self.assertIsNone(evidence.get('reuse_observed'), 'discovery is not yet a certified reuse')

    def test_existing_loader_old_matching_partial_frame_is_rejected(self):
        evidence = self.discovery_case(probe.Baseline(processes={99: 123}), old_frame=True, returned=True)
        self.assertNotIn('candidate_loader_origin', evidence)
        self.assertNotIn('target_frame_fresh', evidence['diagnostics']['counts'])

    def test_new_loader_old_frame_is_also_rejected(self):
        evidence = self.discovery_case(old_frame=True, returned=True)
        self.assertNotIn('target_frame_fresh', evidence['diagnostics']['counts'])


class BaselineAndLifecycleGuards(unittest.TestCase):
    def test_origin_requires_pid_and_starttime(self):
        baseline = probe.Baseline(processes={12: 100})
        self.assertEqual(baseline.origin(12, 100), 'baseline')
        self.assertEqual(baseline.origin(12, 101), 'new')
        self.assertEqual(baseline.origin(13, 100), 'new')

    def test_snapshot_rejects_old_frame_even_if_it_later_grows_to_target_size(self):
        fd = os.memfd_create('glycin-frame', os.MFD_CLOEXEC)
        try:
            os.ftruncate(fd, 12)
            with mock.patch('active_decode_close.descendants', return_value=set()):
                baseline = probe.snapshot_baseline(os.getpid(), probe.Diagnostics({}))
            info = os.fstat(fd)
            self.assertIn((info.st_dev, info.st_ino), baseline.frame_ids)
            os.ftruncate(fd, probe.SIZE)
            os.pwrite(fd, probe.COLOR * 32, 0)
            self.assertTrue(probe.partial(fd))
            self.assertFalse(baseline.fresh(fd), 'old inode cannot become a fresh target by resizing')
        finally:
            os.close(fd)

    def test_bwrap_helper_is_an_identity_without_unnecessary_frame_fd_access(self):
        parent = os.getpid()
        actual_stat, actual_link, actual_iterdir = probe.stat, os.readlink, Path.iterdir
        accesses = []
        def entries(path):
            accesses.append(str(path))
            if str(path) == '/proc/99/fd':
                raise PermissionError('helper fd should never be inspected')
            return actual_iterdir(path)
        def link(path):
            return '/usr/bin/bwrap' if str(path) == '/proc/99/exe' else actual_link(path)
        with mock.patch('active_decode_close.descendants', return_value={99}), \
             mock.patch('active_decode_close.stat', side_effect=lambda pid: ('S', 123) if pid == 99 else actual_stat(pid)), \
             mock.patch('active_decode_close.os.readlink', side_effect=link), \
             mock.patch('active_decode_close.Path.iterdir', side_effect=entries, autospec=True):
            baseline = probe.snapshot_baseline(parent, probe.Diagnostics({}))
        self.assertEqual(baseline.origin(99, 123), 'baseline')
        self.assertNotIn('/proc/99/fd', accesses)

    def test_unreadable_baseline_does_not_allow_freshness_claim(self):
        with mock.patch('active_decode_close.descendants', return_value=set()), \
             mock.patch('active_decode_close.Path.iterdir', side_effect=PermissionError('denied')):
            with self.assertRaisesRegex(probe.NotRun, 'inventory'):
                probe.snapshot_baseline(os.getpid(), probe.Diagnostics({}))

    def test_fresh_complete_frame_is_still_not_active(self):
        with tempfile.TemporaryFile() as output:
            output.truncate(probe.SIZE)
            os.pwrite(output.fileno(), probe.COLOR * 32, 0)
            os.pwrite(output.fileno(), probe.COLOR * 32, probe.SIZE - 96)
            self.assertTrue(probe.Baseline().fresh(output.fileno()))
            self.assertFalse(probe.partial(output.fileno()))

    def test_old_completed_frame_does_not_become_target_activity(self):
        with tempfile.TemporaryFile() as output:
            output.truncate(probe.SIZE)
            os.pwrite(output.fileno(), probe.COLOR * 32, 0)
            os.pwrite(output.fileno(), probe.COLOR * 32, probe.SIZE - 96)
            info = os.fstat(output.fileno())
            baseline = probe.Baseline(frame_ids={(info.st_dev, info.st_ino)})
            self.assertFalse(baseline.fresh(output.fileno()))
            self.assertFalse(probe.partial(output.fileno()))

    def test_target_telemetry_is_not_mistaken_for_close_protocol(self):
        events, telemetry = queue.Queue(), queue.Queue()
        stream = io.StringIO('PIV09 TARGET_LOAD_ENTERED\nPIV09 TARGET_LOAD_RETURNED error=decode failed\nPIV09 WINDOW_CLOSED entered=true returned_at_checkpoint=false\n')
        with mock.patch('builtins.print'):
            probe.read_events(stream, events, telemetry)
        self.assertTrue(events.get_nowait().startswith('WINDOW_CLOSED'))
        self.assertEqual(events.get_nowait(), 'EOF')
        evidence = {}
        diagnostics = probe.Diagnostics(evidence)
        probe.drain_target_events(telemetry, evidence, diagnostics)
        self.assertEqual(diagnostics.data['counts']['target_load_entered'], 1)
        self.assertIn('error=decode failed', evidence['target_load_return'])

    def test_retained_baseline_process_is_not_itself_a_target_memory_leak(self):
        with tempfile.TemporaryFile() as output:
            output.truncate(64)
            identity = (os.fstat(output.fileno()).st_dev, os.fstat(output.fileno()).st_ino)
            self.assertTrue(probe.target_references(os.getpid(), identity))
            unrelated = (identity[0], identity[1] + 10**9)
            self.assertFalse(probe.target_references(os.getpid(), unrelated))

    def test_mapping_remains_a_reference_after_its_fd_closes(self):
        fd = os.memfd_create('glycin-frame', os.MFD_CLOEXEC)
        os.ftruncate(fd, 4096)
        identity = (os.fstat(fd).st_dev, os.fstat(fd).st_ino)
        memory = mmap.mmap(fd, 4096)
        os.close(fd)
        try:
            refs = probe.target_references(os.getpid(), identity)
            self.assertTrue(any('mapping' in item for item in refs))
        finally:
            memory.close()
        self.assertFalse(probe.target_references(os.getpid(), identity))

    def test_pid_reuse_does_not_keep_the_original_child_alive(self):
        with mock.patch('active_decode_close.stat', return_value=('R', 101)):
            self.assertFalse(probe.surviving_identities({12: 100}))
            self.assertEqual(probe.surviving_identities({12: 101}), {12: 101})


if __name__ == '__main__':
    unittest.main()
