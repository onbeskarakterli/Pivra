"""Debugger results and handle associations. Associations are not leak verdicts."""
import json
import re

def records_from(log, sidecar):
    text = sidecar.read_text(errors='replace') if sidecar.exists() else None
    lines = text.splitlines() if text is not None else [line.removeprefix('GDB_EVENT ') for line in log.splitlines() if line.startswith('GDB_EVENT ')]
    records = []
    for line in lines:
        try:
            value = json.loads(line)
            if value.get('schema') != 1: raise ValueError('unknown debugger schema')
            records.append(value)
        except (ValueError, AttributeError):
            records.append(dict(kind='protocol_error', reason='unreadable debugger event'))
    return records

def outcome(code, records):
    results = [r for r in records if r.get('kind')=='inferior_result']
    base = dict(status='BLOCKED', application_status='UNKNOWN', debugger_status='INCOMPLETE',
                inferior_result=results[-1] if results else None,
                debugger_version=next((r.get('version') for r in records if r.get('kind')=='debugger_ready'),None))
    if len(results)!=1 or any(r.get('kind')=='protocol_error' for r in records):
        return dict(base, reason='debugger did not provide exactly one verified inferior result; raw debugger exit is not an application exit')
    result = results[0]
    if result.get('state')=='signal':
        if result.get('signal') not in ('SIGABRT','SIGBUS','SIGSEGV','SIGILL'):
            return dict(base,reason='nonfatal/unclassified debugger signal stop')
        return dict(base,status='FAIL',application_status='SIGNAL',debugger_status='COMPLETE',reason='inferior signal stop confirmed')
    if result.get('state')!='exited' or type(result.get('exit_code')) is not int:
        return dict(base,reason='debugger stopped/failed before a normal inferior exit was observed')
    application_exit = result['exit_code']
    app = 'PASS' if application_exit==0 else 'NOT RUN' if application_exit==77 else 'FAIL'
    if code != application_exit:
        return dict(base,application_status=app,reason='debugger/inferior exit mismatch')
    return dict(base,status=app,application_status=app,debugger_status='COMPLETE',reason='inferior exit confirmed')

def fields(line):
    return dict(re.findall(r'([a-z_]+)=([^\s]+)',line))

def number(value):
    try: return int(str(value),0)
    except (ValueError,TypeError): return None

def in_lifetime(record, key, obj, start, end, pid):
    instant = number(record.get('time_us'))
    return obj is not None and number(record.get(key))==obj and number(record.get('pid'))==pid and instant is not None and start<=instant<=end

def ownership(records, log):
    maps, retirements, releases = [], [], []
    for line in log.splitlines():
        data = fields(line)
        if line.startswith('PIVRA_GPU_CONTEXT_OWNER '): maps.append(data)
        elif line.startswith('PIVRA_GPU_CONTEXT_RELEASE '): releases.append(data)
        elif line.startswith('PIVRA_GPU_LIFECYCLE ') and data.get('stage') in ('context-retired','gdk-context-retired'): retirements.append(data)
    returns = {r.get('call_id'):r for r in records if r.get('kind')=='egl_return'}
    observed = []
    for event in records:
        if event.get('kind')!='observer_breakpoint' or event.get('symbol') not in ('eglDestroyContext','eglTerminate'): continue
        display, context, instant = number(event.get('display')),number(event.get('context')),event.get('time_us',0)
        pid = event.get('pid')
        transitions = [r for r in records if r.get('kind')=='egl_return' and r.get('symbol') in ('eglInitialize','eglTerminate') and r.get('return_value')==1 and r.get('pid')==pid and number(r.get('display'))==display and r.get('time_us',0)<instant]
        last_transition = max(transitions,key=lambda r:r['time_us'],default=None)
        requests = [r for r in records if r.get('kind')=='observer_breakpoint' and r.get('symbol') in ('eglInitialize','eglTerminate') and r.get('pid')==pid and number(r.get('display'))==display and r.get('time_us',0)<instant]
        last_request = max(requests,key=lambda r:r['time_us'],default=None)
        creations = [r for r in records if r.get('kind')=='egl_return' and r.get('symbol')=='eglCreateContext' and r.get('pid')==pid and number(r.get('display'))==display and number(r.get('return_value'))==context and r.get('time_us',0)<instant]
        generation = max((r['time_us'] for r in creations),default=0)
        candidates = []
        if event['symbol']=='eglDestroyContext' and display and context:
            for entry in maps:
                registered = number(entry.get('time_us'))
                if registered is None or not generation<=registered<=instant: continue
                # GstGLPlatform's public EGL value is 1; GLX display handles
                # must not be associated with an EGLDisplay by numeric accident.
                if number(entry.get('platform'))!=1: continue
                if number(entry.get('pid')) != pid or number(entry.get('native_display'))!=display: continue
                for role,handle,obj in [('pivra-worker','native_context','object'),('gdk-share-context','share_context','gdk_object')]:
                    if number(entry.get(handle))!=context: continue
                    retired = [r for r in retirements if in_lifetime(r,'owner',number(entry.get(obj)),registered,instant,pid)]
                    released = [r for r in releases if in_lifetime(r,'object',number(entry.get('object')),registered,instant,pid)]
                    candidates.append(dict(role=role,object=entry.get(obj),sink=entry.get('owner'),registered_us=registered,
                                           retirement_observed_before_request=bool(retired),
                                           last_reference_snapshot=released[-1] if role=='pivra-worker' and released else None))
        if not generation and candidates:
            latest=max(c['registered_us'] for c in candidates)
            candidates=[c for c in candidates if not any(in_lifetime(r,'owner',number(c['object']),c['registered_us'],latest,pid) for r in retirements)]
        observed.append(dict(symbol=event['symbol'],call_id=event.get('call_id'),time_us=instant,
                             display=event.get('display'),context=event.get('context'),pid=pid,ptid=event.get('ptid'),
                             request_return=returns.get(event.get('call_id'),{}).get('return_value'),
                             generation_start_us=generation or None,owner_candidates=candidates,
                             display_last_successful_transition=last_transition,
                             display_last_request=last_request,
                             caveat='request/association only; EGL may defer destruction; absent retirement trace does not prove a live reference'))
    return observed
