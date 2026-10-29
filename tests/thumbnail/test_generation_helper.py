"""Real C sidecar + real GLib/GIO/GdkPixbuf, GNOME API stub only.
This certifies IPC/limits/identity guards, NOT GNOME generation or source decode.
"""
import os
import resource
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[2]
HEADER='''#include <gio/gio.h>
#include <gdk-pixbuf/gdk-pixbuf.h>
#include <time.h>
typedef GObject GnomeDesktopThumbnailFactory;
typedef enum { GNOME_DESKTOP_THUMBNAIL_SIZE_NORMAL } GnomeDesktopThumbnailSize;
GnomeDesktopThumbnailFactory *gnome_desktop_thumbnail_factory_new(GnomeDesktopThumbnailSize s);
char *gnome_desktop_thumbnail_path_for_uri(const char*,GnomeDesktopThumbnailSize);
char *gnome_desktop_thumbnail_factory_lookup(GnomeDesktopThumbnailFactory*,const char*,time_t);
gboolean gnome_desktop_thumbnail_factory_can_thumbnail(GnomeDesktopThumbnailFactory*,const char*,const char*,time_t);
GdkPixbuf *gnome_desktop_thumbnail_factory_generate_thumbnail(GnomeDesktopThumbnailFactory*,const char*,const char*,GCancellable*,GError**);
gboolean gnome_desktop_thumbnail_factory_save_thumbnail(GnomeDesktopThumbnailFactory*,GdkPixbuf*,const char*,time_t,GCancellable*,GError**);
'''
STUB='''#include <libgnome-desktop/gnome-desktop-thumbnail.h>
#include <sys/resource.h>
#include <stdlib.h>
#include <unistd.h>
GnomeDesktopThumbnailFactory *gnome_desktop_thumbnail_factory_new(GnomeDesktopThumbnailSize s) {
 (void)s; struct rlimit r;
 if(getenv("FACTORY_ENTERED")) g_file_set_contents(getenv("FACTORY_ENTERED"),"entered",-1,NULL);
 if(getrlimit(RLIMIT_AS,&r)||r.rlim_cur!=2048UL*1024*1024||r.rlim_max!=2048UL*1024*1024) _exit(20);
 if(getrlimit(RLIMIT_CPU,&r)||r.rlim_cur!=15||r.rlim_max!=15) _exit(21);
 if(getrlimit(RLIMIT_FSIZE,&r)||r.rlim_cur!=256UL*1024*1024||r.rlim_max!=256UL*1024*1024) _exit(22);
 return g_object_new(G_TYPE_OBJECT,NULL);
}
char *gnome_desktop_thumbnail_path_for_uri(const char *u,GnomeDesktopThumbnailSize s) { (void)u;(void)s;return g_strdup(getenv("STAMP")); }
char *gnome_desktop_thumbnail_factory_lookup(GnomeDesktopThumbnailFactory *f,const char *u,time_t t) {
 (void)f;(void)u;(void)t; return getenv("CACHED")?g_strdup("cached.png"):NULL;
}
gboolean gnome_desktop_thumbnail_factory_can_thumbnail(GnomeDesktopThumbnailFactory *f,const char *u,const char *m,time_t t) {
 (void)f;(void)u;(void)m;(void)t; return !getenv("INELIGIBLE");
}
GdkPixbuf *gnome_desktop_thumbnail_factory_generate_thumbnail(GnomeDesktopThumbnailFactory *f,const char *u,const char *m,GCancellable *c,GError **e) {
 (void)f;(void)m;(void)c;
 if(getenv("DEBUG_SCRIPT")) {
  g_log("GnomeDesktop",G_LOG_LEVEL_DEBUG,"About to launch script: bwrap --unshare-all /usr/bin/test-thumbnailer");
  if(getenv("FAIL")) g_log("GnomeDesktop",G_LOG_LEVEL_DEBUG,"Failed to launch script: underlying decoder error\\nsecond line");
 }
 if(getenv("DEBUG_FLOOD")) {
  char *large=g_strnfill(10000,'x');
  for(int i=0;i<100;i++) g_log("GnomeDesktop",G_LOG_LEVEL_DEBUG,"Failed to launch script: %s",large);
  g_free(large);
 }
 if(getenv("FAIL")) { g_set_error_literal(e,G_IO_ERROR,G_IO_ERROR_FAILED,"controlled error");return NULL; }
 if(getenv("CHANGE")) {GFile *file=g_file_new_for_uri(u);char *path=g_file_get_path(file);g_file_set_contents(path,"replacement",-1,NULL);g_free(path);g_object_unref(file);}
 return gdk_pixbuf_new(GDK_COLORSPACE_RGB,TRUE,8,getenv("OVERSIZED")?1025:64,32);
}
gboolean gnome_desktop_thumbnail_factory_save_thumbnail(GnomeDesktopThumbnailFactory *f,GdkPixbuf *p,const char *u,time_t t,GCancellable *c,GError **e) {
 (void)f;(void)p;(void)u;(void)t;(void)c;
 if(getenv("SAVE_FAIL")) {g_set_error_literal(e,G_IO_ERROR,G_IO_ERROR_FAILED,"controlled save error");return FALSE;}
 gboolean saved=g_file_set_contents(getenv("STAMP"),"saved",-1,e);
 if(getenv("CHANGE_ON_SAVE")) {GFile *file=g_file_new_for_uri(u);char *path=g_file_get_path(file);g_file_set_contents(path,"late replacement",-1,NULL);g_free(path);g_object_unref(file);}
 return saved;
}
'''
class Generation(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.temp=tempfile.TemporaryDirectory(prefix='pivra-generation-api-')
  cls.addClassCleanup(cls.temp.cleanup)
  cls.root=Path(cls.temp.name)
  header=cls.root/'libgnome-desktop/gnome-desktop-thumbnail.h';header.parent.mkdir();header.write_text(HEADER)
  stub=cls.root/'stub.c';stub.write_text(STUB)
  flags=shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs','gio-2.0','gobject-2.0','gdk-pixbuf-2.0'],text=True))
  cls.binary=cls.root/'helper'
  subprocess.run(['cc','-O2','-Wall','-Wextra','-Werror',f'-I{cls.root}',str(ROOT/'native/thumbnail_factory.c'),str(stub),'-o',str(cls.binary),*flags],check=True)
 def run_case(self,mode='missing',limits=None,**extra):
  with tempfile.TemporaryDirectory(dir=self.root) as tmp:
   source=Path(tmp)/'source.png';source.write_bytes(b'synthetic API input; not decoded')
   stamp=Path(tmp)/'saved'
   def apply_limits():
    for which,value in (limits or []):
     resource.setrlimit(which,(value,value))
   result=subprocess.run([str(self.binary),source.as_uri(),'image/png',str(int(source.stat().st_mtime)),mode],env={**os.environ,**extra,'STAMP':str(stamp)},capture_output=True,text=True,timeout=5,preexec_fn=apply_limits if limits else None)
   self.source_ctime=source.stat().st_ctime_ns
   self.cache_mtime=stamp.stat().st_mtime_ns if stamp.exists() else 0
   return result,stamp.exists()
 def test_generate_limits_and_save(self):
  p,saved=self.run_case();self.assertEqual((p.returncode,p.stdout,saved),(0,'generated\n',True))
 def test_2gib_address_space_and_other_limits_are_reported(self):
  p,saved=self.run_case(PIVRA_TRACE_GNOME_THUMB='1')
  self.assertEqual((p.returncode,saved),(0,True))
  self.assertIn('limits_as=2147483648 limits_cpu=15 limits_fsize=268435456',p.stderr)
 def assert_limit_failure_stops_before_factory(self,which,value):
  marker=self.root/f'entered-{which}'
  p,saved=self.run_case(limits=[(which,value)],FACTORY_ENTERED=str(marker))
  self.assertEqual((p.returncode,p.stdout,saved),(2,'',False))
  self.assertFalse(marker.exists(),'GNOME factory entered after limit failure')
 def test_inherited_address_space_hard_limit_fails_closed(self):
  self.assert_limit_failure_stops_before_factory(resource.RLIMIT_AS,1024**3)
 def test_inherited_cpu_hard_limit_fails_closed(self):
  self.assert_limit_failure_stops_before_factory(resource.RLIMIT_CPU,10)
 def test_inherited_output_hard_limit_fails_closed(self):
  self.assert_limit_failure_stops_before_factory(resource.RLIMIT_FSIZE,128*1024**2)
 def test_cache_hit_does_not_regenerate(self):
  p,saved=self.run_case(CACHED='1');self.assertEqual((p.returncode,p.stdout,saved),(0,'cached\n',False))
 def test_stale_forces_generation_despite_seconds_cache_match(self):
  p,saved=self.run_case('stale',CACHED='1');self.assertEqual((p.returncode,p.stdout,saved),(0,'generated\n',True))
 def test_ineligible_is_explicit(self):
  p,saved=self.run_case(INELIGIBLE='1');self.assertEqual((p.returncode,saved),(3,False))
 def test_generation_failure_does_not_save(self):
  p,saved=self.run_case(FAIL='1');self.assertEqual((p.returncode,saved),(4,False));self.assertIn('controlled error',p.stdout)
 def test_oversized_generated_frame_is_rejected_before_save(self):
  p,saved=self.run_case(OVERSIZED='1');self.assertEqual((p.returncode,saved),(5,False))
  self.assertIn('source changed or output bound',p.stdout)
 def test_cache_save_failure_is_not_generation_success(self):
  p,saved=self.run_case(SAVE_FAIL='1');self.assertEqual((p.returncode,saved),(6,False))
  self.assertIn('controlled save error',p.stdout)
 def test_debug_capture_does_not_corrupt_success_protocol(self):
  p,saved=self.run_case(DEBUG_SCRIPT='1',PIVRA_TRACE_GNOME_THUMB='1')
  self.assertEqual((p.returncode,p.stdout,saved),(0,'generated\n',True))
  self.assertIn('GNOME_FACTORY_CONTEXT mime=image/png',p.stderr)
  self.assertIn('About to launch script: bwrap --unshare-all',p.stderr)
 def test_debug_capture_preserves_underlying_failure_and_domain(self):
  p,saved=self.run_case(DEBUG_SCRIPT='1',FAIL='1',PIVRA_TRACE_THUMBNAILS='1')
  self.assertEqual((p.returncode,saved),(4,False))
  self.assertIn('underlying decoder error',p.stderr)
  self.assertIn('GNOME_FACTORY_ERROR phase=generate domain=g-io-error-quark code=',p.stderr)
  self.assertNotIn('second line',p.stderr.splitlines())
 def test_diagnostics_are_bounded_even_for_verbose_library_messages(self):
  p,saved=self.run_case(DEBUG_FLOOD='1',PIVRA_TRACE_GNOME_THUMB='1')
  self.assertEqual((p.returncode,p.stdout,saved),(0,'generated\n',True))
  self.assertEqual(p.stderr.count('GNOME_FACTORY_LOG'),16)
  self.assertLess(len(p.stderr),34000)
 def test_normal_failure_retains_existing_protocol_without_extra_diagnostics(self):
  p,saved=self.run_case(FAIL='1')
  self.assertEqual((p.returncode,p.stdout,saved),(4,'failed:controlled error\n',False))
  self.assertEqual(p.stderr,'')
 def test_source_change_during_save_discards_own_cache_artifact(self):
  p,saved=self.run_case(CHANGE_ON_SAVE='1');self.assertEqual((p.returncode,saved),(5,False))
  self.assertIn('source changed during cache save',p.stdout)
 def test_source_change_during_generation_prevents_save(self):
  p,saved=self.run_case(CHANGE='1');self.assertEqual((p.returncode,saved),(5,False))
if __name__=='__main__':unittest.main()
