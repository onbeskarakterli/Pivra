"""Bounded v0.2.6 checks. Software sessions never establish native AMD acceptance."""
import argparse
import ctypes
import json
import os
import signal
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'tests/video_gpu'))
from lifecycle import session

EXAMPLES = ['thumbnail_memory_probe', 'preload_probe', 'thumbnail_overlay_probe',
            'view_modes_probe', 'async_cancel_probe', 'window_failure_race_probe']

def outside_pointer(env, *, move=True):
    """Only the private 1280x1024 Xvfb created by --session x11."""
    lib = ctypes.CDLL('libX11.so.6')
    lib.XOpenDisplay.argtypes = [ctypes.c_char_p]
    lib.XOpenDisplay.restype = ctypes.c_void_p
    lib.XDefaultRootWindow.argtypes = [ctypes.c_void_p]
    lib.XDefaultRootWindow.restype = ctypes.c_ulong
    lib.XWarpPointer.argtypes = [ctypes.c_void_p,ctypes.c_ulong,ctypes.c_ulong,*([ctypes.c_int]*6)]
    lib.XQueryPointer.argtypes = [ctypes.c_void_p,ctypes.c_ulong,ctypes.POINTER(ctypes.c_ulong),ctypes.POINTER(ctypes.c_ulong),*([ctypes.POINTER(ctypes.c_int)]*4),ctypes.POINTER(ctypes.c_uint)]
    lib.XSync.argtypes = [ctypes.c_void_p,ctypes.c_int]
    lib.XCloseDisplay.argtypes = [ctypes.c_void_p]
    display = lib.XOpenDisplay(env['DISPLAY'].encode())
    if not display: raise RuntimeError('Cannot verify private X11 pointer')
    try:
        root = lib.XDefaultRootWindow(display)
        if move: lib.XWarpPointer(display,0,root,0,0,0,0,1279,1023)
        lib.XSync(display,0)
        a,b = ctypes.c_ulong(),ctypes.c_ulong()
        x,y,wx,wy = (ctypes.c_int() for _ in range(4))
        mask = ctypes.c_uint()
        ok=lib.XQueryPointer(display,root,ctypes.byref(a),ctypes.byref(b),ctypes.byref(x),ctypes.byref(y),ctypes.byref(wx),ctypes.byref(wy),ctypes.byref(mask))
        if not ok or (x.value,y.value)!=(1279,1023): raise RuntimeError(f'X11 pointer position not verified: actual={(x.value,y.value)}')
        return [x.value,y.value]
    finally: lib.XCloseDisplay(display)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--session',choices=['native','x11','sway'],default='native')
    p.add_argument('--software',action='store_true')
    p.add_argument('--output',type=Path)
    a=p.parse_args()
    if a.session!='native' and not a.software: p.error('Controlled headless sessions require --software')
    if a.session=='native' and (a.software or os.environ.get('PIVRA_GPU_ALLOW_SOFTWARE_TEST')=='1'):
        p.error('Native acceptance refuses inherited software-test permission')
    out=a.output or Path(tempfile.mkdtemp(prefix='pivra-v026-acceptance-'))
    if a.output: out.mkdir(parents=True,exist_ok=False)
    print('Reports:',out,flush=True)
    metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--no-deps','--format-version','1'],cwd=ROOT))
    target=Path(metadata['target_directory'])/'debug/examples'
    build=['cargo','build','--locked']
    for example in EXAMPLES: build+=['--example',example]
    subprocess.run(build,cwd=ROOT,check=True)
    rows=[]
    report=dict(source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
                dirty=bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT)),
                session=a.session,software_control=a.software,device_vram='NOT MEASURED',results=rows)
    def run(name,cmd,env,deadline=120):
        log=out/(name+'.log')
        with log.open('w') as stream:
            proc=subprocess.Popen(cmd,cwd=ROOT,env=env,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
            try: code=proc.wait(timeout=deadline)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid,signal.SIGTERM)
                try: proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(proc.pid,signal.SIGKILL);proc.wait()
                code=124
        status='PASS' if code==0 else 'NOT RUN' if code==77 else 'FAIL'
        rows.append(dict(name=name,status=status,exit=code,log=log.name))
        print(status,name,'exit='+str(code),log,flush=True)
        if status=='FAIL': print('\n'.join(log.read_text(errors='replace').splitlines()[-12:]),flush=True)
    env=dict(os.environ,GTK_A11Y='none',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),XDG_DATA_HOME=str(out/'data'))
    minor=int(subprocess.check_output(['pkg-config','--modversion','gtk4'],text=True).split('.')[1])
    env['GSK_RENDERER']='ngl' if minor<20 else 'gl'
    if a.software: env.update(LIBGL_ALWAYS_SOFTWARE='1',PIVRA_GPU_ALLOW_SOFTWARE_TEST='1')
    with session(a.session,env,out,floating=True) as active:
        if a.session=='x11':
            report['initial_pointer']=outside_pointer(active)
            report['after_client_disconnect_pointer']=outside_pointer(active,move=False)
        for name,args in [
            ('thumbnail-memory',[]),('cache-churn',['--cache-churn-benchmark']),
            ('preload-controlled',['--controlled']),('catalog',['--catalog-regression']),
            ('view-modes',['--synthetic','--zoom-pan']),('async-cancel',['--controlled']),('window-races',[])]:
            example={'thumbnail-memory':'thumbnail_memory_probe','cache-churn':'preload_probe','preload-controlled':'preload_probe',
                     'catalog':'thumbnail_overlay_probe','view-modes':'view_modes_probe','async-cancel':'async_cancel_probe','window-races':'window_failure_race_probe'}[name]
            if a.session=='x11' and name=='catalog': report['catalog_pointer']=outside_pointer(active,move=False)
            run(name,[str(target/example),*args],active)
        lifecycle=['bash','tests/run_video_context_lifecycle.sh']+(['--software'] if a.software else [])
        run('video-lifecycle-nine-gates',lifecycle,active,240)
        if a.session=='native' and rows[-1]['status']=='PASS':
            # Real secure decode, thumbnail, 64MP and hardware baseline. Never
            # bypass Bwrap; environment failures remain in the collector report.
            run('native-performance',['bash','tests/run_performance_baseline.sh','--native','--output',str(out/'performance')],active,360)
    if a.session=='native' and not any(r['name']=='native-performance' for r in rows):
        rows.append(dict(name='native-performance',status='NOT RUN',reason='video lifecycle prerequisite failed; repeated failing decode experiment avoided'))
    report['all_checks_pass']=all(r['status']=='PASS' for r in rows)
    report['native_acceptance']=report['all_checks_pass'] and not a.software and a.session=='native'
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    return 1 if any(r['status']=='FAIL' for r in rows) else 77 if any(r['status']=='NOT RUN' for r in rows) else 0

if __name__=='__main__': raise SystemExit(main())
