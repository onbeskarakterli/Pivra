# Read-only observation of one inferior; no inferior GTK/GL calls or overrides.
set pagination off
set confirm off
set debuginfod enabled off
set breakpoint pending on
set disable-randomization off
set stop-on-solib-events 0
python
import runpy
runpy.run_path("tests/video_gpu/context_trace.py", run_name="__main__")
end
