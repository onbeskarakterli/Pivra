"""Runner guards, including actual Xvfb last-client-disconnect regression."""
import importlib.util
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

SCRIPT=Path(__file__).parent/'performance/v026_acceptance.py'
spec=importlib.util.spec_from_file_location('v026_acceptance',SCRIPT)
runner=importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)

class AcceptanceRunner(unittest.TestCase):
    def rejected(self,args):
        child=subprocess.run([sys.executable,'-B',str(SCRIPT),*args],capture_output=True,text=True)
        self.assertEqual(child.returncode,2)
        return child.stderr

    def test_native_mode_rejects_software_permission(self):
        self.assertIn('Native acceptance refuses',self.rejected(['--software']))

    def test_headless_mode_requires_explicit_software_label(self):
        self.assertIn('require --software',self.rejected(['--session','x11']))

    @unittest.skipUnless(shutil.which('Xvfb'),'NOT RUN Xvfb unavailable')
    def test_pointer_survives_last_client_disconnect(self):
        with tempfile.TemporaryDirectory() as temp:
            with runner.session('x11',os.environ.copy(),Path(temp)) as env:
                self.assertEqual(runner.outside_pointer(env),[1279,1023])
                self.assertEqual(runner.outside_pointer(env,move=False),[1279,1023])

    def test_native_catalog_requires_secure_mode_and_preserves_real_provider(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)
            env={'XDG_CURRENT_DESKTOP':'GNOME','PIVRA_THUMBNAIL_GENERATOR':'/trusted/real-helper'}
            command,actual,kind=runner.catalog_case(root,env,root,native=True)
            self.assertEqual(command,[str(root/'thumbnail_overlay_probe'),'--catalog-regression','--native-thumbnails'])
            self.assertEqual(actual,env)
            self.assertEqual(env['PIVRA_THUMBNAIL_GENERATOR'],'/trusted/real-helper')
            self.assertIn('secure-decoder',kind)
            self.assertFalse((root/'config').exists())

    def test_controlled_catalog_refuses_generation_even_if_host_can_generate(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)
            env={'XDG_CURRENT_DESKTOP':'GNOME','PIVRA_THUMBNAIL_GENERATOR':'/trusted/real-helper'}
            command,actual,kind=runner.catalog_case(root,env,root,native=False)
            self.assertNotIn('--native-thumbnails',command)
            self.assertEqual(env['PIVRA_THUMBNAIL_GENERATOR'],'/trusted/real-helper')
            self.assertEqual(actual['XDG_CURRENT_DESKTOP'],'GNOME')
            child=subprocess.run([actual['PIVRA_THUMBNAIL_GENERATOR'],'--generate','file:///test','image/png','1'],env={**os.environ,**actual},capture_output=True,text=True)
            self.assertEqual(child.returncode,4)
            self.assertIn('CONTROLLED catalog',child.stderr)
            self.assertIn('synthetic',kind)
