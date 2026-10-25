"""Native fixtures/parent+child sampling. Partial memfd + pidfd STOP checkpoint for cancellation.
No host image decoder. Missing evidence never counts as a cancellation PASS.
"""
import argparse,json,os,select,signal,struct,subprocess,tempfile,time,zlib
from pathlib import Path

def fixture(path,w,h):
    def chunk(f,k,d):f.write(struct.pack('>I',len(d))+k+d+struct.pack('>I',zlib.crc32(k+d)))
    with path.open('wb') as f:
        f.write(b'\x89PNG\r\n\x1a\n');chunk(f,b'IHDR',struct.pack('>IIBBBBB',w,h,8,2,0,0,0))
        c=zlib.compressobj();row=b'\0'+bytes.fromhex('1153bf')*w
        for _ in range(h):
            data=c.compress(row)
            if data:chunk(f,b'IDAT',data)
        chunk(f,b'IDAT',c.flush());chunk(f,b'IEND',b'')

def processes():
    result={}
    for p in Path('/proc').iterdir():
        if not p.name.isdigit():continue
        try:
            fields=(p/'stat').read_text().rsplit(')',1)[1].split()
            result[int(p.name)]=(int(fields[1]),fields[19],fields[0])
        except (OSError,ValueError,IndexError):pass
    return result

def descendants(root,table):
    found={root}
    while True:
        new={pid for pid,(parent,_,_) in table.items() if parent in found}
        if new<=found:return found
        found|=new

def memory(pid):
    values={}
    for name in ['status','smaps_rollup']:
        try:
            for line in Path(f'/proc/{pid}/{name}').read_text().splitlines():
                key,_,v=line.partition(':')
                if key in ['VmRSS','VmHWM','Rss','Pss']:values[key]=int(v.split()[0])*1024
        except (OSError,ValueError):pass
    return values

def partial(pid,size):
    try:
        maps=Path(f'/proc/{pid}/maps').read_text()
        for fd in Path(f'/proc/{pid}/fd').iterdir():
            if 'memfd:glycin-frame' not in os.readlink(fd):continue
            with fd.open('rb',buffering=0) as f:
                m=os.fstat(f.fileno())
                if m.st_size!=size:continue
                if not any(len(parts:=line.split())>=5 and parts[4]==str(m.st_ino) and parts[3]==f'{os.major(m.st_dev):02x}:{os.minor(m.st_dev):02x}' and 'w' in parts[1] for line in maps.splitlines()):continue
                prefix=f.read(12);f.seek(-12,2);tail=f.read(12)
                if prefix==bytes.fromhex('1153bf')*4 and tail==bytes(12):return True
    except (OSError,ValueError):pass
    return False

def cgroup_peak():
    try:return {'bytes':int(Path('/sys/fs/cgroup/memory.peak').read_text()),'scope':'existing shared cgroup, not isolated job peak; not reset'}
    except (OSError,ValueError):return 'NOT MEASURED'

def job(binary,path,w,h,cancel,extra=()):
    samples=[];handles={};certificate=None;errors=0;peak_before=cgroup_peak();window_protocol='--fd06-active-close' in extra;finished=False;window_deadline=None
    with tempfile.TemporaryFile(mode='w+') as out,tempfile.TemporaryFile(mode='w+') as err:
        p=subprocess.Popen([binary,*extra,str(path)]+(['--cancel-active']if cancel else []),stdin=subprocess.PIPE,stdout=out,stderr=err,text=True,env={**os.environ,'PIVRA_TRACE_FD05A':'1'})
        parent_identity=processes().get(p.pid)
        limit=time.monotonic()+100
        try:
            while p.poll() is None and time.monotonic()<limit:
                table=processes();children=descendants(p.pid,table)-{p.pid};row={'t':time.monotonic(),'parent':memory(p.pid),'loaders':{}}
                for pid in children:
                    try:
                        identity=(pid,table[pid][1])
                        if identity not in handles:handles[identity]=os.pidfd_open(pid)
                        fresh=processes()
                        if pid not in fresh or fresh[pid][1]!=identity[1] or pid not in descendants(p.pid,fresh):continue
                        cmd=Path(f'/proc/{pid}/cmdline').read_bytes()
                        if b'glycin-image-rs' not in cmd or b'bwrap' in cmd:continue
                        row['loaders'][str(identity)]=memory(pid)
                        if cancel and certificate is None and partial(pid,w*h*3):
                            fd=handles[identity];signal.pidfd_send_signal(fd,signal.SIGSTOP)
                            try:
                                stop_deadline=time.monotonic()+1
                                current=processes().get(pid)
                                while current and current[1]==identity[1] and current[2] not in ('T','t') and time.monotonic()<stop_deadline:
                                    time.sleep(.001);current=processes().get(pid)
                                if current and current[1]==identity[1] and current[2] in ('T','t') and partial(pid,w*h*3):
                                    certificate={'pid':pid,'starttime':identity[1],'bytes':w*h*3,'partial_after_stop':True}
                                    p.stdin.write('cancel\n');p.stdin.flush()
                                    # Wait for explicit cancellation issuance before resuming decoder.
                                    deadline=time.monotonic()+3
                                    while time.monotonic()<deadline:
                                        out.flush();out.seek(0)
                                        if 'CANCEL_ISSUED' in out.read():break
                                        time.sleep(.001)
                                    else:raise RuntimeError('cancel acknowledgement absent')
                            finally:
                                try:signal.pidfd_send_signal(fd,signal.SIGCONT)
                                except ProcessLookupError:pass
                    except (OSError,ValueError):errors+=1
                if window_protocol and certificate and not finished:
                    out.seek(0);text=out.read()
                    if 'WINDOW_CLOSED_OTHER_ALIVE' in text:
                        target=handles[(certificate['pid'],certificate['starttime'])]
                        if select.select([target],[],[],0)[0]:
                            certificate['loader_exited_while_other_window_alive']=True
                            p.stdin.write('finish\n');p.stdin.flush();finished=True
                        else:
                            if window_deadline is None:window_deadline=time.monotonic()+5
                            if time.monotonic()>window_deadline:raise RuntimeError('loader still alive while other window open')
                samples.append(row)
                if cancel and certificate is None and time.monotonic()>limit-90:
                    p.stdin.close() # Finished/too-fast decode is NOT active-decode evidence.
                time.sleep(.005)
            if p.poll() is None:raise RuntimeError('native job deadline exceeded')
            out.seek(0);err.seek(0);stdout=out.read();stderr=err.read();code=p.returncode
            cleanup=True
            end=time.monotonic()+5
            while time.monotonic()<end:
                table=processes();live=[pid for (pid,start) in handles if pid in table and table[pid][1]==start and table[pid][2]!='Z']
                if not live:break
                time.sleep(.02)
            else:cleanup=False
            if not cleanup:code=1
            if cancel and (certificate is None or 'PASS CANCELLED' not in stdout or not cleanup):code=(77 if p.returncode in (0,77) else p.returncode) if certificate is None else 1
            return {'parent_pid':p.pid,'parent_starttime':parent_identity[1]if parent_identity else'NOT MEASURED','size':[w,h],'format':path.suffix,'cancel':cancel,'exit':code,'stdout':stdout,'stderr':stderr,'certificate':certificate,'cleanup':cleanup,'tracked_descendants':[{'pid':pid,'starttime':start}for pid,start in handles],'proc_errors':errors,'samples':samples,'child_rss':'MEASURED'if any(('Rss' in v or 'VmRSS' in v)for s in samples for v in s['loaders'].values())else'NOT MEASURED','child_pss':'MEASURED'if any('Pss' in v for s in samples for v in s['loaders'].values())else'NOT MEASURED','cancel_stage':('certified_partial_frame'if certificate else'NOT RUN partial_frame_checkpoint')if cancel else'not_requested','max_observed_loader_processes':max((len(s['loaders']) for s in samples),default=0),'cgroup_peak_before':peak_before,'cgroup_peak_after':cgroup_peak()}
        finally:
            if p.poll() is None:
                p.terminate()
                try:p.wait(timeout=5)
                except subprocess.TimeoutExpired:p.kill();p.wait(timeout=5)
            for fd in handles.values():os.close(fd)

def main():
    a=argparse.ArgumentParser();a.add_argument('--report',default=None);args=a.parse_args();results=[]
    with tempfile.TemporaryDirectory(prefix='pivra-fd05a-') as tmp:
        root=Path(tmp)
        for suffix,w,h in [(s,w,h) for s in ('png','jpg') for w,h in [(4000,3000),(6000,4000),(8000,6000),(10000,5000)]]:
            path=root/f'{w}x{h}.{suffix}'
            if suffix=='png':fixture(path,w,h)
            else:
                made=subprocess.run(['target/debug/examples/fd05a_memory_probe','--fixture-jpeg',str(w),str(h),str(path)],capture_output=True,text=True)
                if made.returncode:
                    results.append({'exit':made.returncode,'fixture':'JPEG generation failed','stderr':made.stderr});print(made.stderr);break
            r=job('target/debug/examples/fd05a_memory_probe',path,w,h,False);results.append(r);print(r['stdout'],r['stderr'],flush=True)
            if r['exit']:break
        else:
            path=root/'10000x5000.png';r=job('target/debug/examples/fd05a_memory_probe',path,10000,5000,True);results.append(r);print(r['stdout'],r['stderr'],flush=True)
    if args.report:
        fd=os.open(args.report,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600);report=args.report
    else:fd,report=tempfile.mkstemp(prefix='pivra-fd05a-report-',suffix='.json')
    with os.fdopen(fd,'w') as f:json.dump(results,f,indent=2)
    print('Report:',report)
    return next((r['exit'] for r in results if r['exit']),0)
if __name__=='__main__':raise SystemExit(main())
