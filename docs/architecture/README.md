# Pivra architecture on dev

GTK4 and libadwaita own widgets and presentation. The GTK main context owns
window state, GIO actions, signal callbacks and updates to the widget tree.
Workers do disk enumeration, image loading and bounded thumbnail work. Results
are checked against the current request and window before changing the UI.

| Area | Implementation |
| --- | --- |
| Entry and windows | `src/main.rs`, `src/app.rs` and `src/input.rs` validate launch arguments and route local paths, file URIs and GIO opens. `src/window.rs` coordinates the window actions, catalog selection, title, toolbar and viewer. `src/window_state.rs` persists normal geometry and maximize state. |
| Open and external applications | `src/window/file_chooser.rs` reuses a per-window GtkFileDialog descriptor for empty Open File, Ctrl+O and context Open. Last-folder preparation runs independently and the chosen file routes through the same `open-file` action. `application_chooser.rs` lists and launches installed applications via GIO. `reveal.rs` calls FileManager1 asynchronously with the current file URI. |
| Context menu | `src/window/context_menu.rs` and its `rows.rs` and `submenu.rs` helpers bind existing actions to GTK popovers. The submenu is an adjacent child popup. Selected URI and actual toplevel focus determine dismissal, and opening dialogs explicitly dismisses the menu chain. |
| Catalog and folders | `src/catalog.rs` owns ordered selection. `src/discovery.rs` and `src/natural.rs` enumerate and order candidates. `src/window/folder_watch.rs` coalesces GIO monitor events, runs scans on a worker and reconciles catalog changes in place. Catalog admission is extension based. Decode decides whether a file can actually be displayed. |
| Image presentation | `src/image_view.rs` owns the current texture and media lifecycle. `src/image_view/presentation.rs`, `src/viewing.rs` and `src/actual_paintable.rs` implement Fit/Fill/100%, resampling and geometry. `src/image_view/interaction.rs` handles cursor zoom, panning and video gestures. `src/minimap.rs` draws a bounded view from the existing texture and shared pan state. When mapped, it suppresses thumbnail and arrow overlays without changing user preferences. |
| Decoder and animation | `src/image_loader.rs` is the secure Glycin/Bubblewrap boundary. `src/image_requests.rs` manages cancellation, latest-request identity and bounded admission. `src/animation.rs` advances supported animation frames with bounded buffering. `src/raw_loader.rs` and `raw/` contain the optional, sandboxed experimental DNG path. |
| Image cache and preload | `src/image_cache.rs` tracks file identity, LRU and pinning under a 128 MiB default budget and 64 MiB entry limit. `src/preload.rs` schedules adjacent photos, cancels obsolete work and can promote a matching live preload. `src/image_requests.rs` retains foreground admission and blocks stale completions. Texture charges are estimates. |
| Thumbnail strip | `src/thumbnail_overlay.rs` owns visibility, pinning, resize and filter controls. `src/thumbnail_strip.rs` schedules visible and bounded hidden work, validates cached entries and retains up to 64 textures under a 16 MiB charge. `src/media_filter.rs` filters the projection while retaining the catalog and viewer selection. `src/thumbnail_manager.rs` selects the GNOME-first or Freedesktop provider. `src/thumbnail.rs`, `src/freedesktop_thumbnail.rs`, `src/freedesktop_generator.rs` and `src/freedesktop_writer.rs` implement provider cache and generation behavior. |
| Video | `src/gpu_video.rs` and `native/video/` handle eligibility, the OpenGL BT.1886 adapter and stock GTK fallback. `src/video_controls.rs` owns transport controls. `src/video_inhibit.rs` and `src/inhibit.rs` manage display-sleep inhibition while video plays. Read the [GTK upstream record](../upstream/GTK_4.24.1_WAYLAND_EGL_OWNERSHIP.md) for the native Wayland limitation. |
| Slideshow | `src/slideshow.rs` contains timing, transition options and EOS policy. `src/slideshow_window.rs` coordinates window actions, photo deadlines, video completion and paused inhibition. `src/window/slideshow_interaction.rs` sends photo clicks to manual navigation. `src/photo_transition.rs` uses a GTK frame clock and rendered photo snapshots for the single transition layer. `src/fullscreen.rs` coordinates fullscreen state. |
| Metadata | `src/media_information.rs` records bounded fields from existing image metadata and video discovery. `src/information_panel.rs` uses those fields and an asynchronous file metadata query. Opening the panel does not start a second photo decode. |
| Rename, Trash and Undo | `src/rename_dialog.rs` prepares one selection-bound dialog. `src/rename.rs` performs identity-checked no-clobber rename. The Trash action in `src/window.rs` uses GIO, reconciles the catalog and resolves the precise new trash entry for Undo when the backend supports it. |
| Diagnostics | `src/performance.rs` emits bounded opt-in monotonic events. `src/window/file_chooser/trace.rs` observes local chooser and portal request boundaries and `src/image_view/presentation_trace.rs` records GTK frame evidence. `src/thumbnail_diagnostics.rs` traces thumbnail admission and failure. |

Keep independent features in dedicated modules where practical. Leave
coordination in the window or view and reuse existing GIO actions for alternate
entry points. Keep cancellation and ownership with the component that starts
the request. Stable behavior and version-specific scope belong in the
[release notes](../releases/README.md).
