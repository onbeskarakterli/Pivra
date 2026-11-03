# Application-owned stock-GTK GL video adapter

`gtkgstsink*` and `gtkgstpaintable*` originate from GNOME GTK **4.18.6**
`modules/media`, with retained copyright notices, namespaced types/symbols,
GL-only transport, bounded pending presentation and the Pivra transfer stage.
The pinned original source hashes remain in
`tests/video_color/backend_probe.py:FILES`. `gtkgstmediafile*` retains the
original interface/header and has a new direct `playbin3` controller; it does
not use GstPlayer/GstPlay. `bridge.c`, `policy.h` and `filter.c` implement
application selection and GPU transfer policy.

GTK-derived files: **LGPL-2.1-or-later**, see `COPYING.LGPL-2.1`.
GStreamer GL APIs used here are LGPL-2.1-or-later. The additional LGPL-2.0
text is provided for relevant runtime components; consult installed component
notices, not this directory alone, for the complete runtime distribution.
The application remains GPL-3.0-only; these notices do not relicense GTK.

The normal Cargo build compiles these C sources into a static adapter archive
and links system GTK/GStreamer shared libraries. No downloaded source, runtime
module path, Python or custom GTK build is required. Build requires a C compiler,
`ar`, pkg-config, GTK development files and GStreamer >=1.24 GL, video,
allocators and pbutils development APIs. Missing APIs retain stock-only support
with a diagnostic. C compilation errors stop the build. For a controlled
stock-only build: `PIVRA_DISABLE_GPU_BUILD=1 cargo build --locked`.

Distributors must retain upstream notices/license texts, provide corresponding
modified adapter source and build instructions, and satisfy applicable LGPL
relinking obligations. Rebuilding this GPL application from complete source
with modified LGPL adapter sources is supported by `cargo build --locked` with
the same native development APIs. Distribution/license-document collection and
Arch packaging changes remain a separate task; no packaging files are changed
by this integration.

See `docs/diagnostics/VIDEO_COLOR_STOCK_GTK_INTEGRATION.md` for eligibility,
lifecycle, test evidence and remaining native acceptance requirements.
