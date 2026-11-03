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


class TapParsingTests(unittest.TestCase):
    def test_actual_gtk_tap_listing(self):
        output = '''TAP version 14
# random seed: R02S1234567890
# Start of colorstate tests
# /colorstate/equal
# /colorstate/convert-dmabuf/srgb/srgb
# End of colorstate tests
1..2
'''
        self.assertEqual(baseline.registered_cases(output), [
            '/colorstate/equal', '/colorstate/convert-dmabuf/srgb/srgb'])

    def test_bare_paths_and_whitespace(self):
        self.assertEqual(baseline.registered_cases(
            '  # /colorstate/equal  \n/colorstate/cicp\n'),
            ['/colorstate/equal', '/colorstate/cicp'])

    def test_diagnostic_is_not_registered_case(self):
        self.assertEqual(baseline.registered_cases(
            '# failed /colorstate/equal\n# /colorstate/equal diagnostic\n'), [])

    def test_skip_directive_case_and_spacing(self):
        for directive in ('SKIP', 'skip', 'Skip'):
            for gap in (' ', ''):
                with self.subTest(directive=directive, gap=gap):
                    self.assertTrue(baseline.skipped(
                        f'ok 1 /colorstate/equal #{gap}{directive} unavailable\n'))

    def test_zero_test_skip_plan(self):
        self.assertTrue(baseline.skipped('1..0 # SKIP no display\n'))

    def test_pass_or_diagnostic_is_not_skip(self):
        self.assertFalse(baseline.skipped('ok 1 /colorstate/equal\n'))
        self.assertFalse(baseline.skipped('# SKIP mentioned in a diagnostic\n'))
        self.assertFalse(baseline.skipped('ok 1 /colorstate/equal # skippedness\n'))

    def test_skip_cannot_become_successful_baseline(self):
        exit_code = 0
        if baseline.skipped('ok 1 /colorstate/equal # skip unsupported\n'):
            exit_code = None
        self.assertIn('INCONCLUSIVE', baseline.classify(exit_code, 0))
