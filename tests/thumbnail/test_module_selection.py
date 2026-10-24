"""Native-entry driver regression using fake pkg-config/compiler, real Rust provider.
Factory stubs are not real GNOME generation evidence.
"""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[2]

class Selection(unittest.TestCase):
    def setUp(self):
        tmp=tempfile.TemporaryDirectory(prefix='pivra-th01-modules-')
        self.addCleanup(tmp.cleanup)
        self.root=Path(tmp.name);self.bin=self.root/'bin';self.bin.mkdir()
        self.log=self.root/'compilers.jsonl'
        self.program('pkg-config', '''import os,sys
if '--list-all' in sys.argv:
    if os.environ.get('FAIL_LIST'):sys.exit(2)
    for m in os.environ.get('MODULES','gnome-desktop-3.0,gnome-desktop-4').split(','):
        if m:print(m,'module')
    # Exceeds a pipe buffer. An early awk exit used to break the producer.
    for n in range(12000):print('unrelated-'+str(n),'description'*16)
elif '--cflags' in sys.argv:
    print('-DMODULE_4' if sys.argv[-1]=='gnome-desktop-4' else '-DMODULE_3')
else:sys.exit(2)
''')
        self.program('cc', '''import json,os,pathlib,sys
kind='4' if '-DMODULE_4' in sys.argv else '3'
with open(os.environ['COMPILE_LOG'],'a') as f:f.write(json.dumps(sys.argv[1:])+'\\n')
if kind in os.environ.get('FAIL_CC',''):sys.exit(1)
p=pathlib.Path(sys.argv[sys.argv.index('-o')+1])
p.write_text("#!/bin/sh\\nprintf '"+os.environ.get('ELIGIBLE','1')+"\\\\n'\\n")
p.chmod(0o700)
''')
        self.program('cargo','pass\n')
    def program(self,name,body):
        path=self.bin/name
        path.write_text('#!/usr/bin/env python3\n'+body)
        path.chmod(0o700)
    def run_script(self,**extra):
        env={**os.environ,'PATH':str(self.bin)+os.pathsep+os.environ['PATH'],
             'COMPILE_LOG':str(self.log),'XDG_CURRENT_DESKTOP':'GNOME',
             'XDG_CACHE_HOME':str(self.root/'cache'),**extra}
        result=subprocess.run(['bash','tests/run_thumbnail_provider_probe.sh'],cwd=ROOT,env=env,capture_output=True,text=True,timeout=10)
        self.assertEqual(result.returncode,0,result.stdout+result.stderr)
        compiled=[json.loads(line) for line in self.log.read_text().splitlines()] if self.log.exists() else []
        return result.stdout,result.stderr,compiled
    def test_two_modules_large_listing_no_sigpipe_selects_only_four(self):
        out,_,calls=self.run_script()
        self.assertIn('GNOME factory module: gnome-desktop-4\n',out)
        self.assertEqual(len(calls),1);self.assertIn('-DMODULE_4',calls[0])
        self.assertIn('factory=Eligible',out);self.assertIn('capability=CacheAndGenerate',out)
        self.assertIn('OUTCOME Missing',out)
    def test_unbuildable_four_falls_back_to_three(self):
        out,err,calls=self.run_script(FAIL_CC='4')
        self.assertIn('GNOME factory module: gnome-desktop-3.0\n',out)
        self.assertIn('not buildable: gnome-desktop-4',err);self.assertEqual(len(calls),2)
    def test_no_modules_is_controlled_cache_only(self):
        out,_,calls=self.run_script(MODULES='')
        self.assertEqual(calls,[]);self.assertIn('factory=NotConfigured',out)
        self.assertIn('capability=CacheOnly',out)
    def test_all_builds_fail_without_forcing_capability(self):
        out,_,calls=self.run_script(FAIL_CC='43')
        self.assertEqual(len(calls),2);self.assertIn('unavailable/unbuildable',out)
        self.assertNotIn('capability=CacheAndGenerate',out)
    def test_factory_false_is_observable_and_not_forced_true(self):
        out,_,_=self.run_script(ELIGIBLE='0')
        self.assertIn('factory=Ineligible',out);self.assertIn('mime: "image/png"',out)
        self.assertIn('generation_eligible=false',out);self.assertIn('capability=CacheOnly',out)
        self.assertIn('OUTCOME Missing',out)
    def test_listing_error_is_controlled_fallback(self):
        out,_,calls=self.run_script(FAIL_LIST='1')
        self.assertEqual(calls,[]);self.assertIn('factory=NotConfigured',out)

if __name__=='__main__':unittest.main()
