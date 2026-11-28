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
    parser.add_argument('--development',choices=['open','info','minimap','layout','navigation','shortcuts','zoom-state','context-menu','slideshow-click','slideshow-effects','opening','reveal','open-latency','file-actions','undo','video-mouse','video-speed','video-frame','comparison','multi-selection','all'])
    args=parser.parse_args()
    if args.development in ('video-mouse', 'undo', 'video-frame', 'comparison', 'multi-selection') and args.session != 'x11':
        parser.error(f'{args.development} requires real X11 input from xdotool')
    if os.environ.get('PIVRA_TOOLBAR_PRIVATE_BUS')!='1':
        fixture_parent = None
        if args.development == 'undo' and Path(tempfile.gettempdir()).stat().st_dev != Path.home().stat().st_dev:
            # GLib disallows trashing files from a system-internal /tmp mount.
            fixture_parent = ROOT / 'target' if ROOT.stat().st_dev == Path.home().stat().st_dev else Path.home()
            fixture_parent.mkdir(exist_ok=True)
        out=Path(tempfile.mkdtemp(prefix='pivra-toolbar-ux-', dir=fixture_parent))
        env=dict(os.environ,PIVRA_TOOLBAR_PRIVATE_BUS='1',PIVRA_TOOLBAR_REPORT=str(out),
                 XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),XDG_DATA_HOME=str(out/'data'))
        return subprocess.run(['dbus-run-session','--',sys.executable,'-B',__file__,*sys.argv[1:]],cwd=ROOT,env=env).returncode
    out=Path(os.environ['PIVRA_TOOLBAR_REPORT']);print('Logs:',out,flush=True)
    video=out/'video.mp4'
    fixture = ['ffmpeg','-hide_banner','-loglevel','error','-f','lavfi','-i','testsrc2=size=192x96:rate=12']
    if args.development == 'video-frame':
        fixture[-1] = r'color=red:size=192x96:rate=12,drawbox=w=iw:h=ih:color=blue:t=fill:enable=gte(t\,4)'
    if args.development in ('video-speed', 'video-frame'):
        fixture += ['-f','lavfi','-i','sine=frequency=440:sample_rate=48000',
                    '-c:a','aac','-x264-params',
                    'colorprim=bt709:transfer=bt709:colormatrix=bt709:fullrange=off']
    duration = {'video-mouse': '25', 'video-speed': '12', 'video-frame': '12'}.get(args.development, '1')
    if args.development not in ('undo', 'comparison'):
        subprocess.run(fixture + ['-t',duration,'-c:v','libx264','-pix_fmt','yuv420p',str(video)],check=True)
    subprocess.run(['cargo','build','--locked','--example','slideshow_probe'],cwd=ROOT,check=True)
    env=dict(os.environ,GTK_A11Y='none',PIVRA_TRACE_SLIDESHOW='1')
    if args.development == 'open-latency':
        import shutil
        media=out/'same-media';media.mkdir()
        for name in ('1.png','2.png','10.png'):
            shutil.copyfile(ROOT/'tests/fixtures/images/wide.png',media/name)
        env.update(PIVRA_OPEN_LATENCY_DIR=str(media),PIVRA_PERF='1',PIVRA_TRACE_OPEN='1')
    with session(args.session,env,out,floating=True) as active:
        with window_manager(args.session,active,out):
            modes = (['development-'+s for s in ['open','info','minimap']] if args.development == 'all'
                     else ['development-open-direct','development-open-chooser'] if args.development == 'open-latency'
                     else ['development-'+args.development] if args.development else ['toolbar-ux','toolbar-ux-restore'])
            for mode in modes:
                result=run(mode,['target/debug/examples/slideshow_probe','--controlled','--'+mode,video],active,out,timeout=60)
                if result['status']!='PASS':return result['exit'] if result['exit']>0 else 1
                if args.development == 'video-frame':
                    result = run(mode+'-stock', ['target/debug/examples/slideshow_probe', '--controlled', '--'+mode, video],
                                 dict(active, PIVRA_VIDEO_BACKEND='stock'), out, timeout=60)
                    if result['status'] != 'PASS':
                        return result['exit'] if result['exit'] > 0 else 1
                if args.development == 'open-latency':
                    import json
                    text=(out/(mode+'.log')).read_text()
                    events=[json.loads(line) for line in text.splitlines() if line.startswith('{"pivra_perf":')]
                    requests=[e['value'] for e in events if e['event']=='image_requested']
                    frames=[e['value'] for e in events if e['event']=='image_frame_after_paint']
                    assert len(requests)==4 and requests[0]==requests[2] and requests[1]==requests[3] and requests[0]!=requests[1], 'unexpected selected-media sequence'
                    assert frames==requests, 'missing, duplicated or stale GTK frame observation'
                    assert any(e['event']=='folder_catalog_populated' and e['value']==3 for e in events), 'wrong shared folder catalog'
                    if mode.endswith('chooser'):
                        for stage in ('action-activated','popup-dismiss-returned','file-dialog-descriptor-reused','gtk-presentation-request','gtk-presentation-returned','local-chooser-mapped','local-chooser-after-paint','selection-completed','local-chooser-unmapped'):
                            assert 'stage='+stage in text, 'missing diagnostic stage: '+stage
    print(('PASS focused development '+args.development+' GTK gate' if args.development else 'PASS controlled toolbar and fresh-process persistence')+'; native GNOME visual acceptance NOT TESTED',flush=True)
    return 0

if __name__=='__main__':raise SystemExit(main())
