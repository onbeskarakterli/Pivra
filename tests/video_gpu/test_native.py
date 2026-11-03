import importlib.util
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('native', Path(__file__).with_name('native.py'))
native = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native)


class IsolationTests(unittest.TestCase):
    def test_removes_cpu_and_injection_environment(self):
        with patch.dict(os.environ, {**dict.fromkeys(native.CPU_ENV, 'bad'),
                                    'LD_PRELOAD': 'bad', 'LD_LIBRARY_PATH': 'bad'}):
            env = native.environment(Path('/private/lib'))
        self.assertEqual(env['LD_LIBRARY_PATH'], '/private/lib')
        for key in (*native.CPU_ENV, 'LD_PRELOAD'):
            self.assertNotIn(key, env)

    def test_stock_does_not_inherit_private_sdk(self):
        self.assertEqual(native.environment(None)['LD_LIBRARY_PATH'], '')

    def test_dead_process_does_not_prove_library_loaded(self):
        self.assertEqual(native.mapped_gtk(999999999), [])

    def test_control_failure_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            result = native.command(['/bin/sh', '-c', 'exit 23'], os.environ.copy(),
                                    Path(directory) / 'log')
        self.assertEqual(result['exit'], 23)

    def test_control_timeout_is_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            result = native.command(['/bin/sleep', '1'], os.environ.copy(),
                                    Path(directory) / 'log', timeout=0.01)
        self.assertEqual(result['status'], 'FAIL timeout')
