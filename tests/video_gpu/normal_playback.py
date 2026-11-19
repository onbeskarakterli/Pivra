"""Normal GPU playback vs immutable v0.2.5; never conflates mixed stress or visual acceptance."""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tarfile
import tempfile

from lifecycle import ROOT, flags, run, session

BASELINE='726f6948cb391e620a4733348ab164ad3a7ea295'
PRE_FIX='d9d2fdadd2cbbe03b9399024202e5f96c75c83ff'
GPU_MARKERS=('BACKEND selected=PivraGpuMediaFile','PIVRA_GPU_DRAW BT1886 GLMemory -> GLMemory',
             'PIVRA_GPU_PRESENT GLMemory sync-meta retained','PASS MediaStream play/pause/loop/volume/mute')

PRODUCT_MARKERS=('PIVRA_GPU_BACKEND automatic type=PivraGpuMediaFile',*GPU_MARKERS[1:3])

def check_normal(result, log, markers=GPU_MARKERS):
    missing=[marker for marker in markers if marker not in log]
    if result['status']=='PASS' and missing:
        result=dict(result,status='FAIL',reasons=[*result['reasons'],'normal GPU evidence missing: '+', '.join(missing)])
    return result

def acceptance(rows):
    # The old development tree is an observation, not a requirement that a
    # demonstrated old failure must still pass. Its raw result stays in report.
    gates=[row for row in rows if row.get('role')!='pre-fix-reference']
    if not gates: return 77
    return 1 if any(r['status']=='FAIL' for r in gates) else 77 if any(r['status'] in ('NOT RUN','BLOCKED') for r in gates) else 0

def build_binding(source, destination, missing_api=False):
    command=[*shlex.split(os.environ.get('CC','cc')),'-O2','-Wall','-Wextra','-Werror','-Wno-unused-parameter',
             '-DGTK_COMPILATION','-I'+str(source/'native/video'),*shlex.split(os.environ.get('CPPFLAGS','')),*flags('--cflags')]
    if missing_api: command+=['-Dgst_gl_display_egl_set_foreign=pivra_test_missing_egl_set_foreign']
    command+=shlex.split(subprocess.check_output(['pkg-config','--cflags','epoxy'],text=True))
    command += [str(ROOT/'tests/video_gpu/initial_binding.c'),str(source/'native/video/gtkgstpaintable.c'),
                str(source/'native/video/gtkgstsink.c'),'-o',str(destination),*shlex.split(os.environ.get('LDFLAGS','')),
                *flags('--libs'),*shlex.split(subprocess.check_output(['pkg-config','--libs','epoxy'],text=True)),'-lm']
    subprocess.run(command,cwd=ROOT,check=True)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--session',choices=['native','x11','sway'],default='native')
    parser.add_argument('--software',action='store_true')
    parser.add_argument('--video',type=Path,help='read-only eligible BT.709 H264 video; otherwise generate a static full-range ramp')
    parser.add_argument('--manual',action='store_true',help='also open baseline then dev product windows; close each to continue')
    args=parser.parse_args()
    if args.session!='native' and not args.software: parser.error('headless controls require --software')
    if not args.software and os.environ.get('PIVRA_GPU_ALLOW_SOFTWARE_TEST')=='1': parser.error('software permission requires explicit --software')
    if args.manual and (args.software or args.session!='native' or not args.video): parser.error('--manual requires a real native session and --video')
    for key in ('PIVRA_VIDEO_BACKEND','PIVRA_VIDEO_TEST_MODULE_DIR','GTK_MEDIA'):
        if os.environ.get(key): parser.error('unset '+key+' for automatic stock-GTK GPU acceptance')
    if args.video and not args.video.is_file(): parser.error('video is not a file')
    out=Path(tempfile.mkdtemp(prefix='pivra-normal-video-'));print('Reports:',out,flush=True)
    sources={}
    target=Path(json.loads(subprocess.check_output(['cargo','metadata','--locked','--no-deps','--format-version','1'],cwd=ROOT))['target_directory'])
    build=['cargo','build','--locked','--bin','pivra','--example','video_color_probe','--target-dir',str(target)]
    with (out/'build.log').open('w') as log:
        for label,commit in [('v025',BASELINE),('pre-fix',PRE_FIX)]:
            old=out/label;old.mkdir();sources[label]=old
            archive=out/(label+'-source.tar')
            subprocess.run(['git','archive','--format=tar','--output',str(archive),commit],cwd=ROOT,check=True)
            with tarfile.open(archive) as tar: tar.extractall(old,filter='data')
            subprocess.run([*build,'--manifest-path',str(old/'Cargo.toml')],cwd=old,stdout=log,stderr=subprocess.STDOUT,check=True)
            for name in ['pivra','examples/video_color_probe']:
                shutil.copy2(target/'debug'/name,out/(label+'-'+Path(name).name))
        subprocess.run([*build,'--example','integrated_video_probe'],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,check=True)
    for name in ['pivra','examples/video_color_probe','examples/integrated_video_probe']:
        shutil.copy2(target/'debug'/name,out/('dev-'+Path(name).name))
    for label,source in [*sources.items(),('dev',ROOT)]:
        build_binding(source,out/(label+'-binding'))
    build_binding(ROOT,out/'compat-binding',missing_api=True)
    if args.video:
        video=args.video.resolve();extra=[]
    else:
        subprocess.run(['python3','-B','tests/video_gpu/backend/fixtures.py',str(out/'fixtures')],cwd=ROOT,check=True)
        video=out/'fixtures/ramp.mp4';extra=['--check-bt1886-ramp']
    env=dict(os.environ,GTK_A11Y='none',PIVRA_TRACE_VIDEO='1',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),XDG_DATA_HOME=str(out/'data'))
    env['GSK_RENDERER']='ngl' if int(subprocess.check_output(['pkg-config','--modversion','gtk4'],text=True).split('.')[1])<20 else 'gl'
    if args.software: env.update(LIBGL_ALWAYS_SOFTWARE='1',PIVRA_GPU_ALLOW_SOFTWARE_TEST='1')
    setter=hasattr(ctypes.CDLL('libgstgl-1.0.so.0'),'gst_gl_display_egl_set_foreign')
    rows=[]
    with session(args.session,env,out,floating=True) as active:
        for name,binary,options in [('baseline-initialization','v025-binding',[]),
                                    ('pre-fix-initialization','pre-fix-binding',['--require-borrowed']),
                                    ('dev-initialization','dev-binding',['--require-borrowed','--expect-wayland'] if setter else ['--require-borrowed','--expect-egl']),
                                    ('pre126-api-compatibility','compat-binding',['--require-borrowed','--expect-egl'])]:
            result=run(name,[out/binary,*options],active,out,timeout=30)
            if name.startswith('pre-fix-'): result['role']='pre-fix-reference'
            rows.append(result)
        for label in ['v025','pre-fix','dev']:
            name=label+'-normal-playback'
            result=run(name,[out/(label+'-video_color_probe'),video,'--check-controls',*extra],active,out)
            result=check_normal(result,(out/(name+'.log')).read_text(errors='replace'))
            if label=='pre-fix': result['role']='pre-fix-reference'
            rows.append(result)
            print('NORMAL_RESULT',label,result['status'],result['reasons'],flush=True)
        rows.append(run('dev-normal-production-window',[out/'dev-integrated_video_probe','--normal-only',video],active,out))
        if args.manual and acceptance(rows)==0:
            for label in ['v025','dev']:
                print('MANUAL',label,'normal product window; check color, audio, seek and controls; close to continue. Avoid stock/media navigation in this isolated normal check.',flush=True)
                name=label+'-manual-product'
                result=run(name,[out/(label+'-pivra'),video],active,out,timeout=600)
                rows.append(check_normal(result,(out/(name+'.log')).read_text(errors='replace'),PRODUCT_MARKERS))
    report=dict(baseline=BASELINE,pre_fix=PRE_FIX,dev=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
                dirty=bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT)),software_control=args.software,
                session=args.session,foreign_setter_available=setter,results=rows,
                executable_sha256={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in out.iterdir() if p.is_file() and os.access(p,os.X_OK)},
                product_versions={label:subprocess.check_output([str(out/(label+'-pivra')),'--version'],text=True).strip() for label in ['v025','pre-fix','dev']},
                normal_checks_pass=acceptance(rows)==0,
                mixed_lifecycle='NOT RUN by this normal-only command; previous native ownership failure remains OPEN',
                native_visual_acceptance='PENDING user report; process exit and software GL are not visual acceptance')
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print('Mixed lifecycle remains a separate gate: bash tests/run_video_context_lifecycle.sh',flush=True)
    return acceptance(rows)

if __name__=='__main__': raise SystemExit(main())
