#!/usr/bin/env python3
"""Private, bounded performance collection. No user media or cache is touched."""
import argparse
import json
import math
import os
from pathlib import Path
import platform
import random
import shutil
import signal
import statistics
import struct
import subprocess
import tempfile
import time
import zlib

ROOT=Path(__file__).resolve().parents[2]
BINS=Path(os.environ.get('PIVRA_PERF_BIN_DIR',ROOT/'target/release/examples'))
def summary(values):
    values=sorted(values)
    return dict(n=len(values),median_us=statistics.median(values),p95_us=values[max(0,math.ceil(.95*len(values))-1)],max_us=max(values))
def png(path,width,height):
    def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
    compressor=zlib.compressobj(1);parts=[];row=b'\0'+bytes((40,110,180))*width
    for _ in range(height):parts.append(compressor.compress(row))
    parts.append(compressor.flush())
    path.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,8,2,0,0,0))+chunk(b'IDAT',b''.join(parts))+chunk(b'IEND',b''))
def process(pid):
    try:
        stat=Path(f'/proc/{pid}/stat').read_text().rsplit(')',1)[1].split()
        rss=next(int(line.split()[1])*1024 for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('VmRSS:'))
        return rss,int(stat[11])+int(stat[12])
    except (OSError,ValueError,StopIteration):return 0,0

def run_case(name,cmd,env,out,timeout=90):
    started=time.monotonic();samples=[]
    with (out/(name+'.stdout')).open('w') as stdout,(out/(name+'.stderr')).open('w') as stderr,(out/(name+'.resources.jsonl')).open('w') as resources:
        child=subprocess.Popen(cmd,cwd=ROOT,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
        terminated=False
        try:
            while True:
                pid,status,usage=os.wait4(child.pid,os.WNOHANG)
                if pid:
                    child.returncode=os.waitstatus_to_exitcode(status)
                    break
                elapsed=time.monotonic()-started
                sample=process(child.pid);samples.append(sample)
                resources.write(json.dumps(dict(elapsed_s=elapsed,pid=child.pid,rss_bytes=sample[0],cpu_ticks=sample[1]))+'\n')
                if elapsed>timeout and not terminated:
                    os.killpg(child.pid,signal.SIGTERM);terminated=True
                if elapsed>timeout+5:os.killpg(child.pid,signal.SIGKILL)
                time.sleep(.02)
        finally:
            if child.returncode is None:
                os.killpg(child.pid,signal.SIGKILL);child.wait()
    result=dict(wait4_max_rss_bytes=usage.ru_maxrss*1024,wait4_cpu_seconds=usage.ru_utime+usage.ru_stime,timed_out=terminated,name=name,command=cmd,exit=child.returncode,status='PASS' if child.returncode==0 else 'NOT RUN' if child.returncode==77 else 'FAIL',elapsed_s=time.monotonic()-started,parent_peak_sampled_rss_bytes=max((v[0] for v in samples),default=0),parent_max_sampled_cpu_ticks=max((v[1] for v in samples),default=0))
    text=(out/(name+'.stderr')).read_text(errors='replace')
    if terminated:result.update(status='FAIL',reason='bounded benchmark deadline exceeded')
    if result['status']=='FAIL' and any(marker in text for marker in ('software driver rejected','no shared GL context','bwrap:','No loaders configured')):
        result.update(status='NOT RUN',reason='required secure-decoder or hardware GL capability unavailable; inspect stderr, no bypass enabled')
    events=[]
    for line in (out/(name+'.stderr')).read_text(errors='replace').splitlines():
        try:
            event=json.loads(line)
            if event.get('pivra_perf')==1:events.append(event)
        except (ValueError,AttributeError):pass
    result['diagnostic_events']=len(events)
    result['diagnostics_truncated']=len(events)>=4096
    result['diagnostic_values']={event:[e['value'] for e in events if e['event']==event][-1] for event in sorted({e['event'] for e in events if e['status'] in ('snapshot','observed_count')})}
    result['diagnostic_ranges']={event:dict(min=min(e['value'] for e in events if e['event']==event),max=max(e['value'] for e in events if e['event']==event),last=value) for event,value in result['diagnostic_values'].items()}
    rows=[]
    for line in (out/(name+'.stdout')).read_text(errors='replace').splitlines():
        try: rows.append(json.loads(line))
        except ValueError: pass
    result['scenario_latencies']={event:summary([r['elapsed_us'] for r in rows if r.get('scenario')==event and r.get('success') is True]) for event in sorted({r['scenario'] for r in rows if r.get('success') is True and 'elapsed_us' in r})}
    result['pipeline_results']=[r for r in rows if r.get('scenario')=='real_thumbnail_pipeline']
    result['diagnostic_latencies']={event:summary([v['elapsed_us'] for v in events if v['event']==event and v['elapsed_us']>0]) for event in sorted({e['event'] for e in events if e['elapsed_us']>0})}
    return result

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--native',action='store_true');p.add_argument('--output',type=Path);a=p.parse_args()
    out=a.output or Path(tempfile.mkdtemp(prefix='pivra-performance-'))
    if a.output:out.mkdir(parents=True,exist_ok=False)
    if (out/'report.json').exists():p.error('refusing to overwrite existing report')
    report=dict(schema=1,source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),dirty=bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT)),fixtures=dict(scan='64x32 constant RGB PNG and empty MP4 entries, 4:1 ratio; insertion shuffled with seed 2601; enumeration/model timings only',native='1600x1200 PNG, 8000x8000=64MP RGB PNG; generated H264 BT709/sRGB/unknown/full-range 192x96 videos'),environment=dict(platform=platform.platform(),cpu_count=os.cpu_count(),clk_tck=os.sysconf('SC_CLK_TCK'),software_gl_test_permission=os.environ.get('PIVRA_GPU_ALLOW_SOFTWARE_TEST')=='1'),cases=[],limitations=['Warm filesystem caches are not flushed; first scan is not a guaranteed disk-cold read.','RSS samples exclude decoder child processes and GPU VRAM; shared mappings are not PSS. wait4 max RSS/CPU can include reaped descendants and launcher pre-exec mappings, and is not a live process-tree sum or steady-state RSS.','20ms sampling can miss peaks/short processes; CPU is process ticks, not utilization percent.','Diagnostic I/O adds overhead; compare enabled/disabled runs separately.','p95 of 10-11 samples is exploratory, not a production SLA.'])
    env=os.environ.copy();env.update(PIVRA_PERF='1',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),XDG_DATA_HOME=str(out/'data'))
    with tempfile.TemporaryDirectory(prefix='pivra-perf-fixtures-') as temp:
        fixture=Path(temp);png(fixture/'small.png',64,32)
        for count in (100,1000,5000):
            directory=fixture/str(count);directory.mkdir()
            order=list(range(count));random.Random(2601).shuffle(order)
            for i in order:
                # Scan/model fixtures: MP4 entries intentionally contain no media;
                # these measure enumeration/sort, never decoder performance.
                (directory/(str(i)+('.mp4' if i%5==0 else '.png'))).write_bytes(b'' if i%5==0 else (fixture/'small.png').read_bytes())
            for enabled in ('0','1'):
                local=dict(env,PIVRA_PERF=enabled)
                case=run_case(f'scan-{count}-trace{enabled}',[str(BINS/'performance_probe'),str(directory)],local,out)
                rows=[json.loads(line) for line in (out/(case['name']+'.stdout')).read_text().splitlines() if line.startswith('{')]
                case['latencies']={scenario:summary([r['elapsed_us'] for r in rows if r['scenario']==scenario]) for scenario in ('scan','model_navigation_2000')}
                scans=[r for r in rows if r['scenario']=='scan']
                case['first_scan_us']=scans[0]['elapsed_us']
                case['repeat_scan']=summary([r['elapsed_us'] for r in scans[1:]])
                report['cases'].append(case)
        if a.native:
            png(fixture/'1.png',1600,1200);shutil.copyfile(fixture/'1.png',fixture/'2.png');png(fixture/'64mp.png',8000,8000)
            report['cases'].append(run_case('secure-image-gui',[str(BINS/'performance_gui_probe'),str(fixture)],env,out,180))
            # Existing genuine thumbnail cache/generation path, separate from image decode.
            if report['cases'][-1]['exit']==0:
                report['cases'].append(run_case('thumbnail-metadata-only',[str(BINS/'thumbnail_provider_probe'),str(fixture/'1.png')],env,out))
            else:report['cases'].append(dict(name='thumbnail-metadata',status='NOT RUN',reason='secure image gate unavailable'))
            report['cases'].append(dict(name='thumbnail-system-generation',status='NOT RUN',reason='see real_thumbnail_pipeline in secure-image-gui; metadata probe alone is not generation evidence'))
            video=fixture/'video';video.mkdir()
            subprocess.run(['python3','tests/video_color/backend_probe.py','--help'],cwd=ROOT,stdout=subprocess.DEVNULL,check=True)
            import importlib.util
            spec=importlib.util.spec_from_file_location('video_fixtures',ROOT/'tests/video_color/backend_probe.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);module.fixtures(video)
            env['GSK_RENDERER']='ngl' if int(subprocess.check_output(['pkg-config','--modversion','gtk4'],text=True).split('.')[1])<20 else 'gl'
            env['PIVRA_TRACE_VIDEO']='1'
            report['cases'].append(run_case('video-lifecycle',[str(BINS/'integrated_video_probe'),*[str(video/(n+'.mp4')) for n in ('bt709','srgb','unknown','full709')]],env,out))
            if report['cases'][-1]['status']=='NOT RUN':
                report['cases'].append(dict(name='video-seek',status='NOT RUN',reason='video GL prerequisite unavailable'))
            else: report['cases'].append(run_case('video-seek',[str(BINS/'video_color_probe'),str(video/'bt709.mp4'),'--check-controls'],env,out))
        else:
            report['cases'].append(dict(name='native-image-64mp-windows-thumbnail-video',status='NOT RUN',reason='use --native on the target graphical session'))
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(out/'report.json')
    return int(any(c['status']=='FAIL' for c in report['cases']))
if __name__=='__main__':raise SystemExit(main())
