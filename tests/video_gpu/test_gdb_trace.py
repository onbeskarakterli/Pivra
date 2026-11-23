"""Real debugger controls; EGL-named C stubs are not GPU acceptance evidence."""
import contextlib
import io
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile
import unittest

import lifecycle as runner
from gdb_report import records_from

class RealGdb(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if not shutil.which('gdb') or not shutil.which('cc'):
            raise unittest.SkipTest('NOT RUN real GDB controls: gdb/cc unavailable')
        cls.temp = tempfile.TemporaryDirectory(prefix='pivra-gdb-tests-')
        cls.root = Path(cls.temp.name)
        for suffix,flags in [('debug',['-g']),('minimal',[])]:
            subprocess.run([*shlex.split(os.environ.get('CC','cc')),'-O0','-Wall','-Wextra','-Werror',*flags,'tests/video_gpu/gdb_control.c','-o',str(cls.root/suffix)],cwd=runner.ROOT,check=True)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def controlled(self, mode=None, minimal=False, unmanaged=False):
        with tempfile.TemporaryDirectory(dir=self.root) as directory, contextlib.redirect_stdout(io.StringIO()):
            out = Path(directory)
            command = ['gdb','-nx','--quiet','--batch','--return-child-result']
            if unmanaged:
                command += ['-ex','set disable-randomization off','-ex','break eglDestroyContext','-ex','run']
            else:
                command += ['-x',str(runner.ROOT/'tests/video_gpu/context_trace.gdb')]
            command += ['--args',str(self.root/('minimal' if minimal else 'debug'))]
            if mode: command.append(mode)
            result = runner.run('control',command,os.environ.copy(),out,debugger=True)
            log = (out/'control.log').read_text()
            records = records_from(log,out/'control.gdb.jsonl')
            return result,log,records

    def test_observer_breakpoints_continue_to_real_exit(self):
        result,log,events = self.controlled()
        self.assertEqual(result['status'],'PASS')
        self.assertEqual(result['debugger_result']['inferior_result']['exit_code'],0)
        self.assertIn('CONTROL_AFTER_INTENTIONAL_BREAKPOINTS',log)
        requests=[e for e in events if e['kind']=='observer_breakpoint']
        self.assertEqual({e['symbol'] for e in requests},{'eglCreateContext','eglDestroyContext','eglInitialize','eglTerminate'})
        self.assertTrue(all(e['auto_continue'] for e in requests))
        destroy=next(e for e in requests if e['symbol']=='eglDestroyContext')
        self.assertEqual((destroy['display'],destroy['context']),('0x1111','0x2222'))

    def test_actual_abort_is_an_inferior_failure(self):
        result,_,_ = self.controlled('abort')
        self.assertEqual(result['status'],'FAIL')
        self.assertEqual(result['debugger_result']['inferior_result']['signal'],'SIGABRT')
        self.assertEqual(result['debugger_result']['application_status'],'SIGNAL')

    def test_actual_nonzero_exit_is_not_a_debugger_stop(self):
        result,_,_ = self.controlled('exit7')
        self.assertEqual(result['status'],'FAIL')
        self.assertEqual(result['debugger_result']['inferior_result']['exit_code'],7)

    def test_real_application_exit_255_remains_failure(self):
        result,_,_ = self.controlled('exit255')
        self.assertEqual(result['status'],'FAIL')
        self.assertEqual(result['debugger_result']['inferior_result']['exit_code'],255)

    def test_signal_trap_is_not_assumed_fatal_application_termination(self):
        result,_,_ = self.controlled('trap')
        self.assertEqual((result['status'],result['debugger_result']['application_status']),('BLOCKED','UNKNOWN'))

    def test_unmanaged_intentional_stop_is_blocked_not_application_failure(self):
        result,log,_ = self.controlled(unmanaged=True)
        self.assertEqual(result['status'],'BLOCKED')
        self.assertEqual(result['debugger_result']['application_status'],'UNKNOWN')
        self.assertNotEqual(result['exit'],0)
        self.assertNotIn('CONTROL_AFTER_INTENTIONAL_BREAKPOINTS',log)

    def test_minimal_symbols_use_explicit_abi_fallback(self):
        result,_,events = self.controlled(minimal=True)
        self.assertEqual(result['status'],'PASS')
        entry=next(e for e in events if e['kind']=='observer_breakpoint' and e['symbol']=='eglDestroyContext')
        if not entry['display_source'].startswith('linux-ABI-register:'):
            self.skipTest('ABI register fallback unavailable on this architecture')
        self.assertEqual((entry['display'],entry['context']),('0x1111','0x2222'))
        returned=next(e for e in events if e['kind']=='egl_return' and e['symbol']=='eglCreateContext')
        self.assertEqual(returned['return_value'],'0x2222')
