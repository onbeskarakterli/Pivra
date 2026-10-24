"""Metadata/driver tests only. No host JPEG pixel decoding."""
import struct
import subprocess
import unittest
from pathlib import Path
from unittest.mock import Mock, patch
import probe

JPEG = (Path(__file__).resolve().parents[1]/'fixtures/images/no-exif.jpg').read_bytes()

def tiff(order='<', offset=56, compression=6, count=1):
    def tag(number, kind, value):
        raw = struct.pack(order+'H',value)+b'\0\0' if kind == 3 else struct.pack(order+'I',value)
        return struct.pack(order+'HHI',number,kind,count)+raw
    return ((b'II' if order == '<' else b'MM')+struct.pack(order+'HI',42,8)
            +struct.pack(order+'HI',0,14)+struct.pack(order+'H',3)
            +tag(0x103,3,compression)+tag(0x201,4,offset)
            +tag(0x202,4,len(JPEG))+struct.pack(order+'I',0)+JPEG)

def embedded(t):
    payload = b'Exif\0\0'+t
    return JPEG[:2]+b'\xff\xe1'+struct.pack('>H',len(payload)+2)+payload+JPEG[2:]

class Extraction(unittest.TestCase):
    def test_both_endian_real_embedded_jpeg_bytes(self):
        for order in ('<','>'):
            data = embedded(tiff(order))
            original = bytes(data)
            self.assertEqual(probe.thumbnail(data),JPEG)
            self.assertEqual(data,original)
    def test_no_thumbnail_controlled_fallback(self):
        self.assertIsNone(probe.thumbnail(JPEG))
        self.assertIsNone(probe.exif_thumbnail(b'II'+struct.pack('<HIHI',42,8,0,0)))
    def test_bad_offset_length_type_and_compression(self):
        for data in (tiff(offset=55),tiff(offset=2**32-1),tiff(compression=1),
                     tiff(count=2),tiff()[:-1],b'II\0\0'+b'\0'*20):
            with self.assertRaises(ValueError): probe.exif_thumbnail(data)
    def test_truncated_headers_and_segments(self):
        for data in (b'',b'not JPEG',b'\xff\xd8',b'\xff\xd8\xff',
                     b'\xff\xd8\xff\xe1\xff\xffshort'):
            with self.assertRaises(ValueError): probe.thumbnail(data)
    def test_cycles_counts_and_bad_envelope(self):
        for position,value in ((10,8),(4,2**32-1),(52,14),(40,2**32-1)):
            data=bytearray(tiff());struct.pack_into('<I',data,position,value)
            with self.assertRaises(ValueError):probe.exif_thumbnail(data)
        data=bytearray(tiff());struct.pack_into('<H',data,14,129)
        with self.assertRaises(ValueError):probe.exif_thumbnail(data)
        data=bytearray(tiff());data[-1]=0
        with self.assertRaises(ValueError):probe.exif_thumbnail(data)
    @patch('os.killpg')
    @patch.object(probe.subprocess,'Popen')
    def test_timeout_cancels_only_dedicated_sandbox_and_joins(self,popen,kill):
        child=Mock(pid=123)
        child.communicate.side_effect=[subprocess.TimeoutExpired('bwrap',20),(None,None)]
        popen.return_value=child
        with self.assertRaises(subprocess.TimeoutExpired):probe.decode(Path('/test/decoder'),JPEG,8)
        self.assertEqual(popen.call_args.args[0][0],'bwrap')
        self.assertIn('--unshare-all',popen.call_args.args[0])
        self.assertTrue(popen.call_args.kwargs['start_new_session'])
        kill.assert_called_once_with(123,probe.signal.SIGTERM)
        self.assertEqual(child.communicate.call_count,2)

if __name__ == '__main__':unittest.main()
