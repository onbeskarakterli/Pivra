import mmap,os,struct,tempfile,unittest
from pathlib import Path
import benchmark as b
class Observer(unittest.TestCase):
    def test_pid_starttime_and_parent(self):
        pid=os.getpid();row=b.processes()[pid];self.assertEqual(row[0],os.getppid());self.assertGreater(int(row[1]),0)
    def test_descendants_do_not_include_unrelated_processes(self):
        table={1:(0,'1','S'),2:(1,'2','S'),3:(2,'3','S'),4:(9,'4','S')}
        self.assertEqual(b.descendants(1,table),{1,2,3})
    def test_partial_requires_correct_size_prefix_zero_tail_and_writable_mapping(self):
        fd=os.memfd_create('glycin-frame',os.MFD_CLOEXEC)
        try:
            os.ftruncate(fd,96)
            with mmap.mmap(fd,96) as m:
                self.assertFalse(b.partial(os.getpid(),96))
                m[:12]=bytes.fromhex('1153bf')*4
                self.assertTrue(b.partial(os.getpid(),96))
                self.assertFalse(b.partial(os.getpid(),97))
                m[84:]=bytes.fromhex('1153bf')*4
                self.assertFalse(b.partial(os.getpid(),96))
                m[84:]=bytes(12);m[:12]=bytes([1])*12
                self.assertFalse(b.partial(os.getpid(),96))
                m[:12]=bytes.fromhex('1153bf')*4
            self.assertFalse(b.partial(os.getpid(),96),'closed writable mapping is not active evidence')
        finally:os.close(fd)
    def test_window_pass_requires_certificate_ack_loader_exit_and_finish(self):
        cert={'partial_after_stop':True,'cancel_acknowledged':True,'loader_exited_while_other_window_alive':True,'other_window_verified':True}
        self.assertTrue(b.window_protocol_complete(cert,True))
        self.assertFalse(b.window_protocol_complete(None,True))
        self.assertFalse(b.window_protocol_complete(cert,False))
        for key in cert:
            incomplete={**cert,key:False}
            self.assertFalse(b.window_protocol_complete(incomplete,True),key)
    def test_pool_observation_cutoff_does_not_classify_pid_as_failure(self):
        self.assertEqual(b.loader_exit_progress(False,5),'observing_pool_ownership')
        self.assertEqual(b.loader_exit_progress(False,29.999),'observing_pool_ownership')
        self.assertEqual(b.loader_exit_progress(False,30),'collecting_exit_diagnostics')
        self.assertEqual(b.loader_exit_progress(False,34.999),'collecting_exit_diagnostics')
        with self.assertRaisesRegex(b.PoolObservationIncomplete,'ownership unresolved'):
            b.loader_exit_progress(False,35)
        self.assertEqual(b.loader_exit_progress(True,35),'exited')
        cert={'partial_after_stop':True,'cancel_acknowledged':True}
        self.assertFalse(b.window_protocol_complete(cert,False),'pool waiting is never PASS')
    def test_real_pidfd_must_signal_process_exit(self):
        import select,subprocess,sys
        child=subprocess.Popen([sys.executable,'-c','import sys;sys.stdin.read()'],stdin=subprocess.PIPE)
        fd=os.pidfd_open(child.pid)
        try:
            self.assertFalse(select.select([fd],[],[],0)[0])
            self.assertNotEqual(b.loader_exit_progress(False,5),'exited')
            child.stdin.close();child.wait(timeout=5)
            self.assertTrue(select.select([fd],[],[],0)[0])
            self.assertEqual(b.loader_exit_progress(True,0),'exited')
        finally:
            if child.poll() is None:child.kill();child.wait()
            os.close(fd)
    def test_observer_failure_is_visible_in_terminal_message(self):
        r={'exit':1,'observer_failure':'loader still alive while other window open'}
        self.assertEqual(b.failure_message(r),'FAIL FD06 active close: loader still alive while other window open')
        self.assertIsNone(b.failure_message({'exit':0}))
        self.assertIn('NOT RUN',b.failure_message({'exit':77,'stderr':'NOT RUN checkpoint absent'}))
        self.assertIn('other window render pixels changed',b.failure_message({'exit':1,'stderr':'FAIL FD06 active close: other window render pixels changed'}))
    def test_process_observation_checks_identity_and_keeps_idle_distinct(self):
        pid=os.getpid();start=b.processes()[pid][1]
        info=b.observe_loader(pid,start,123)
        self.assertEqual(info['identity'],'matched')
        self.assertIn('cpu_ticks',info)
        self.assertFalse(info['partial_target_frame'])
        self.assertEqual(b.observe_loader(pid,'wrong-starttime',123),{'identity':'exited_or_changed'})
        message=b.failure_message({'exit':77,'observer_failure':'live pooled loader ownership unresolved'})
        self.assertTrue(message.startswith('NOT RUN'))
    def test_streamed_fixture_has_exact_header(self):
        with tempfile.TemporaryDirectory() as d:
            path=Path(d)/'source.png';b.fixture(path,64,32);data=path.read_bytes()
            self.assertEqual(data[:8],b'\x89PNG\r\n\x1a\n');self.assertEqual(struct.unpack('>II',data[16:24]),(64,32));self.assertEqual(data[25],2)
if __name__=='__main__':unittest.main()
