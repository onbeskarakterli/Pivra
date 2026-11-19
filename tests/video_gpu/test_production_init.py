import os
from pathlib import Path
import subprocess
import sys
import unittest
from production_init import verify


class ProductionInitEvidence(unittest.TestCase):
    def test_exit_zero_is_not_proof_of_recovery(self):
        row = verify(dict(status='PASS', reasons=[]), '', ['actual rendered frame', 'stock replacement'])
        self.assertEqual(row['status'], 'FAIL')
        self.assertIn('actual rendered frame', row['reasons'][0])

    def test_evidence_cannot_hide_failure_or_skip(self):
        for status in ['FAIL', 'NOT RUN', 'BLOCKED', 'PASS']:
            self.assertEqual(verify(dict(status=status, reasons=[]), 'frame', ['frame'])['status'], status)

    def test_headless_cannot_be_mistaken_for_hardware(self):
        p = subprocess.run([sys.executable, '-B', str(Path(__file__).with_name('production_init.py')), '--session', 'sway'], capture_output=True, text=True)
        self.assertEqual(p.returncode, 2)
        self.assertIn('require --software', p.stderr)

    def test_stock_override_cannot_mask_gpu_failure(self):
        p = subprocess.run([sys.executable, '-B', str(Path(__file__).with_name('production_init.py'))],
                           env={**os.environ, 'PIVRA_VIDEO_BACKEND': 'stock'}, capture_output=True, text=True)
        self.assertEqual(p.returncode, 2)
        self.assertIn('unset PIVRA_VIDEO_BACKEND', p.stderr)
