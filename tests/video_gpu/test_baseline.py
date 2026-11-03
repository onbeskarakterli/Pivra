from pathlib import Path
import signal
import sys
import unittest

sys.path.insert(0, str(Path(__file__).parent))
import baseline


class ClassificationTests(unittest.TestCase):
    def test_both_sigbus_does_not_exonerate_patch(self):
        result = baseline.classify(-signal.SIGBUS, -signal.SIGBUS)
        self.assertIn('BASELINE REPRODUCIBLE', result)
        self.assertIn('not excluded', result)

    def test_patched_only_blocks(self):
        self.assertIn('integration blocked', baseline.classify(0, -signal.SIGBUS))

    def test_missing_or_skipped_baseline_is_inconclusive(self):
        self.assertIn('INCONCLUSIVE', baseline.classify(None, -signal.SIGBUS))

    def test_other_signal_is_not_sigbus(self):
        self.assertIn('INCONCLUSIVE', baseline.classify(-signal.SIGSEGV, -signal.SIGBUS))

    def test_success_is_not_a_fix(self):
        self.assertIn('NOT REPRODUCED', baseline.classify(0, 0))
