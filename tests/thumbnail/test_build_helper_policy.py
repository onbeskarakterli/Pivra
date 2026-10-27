"""Build policy and actual resolver body, not a GNOME API/decoder acceptance test."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class BuildHelperPolicy(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(prefix='pivra-helper-policy-')
        cls.addClassCleanup(cls.temp.cleanup)
        cls.root = Path(cls.temp.name)
        cls.rustc = shutil.which('rustc')
        if not cls.rustc:
            raise RuntimeError('rustc is required; activate the Rust test environment')
        cls.build_script = cls.root / 'build-script'
        subprocess.run([cls.rustc, '--edition=2024', str(ROOT / 'build.rs'),
                        '-o', str(cls.build_script)], check=True)
        cls.tools = cls.root / 'tools'
        cls.tools.mkdir()
        (cls.tools / 'pkg-config').write_text('#!/bin/sh\nexit "${NO_API:-0}"\n')
        (cls.tools / 'cc').write_text(
            '#!/bin/sh\nprintf "called\\n" >> "$CC_LOG"\nexit 0\n')
        for path in cls.tools.iterdir():
            path.chmod(0o755)
        source = (ROOT / 'src/thumbnail.rs').read_text()
        start = source.index('pub fn generation_helper()')
        end = source.index('\n/// Worker-only', start)
        # Compile the exact current production resolver; no rewritten substitute.
        cls.resolver_source = cls.root / 'resolver.rs'
        cls.resolver_source.write_text(
            'use std::path::PathBuf;\n' + source[start:end] +
            '\nfn main() { match generation_helper() {'
            'Some(path) => println!("{}", path.display()), '
            'None => println!("NONE"), } }\n')

    def build_policy(self, name, mode=None, no_api=False):
        folder = self.root / name
        folder.mkdir()
        out = folder / 'unique-build-root' / 'out'
        out.mkdir(parents=True)
        log = folder / 'cc.log'
        env = os.environ.copy()
        env.pop('PIVRA_DISTRIBUTION_BUILD', None)
        env.update(PATH=str(self.tools) + os.pathsep + env['PATH'],
                   OUT_DIR=str(out), CC_LOG=str(log), NO_API=str(int(no_api)))
        if mode is not None:
            env['PIVRA_DISTRIBUTION_BUILD'] = mode
        result = subprocess.run([str(self.build_script)], env=env, check=True,
                                capture_output=True, text=True)
        self.assertIn('cargo:rerun-if-env-changed=PIVRA_DISTRIBUTION_BUILD', result.stdout)
        prefix = 'cargo:rustc-env=PIVRA_BUILT_THUMBNAIL_FACTORY='
        values = [line[len(prefix):] for line in result.stdout.splitlines()
                  if line.startswith(prefix)]
        self.assertEqual(len(values), 1)
        return folder, out / 'pivra-thumbnail-factory', values[0], log

    def resolver(self, folder, fallback):
        binary = folder / 'installed' / 'pivra'
        binary.parent.mkdir()
        env = {**os.environ, 'PIVRA_BUILT_THUMBNAIL_FACTORY': fallback}
        subprocess.run([self.rustc, '--edition=2024', str(self.resolver_source),
                        '-o', str(binary)], env=env, check=True)
        return binary

    def resolve(self, binary, override=None):
        env = os.environ.copy()
        env.pop('PIVRA_THUMBNAIL_GENERATOR', None)
        if override is not None:
            env['PIVRA_THUMBNAIL_GENERATOR'] = str(override)
        return subprocess.check_output([str(binary)], env=env, text=True).strip()

    def test_development_keeps_absolute_fallback_and_sibling_priority(self):
        folder, expected, value, log = self.build_policy('development')
        self.assertEqual(value, str(expected))
        self.assertTrue(log.exists())
        binary = self.resolver(folder, value)
        self.assertEqual(self.resolve(binary), str(expected))
        self.assertIn(str(expected).encode(), binary.read_bytes())
        sibling = binary.with_name('pivra-thumbnail-factory')
        sibling.write_text('discovery-only fixture, not a native helper')
        self.assertEqual(self.resolve(binary), str(sibling))
        override = folder / 'explicit-override'
        self.assertEqual(self.resolve(binary, override), str(override))

    def test_distribution_excludes_build_path_and_keeps_sibling_and_override(self):
        folder, expected, value, log = self.build_policy('distribution', '1')
        self.assertEqual(value, '')
        self.assertFalse(log.exists())
        binary = self.resolver(folder, value)
        self.assertEqual(self.resolve(binary), 'NONE')
        self.assertNotIn(str(expected).encode(), binary.read_bytes())
        self.assertNotIn(b'/out/pivra-thumbnail-factory', binary.read_bytes())
        sibling = binary.with_name('pivra-thumbnail-factory')
        sibling.write_text('discovery-only fixture, not a native helper')
        self.assertEqual(self.resolve(binary), str(sibling))
        self.assertEqual(self.resolve(binary, folder / 'override'), str(folder / 'override'))

    def test_development_without_api_keeps_empty_fallback(self):
        folder, _, value, log = self.build_policy('no-api', no_api=True)
        self.assertEqual(value, '')
        self.assertFalse(log.exists())
        self.assertEqual(self.resolve(self.resolver(folder, value)), 'NONE')

    def test_only_explicit_one_enables_distribution_policy(self):
        _, expected, value, log = self.build_policy('explicit-zero', '0')
        self.assertEqual(value, str(expected))
        self.assertTrue(log.exists())


if __name__ == '__main__':
    unittest.main()
