"""Classifier/source-integrity tests; mocked devices do not prove a real GPU."""
import contextlib
import io
import os
from pathlib import Path
import stat
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import probe


class HardwareGate(unittest.TestCase):
    def report(self, renderer="AMD Radeon RX", vendor="AMD", node="/dev/dri/renderD128"):
        return {"renderer": renderer, "vendor": vendor, "drm_render_node": node}

    def test_software_never_passes(self):
        for name in ["llvmpipe", "softpipe", "swrast", "lavapipe", "SwiftShader"]:
            with self.subTest(name=name):
                self.assertFalse(probe.hardware(self.report(renderer=name)))

    def test_unknown_renderer_not_proof(self):
        self.assertFalse(probe.hardware(self.report(renderer="unknown", vendor="Mesa")))

    def test_missing_node(self):
        self.assertFalse(probe.hardware(self.report(node="")))

    def test_non_drm_device(self):
        self.assertFalse(probe.hardware(self.report(node="/dev/null")))

    def test_regular_file_not_device(self):
        with patch.object(Path, "stat", return_value=SimpleNamespace(st_mode=stat.S_IFREG, st_rdev=0)):
            self.assertFalse(probe.hardware(self.report()))

    def test_wrong_device_major(self):
        with patch.object(Path, "stat", return_value=SimpleNamespace(st_mode=stat.S_IFCHR, st_rdev=os.makedev(1, 3))):
            self.assertFalse(probe.hardware(self.report()))

    def test_device_missing_or_denied(self):
        for error in [FileNotFoundError(), PermissionError()]:
            with patch.object(Path, "stat", side_effect=error):
                self.assertFalse(probe.hardware(self.report()))

    def test_known_driver_and_drm_device(self):
        with patch.object(Path, "stat", return_value=SimpleNamespace(st_mode=stat.S_IFCHR, st_rdev=os.makedev(226, 128))):
            self.assertTrue(probe.hardware(self.report()))

    def test_prepare_only_does_not_pass_hardware(self):
        with patch("sys.argv", ["probe", "--prepare-only", "--require-hardware"]), contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit) as result:
                probe.main()
        self.assertEqual(result.exception.code, 2)

    def test_source_mismatch_rejected_before_copy(self):
        with tempfile.TemporaryDirectory(prefix="pivra-gpu-unit-") as directory:
            root=Path(directory); source=root/"source"; source.mkdir(); output=root/"output"; output.mkdir()
            (source/"example.c").write_text("unexpected source")
            with patch.object(probe, "SOURCE_SHA256", {"example.c": "0"*64}):
                with self.assertRaisesRegex(AssertionError, "unverified upstream source"):
                    probe.prepare(output, source)
            self.assertFalse((output/"gtk-4.24.1").exists())


if __name__ == "__main__":
    unittest.main()
