"""Benchmark format/statistics and failure handling; no native pixel claims."""
import importlib.util
import json
import os
from pathlib import Path
import struct
import sys
import tempfile
import unittest
import zlib
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('performance_collection',ROOT/'tests/performance/collect.py')
collector=importlib.util.module_from_spec(spec);spec.loader.exec_module(collector)
class Performance(unittest.TestCase):
    def test_percentiles_nearest_rank(self):
        self.assertEqual(collector.summary(range(1,21)),dict(n=20,median_us=10.5,p95_us=19,max_us=20))
    def test_png_valid_crc_and_dimensions(self):
        with tempfile.TemporaryDirectory() as d:
            path=Path(d)/'image.png';collector.png(path,8,4);data=path.read_bytes();self.assertEqual(data[:8],b'\x89PNG\r\n\x1a\n');offset=8;pixels=b''
            while offset<len(data):
                length=struct.unpack('>I',data[offset:offset+4])[0];kind=data[offset+4:offset+8];payload=data[offset+8:offset+8+length]
                self.assertEqual(struct.unpack('>I',data[offset+8+length:offset+12+length])[0],zlib.crc32(kind+payload))
                if kind==b'IHDR':self.assertEqual(struct.unpack('>II',payload[:8]),(8,4))
                if kind==b'IDAT':pixels+=payload
                offset+=12+length
            self.assertEqual(len(zlib.decompress(pixels)),4*(1+8*3))
    def test_real_failure_is_not_a_pass_and_no_vram_label(self):
        with tempfile.TemporaryDirectory() as d:
            result=collector.run_case('failure',[sys.executable,'-c','raise SystemExit(3)'],os.environ.copy(),Path(d))
            self.assertEqual(result['status'],'FAIL');self.assertEqual(result['exit'],3)
            self.assertIn('wait4_max_rss_bytes',result);self.assertNotIn('vram_bytes',result)
    def test_explicit_not_run_and_diagnostic_parse(self):
        with tempfile.TemporaryDirectory() as d:
            event=dict(pivra_perf=1,event='preload_queue_depth',status='snapshot',value=2,elapsed_us=0)
            code='import sys; print('+repr(json.dumps(event))+',file=sys.stderr); raise SystemExit(77)'
            result=collector.run_case('blocked',[sys.executable,'-c',code],os.environ.copy(),Path(d))
            self.assertEqual(result['status'],'NOT RUN')
            self.assertEqual(result['diagnostic_ranges']['preload_queue_depth'],dict(min=2,max=2,last=2))
            self.assertTrue((Path(d)/'blocked.resources.jsonl').exists())
if __name__=='__main__':unittest.main()
