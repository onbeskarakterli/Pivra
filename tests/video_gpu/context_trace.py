"""Executed inside GDB, not the application. Observer breakpoints auto-continue."""
import json
import os
import sys
import time
import gdb

sequence = 0
calls = 0
last_stop = None
exit_result = None
return_points = {}
pending_returns = {}
sidecar = open(os.environ['PIVRA_GDB_EVENTS'], 'w') if os.environ.get('PIVRA_GDB_EVENTS') else None

def emit(kind, **fields):
    global sequence
    sequence += 1
    inferior = gdb.selected_inferior()
    thread = gdb.selected_thread()
    event = dict(schema=1, seq=sequence, kind=kind, time_us=time.monotonic_ns()//1000,
                 pid=inferior.pid, thread=thread.global_num if thread else None,
                 ptid=list(thread.ptid) if thread else None, **fields)
    text = json.dumps(event, separators=(',', ':'))
    if sidecar:
        sidecar.write(text+'\n'); sidecar.flush()
    gdb.write('GDB_EVENT '+text+'\n')

def stack():
    frames = []
    try:
        frame = gdb.newest_frame()
        while frame and len(frames) < 16:
            pc = frame.pc()
            frames.append(dict(function=frame.name(), pc=hex(pc),
                               module=gdb.solib_name(pc) or gdb.current_progspace().filename))
            frame = frame.older()
        return dict(stack=frames)
    except Exception as error:
        return dict(stack=frames, stack_error=str(error))

def abi_register(index, returning=False):
    arch = gdb.selected_frame().architecture().name()
    if 'x86-64' in arch:
        registers = ['rdi','rsi','rdx','rcx','r8','r9']; result = 'rax'
    elif 'aarch64' in arch:
        registers = ['x0','x1','x2','x3','x4','x5']; result = 'x0'
    else:
        return None, 'unsupported ABI: '+arch
    register = result if returning else registers[index]
    return int(gdb.parse_and_eval('$'+register)), 'linux-ABI-register:'+register

def argument(index, names):
    frame = gdb.newest_frame()
    for name in names:
        try:
            return int(frame.read_var(name)), 'debug-info:'+name
        except (gdb.error, ValueError):
            pass
    return abi_register(index)

class CallReturn(gdb.Breakpoint):
    """Global call-site observer, never a thread-bound temporary breakpoint.

    GDB 16.3 aborted in find_program_space_for_breakpoint when a GL thread exited
    with Python FinishBreakpoints. Retain only numeric thread IDs/return PCs.
    """
    def __init__(self, pc):
        super().__init__('*'+hex(pc),internal=True)
        self.pc = pc

    def stop(self):
        thread = gdb.selected_thread()
        key = (gdb.selected_inferior().num,thread.global_num,self.pc)
        entries = pending_returns.get(key,[])
        if not entries: return False
        symbol,call_id,fields = entries.pop()
        try:
            value,source = abi_register(0,returning=True)
            if value is not None and symbol!='eglCreateContext': value &= 0xffffffff
            emit('egl_return',symbol=symbol,call_id=call_id,
                 return_value=hex(value) if symbol=='eglCreateContext' and value is not None else value,
                 return_source=source,**fields)
        except Exception as error:
            emit('trace_error',symbol=symbol,call_id=call_id,error=str(error))
        return False

class EglObserver(gdb.Breakpoint):
    def __init__(self, symbol):
        super().__init__(symbol, internal=True)
        self.symbol = symbol; self.samples = 0

    def stop(self):
        global calls
        try:
            if self.samples >= 128:
                self.enabled = False
                emit('trace_limit', symbol=self.symbol, limit=128)
                return False
            self.samples += 1; calls += 1
            display, display_source = argument(0, ['dpy','display'])
            fields = dict(display=hex(display) if display is not None else None,
                          display_source=display_source)
            if self.symbol in ('eglCreateContext','eglDestroyContext'):
                index = 2 if self.symbol=='eglCreateContext' else 1
                names = ['share_context','share','share_list'] if index==2 else ['ctx','context']
                context, source = argument(index, names)
                key = 'share_context' if index==2 else 'context'
                fields.update({key:hex(context) if context is not None else None, key+'_source':source})
            emit('observer_breakpoint', symbol=self.symbol, call_id=calls,
                 auto_continue=True, **fields, **stack())
            caller = gdb.newest_frame().older()
            if caller:
                pc = caller.pc()
                if pc not in return_points: return_points[pc] = CallReturn(pc)
                key = (gdb.selected_inferior().num,gdb.selected_thread().global_num,pc)
                pending_returns.setdefault(key,[]).append((self.symbol,calls,fields))
        except Exception as error:
            # A backtrace/ABI extraction error must not stop an observer breakpoint.
            emit('trace_error', symbol=self.symbol, error=str(error))
        return False

def stopped(event):
    global last_stop
    if isinstance(event, gdb.SignalEvent):
        last_stop = dict(state='signal', signal=event.stop_signal)
        emit('signal_stop', signal=event.stop_signal)
    elif isinstance(event, gdb.BreakpointEvent):
        own = all(isinstance(bp,(EglObserver,CallReturn)) for bp in event.breakpoints)
        last_stop = dict(state='debugger_stopped', intentional=own,
                         breakpoints=[bp.number for bp in event.breakpoints])
        emit('breakpoint_stop', **last_stop)
    else:
        last_stop = dict(state='debugger_stopped', intentional=False, reason=type(event).__name__)

def exited(event):
    global exit_result
    code = getattr(event, 'exit_code', None)
    exit_result = dict(state='exited', exit_code=code) if code is not None else dict(state='exit_unclassified')
    emit('inferior_exited', **exit_result)

def finish(result, debugger_code):
    emit('inferior_result', **result)
    # Never quit reentrantly from a stop callback.
    if result['state'] != 'exited' and gdb.selected_inferior().threads():
        gdb.execute('kill', to_string=True)
    gdb.execute('quit '+str(debugger_code))

def main():
    gdb.events.stop.connect(stopped); gdb.events.exited.connect(exited)
    for symbol in ('eglInitialize','eglCreateContext','eglDestroyContext','eglTerminate'):
        EglObserver(symbol)
    emit('debugger_ready', version=gdb.VERSION, python_version=sys.version.split()[0], return_observer='global-callsite')
    gdb.execute('run')
    for _ in range(16):
        if exit_result:
            code = exit_result.get('exit_code')
            finish(exit_result, code if code is not None else 125)
        if last_stop and last_stop['state']=='signal':
            signal = last_stop['signal']
            gdb.write('GDB_INFERIOR_SIGNAL '+signal+'\n')
            try: gdb.execute('thread apply all bt 20')
            except gdb.error as error: emit('trace_error', error=str(error))
            number = {'SIGABRT':6,'SIGBUS':7,'SIGSEGV':11,'SIGILL':4}.get(signal)
            if number:
                finish(last_stop, 128+number)
            finish(dict(state='debugger_stopped',signal=signal,reason='nonfatal/unclassified signal stop'),125)
        if last_stop and last_stop.get('intentional'):
            gdb.execute('continue')
            continue
        finish(last_stop or dict(state='debugger_stopped', reason='run returned without inferior exit or signal'),125)
    finish(dict(state='debugger_stopped',reason='repeated debugger stops'),125)

try:
    main()
except Exception as error:
    finish(dict(state='debugger_error',error=str(error)),125)
