"""Source-review and native workflow policy, not actual makepkg acceptance."""
import importlib.util
import io
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('release_validation', ROOT / 'packaging/arch/validate-release.py')
validation = importlib.util.module_from_spec(spec); spec.loader.exec_module(validation)

class SourceReview(unittest.TestCase):
    def test_no_moving_ref_or_url_inputs(self):
        for value in ('dev', 'main', 'v0.2.5', 'a'*39, 'A'*40, 'https://example.org/archive'):
            with self.subTest(value=value), self.assertRaises(ValueError):
                validation.source_url(value)
        self.assertEqual(validation.source_url('a'*40), 'https://github.com/onbeskarakterli/Pivra/archive/'+'a'*40+'.tar.gz')

    def info(self):
        # Controlled fixture digests, never release evidence.
        return '\n'.join([
            'pkgbase = pivra', 'pkgname = pivra', 'pkgver = 0.2.5', 'pkgrel = 1',
            'arch = x86_64', 'license = GPL-3.0-only', 'options = !debug',
            'source = pivra-0.2.5.tar.gz::'+validation.source_url('a'*40),
            'source = collect-licenses.py', 'sha256sums = '+'b'*64, 'sha256sums = '+'c'*64,
            *['depends = '+p for p in ('gtk4>=4.16','gnome-desktop-4','glycin','gstreamer>=1.24',
                'gst-plugins-base-libs>=1.24','gst-plugins-good','gst-plugins-bad-libs','gst-libav')]])

    def test_srcinfo_must_match_verified_inputs(self):
        validation.verify_srcinfo(self.info(), 'a'*40, 'b'*64, 'c'*64)
        for old, new in [('pkgver = 0.2.5','pkgver = 0.2.4'), ('options = !debug','options = debug'),
                         ('pkgname = pivra','pkgname = pivra-debug'), ('b'*64,'d'*64),
                         ('depends = gst-libav','depends = different'), ('a'*40,'e'*40)]:
            with self.subTest(old=old), self.assertRaises(ValueError):
                validation.verify_srcinfo(self.info().replace(old,new), 'a'*40,'b'*64,'c'*64)

    def test_archive_links_and_traversal_are_rejected(self):
        for name, kind in [('Pivra-'+'a'*40+'/../escape', tarfile.REGTYPE),
                           ('Pivra-'+'a'*40+'/link',tarfile.SYMTYPE), ('Other/file',tarfile.REGTYPE)]:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as d:
                path=Path(d)/'source.tar.gz'
                with tarfile.open(path,'w:gz') as archive:
                    info=tarfile.TarInfo(name);info.type=kind;info.linkname='/tmp/outside'
                    archive.addfile(info,io.BytesIO(b''))
                with self.assertRaises(ValueError):validation.extract_source(path,Path(d)/'out','a'*40)
                self.assertFalse((Path(d)/'escape').exists())

    def test_regular_source_extraction_and_hash(self):
        with tempfile.TemporaryDirectory() as d:
            path=Path(d)/'source.tar.gz'
            with tarfile.open(path,'w:gz') as archive:
                data=b'[package]\nversion="0.2.5"\n';info=tarfile.TarInfo('Pivra-'+'a'*40+'/Cargo.toml')
                info.size=len(data);info.mode=0o644;archive.addfile(info,io.BytesIO(data))
            root=validation.extract_source(path,Path(d)/'out','a'*40)
            self.assertEqual((root/'Cargo.toml').read_bytes(),data)
            self.assertEqual(len(validation.digest(path)),64)

    def test_workflow_has_no_install_or_dependency_sync_actions(self):
        text=(ROOT/'packaging/arch/validate-release.py').read_text()
        self.assertNotIn("'--syncdeps'",text)
        self.assertNotIn("'--install'",text)
        self.assertNotIn("'pacman', '-U'",text)
        self.assertIn("'--approved-source'",text)
        self.assertIn("'--verifysource'",text)
        self.assertIn("'--cleanbuild'",text)
        self.assertIn("'--check'",text)

if __name__=='__main__':unittest.main()
