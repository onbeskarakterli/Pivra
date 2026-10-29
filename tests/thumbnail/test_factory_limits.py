"""Real helper compiled with real GObject and a GNOME factory API stub.
This tests IPC/resource policy and linking, not actual GNOME factory behavior.
"""
import os
from pathlib import Path
import shlex
import resource
import signal
import subprocess
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[2]
HEADER='''#include <glib-object.h>
#include <time.h>
typedef GObject GnomeDesktopThumbnailFactory;
typedef enum { GNOME_DESKTOP_THUMBNAIL_SIZE_NORMAL } GnomeDesktopThumbnailSize;
GnomeDesktopThumbnailFactory *gnome_desktop_thumbnail_factory_new(GnomeDesktopThumbnailSize size);
gboolean gnome_desktop_thumbnail_factory_can_thumbnail(GnomeDesktopThumbnailFactory *f,const char *u,const char *m,time_t t);
'''
STUB='''#define _GNU_SOURCE
#include <libgnome-desktop/gnome-desktop-thumbnail.h>
#include <sys/mman.h>
#include <sys/resource.h>
#include <unistd.h>
#include <stdlib.h>
GnomeDesktopThumbnailFactory *gnome_desktop_thumbnail_factory_new(GnomeDesktopThumbnailSize size) {
    (void)size;
    struct rlimit r;
    if (getrlimit(RLIMIT_FSIZE,&r) || r.rlim_cur != 1024*1024 || r.rlim_max != 1024*1024) _exit(10);
    if (getrlimit(RLIMIT_AS,&r) || r.rlim_cur != 256UL*1024*1024) _exit(11);
    if (getrlimit(RLIMIT_CPU,&r) || r.rlim_cur != 1) _exit(12);
    if (getenv("ZERO_LIMIT")) { struct rlimit zero={0,0}; if(setrlimit(RLIMIT_FSIZE,&zero)) _exit(14); }
    int fd=memfd_create("factory-metadata-test",MFD_CLOEXEC);
    if (fd<0 || ftruncate(fd,4096)) _exit(13);
    close(fd);
    if (getenv("FAIL_FACTORY")) return NULL;
    return g_object_new(G_TYPE_OBJECT,NULL);
}
gboolean gnome_desktop_thumbnail_factory_can_thumbnail(GnomeDesktopThumbnailFactory *f,const char *u,const char *m,time_t t) {
    (void)f;(void)u;(void)m;(void)t;
    return getenv("INELIGIBLE")==NULL;
}
'''

class Limits(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp=tempfile.TemporaryDirectory(prefix='pivra-th01-limits-')
        cls.addClassCleanup(cls.temp.cleanup)
        folder=Path(cls.temp.name)
        (folder/'libgnome-desktop').mkdir()
        (folder/'libgnome-desktop/gnome-desktop-thumbnail.h').write_text(HEADER)
        (folder/'stub.c').write_text(STUB)
        cls.binary=folder/'factory'
        flags=shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs','gobject-2.0'],text=True))
        subprocess.run(['cc','-O2','-Wall','-Wextra','-Werror','-I'+str(folder),str(ROOT/'tests/thumbnail/factory_probe.c'),str(folder/'stub.c'),'-o',str(cls.binary),*flags],check=True,capture_output=True,text=True)
    def run_helper(self,**env):
        return subprocess.run([str(self.binary),'file:///test.png','image/png','1'],env={**os.environ,**env},capture_output=True,text=True,timeout=3,preexec_fn=lambda:resource.setrlimit(resource.RLIMIT_CORE,(0,0)))
    def test_old_zero_ceiling_reproduces_kernel_sigxfsz(self):
        p=self.run_helper(ZERO_LIMIT='1');self.assertEqual(p.returncode,-signal.SIGXFSZ)
    def test_small_memfd_initialization_no_sigxfsz_and_true(self):
        p=self.run_helper();self.assertEqual(p.returncode,0,p.stderr);self.assertEqual(p.stdout,'1\n')
    def test_false_is_preserved(self):
        p=self.run_helper(INELIGIBLE='1');self.assertEqual(p.returncode,0,p.stderr);self.assertEqual(p.stdout,'0\n')
    def test_real_factory_failure_is_not_success(self):
        p=self.run_helper(FAIL_FACTORY='1');self.assertEqual(p.returncode,2);self.assertEqual(p.stdout,'')
    def test_provider_preserves_real_helper_failure(self):
        with tempfile.TemporaryDirectory(prefix='pivra-factory-failed-') as cache:
            env={**os.environ,'XDG_CURRENT_DESKTOP':'GNOME','XDG_CACHE_HOME':cache,
                 'PIVRA_THUMBNAIL_FACTORY_PROBE':str(self.binary),'FAIL_FACTORY':'1'}
            p=subprocess.run([str(ROOT/'target/debug/examples/thumbnail_provider_probe'),str(ROOT/'tests/fixtures/images/large.png')],env=env,capture_output=True,text=True,timeout=3)
            self.assertEqual(p.returncode,0,p.stderr)
            self.assertIn('factory=Failed',p.stdout)
            self.assertIn('exit status: 2',p.stdout)
            self.assertIn('OUTCOME Missing',p.stdout)
            self.assertNotIn('capability=CacheAndGenerate',p.stdout)

if __name__=='__main__':unittest.main()
