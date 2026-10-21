#!/usr/bin/env python3
"""Observer guard tests only; these never certify a real decoder or GUI PASS."""
import os
from pathlib import Path
import struct
import tempfile
import unittest
import zlib

import active_decode_close as probe


class ObserverGuards(unittest.TestCase):
    def test_only_exact_prefix_with_unwritten_tail_is_a_candidate(self):
        with tempfile.TemporaryFile() as output:
            output.truncate(probe.SIZE)
            self.assertFalse(probe.partial(output.fileno()), 'zero allocation is not decode')
            os.pwrite(output.fileno(), probe.COLOR * 32, 0)
            self.assertTrue(probe.partial(output.fileno()))
            os.pwrite(output.fileno(), probe.COLOR * 32, probe.SIZE - 96)
            self.assertFalse(probe.partial(output.fileno()), 'completed output is not active')

    def test_wrong_pixel_format_or_source_is_rejected(self):
        with tempfile.TemporaryFile() as output:
            output.truncate(probe.SIZE)
            os.pwrite(output.fileno(), bytes([255]) * 96, 0)
            self.assertFalse(probe.partial(output.fileno()))

    def test_wrong_size_is_rejected(self):
        with tempfile.TemporaryFile() as output:
            output.truncate(probe.SIZE - 1)
            os.pwrite(output.fileno(), probe.COLOR * 32, 0)
            self.assertFalse(probe.partial(output.fileno()))

    def test_generated_rgb_png_crc_dimensions_and_scanlines(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / 'small.png'
            probe.png(path, width=8, height=3)
            data = path.read_bytes()
        self.assertEqual(data[:8], b'\x89PNG\r\n\x1a\n')
        chunks = []
        offset = 8
        compressed = b''
        while offset < len(data):
            size = struct.unpack('>I', data[offset:offset+4])[0]
            kind = data[offset+4:offset+8]
            value = data[offset+8:offset+8+size]
            crc = struct.unpack('>I', data[offset+8+size:offset+12+size])[0]
            self.assertEqual(crc, zlib.crc32(kind + value))
            chunks.append(kind)
            if kind == b'IHDR':
                self.assertEqual(struct.unpack('>IIBBBBB', value), (8, 3, 8, 2, 0, 0, 0))
            if kind == b'IDAT':
                compressed += value
            offset += size + 12
        self.assertEqual(chunks[0], b'IHDR')
        self.assertEqual(chunks[-1], b'IEND')
        self.assertEqual(zlib.decompress(compressed), (b'\0' + probe.COLOR * 8) * 3)

    def test_linux_process_identity_is_not_just_pid(self):
        state, birth = probe.stat(os.getpid())
        self.assertIn(state, 'RSDIT')
        self.assertGreater(birth, 0)
        self.assertNotIn(os.getpid(), probe.descendants(os.getpid()))


if __name__ == '__main__':
    unittest.main()
