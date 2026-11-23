import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from gtk_egl_isolated import provenance, require_loader


class IsolatedGtkProvenance(unittest.TestCase):
    def fixture(self, root):
        source = root/'source/gdk/gdkdisplay.c'; source.parent.mkdir(parents=True)
        source.write_bytes(b'unit source')
        build = root/'build'
        (build/'meson-info').mkdir(parents=True)
        (build/'meson-info/meson-info.json').write_text(json.dumps(dict(directories=dict(source=str(root/'source')))))
        library = build/'gtk/libgtk-4.so.1'; library.parent.mkdir()
        library.write_bytes(b'unit metadata fixture; not a real GTK library')
        return build, source, library

    def test_wrong_source_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            build, _, _ = self.fixture(Path(directory))
            with self.assertRaisesRegex(RuntimeError, 'source mismatch'):
                provenance(build, 'wrong')

    def test_old_color_patch_cannot_be_mistaken_for_egl_patch(self):
        with tempfile.TemporaryDirectory() as directory:
            build, source, library = self.fixture(Path(directory))
            library.write_bytes(b'video-bt1886 GPU CICP op')
            with self.assertRaisesRegex(RuntimeError, 'EGL-only'):
                provenance(build, hashlib.sha256(source.read_bytes()).hexdigest())

    def test_library_outside_private_build_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            build, source, library = self.fixture(Path(directory))
            other = Path(directory)/'not-private.so'; other.write_bytes(b'fixture')
            library.unlink(); library.symlink_to(other)
            with self.assertRaisesRegex(RuntimeError, 'private EGL-only'):
                provenance(build, hashlib.sha256(source.read_bytes()).hexdigest())

    def test_loader_must_resolve_exact_library(self):
        with patch('subprocess.check_output', return_value='libgtk-4.so.1 => /other/libgtk-4.so.1 (0x123)\n'):
            with self.assertRaisesRegex(RuntimeError, 'wrong runtime GTK'):
                require_loader(Path('/test'), '/expected/libgtk-4.so.1', {})

    def test_builder_refuses_system_workspace_before_writing(self):
        with tempfile.NamedTemporaryFile() as video:
            result = subprocess.run([sys.executable, '-B', str(Path(__file__).with_name('gtk_egl_build.py')),
                                     '--workspace', '/usr/lib/pivra-test', '--video', video.name], capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            self.assertIn('private workspace', result.stderr)
