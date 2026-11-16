# Read-only EGL lifecycle observation in this one inferior; no GTK replacement.
set pagination off
set confirm off
set debuginfod enabled off
set breakpoint pending on
break eglTerminate
commands
  silent
  printf "GDB_EGL_TERMINATE caller stack\n"
  bt 16
  continue
end
break eglDestroyContext
commands
  silent
  printf "GDB_EGL_DESTROY_CONTEXT caller stack\n"
  bt 12
  continue
end
python
import gdb
def fatal_stop(event):
    if isinstance(event, gdb.SignalEvent):
        codes = {"SIGABRT": 6, "SIGBUS": 7, "SIGSEGV": 11, "SIGILL": 4}
        if event.stop_signal in codes:
            gdb.write("GDB_INFERIOR_SIGNAL " + event.stop_signal + "\n")
            gdb.execute("thread apply all bt 20")
            gdb.execute("quit " + str(128 + codes[event.stop_signal]))
gdb.events.stop.connect(fatal_stop)
end
run
