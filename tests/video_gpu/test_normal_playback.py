"""Normal evidence requirements remain distinct from historical observations."""
import os
from pathlib import Path
import subprocess
import sys
import unittest
from normal_playback import GPU_MARKERS, PRODUCT_MARKERS, check_normal, acceptance

class NormalPlayback(unittest.TestCase):
    def test_exit_zero_without_gpu_or_controls_evidence_is_failure(self):
        result=check_normal(dict(status='PASS',reasons=[]),'BACKEND selected=GtkGstMediaFile')
        self.assertEqual(result['status'],'FAIL')
        self.assertIn('normal GPU evidence missing',result['reasons'][0])

    def test_actual_gpu_markers_do_not_hide_failure_or_not_run(self):
        log='\n'.join(GPU_MARKERS)
        self.assertEqual(check_normal(dict(status='PASS',reasons=[]),log)['status'],'PASS')
        for status in ('FAIL','NOT RUN','BLOCKED'):
            self.assertEqual(check_normal(dict(status=status,reasons=[]),log)['status'],status)

    def test_old_failure_is_preserved_without_masking_current_or_baseline_failure(self):
        old=dict(status='FAIL',role='pre-fix-reference')
        good=dict(status='PASS')
        self.assertEqual(acceptance([old,good,good]),0)
        self.assertEqual(old['status'],'FAIL')
        self.assertEqual(acceptance([]),77)
        self.assertEqual(acceptance([old]),77)
        self.assertEqual(acceptance([old,dict(status='FAIL')]),1)
        self.assertEqual(acceptance([old,dict(status='NOT RUN')]),77)

    def test_manual_product_requires_actual_gpu_draw_and_backend(self):
        self.assertEqual(check_normal(dict(status='PASS',reasons=[]),'PIVRA_GPU_STOCK type=GtkGstMediaFile',PRODUCT_MARKERS)['status'],'FAIL')
        self.assertEqual(check_normal(dict(status='PASS',reasons=[]),'\n'.join(PRODUCT_MARKERS),PRODUCT_MARKERS)['status'],'PASS')

    def test_headless_control_needs_explicit_software_label(self):
        child=subprocess.run([sys.executable,'-B',str(Path(__file__).with_name('normal_playback.py')),'--session','sway'],capture_output=True,text=True)
        self.assertEqual(child.returncode,2)
        self.assertIn('require --software',child.stderr)

    def test_stock_override_cannot_be_reported_as_gpu_acceptance(self):
        child=subprocess.run([sys.executable,'-B',str(Path(__file__).with_name('normal_playback.py'))],env={**os.environ,'PIVRA_VIDEO_BACKEND':'stock'},capture_output=True,text=True)
        self.assertEqual(child.returncode,2)
        self.assertIn('unset PIVRA_VIDEO_BACKEND',child.stderr)
