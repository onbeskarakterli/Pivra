import mmap,os,struct,tempfile,unittest
from pathlib import Path
import benchmark as b
class Observer(unittest.TestCase):
    def test_pid_starttime_and_parent(self):
        pid=os.getpid();row=b.processes()[pid];self.assertEqual(row[0],os.getppid());self.assertGreater(int(row[1]),0)
    def test_descendants_do_not_include_unrelated_processes(self):
        table={1:(0,'1','S'),2:(1,'2','S'),3:(2,'3','S'),4:(9,'4','S')}
        self.assertEqual(b.descendants(1,table),{1,2,3})
    def test_partial_requires_correct_size_prefix_zero_tail_and_writable_mapping(self):
        fd=os.memfd_create('glycin-frame',os.MFD_CLOEXEC)
        try:
            os.ftruncate(fd,96)
            with mmap.mmap(fd,96) as m:
                self.assertFalse(b.partial(os.getpid(),96))
                m[:12]=bytes.fromhex('1153bf')*4
                self.assertTrue(b.partial(os.getpid(),96))
                self.assertFalse(b.partial(os.getpid(),97))
                m[84:]=bytes.fromhex('1153bf')*4
                self.assertFalse(b.partial(os.getpid(),96))
                m[84:]=bytes(12);m[:12]=bytes([1])*12
                self.assertFalse(b.partial(os.getpid(),96))
                m[:12]=bytes.fromhex('1153bf')*4
            self.assertFalse(b.partial(os.getpid(),96),'closed writable mapping is not active evidence')
        finally:os.close(fd)
    def test_streamed_fixture_has_exact_header(self):
        with tempfile.TemporaryDirectory() as d:
            path=Path(d)/'source.png';b.fixture(path,64,32);data=path.read_bytes()
            self.assertEqual(data[:8],b'\x89PNG\r\n\x1a\n');self.assertEqual(struct.unpack('>II',data[16:24]),(64,32));self.assertEqual(data[25],2)
if __name__=='__main__':unittest.main()
