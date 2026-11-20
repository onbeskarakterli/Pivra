import unittest
from wayland_config import NAMES, SUCCESS, classify


def rows(**counts):
    return [dict(name=name, ok=True, error=SUCCESS, count=counts.get(name, 0)) for name in NAMES]


class WaylandConfigEvidence(unittest.TestCase):
    def test_missing_or_duplicate_observations_are_incomplete(self):
        self.assertEqual(classify(rows()[:-1]), 'INCOMPLETE')
        self.assertEqual(classify(rows()+[rows()[0]]), 'INCOMPLETE')

    def test_egl_error_cannot_be_called_missing_es1(self):
        data = rows(explicit_gl=8)
        data[0].update(ok=False, error=0x3008)
        self.assertEqual(classify(data), 'EGL_QUERY_ERROR')

    def test_query_size_difference_is_not_an_api_mismatch(self):
        self.assertEqual(classify(rows(gtk_sdr_size_zero=4, explicit_gl=4)), 'QUERY_SIZE_DIVERGENCE')

    def test_explicit_es1_control_must_match(self):
        self.assertEqual(classify(rows(explicit_es1=3, explicit_gl=5)), 'IMPLICIT_ES1_CONTROL_MISMATCH')

    def test_modern_only_driver_exposes_implicit_filter(self):
        for name in ('explicit_es2', 'explicit_gl'):
            self.assertEqual(classify(rows(**{name: 4}, any_api=8)), 'IMPLICIT_ES1_FILTER_MISMATCH')

    def test_matching_configs_do_not_prove_playback_fixed(self):
        self.assertEqual(classify(rows(gtk_sdr_implicit=8, gtk_sdr_size_zero=8, explicit_es1=8, explicit_gl=8)), 'GTK_SDR_CONFIG_FAILURE_NOT_REPRODUCED')

    def test_alpha_and_api_failures_are_distinct(self):
        self.assertEqual(classify(rows(any_api_no_alpha=3)), 'RGBA8_REQUIREMENT_MISMATCH')
        self.assertEqual(classify(rows(any_api=3)), 'NO_EXPLICIT_MODERN_API_MATCH')
        self.assertEqual(classify(rows()), 'NO_MATCHING_WINDOW_RGB_CONFIG')
