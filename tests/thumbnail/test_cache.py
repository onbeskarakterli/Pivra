"""Real GIO metadata queries in isolated process caches, not native GNOME/decoder evidence."""
import hashlib
import os
from pathlib import Path
import struct
import subprocess
import tempfile
import unittest
import zlib

ROOT=Path(__file__).resolve().parents[2]
BINARY=ROOT/'target/debug/examples/thumbnail_provider_probe'
PIXELS=(ROOT/'tests/fixtures/pixels-2x2.png').read_bytes()

def chunk(kind,data):
    return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))

class Cache(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='pivra-th01-')
        self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name)
        self.source=self.root/'örnek #?.png';self.source.write_bytes(PIXELS)
        self.cache=self.root/'cache'
    def thumbnail(self,uri=None,mtime=None,size=None):
        uri=self.source.as_uri() if uri is None else uri
        meta=chunk(b'tEXt',b'Thumb::URI\0'+uri.encode())
        meta+=chunk(b'tEXt',b'Thumb::MTime\0'+str(int(self.source.stat().st_mtime) if mtime is None else mtime).encode())
        meta+=chunk(b'tEXt',b'Thumb::Size\0'+str(self.source.stat().st_size if size is None else size).encode())
        path=self.cache/'thumbnails/normal'/(hashlib.md5(self.source.as_uri().encode(),usedforsecurity=False).hexdigest()+'.png')
        path.parent.mkdir(parents=True,exist_ok=True)
        path.write_bytes(PIXELS[:33]+meta+PIXELS[33:]);return path
    def run_query(self,expected,source=None,desktop='GNOME',factory=None):
        env={**os.environ,'XDG_CACHE_HOME':str(self.cache),'XDG_CURRENT_DESKTOP':desktop}
        env.pop('PIVRA_THUMBNAIL_FACTORY_PROBE',None)
        if factory is not None:
            env['PIVRA_THUMBNAIL_FACTORY_PROBE']=str(factory)
        process=subprocess.run([str(BINARY),str(source or self.source)],env=env,capture_output=True,text=True,timeout=10)
        self.assertEqual(process.returncode,0,process.stderr)
        self.assertIn(expected,process.stdout,process.stdout+process.stderr)
        return process.stdout
    def test_real_valid_cache(self):
        self.thumbnail();self.run_query('HIT metadata/container valid')
    def test_empty_cache_is_cache_only(self):
        self.run_query('capability=CacheOnly');self.run_query('OUTCOME Missing')
    def test_stale_and_wrong_uri_and_size(self):
        for values in ({'mtime':0},{'uri':'file:///wrong'},{'size':1}):
            self.thumbnail(**values);self.run_query('OUTCOME Stale')
    def test_corrupt_thumbnail(self):
        path=self.thumbnail();data=bytearray(path.read_bytes());data[-1]^=1;path.write_bytes(data)
        self.run_query('OUTCOME Invalid')
    def test_deleted_source(self):
        self.thumbnail();self.source.unlink();self.run_query('OUTCOME SourceUnavailable')
    def test_rename_changes_uri(self):
        self.thumbnail();renamed=self.source.with_name('renamed.png');self.source.rename(renamed)
        self.run_query('OUTCOME Missing',source=renamed)
    def assert_hit(self):
        out = self.run_query('HIT metadata/container valid')
        self.assertIn('current=true', out)
        self.assertNotIn('OUTCOME ', out)

    def replace(self, data, mtime_ns):
        old = self.source.stat()
        new = self.root / 'save'
        new.write_bytes(data)
        os.utime(new, ns=(old.st_atime_ns, mtime_ns))
        new.replace(self.source)
        self.assertNotEqual(self.source.stat().st_ino, old.st_ino)
        self.assertEqual(self.source.stat().st_mtime_ns, mtime_ns)

    def pixel_png(self, rgb):
        # Valid 2x2 RGB PNG; level=0 keeps differently colored files equal-size.
        header = struct.pack('>IIBBBBB', 2, 2, 8, 2, 0, 0, 0)
        rows = (b'\0' + bytes(rgb) * 2) * 2
        return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', header)
                + chunk(b'IDAT', zlib.compress(rows, level=0)) + chunk(b'IEND', b''))

    def changed_pixels(self):
        before = self.pixel_png((20, 80, 140))
        after = self.pixel_png((200, 30, 90))
        self.assertEqual(len(before), len(after))
        self.assertNotEqual(before, after)
        self.source.write_bytes(before)
        return before, after

    def test_identical_atomic_replacement_preserved_mtime_reuses_valid_cache(self):
        cache = self.thumbnail()
        old = self.source.stat()
        cache_bytes = cache.read_bytes()
        cache_identity = (cache.stat().st_ino, cache.stat().st_mtime_ns)
        self.assert_hit()
        self.replace(PIXELS, old.st_mtime_ns)
        self.assertEqual(self.source.read_bytes(), PIXELS)
        self.assertEqual(self.source.stat().st_size, old.st_size)
        self.assert_hit()
        self.assertEqual(cache.read_bytes(), cache_bytes)
        self.assertEqual((cache.stat().st_ino, cache.stat().st_mtime_ns), cache_identity)

    def test_changed_pixels_in_place_new_mtime_is_stale(self):
        before, after = self.changed_pixels()
        self.thumbnail()
        old = self.source.stat()
        self.source.write_bytes(after)
        os.utime(self.source, ns=(old.st_atime_ns, old.st_mtime_ns + 2_000_000_000))
        self.assertEqual(self.source.stat().st_ino, old.st_ino)
        self.assertEqual(self.source.stat().st_size, old.st_size)
        self.assertNotEqual(self.source.read_bytes(), before)
        self.run_query('OUTCOME Stale')

    def test_same_size_atomic_replacement_changed_pixels_new_mtime_is_stale(self):
        before, after = self.changed_pixels()
        self.thumbnail()
        old = self.source.stat()
        self.replace(after, old.st_mtime_ns + 2_000_000_000)
        self.assertEqual(self.source.stat().st_size, old.st_size)
        self.assertNotEqual(self.source.read_bytes(), before)
        self.run_query('OUTCOME Stale')

    def test_changed_size_preserved_mtime_is_stale(self):
        self.thumbnail()
        old = self.source.stat()
        # Add a valid ancillary chunk: source is still PNG, but Thumb::Size is old.
        enlarged = PIXELS[:33] + chunk(b'tEXt', b'Comment\0changed') + PIXELS[33:]
        self.replace(enlarged, old.st_mtime_ns)
        self.assertNotEqual(self.source.stat().st_size, old.st_size)
        self.run_query('OUTCOME Stale')

    def test_mtime_only_change_invalidates_identical_pixels(self):
        self.thumbnail()
        old = self.source.stat()
        os.utime(self.source, ns=(old.st_atime_ns, old.st_mtime_ns + 2_000_000_000))
        self.assertEqual(self.source.read_bytes(), PIXELS)
        self.run_query('OUTCOME Stale')

    def test_trash_like_move_and_restore_reuses_valid_cache(self):
        # Private rename/restore models filesystem effects, not desktop Trash UI.
        cache = self.thumbnail()
        os.utime(cache, ns=(1_000_000_000, 1_000_000_000))
        old = self.source.stat()
        cached = cache.read_bytes()
        trash = self.root / 'private-trash'
        trash.mkdir()
        moved = trash / self.source.name
        self.source.rename(moved)
        self.run_query('OUTCOME SourceUnavailable')
        moved.rename(self.source)
        self.assertEqual(self.source.stat().st_ino, old.st_ino)
        self.assertEqual(self.source.stat().st_mtime_ns, old.st_mtime_ns)
        self.assertEqual(self.source.read_bytes(), PIXELS)
        self.assertGreater(self.source.stat().st_ctime_ns, cache.stat().st_mtime_ns)
        self.assert_hit()
        self.assertEqual(cache.read_bytes(), cached)
        self.assertEqual(cache.stat().st_mtime_ns, 1_000_000_000)

    def test_permission_metadata_change_does_not_invalidate_pixels(self):
        cache = self.thumbnail()
        os.utime(cache, ns=(1_000_000_000, 1_000_000_000))
        old = self.source.stat()
        self.source.chmod((old.st_mode & 0o777) ^ 0o040)
        self.assertNotEqual(self.source.stat().st_mode, old.st_mode)
        self.assertEqual(self.source.stat().st_ino, old.st_ino)
        self.assertEqual(self.source.stat().st_mtime_ns, old.st_mtime_ns)
        self.assertEqual(self.source.read_bytes(), PIXELS)
        self.assertGreater(self.source.stat().st_ctime_ns, cache.stat().st_mtime_ns)
        self.assert_hit()

    def test_unsupported_desktop(self):
        self.run_query('capability=Unavailable',desktop='KDE')
    def factory(self,output='1',status=0):
        # Controlled metadata IPC only; never decodes or generates an image.
        helper=self.root/'metadata-helper'
        self.calls=self.root/'metadata-calls'
        helper.write_text("#!/bin/sh\nprintf '%s\\n' \"$@\" >> '"+str(self.calls)+"'\nprintf '%s\\n' '"+output+"'\nexit "+str(status)+"\n")
        helper.chmod(0o700)
        return helper
    def test_colon_separated_case_insensitive_gnome_selects_provider(self):
        out=self.run_query('capability=CacheAndGenerate',desktop='ubuntu:gNoMe',factory=self.factory())
        self.assertIn('factory=Eligible',out)
        self.assertIn('OUTCOME Missing',out)
        self.assertEqual(self.calls.read_text().splitlines(),
                         [self.source.as_uri(),'image/png',str(int(self.source.stat().st_mtime))])
    def test_desktop_substring_is_not_gnome_and_helper_not_called(self):
        for desktop in ('KDE','X-GNOME','GNOME-Classic',''):
            with self.subTest(desktop=desktop):
                self.run_query('OUTCOME UnsupportedDesktop',desktop=desktop,factory=self.factory())
                self.assertFalse(self.calls.exists())
    def test_valid_cache_survives_factory_failure(self):
        self.thumbnail()
        out=self.run_query('HIT metadata/container valid',factory=self.factory(status=2))
        self.assertIn('factory=Failed',out)
        self.assertNotIn('capability=CacheAndGenerate',out)
    def test_valid_cache_survives_ineligible_factory(self):
        self.thumbnail()
        out=self.run_query('HIT metadata/container valid',factory=self.factory(output='0'))
        self.assertIn('factory=Ineligible',out)
        self.assertIn('generation_eligible=false',out)
    def test_malformed_helper_protocol_is_not_generation_capability(self):
        out=self.run_query('OUTCOME Missing',factory=self.factory(output='eligible'))
        self.assertIn('factory=Failed',out)
        self.assertNotIn('capability=CacheAndGenerate',out)
    def test_missing_helper_keeps_missing_cache_explicit(self):
        out=self.run_query('OUTCOME Missing',factory=self.root/'absent-helper')
        self.assertIn('factory=Failed',out)
        self.assertNotIn('capability=CacheAndGenerate',out)
    def test_standard_failed_marker(self):
        path=self.thumbnail();failure=self.cache/'thumbnails/fail/gnome-thumbnail-factory'/path.name
        failure.parent.mkdir(parents=True);path.rename(failure)
        self.run_query('OUTCOME FailedCached')
    def test_symlink_cache_is_rejected(self):
        path=self.thumbnail();target=self.root/'linked.png';path.rename(target);path.symlink_to(target)
        self.run_query('OUTCOME Invalid')
    def test_oversized_cache(self):
        path=self.thumbnail()
        with path.open('ab') as file:file.truncate(2*1024*1024+1)
        self.run_query('OUTCOME Invalid')

if __name__=='__main__':unittest.main()
