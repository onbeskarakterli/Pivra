"""Source-review and native workflow policy, not actual makepkg acceptance."""
import importlib.util
import io
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import unittest
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('release_validation', ROOT / 'packaging/arch/validate-release.py')
validation = importlib.util.module_from_spec(spec); spec.loader.exec_module(validation)

class SourceReview(unittest.TestCase):
    def test_no_moving_ref_or_url_inputs(self):
        for value in ('dev', 'main', 'v0.2.6', 'a'*39, 'A'*40, 'https://example.org/archive'):
            with self.subTest(value=value), self.assertRaises(ValueError):
                validation.source_url(value)
        self.assertEqual(validation.source_url('a'*40), 'https://github.com/onbeskarakterli/Pivra/archive/'+'a'*40+'.tar.gz')

    def info(self, version, source):
        # Controlled fixture digests, never release evidence.
        return '\n'.join([
            'pkgbase = pivra', 'pkgname = pivra', f'pkgver = {version}', 'pkgrel = 1',
            'arch = x86_64', 'license = GPL-3.0-only', 'options = !debug',
            f'source = {source}',
            'source = collect-licenses.py', 'sha256sums = '+'b'*64, 'sha256sums = '+'c'*64,
            *['depends = '+p for p in ('gtk4>=4.16','gnome-desktop-4','glycin','gstreamer>=1.24',
                'gst-plugins-base-libs>=1.24','gst-plugins-good','gst-plugins-bad-libs','gst-libav')]])

    def test_srcinfo_must_match_verified_inputs(self):
        version = '0.2.8'
        local = f'pivra-{version}.tar.gz'
        remote = local + '::' + validation.source_url('a'*40)
        for source in (local, remote):
            info = self.info(version, source)
            validation.verify_srcinfo(info, version, source, 'b'*64, 'c'*64)
            for old, new in [(f'pkgver = {version}', 'pkgver = 0.2.6'),
                             ('options = !debug', 'options = debug'),
                             ('pkgname = pivra', 'pkgname = pivra-debug'),
                             ('b'*64, 'd'*64), ('depends = gst-libav', 'depends = different'),
                             (source, local if source == remote else remote)]:
                with self.subTest(source=source, old=old), self.assertRaises(ValueError):
                    validation.verify_srcinfo(info.replace(old, new), version, source, 'b'*64, 'c'*64)

    def test_local_archive_includes_only_the_selected_commit(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            repo = root / 'repo'
            repo.mkdir()
            git = ['git', '-C', str(repo)]
            subprocess.run(git + ['init', '-q'], check=True)
            manifest = repo / 'Cargo.toml'
            manifest.write_text('[package]\nversion="0.2.8"\n')
            subprocess.run(git + ['add', 'Cargo.toml'], check=True)
            subprocess.run(git + ['-c', 'user.name=Pivra Test', '-c', 'user.email=test@example.invalid',
                                  'commit', '-qm', 'candidate', '--no-gpg-sign'], check=True)
            commit = subprocess.check_output(git + ['rev-parse', 'HEAD'], text=True).strip()
            manifest.write_text('uncommitted version')
            (repo / 'untracked').touch()
            archive = root / 'candidate.tar.gz'
            validation.archive_local(repo, commit, archive)
            source = validation.extract_source(archive, root / 'extracted', commit)
            self.assertEqual((source / 'Cargo.toml').read_text(), '[package]\nversion="0.2.8"\n')
            self.assertFalse((source / 'untracked').exists())
            with self.assertRaises(ValueError):
                validation.archive_local(repo, 'HEAD', root / 'invalid.tar.gz')

    def test_recipe_separates_local_and_approved_source_modes(self):
        version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['package']['version']
        commit = 'a'*40
        sha256 = 'b'*64
        archive = f'pivra-{version}.tar.gz'
        with tempfile.TemporaryDirectory() as d:
            recipe = Path(d) / 'PKGBUILD'
            shutil.copyfile(ROOT / 'packaging/arch/PKGBUILD', recipe)

            def load():
                return subprocess.run(['bash', '-c',
                    'source "$1" && printf "%s\\n" "${source[0]}" "${sha256sums[0]}" "${options[0]}"',
                    'verify-recipe', str(recipe)], text=True, capture_output=True)

            self.assertIn('BLOCKED', load().stderr)
            candidate = Path(d) / 'candidate-source.conf'
            approval = Path(d) / 'release-source.conf'
            config = f"release_commit='{commit}'\nrelease_sha256='{sha256}'\n"
            candidate.write_text(config)
            self.assertIn('requires its local archive', load().stderr)
            (Path(d) / archive).touch()
            self.assertEqual(load().stdout.splitlines(), [archive, sha256, '!debug'])
            url = validation.source_url(commit)
            candidate.write_text(config + f"release_url='{url}'\n")
            self.assertIn('empty URL', load().stderr)
            approval.write_text(config + f"release_url='{url}'\n")
            self.assertIn('cannot coexist', load().stderr)
            candidate.unlink()
            self.assertEqual(load().stdout.splitlines(), [f'{archive}::{url}', sha256, '!debug'])
            approval.write_text(config + f"release_url='{url.replace(commit, 'dev')}'\n")
            self.assertIn('immutable GitHub', load().stderr)

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
                data=b'[package]\nversion="0.2.6"\n';info=tarfile.TarInfo('Pivra-'+'a'*40+'/Cargo.toml')
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
