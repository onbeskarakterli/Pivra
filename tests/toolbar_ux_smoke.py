#!/usr/bin/env python3
"""Focused toolbar + actual process restart. Controlled pixels, not native decode."""
import argparse
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from slideshow_smoke import window_manager
from video_gpu.lifecycle import session, run, ROOT

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--session',choices=['x11','sway'],required=True)
    parser.add_argument('--development',choices=['open','info','minimap','all'])
    args=parser.parse_args()
    if os.environ.get('PIVRA_TOOLBAR_PRIVATE_BUS')!='1':
        out=Path(tempfile.mkdtemp(prefix='pivra-toolbar-ux-'))
        env=dict(os.environ,PIVRA_TOOLBAR_PRIVATE_BUS='1',PIVRA_TOOLBAR_REPORT=str(out),
                 XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),XDG_DATA_HOME=str(out/'data'))
        return subprocess.run(['dbus-run-session','--',sys.executable,'-B',__file__,*sys.argv[1:]],cwd=ROOT,env=env).returncode
    out=Path(os.environ['PIVRA_TOOLBAR_REPORT']);print('Logs:',out,flush=True)
    video=out/'video.mp4'
    subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-f','lavfi','-i','testsrc2=size=192x96:rate=12',
                    '-t','1','-c:v','libx264','-pix_fmt','yuv420p',str(video)],check=True)
    subprocess.run(['cargo','build','--locked','--example','slideshow_probe'],cwd=ROOT,check=True)
    env=dict(os.environ,GTK_A11Y='none',PIVRA_TRACE_SLIDESHOW='1')
    with session(args.session,env,out,floating=True) as active:
        with window_manager(args.session,active,out):
            modes = (['development-'+s for s in ['open','info','minimap']] if args.development == 'all'
                     else ['development-'+args.development] if args.development else ['toolbar-ux','toolbar-ux-restore'])
            for mode in modes:
                result=run(mode,['target/debug/examples/slideshow_probe','--controlled','--'+mode,video],active,out,timeout=60)
                if result['status']!='PASS':return result['exit'] if result['exit']>0 else 1
    print(('PASS focused development '+args.development+' GTK gate' if args.development else 'PASS controlled toolbar and fresh-process persistence')+'; native GNOME visual acceptance NOT TESTED',flush=True)
    return 0

if __name__=='__main__':raise SystemExit(main())
