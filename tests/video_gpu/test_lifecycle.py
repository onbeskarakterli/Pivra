"""Runner integrity: real child exits/logs, not GPU or shader acceptance."""
import contextlib
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
