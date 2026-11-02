"""Harness-only regressions: mocked output is not real decoder evidence."""
import subprocess
import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
import backend_probe as backend


class BackendGateTests(unittest.TestCase):
    def setUp(self):
        self.workspace = tempfile.TemporaryDirectory(prefix="pivra-loader-unit-")
        self.addCleanup(self.workspace.cleanup)
        self.root = Path(self.workspace.name)

    def call(self, source="CPU-memory", selected="PivraCandidateMediaFile", marker=True,
             returncode=0, samples=True):
        output = (f"BACKEND selected={selected}\nSOURCE path={source}\n"
                  "PASS MediaStream controls\nmax channel difference=0\n")
        if samples:
            output += "".join(f"sample x={i} y=48 sRGB={[v,v,v,255]}\n" for i,v in enumerate([0,121,253]))
        result = subprocess.CompletedProcess([], returncode, output,
            "PIVRA_COLOR_IMPL id=pivra-color-candidate" if marker else "")
        with patch.object(backend.subprocess, "run", return_value=result) as invoked, contextlib.redirect_stdout(io.StringIO()):
            pixels = backend.probe(self.root, Path("/unused-probe"), "test", Path("/unused-video"), "candidate", True)
            self.assertEqual(invoked.call_args.kwargs["env"]["GTK_MEDIA"], "pivra-color-candidate")
            self.assertEqual(invoked.call_args.kwargs["env"]["PIVRA_VIDEO_TEST_MODULE_DIR"], str(self.root / "candidate/modules"))
            return pixels

    def test_verified_cpu(self):
        self.assertEqual(self.call()[1], [121,121,121,255])

    def test_dmabuf_not_cpu_assertion(self):
        with self.assertRaisesRegex(RuntimeError, "NOT RUN CPU candidate inactive"):
            self.call(source="DMA-BUF")

    def test_gl_not_cpu_assertion(self):
        with self.assertRaisesRegex(RuntimeError, "NOT RUN CPU candidate inactive"):
            self.call(source="GL")

    def test_unknown_path_not_pass(self):
        with self.assertRaisesRegex(RuntimeError, "NOT RUN CPU candidate inactive"):
            self.call(source="UNKNOWN")

    def test_mixed_path_not_pass(self):
        with self.assertRaisesRegex(RuntimeError, "NOT RUN mixed/non-CPU"):
            self.call(source="CPU-memory\nSOURCE path=DMA-BUF")

    def test_builtin_class_not_candidate(self):
        with self.assertRaisesRegex(RuntimeError, "NOT RUN backend provenance"):
            self.call(selected="GtkGstMediaFile")

    def test_missing_implementation_marker(self):
        with self.assertRaisesRegex(RuntimeError, "NOT RUN backend provenance"):
            self.call(marker=False)

    def test_decode_failure_stays_failure(self):
        with self.assertRaises(AssertionError):
            self.call(returncode=1)

    def test_missing_samples_stays_failure(self):
        with self.assertRaises(AssertionError):
            self.call(samples=False)

    def test_module_names_do_not_collide(self):
        for variant in ["stock", "candidate"]:
            directory = self.root / variant
            directory.mkdir()
            text = ('GtkGstMediaFile gtk_gst_media_file_get_type GTK_TYPE_GST_MEDIA_FILE GTK_GST_MEDIA_FILE\n'
                    'G_DECLARE_FINAL_TYPE (GtkGstMediaFile, gtk_gst_media_file, GTK, GST_MEDIA_FILE, GtkMediaFile)\n'
                    '"gstreamer",\n  self->paintable = gtk_gst_paintable_new ();\n')
            source = directory / "gtkgstmediafile.c"
            source.write_text(text)
            backend.isolate_symbols(directory, variant)
            renamed = source.read_text()
            self.assertNotIn("GtkGstMediaFile", renamed)
            self.assertNotIn("gtk_gst_media_file", renamed)
            self.assertNotIn("GTK_TYPE_GST_MEDIA_FILE", renamed)
            self.assertIn("Pivra" + variant.title() + "MediaFile", renamed)
            self.assertIn('"pivra-color-' + variant + '",', renamed)
            self.assertIn("PIVRA, " + variant.upper() + "_MEDIA_FILE", renamed)
            self.assertIn("GtkMediaFile", renamed)  # Public GTK parent is preserved.

    def test_visual_requires_video(self):
        with patch("sys.argv", ["probe", "--visual"]), contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit) as raised:
                backend.main()
        self.assertEqual(raised.exception.code, 2)

    def test_visual_not_build_only(self):
        with patch("sys.argv", ["probe", "--visual", "--video", "unused", "--build-only"]), contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit) as raised:
                backend.main()
        self.assertEqual(raised.exception.code, 2)

    def test_visual_launch_uses_verified_module(self):
        video = self.root / "input.mp4"
        binary = self.root / "probe"
        video.touch()
        binary.touch()
        def samples(root, probe, name, clip, variant, enabled):
            values = ([4,121,234] if enabled else [31,140,237]) if name.startswith("full709") else ([0,121,253] if enabled else [0,140,253])
            if name.startswith(("srgb", "unknown")):
                values = [0,128,253]
            return [[v,v,v,255] for v in values]
        with patch("sys.argv", ["probe", "--visual", "--video", str(video), "--probe", str(binary), "--output", str(self.root / "output")]), \
             patch.object(backend, "build"), patch.object(backend, "fixtures"), \
             patch.object(backend, "probe", side_effect=samples), \
             patch.object(backend, "run") as run, patch.object(backend.shutil, "which", return_value="tool"), \
             contextlib.redirect_stdout(io.StringIO()):
            backend.main()
        launch = run.call_args
        self.assertTrue(launch.args[0][0].endswith("/debug/examples/video_color_window"))
        self.assertEqual(launch.args[0][1], str(video))
        self.assertEqual(launch.kwargs["env"]["GTK_MEDIA"], "pivra-color-candidate")
        self.assertEqual(launch.kwargs["env"]["PIVRA_VIDEO_TEST_MODULE_DIR"], str(self.root / "output/candidate/modules"))


if __name__ == "__main__":
    unittest.main()
