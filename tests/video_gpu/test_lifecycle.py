"""Runner integrity: real child exits/logs, not GPU or shader acceptance."""
import contextlib
import hashlib
import importlib.util
import io
import os
from pathlib import Path
import sys
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('context_lifecycle', Path(__file__).with_name('lifecycle.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)

class LifecycleRunner(unittest.TestCase):
    def run_child(self, code, **kwargs):
        with tempfile.TemporaryDirectory() as d, contextlib.redirect_stdout(io.StringIO()):
            return runner.run('child', [sys.executable, '-c', code], os.environ.copy(), Path(d), **kwargs)

    def test_success_and_explicit_not_run(self):
        self.assertEqual(self.run_child('pass')['status'], 'PASS')
        self.assertEqual(self.run_child('raise SystemExit(77)')['status'], 'NOT RUN')

    def test_failure_exit_is_preserved(self):
        r = self.run_child('raise SystemExit(9)')
        self.assertEqual((r['status'], r['exit']), ('FAIL', 9))

    def test_logged_context_failure_cannot_pass_on_exit_zero(self):
        r = self.run_child('print("gdk_gl_context_make_current() failed")')
        self.assertEqual(r['status'], 'FAIL')
        self.assertEqual(r['context_errors'], 1)

    def test_context_restore_failure_cannot_pass_on_exit_zero(self):
        r = self.run_child('print("PIVRA_GPU_CONTEXT_FAILURE stage=restore-previous")')
        self.assertEqual(r['status'], 'FAIL')
        self.assertEqual(r['context_failures'], 1)

    def test_native_abort_and_last_transition_preserved(self):
        r = self.run_child('import os,resource,signal; resource.setrlimit(resource.RLIMIT_CORE,(0,0)); print("LIFECYCLE_STAGE unknown stock route begin",flush=True); os.kill(os.getpid(),signal.SIGABRT)')
        self.assertEqual((r['status'],r['exit']), ('FAIL',-6))
        self.assertEqual(r['last_stage'],'LIFECYCLE_STAGE unknown stock route begin')

    def test_debugger_signal_is_distinct_from_wrapper_exit(self):
        r = self.run_child('print("GDB_INFERIOR_SIGNAL SIGABRT"); raise SystemExit(134)')
        self.assertEqual(r['inferior_signals'], ['SIGABRT'])
        self.assertEqual(r['exit'],134)

    def test_debugger_abort_without_inferior_result_is_not_app_abort(self):
        r = self.run_child('import os,resource,signal;resource.setrlimit(resource.RLIMIT_CORE,(0,0));os.kill(os.getpid(),signal.SIGABRT)', debugger=True)
        self.assertEqual((r['status'],r['debugger_result']['application_status']),('BLOCKED','UNKNOWN'))
        self.assertEqual((r['exit_scope'],r['debugger_exit']),('debugger',-6))

    def test_confirmed_inferior_exit_does_not_hide_gl_warnings(self):
        message='GDB_EVENT {"schema":1,"kind":"inferior_result","state":"exited","exit_code":0}'
        r = self.run_child('print('+repr(message)+'); print("gdk_gl_context_make_current() failed")', debugger=True)
        self.assertEqual(r['status'],'FAIL')
        self.assertEqual(r['debugger_result']['application_status'],'PASS')

    def test_expected_context_error_count_is_exact(self):
        self.assertEqual(self.run_child('print("PIVRA_GPU_CONTEXT_FAILURE injected")', expected_context_failures=1)['status'],'PASS')
        self.assertEqual(self.run_child('print("gdk_gl_context_make_current() failed"); print("PIVRA_GPU_CONTEXT_FAILURE injected")', expected_context_failures=1)['status'],'FAIL')

    def test_library_provenance_hashes_the_observed_file(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'library.so'; path.write_bytes(b'provenance fixture')
            result = self.run_child('print('+repr('LIFECYCLE_LIBRARY '+str(path))+')')
            self.assertEqual(result['libraries_sha256'][str(path)], hashlib.sha256(b'provenance fixture').hexdigest())

    def test_cap_exhaustion_is_explicit(self):
        r = self.run_child('print(\'{"pivra_perf":1,"event":"poll"}\\n\'*4096,end="")')
        self.assertEqual(r['status'], 'FAIL')
        self.assertIn('diagnostic cap reached', r['reasons'])

    def test_expected_error_injection_must_match_exactly(self):
        self.assertEqual(self.run_child('print("PIVRA_GPU_FAIL injected")', expected_gpu_errors=1)['status'], 'PASS')
        self.assertEqual(self.run_child('pass', expected_gpu_errors=1)['status'], 'FAIL')

    def test_timeout_is_failure(self):
        r = self.run_child('import time;time.sleep(10)', timeout=.1)
        self.assertEqual((r['status'], r['exit']), ('FAIL', 124))

if __name__ == '__main__':
    unittest.main()
