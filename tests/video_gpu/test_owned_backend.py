import importlib.util
from pathlib import Path
import sys
import unittest

folder = Path(__file__).parent / 'backend'
sys.path.insert(0, str(folder))
spec = importlib.util.spec_from_file_location('owned_run', folder / 'run.py')
owned = importlib.util.module_from_spec(spec)
spec.loader.exec_module(owned)


class SourcePolicyTests(unittest.TestCase):
    def setUp(self):
        self.video = dict(codec_type='video', color_primaries='bt709',
                          color_transfer='bt709', color_space='bt709',
                          color_range='tv', pix_fmt='yuv420p')

    def test_explicit_full_and_limited_are_eligible(self):
        self.assertTrue(owned.eligible(self.video))
        self.video['color_range'] = 'pc'
        self.assertTrue(owned.eligible(self.video))

    def test_every_missing_metadata_field_rejects(self):
        for key in self.video:
            with self.subTest(key=key):
                v = self.video.copy()
                del v[key]
                self.assertFalse(owned.eligible(v))

    def test_hdr_srgb_and_other_primaries_reject(self):
        for transfer in ('smpte2084', 'arib-std-b67', 'iec61966-2-1', 'unknown'):
            self.assertFalse(owned.eligible({**self.video, 'color_transfer': transfer}))
        self.assertFalse(owned.eligible({**self.video, 'color_primaries': 'bt2020'}))

    def test_alpha_and_high_depth_do_not_enter_8bit_prototype(self):
        for fmt in ('yuva420p', 'yuv420p10le', 'gbrpf32le'):
            self.assertFalse(owned.eligible({**self.video, 'pix_fmt': fmt}))
