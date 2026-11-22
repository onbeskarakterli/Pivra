"""Runner/provenance guards only; not driver or native AMD acceptance."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from gtk_stock_lifecycle import stock_provenance, compare_sources


class StockOwnershipProvenance(unittest.TestCase):
    def fixture(self, root, *, enabled=True):
        source = root/'source'
        (source/'gdk').mkdir(parents=True)
        (source/'gdk/gdkdisplay.c').write_bytes(b'fixed EGL selection')
        (source/'gtk/media').mkdir(parents=True)
        (source/'gtk/media/gtkgstsink.c').write_bytes(b'stock policy fixture')
        build = root/'build'
        (build/'gtk').mkdir(parents=True)
        (build/'gtk/libgtk-4.so.1').write_bytes(b'library fixture only')
        (build/'meson-info').mkdir()
        (build/'meson-info/meson-info.json').write_text(json.dumps(dict(directories=dict(source=str(source)))))
        (build/'config.h').write_text('#define HAVE_GSTREAMER 1\n' if enabled else '/* #undef HAVE_GSTREAMER */\n')
        return build, source

    def metadata(self, build):
        with patch('gtk_stock_lifecycle.PATCHED_SHA256', hashlib.sha256(b'fixed EGL selection').hexdigest()):
            return stock_provenance(build, hashlib.sha256(b'stock policy fixture').hexdigest())

    def test_builtin_backend_is_accepted_without_old_module(self):
        with tempfile.TemporaryDirectory() as directory:
            build, _ = self.fixture(Path(directory))
            self.assertEqual(self.metadata(build)['stock_backend'], 'built-in HAVE_GSTREAMER')
            self.assertFalse((build/'modules/media/libmedia-gstreamer.so').exists())

    def test_missing_builtin_cannot_be_certified_by_old_module_filename(self):
        with tempfile.TemporaryDirectory() as directory:
            build, _ = self.fixture(Path(directory), enabled=False)
            old = build/'modules/media/libmedia-gstreamer.so'; old.parent.mkdir(parents=True)
            old.write_bytes(b'old module placeholder')
            with self.assertRaisesRegex(RuntimeError, 'real built-in'):
                self.metadata(build)

    def test_unreviewed_sink_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            build, source = self.fixture(Path(directory))
            (source/'gtk/media/gtkgstsink.c').write_bytes(b'unreviewed')
            with self.assertRaisesRegex(RuntimeError, 'reviewed ownership policy'):
                self.metadata(build)

    def test_existing_egl_patch_cannot_be_changed(self):
        with tempfile.TemporaryDirectory() as directory:
            build, source = self.fixture(Path(directory))
            (source/'gdk/gdkdisplay.c').write_bytes(b'unreviewed')
            with self.assertRaisesRegex(RuntimeError, 'source mismatch'):
                self.metadata(build)

    def test_only_stock_sink_difference_allowed_between_variants(self):
        with tempfile.TemporaryDirectory() as directory:
            _, control = self.fixture(Path(directory)/'control')
            _, candidate = self.fixture(Path(directory)/'candidate')
            (candidate/'gtk/media/gtkgstsink.c').write_bytes(b'other allowed sink variant')
            compare_sources(control, candidate)
            (candidate/'gdk/gdkdisplay.c').write_bytes(b'shader or EGL change')
            with self.assertRaisesRegex(RuntimeError, 'unreviewed GTK change'):
                compare_sources(control, candidate)

    def test_added_shader_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            _, control = self.fixture(Path(directory)/'control')
            _, candidate = self.fixture(Path(directory)/'candidate')
            (candidate/'gtk/new.frag').write_bytes(b'unreviewed shader')
            with self.assertRaisesRegex(RuntimeError, 'addition/removal'):
                compare_sources(control, candidate)
