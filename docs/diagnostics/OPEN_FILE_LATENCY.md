# Open chooser and subsequent photo latency

Status: **OPEN — physical GNOME ~2-second chooser and slow-next-photo reports
not reproduced in the controlled SDK; no native performance fix claimed.**
Base: `a6bc6ae1f46944627eb3f149193e570460c5375a` on `dev`.

## Findings and scope

All three entries (Ctrl+O, empty-state Open File, context Open...) bind directly
to `win.choose-media`. The action dismisses the context-menu chain, sets the
cached initial folder and calls `GtkFileDialog::open` immediately on the GTK
context. There is no application idle, timer, sleep, folder scan or decode before
presentation. The existing last-folder preparation runs on a GIO worker. Parent
windows were mapped/realized; modal was true; repeated Cancel preserved the
current viewer. Popover popdown hides immediately in GTK, without a two-second
animation or deferred launcher.

Important correction to the earlier reuse description: Pivra reuses the
**GtkFileDialog descriptor**, not an already built native chooser. GTK 4.24.1
`gtk/gtkfiledialog.c:create_file_chooser()` constructs a fresh GtkFileChooserNative
on each open; its init constructs the fallback GtkFileChooserDialog. This is GTK
API behavior, not a second Pivra opening pipeline.

GTK 4.24.1 `gdk/gdk.c:environment_has_portals/check_portal_interface` performs
synchronous `ListActivatableNames` and portal `Properties.Get(version)` queries
(the latter cached per interface). `gtk/gtkfilechoosernativeportal.c` subsequently
exports the parent and sends `FileChooser.OpenFile` asynchronously. Therefore a
short Pivra preparation does not prove GTK/portal initialization is fast. The
existing SDK trace actually captured a FileChooser version reply in **149 us**,
with `NameHasNoOwner`; it did not exercise a working native GNOME portal.
These source findings are candidates, not attribution of the physical 2 seconds.
No portal has been disabled, bypassed, manually invoked or forcibly selected.

The Open selection callback still validates local input and saves the chosen
folder before replacing the media. These operations are now separately timed;
small local samples did not justify changing their ordering or error semantics.
Directory enumeration/natural sort and symlink discovery already run on a GIO
worker. Folder reconciliation, thumbnail catalog scheduling, foreground decode,
preload and GTK presentation are now correlated without changing budgets,
concurrency, cache policy or media behavior. No duplicate foreground decode,
worker starvation or chooser-caused navigation delay was observed in this small
controlled case; secure/native workloads remain open.

## Opt-in diagnostics

`PIVRA_TRACE_OPEN=1` emits sparse `PIVRA_OPEN` stage timestamps and parent/GTK
backend/version information. Request IDs distinguish first/subsequent invocations.
It observes public GTK toplevel/model/map/unmap and frame-clock events; all
handlers are disconnected after the request. A passive, process-local GIO bus
filter records FileChooser version and OpenFile send/reply serials. It passes
messages through unchanged, observes no other application's connection and
never calls GTK on the D-Bus I/O thread. Async observer setup may warm the local
session connection but does not activate the portal. A missing `observer-ready`
line means bus tracing was unavailable; missing bus events alone do not prove a
local route. No filenames, URIs or message bodies are emitted by these new events.

`PIVRA_PERF=1` adds a common process-local `monotonic_us` to existing bounded JSONL
performance records. `media_selected`, `image_requested`, `image_worker_started`,
`image_memory_cache`, `image_paintable_ready` and `image_frame_after_paint` use a
path hash in `value` to correlate media without retaining a history or pixels.
Other stages identify input validation/preference writes, folder scan queue and
worker duration, catalog population/application, foreground/preload queue wait,
existing monitor/poll events and navigation dispatch. Existing thumbnail stage,
generation/decode and preload counters now share the same monotonic time axis.

Interpretation:

- `action-activated` → `gtk-presentation-request`: Pivra preparation.
- Request → `gtk-presentation-returned`: synchronous GTK/native setup, including
  any synchronous portal query; it is **not** total visibility latency.
- `local-chooser-mapped` and `local-chooser-after-paint`: in-process GTK evidence.
  They are not physical compositor scanout. Portal windows live in another
  process, so these events must remain absent for them.
- Version send → reply: GIO/portal version negotiation. OpenFile send → reply:
  external service acknowledgement, **not** proof of a visible window.
- Selection/cancel completion measures user interaction too; never count the
  whole response interval as startup delay. Local unmap separately identifies
  dismissal. For an external chooser, physical visibility/dismissal remains
  unobservable with GTK's public FileDialog API.
- Image request → matching frame-after-paint: next GTK frame evidence. Its
  `elapsed_us` is paintable-ready → frame; neither value is actual monitor
  scanout or a separate decode. Abandoned/closed requests do not emit success.

## Controlled measurements (microseconds unless stated)

Environment: runtime GTK **4.18.6**, libadwaita 1.7.6, Linux SDK,
X11/Xvfb/xfwm4, software `ngl`, private D-Bus/XDG directories. GTK **4.24.1 source
inspection is distinct** from this SDK runtime. Physical GNOME, secure Glycin
success and hardware GPU timing were not run. Both fresh probe processes used
exactly the same three naturally sorted 64x32 PNG fixture files; foreground
pixels were explicitly controlled. Thumbnail secure loads encountered the known
sandbox restriction, so this is not a thumbnail generation/cache benchmark.
Filesystem caches were not flushed. Single samples do not establish a speedup.

First/repeated real Ctrl+O (`/tmp/pivra-toolbar-ux-l75qz023`):

| Stage from action | First | Second | Third |
| --- | ---: | ---: | ---: |
| Popup dismiss returned | 52 | 34 | 44 |
| GTK request | 119 | 64 | 89 |
| Local map | 27,122 | 18,733 | 20,027 |
| GTK call returned | 27,202 | 18,926 | 20,174 |
| First chooser after-paint | 145,890 | 42,337 | 54,098 |

Second opening accepted a file; the others cancelled. Context-menu Open, in a
separate focused gate (`/tmp/pivra-toolbar-ux-w8a5sv9c`), dismissed in 31 us,
requested GTK in 103 us, mapped at 28,197 us and painted at 67,604 us.

A/B/C sequence, assertion-bearing sample (`/tmp/pivra-toolbar-ux-dc8pl2lv`):

| Stage | A: direct file | B: empty → Open same file |
| --- | ---: | ---: |
| Input validation on Open UI | n/a | 48 |
| Last-folder write on UI | n/a | 36 |
| Scan worker queue wait | 48 | 32 |
| Scan worker including symlinks | 120 | 133 |
| Catalog application on UI | 697 | 647 |
| Initial foreground queue wait | 35 | 61 |
| Initial request → ready callback | 30,724 | 9,045 |
| Initial request → GTK after-paint | 454,149 | 38,208 |
| First Next → rendered node | 17,422 | 18,678 |
| Repeated Next → rendered node | 16,856 | 7,500 |

B's chooser mapped at 25,108 us and painted at 151,170 us. Its selection callback
completed at 726,193 us; that interval includes typing/accepting the filename.
The initial A frame includes first-window/renderer startup, while B has already
rendered the empty window and chooser; the large A/B difference is **not a decode
or Open performance improvement**. Both first Next requests were already memory
cache hits from neighbor preload: this is not a cold-next decode measurement.

## Validation and native next step

PASS: fmt/check; focused X11 shortcuts gate; two fresh-process Open sequence
cases with assertions for exact media/GTK frame order, same three-item catalog,
all chooser stage events and one viewer window; existing context-menu gate with
its shared chooser/Cancel/focus/submenu assertions; `bash -n` on capture script.
The final identity-guard run (`/tmp/pivra-toolbar-ux-kv3gg84u`) also passed both
cases: frame evidence requires the same still-current paintable and is suppressed
for superseded frames. No full suite, clean build, release build, packaging or
native stress run.

Native capture (normal development build, no backend/portal overrides):

```bash
bash tests/trace_open_latency.sh /absolute/path/to/the/same/photo.jpg
```

Close other Pivra processes first. Follow A/B prompts; navigate immediately and
again warm, then repeat chooser opens through Ctrl+O/context Open and cancel.
The script creates private logs under `/tmp/pivra-open-latency.*`, uses the same
original media read-only and never clears user caches or changes settings itself.
Normal application preference persistence remains unchanged. It builds once,
executes the real production app sequentially and preserves nonzero exits.
Script syntax is checked here; the interactive GNOME run is **NOT RUN**.

To attribute the native delay, compare preparation/GTK return, portal serial
round trips and matched image request/frame/queue stages. If the delay is after
an external OpenFile acknowledgement, backend/journal and physical visibility
observations are still required. No exact GNOME external component is blamed
without that capture. Leave both reported performance issues OPEN; no speculative
scheduler changes, portal prewarming trick or chooser replacement is warranted
by the current controlled data.
