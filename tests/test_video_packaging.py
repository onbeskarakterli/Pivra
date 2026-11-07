"""Controlled packaging/build-policy tests, never native GPU/Arch acceptance."""
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
import sys
sys.dont_write_bytecode = True
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]

def module(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result

elf = module('video_binary', 'packaging/linux/verify-video-binary.py')
sources = module('distribution_sources', 'packaging/linux/collect-distribution.py')
preflight = module('release_preflight', 'scripts/release_preflight.py')

class BuildPolicy(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.root = Path(cls.tmp.name)
        cls.binary = cls.root / 'build-policy'
        wrapper = cls.root / 'main.rs'
        wrapper.write_text(f'#[path = {json.dumps(str(ROOT / "native/video/build.rs"))}] mod video; fn main() {{video::build();}}')
        subprocess.run(['rustc', '--edition=2024', str(wrapper), '-o', str(cls.binary)], check=True)
        (cls.root / 'pkg-config').write_text('#!/bin/sh\ncase "$1" in --atleast-version=*) [ "${PKG_ONLY_VERSION:-0}" = 1 ];; *) exit 1;; esac\n')
        (cls.root / 'pkg-config').chmod(0o755)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def run_policy(self, distribution, disabled=False, version_only=False):
        env = os.environ.copy()
        env.update(PATH=str(self.root), PIVRA_DISTRIBUTION_BUILD=str(int(distribution)),
                   PIVRA_DISABLE_GPU_BUILD=str(int(disabled)), PKG_ONLY_VERSION=str(int(version_only)))
        return subprocess.run([str(self.binary)], env=env, capture_output=True, text=True)

    def test_distribution_missing_headers_is_fatal(self):
        result = self.run_policy(True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('GPU backend REQUIRED', result.stderr)

    def test_distribution_missing_headers_after_version_check_is_fatal(self):
        result = self.run_policy(True, version_only=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('missing GTK/GStreamer', result.stderr)

    def test_distribution_cannot_opt_out(self):
        result = self.run_policy(True, True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('conflicts with mandatory GPU', result.stderr)

    def test_developer_stock_only_remains_available(self):
        self.assertEqual(self.run_policy(False, True).returncode, 0)
        self.assertEqual(self.run_policy(False).returncode, 0)

class ElfContract(unittest.TestCase):
    def check(self, marker=True, libraries=None, extra=''):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'pivra'
            path.write_bytes(b'PIVRA_GPU_BACKEND automatic type= PIVRA_GPU_CONTEXT renderer=' if marker else b'stock only')
            dynamic = '\n'.join(f'(NEEDED) Shared library: [{name}]' for name in (elf.REQUIRED if libraries is None else libraries)) + extra
            with mock.patch.object(elf.subprocess, 'run', return_value=mock.Mock(stdout=dynamic)):
                return elf.verify(path)

    def test_shared_native_contract(self):
        self.assertEqual(set(self.check()), elf.REQUIRED)

    def test_no_adapter_or_library_is_rejected(self):
        for kwargs in ({'marker': False}, {'libraries': elf.REQUIRED - {'libgstgl-1.0.so.0'}}):
            with self.subTest(kwargs=kwargs), self.assertRaises(ValueError):
                self.check(**kwargs)

    def test_local_loader_paths_rejected(self):
        for kwargs in ({'extra': '\n(RUNPATH) [/tmp/gtk-fork]'},
                       {'libraries': elf.REQUIRED | {'/tmp/local/libgtk-4.so.1'}}):
            with self.subTest(kwargs=kwargs), self.assertRaises(ValueError):
                self.check(**kwargs)

class SourceCollection(unittest.TestCase):
    def test_snapshot_is_complete_deterministic_and_excludes_build_tree(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d) / 'project'; root.mkdir()
            for name in sources.SOURCE_PATHS:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(name)
            (root / 'target').mkdir(); (root / 'target/private').write_text('not source')
            first, second = Path(d) / 'a.tar.gz', Path(d) / 'b.tar.gz'
            inventory = sources.snapshot(root, first)
            self.assertEqual(inventory, sources.snapshot(root, second))
            self.assertEqual(first.read_bytes(), second.read_bytes())
            self.assertNotIn('target/private', inventory)
            self.assertIn('native', inventory)

    def test_missing_source_is_not_silently_omitted(self):
        with tempfile.TemporaryDirectory() as d, self.assertRaises(ValueError):
            sources.snapshot(Path(d), Path(d) / 'output.tar.gz')

class ReleasePreflight(unittest.TestCase):
    def invoke(self, root, candidate):
        with mock.patch.object(preflight, 'ROOT', root), mock.patch.object(preflight, 'ARCH', root / 'packaging/arch'), \
             mock.patch.object(preflight, 'git', side_effect=lambda *args: 'dev' if args[0] == 'branch' else ''), \
             mock.patch('sys.argv', ['preflight', '--version', '0.2.5'] + (['--candidate'] if candidate else [])), \
             contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            return preflight.main()

    def test_candidate_does_not_approve_unbumped_release(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d); (root / 'packaging/arch').mkdir(parents=True); (root / 'docs/releases').mkdir(parents=True)
            (root / 'Cargo.toml').write_text('[package]\nversion="0.2.4"\n')
            (root / 'Cargo.lock').write_text('name = "pivra"\nversion = "0.2.4"\n')
            (root / 'packaging/arch/PKGBUILD').write_text("pkgver=0.2.4\npkgrel=1\noptions=('!debug')\n")
            (root / 'README.md').write_text('v0.2.4')
            (root / 'docs/releases/PIVRA_V0.2.5_RC_VALIDATION.md').write_text('development candidate')
            self.assertEqual(self.invoke(root, True), 0)
            self.assertEqual(self.invoke(root, False), 1)

    def test_arch_source_approval_still_mandatory(self):
        with tempfile.TemporaryDirectory() as d:
            recipe = Path(d) / 'PKGBUILD'; shutil.copyfile(ROOT / 'packaging/arch/PKGBUILD', recipe)
            result = subprocess.run(['bash', '-c', 'source "$1"', 'test', str(recipe)], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('approved release-source.conf is missing', result.stderr)


class ArchOwnershipGate(unittest.TestCase):
    def run_gate(self, wrong=False):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d)
            tools = {
                'pkg-config': '#!/bin/sh\necho /usr/lib/pkgconfig\n',
                'gst-inspect-1.0': '#!/bin/sh\nprintf "  Filename /plugins/%s.so\\n" "$1"\n',
                'pacman': '#!/bin/sh\ncase "$3" in *pkgconfig/*|*playbin3.so) echo gst-plugins-base-libs;; *glupload.so|*glcolorconvert.so) echo gst-plugins-base;; *qtdemux.so|*autoaudiosink.so) echo gst-plugins-good;; *h264parse.so) echo "${PARSER_OWNER:-gst-plugins-bad-libs}";; *avdec_h264.so) echo gst-libav;; *gst-inspect-1.0) echo gstreamer;; *) exit 1;; esac\n',
            }
            for name, text in tools.items():
                (path / name).write_text(text); (path / name).chmod(0o755)
            env = dict(os.environ, PATH=d+':/usr/bin:/bin')
            if wrong: env['PARSER_OWNER'] = 'unexpected-package'
            return subprocess.run(['/bin/bash', str(ROOT / 'packaging/arch/verify-video-dependencies.sh')], env=env, capture_output=True, text=True)

    def test_controlled_correct_ownership(self):
        result = self.run_gate()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_changed_split_fails_explicitly(self):
        result = self.run_gate(True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('expected=gst-plugins-bad-libs actual=unexpected-package', result.stderr)

class RuntimeGate(unittest.TestCase):
    def test_missing_h264_decoder_fails_without_retry(self):
        with tempfile.TemporaryDirectory() as d:
            tool = Path(d) / 'gst-inspect-1.0'
            tool.write_text('#!/bin/sh\n[ "$1" != avdec_h264 ]\n'); tool.chmod(0o755)
            env = dict(os.environ, PATH=d)
            result = subprocess.run(['/bin/bash', str(ROOT / 'packaging/linux/video-runtime-check.sh')], env=env, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('missing GStreamer element avdec_h264', result.stderr)

if __name__ == '__main__':
    unittest.main()
