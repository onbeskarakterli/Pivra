"""Harness tests only; subprocess mocks are not decoder evidence."""
import subprocess
import unittest
from unittest.mock import patch, Mock
import preview_pipeline_native as native


class Reports(unittest.TestCase):
    @patch.object(native.Path, "write_text")
    @patch.object(native.subprocess, "run")
    def test_success_stays_full_only(self, run, write):
        run.return_value = Mock(returncode=0)
        self.assertEqual(native.main(), 0)
        self.assertEqual(run.call_count, 6)
        self.assertIn("PASS full-only fallback", write.call_args.args[0])
        self.assertIn("NOT RUN: no early preview provider", write.call_args.args[0])

    @patch.object(native.Path, "write_text")
    @patch.object(native.subprocess, "run")
    def test_blocker_stops_without_repeating(self, run, write):
        run.return_value = Mock(returncode=77)
        self.assertEqual(native.main(), 77)
        self.assertEqual(run.call_count, 1)
        self.assertIn('"result": "NOT RUN"', write.call_args.args[0])

    @patch.object(native.Path, "write_text")
    @patch.object(native.subprocess, "run")
    def test_timeout_is_failure(self, run, write):
        run.side_effect = subprocess.TimeoutExpired("probe", 180)
        self.assertEqual(native.main(), 124)
        self.assertEqual(run.call_count, 1)
        self.assertIn('"result": "FAIL"', write.call_args.args[0])


if __name__ == "__main__":
    unittest.main()
