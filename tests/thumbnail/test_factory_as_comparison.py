"""Experiment controls; real kernel VM limits, stub GNOME API, no JPEG decode."""
import copy
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest

from factory_as_comparison import ROOT, LIMIT_LINE, diagnostic_source, classify
from test_generation_helper import HEADER, STUB


class Comparison(unittest.TestCase):
    def test_only_address_space_initializer_changes(self):
        source = (ROOT/'native/thumbnail_factory.c').read_text()
        self.assertEqual(diagnostic_source(source, 2048), source)
        lower = diagnostic_source(source, 1024)
        self.assertEqual(lower.replace('struct rlimit memory={1024UL*1024*1024,1024UL*1024*1024};', LIMIT_LINE), source)
        for invalid in (0, 4096, -1):
            with self.assertRaises(ValueError):
                diagnostic_source(source, invalid)
        with self.assertRaises(ValueError):
            diagnostic_source(source.replace(LIMIT_LINE, ''), 2048)

    def test_evidence_requires_repeat_failure_success_and_real_limits(self):
        trials = [dict(limit_mib=m, observed_as=m*1024**2, exit=e, timeout=False,
                       allocation_error=e==4, generated=e==0, sandbox_command=True)
                  for m, e in [(1024, 4), (2048, 0), (1024, 4)]]
        self.assertTrue(classify(trials, True).startswith('EVIDENCE:'))
        self.assertEqual(classify(trials, False), 'NOT PROVEN')
        self.assertEqual(classify(trials[:2], True), 'NOT PROVEN')
        for index, key, value in [(0,'allocation_error',False), (2,'exit',0),
                                  (1,'generated',False), (1,'timeout',True),
                                  (1,'observed_as',1024**3), (0,'sandbox_command',False)]:
            bad = copy.deepcopy(trials)
            bad[index][key] = value
            self.assertEqual(classify(bad, True), 'NOT PROVEN')

    def test_kernel_virtual_mapping_control_aba(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            header = root/'libgnome-desktop/gnome-desktop-thumbnail.h'
            header.parent.mkdir()
            header.write_text(HEADER)
            # GNOME stub checks actual installed limits and reserves virtual
            # address space PROT_NONE: no 1.2GiB physical allocation or decoder.
            stub = '#include <sys/mman.h>\n#include <errno.h>\n' + STUB
            stub = stub.replace('2048UL*1024*1024', 'EXPECTED_MIB*1024UL*1024')
            stub = stub.replace('(void)f;(void)m;(void)c;', '''(void)f;(void)m;(void)c;
 void *reservation=mmap(NULL,1200UL*1024*1024,PROT_NONE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
 if(reservation==MAP_FAILED) {g_set_error_literal(e,G_IO_ERROR,G_IO_ERROR_FAILED,"controlled mmap ENOMEM");return NULL;}
 munmap(reservation,1200UL*1024*1024);''')
            (root/'stub.c').write_text(stub)
            flags = shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs',
                     'gio-2.0','gobject-2.0','gdk-pixbuf-2.0'], text=True))
            source = root/'source.jpg'
            source.write_bytes(b'controlled API input, not a decoded JPEG')
            for index, mib in enumerate((1024,2048,1024)):
                c_source = root/f'helper-{index}.c'
                c_source.write_text(diagnostic_source((ROOT/'native/thumbnail_factory.c').read_text(), mib))
                binary = root/f'helper-{index}'
                subprocess.run(['cc','-O2','-Wall','-Wextra','-Werror',f'-DEXPECTED_MIB={mib}',
                                '-I'+str(root),str(c_source),str(root/'stub.c'),'-o',str(binary),*flags],
                               check=True, capture_output=True, timeout=15)
                import os
                result = subprocess.run([str(binary),source.as_uri(),'image/jpeg',
                        str(source.stat().st_mtime_ns//10**9),'missing'],
                        env={**os.environ,'STAMP':str(root/f'cache-{index}'), 'PIVRA_TRACE_GNOME_THUMB':'1'},
                        capture_output=True,text=True,timeout=5)
                self.assertEqual(result.returncode, 0 if mib==2048 else 4, result.stderr)
                self.assertIn(f'limits_as={mib*1024**2}', result.stderr)
                if mib==2048:
                    self.assertEqual(result.stdout,'generated\n')
                else:
                    self.assertIn('controlled mmap ENOMEM', result.stdout)


if __name__ == '__main__':
    unittest.main()
