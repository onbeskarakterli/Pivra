#!/usr/bin/env python3
"""Controlled photo pixels + real GTK windows/stock video. Not hardware acceptance."""
import argparse
import contextlib
import shutil
import time
import os
from pathlib import Path
import subprocess
import sys
import tempfile
sys.path.insert(0, str(Path(__file__).parent/'video_gpu'))
from lifecycle import session, run, ROOT
from slideshow_result import result

@contextlib.contextmanager
def window_manager(kind, env, out):
    if kind!='x11':
        yield
        return
    if not shutil.which('xfwm4') or not shutil.which('xprop'):
        raise RuntimeError('NOT RUN: X11 fullscreen tests require xfwm4 and xprop')
    with (out/'wm.log').open('w') as log:
        wm_env=dict(env,LD_LIBRARY_PATH='',GTK_PATH='',XDG_CONFIG_DIRS='/etc/xdg',XDG_DATA_DIRS='/usr/local/share:/usr/share')
        wm=subprocess.Popen(['xfwm4','--replace','--compositor=off'],env=wm_env,stdout=log,stderr=log)
        try:
            deadline=time.monotonic()+10
            while True:
                prop=subprocess.check_output(['xprop','-root','_NET_SUPPORTING_WM_CHECK'],env=env,text=True)
                if 'window id #' in prop:break
                if wm.poll() is not None or time.monotonic()>deadline:raise RuntimeError('X11 window manager did not publish its EWMH readiness')
                time.sleep(.02)
            yield
        finally:
            wm.terminate()
            try:wm.wait(timeout=5)
            except subprocess.TimeoutExpired:wm.kill();wm.wait()

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--session',choices=['x11','sway'],required=True)
    p.add_argument('--eos-regression',action='store_true')
    args=p.parse_args()
    required=['cargo','ffmpeg','dbus-run-session',*(['Xvfb','xfwm4','xprop'] if args.session=='x11' else ['sway'])]
    missing=[name for name in required if not shutil.which(name)]
    if missing:
        print('NOT RUN missing controlled GUI prerequisites: '+', '.join(missing),flush=True)
        return 77
    if os.environ.get('PIVRA_SLIDESHOW_PRIVATE_BUS') != '1':
        out=Path(tempfile.mkdtemp(prefix='pivra-slideshow-control-'))
        env=dict(os.environ,PIVRA_SLIDESHOW_PRIVATE_BUS='1',PIVRA_SLIDESHOW_REPORT_DIR=str(out),
                 XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),XDG_DATA_HOME=str(out/'data'))
        return subprocess.run(['dbus-run-session','--',sys.executable,'-B',__file__,*sys.argv[1:]],cwd=ROOT,env=env).returncode
    out=Path(os.environ['PIVRA_SLIDESHOW_REPORT_DIR'])
    print('Logs:',out,flush=True)
    video=out/'video.mp4'
    subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-f','lavfi','-i','testsrc2=size=192x96:rate=12','-t','5','-c:v','libx264','-pix_fmt','yuv420p',str(video)],check=True)
    subprocess.run(['cargo','build','--locked','--example','slideshow_probe'],cwd=ROOT,check=True)
    env=dict(os.environ,XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),XDG_DATA_HOME=str(out/'data'),GTK_A11Y='none',PIVRA_TRACE_SLIDESHOW='1')
    with session(args.session,env,out,floating=True) as active:
        with window_manager(args.session,active,out):
            extra=['--eos-regression'] if args.eos_regression else []
            row=run('slideshow',['target/debug/examples/slideshow_probe','--controlled',*extra,video],active,out,timeout=60)
    report=result(row['exit'],(out/'slideshow.log').read_text(),cycles=args.eos_regression)
    if report['status']!='PASS':
        print(f'{report["status"]}: {report["reason"]}',flush=True)
        return 77 if report['status']=='NOT RUN' else 1
    if row['status']!='PASS':
        print('FAIL: existing lifecycle log gate rejected this run',flush=True)
        return 1
    print('Physical GNOME inhibition, secure photo decode and AMD GPU: NOT RUN',flush=True)
    return row['exit'] if row['exit']>=0 else 1

if __name__=='__main__':raise SystemExit(main())
