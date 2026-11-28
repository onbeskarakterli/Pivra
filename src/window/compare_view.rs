//! Window-local comparison, using the ordinary secure decoder and transforms.

use super::{context_menu, file_chooser, slideshow, undo::Undo};
use crate::{
    image_loader,
    image_view::{self, ImageView, actual_paintable::ActualPaintable},
};
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use pivra::viewing::{Transform, ViewMode};
use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

pub(super) const OPEN: &str = "compare-images";
pub(super) const SELECTED: &str = "compare-selected-images";
const CLOSE: &str = "close-comparison";
const TEXTURE_BUDGET: usize = 128 * 1024 * 1024;
static COMPARING: AtomicBool = AtomicBool::new(false);

struct DecodePermit;
impl Drop for DecodePermit {
    fn drop(&mut self) {
        COMPARING.store(false, Ordering::Release);
    }
}

pub(super) fn candidate(path: &Path) -> bool {
    pivra::discovery::supported_photo(path)
        && !path.extension().and_then(|s| s.to_str()).is_some_and(|s| {
            ["gif", "apng", "dng"]
                .iter()
                .any(|ext| s.eq_ignore_ascii_case(ext))
        })
}

struct Frame {
    path: PathBuf,
    texture: gdk::Texture,
}

enum FrameSource {
    Ready(gdk::Texture),
    Decoded(image_loader::LoadedFrame),
}

struct Pane {
    picture: gtk::Picture,
    name: gtk::Label,
    mode: gtk::DropDown,
    frame: RefCell<Option<Frame>>,
    transform: Rc<RefCell<Transform>>,
}

impl Pane {
    fn new() -> (Self, gtk::Box) {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_hexpand(true);
        let caption = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let name = gtk::Label::new(None);
        name.set_hexpand(true);
        name.set_xalign(0.);
        name.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        name.set_margin_start(10);
        let mode = gtk::DropDown::from_strings(&ViewMode::ALL.map(ViewMode::label));
        mode.set_tooltip_text(Some("Display mode"));
        caption.append(&name);
        caption.append(&mode);
        container.append(&caption);
        let picture = gtk::Picture::builder()
            .hexpand(true)
            .vexpand(true)
            .can_shrink(true)
            .content_fit(gtk::ContentFit::Fill)
            .overflow(gtk::Overflow::Hidden)
            .build();
        container.append(&picture);
        (
            Self {
                picture,
                name,
                mode,
                frame: RefCell::default(),
                transform: Rc::default(),
            },
            container,
        )
    }

    fn mode(&self) -> ViewMode {
        ViewMode::ALL[self.mode.selected().min(2) as usize]
    }

    fn dimensions(&self) -> Option<((f64, f64), (f64, f64))> {
        let frame = self.frame.borrow();
        let texture = &frame.as_ref()?.texture;
        Some((
            (texture.width().into(), texture.height().into()),
            (self.picture.width().into(), self.picture.height().into()),
        ))
    }

    fn show(&self) {
        if let Some(frame) = self.frame.borrow().as_ref() {
            let label = frame.path.file_name().unwrap_or_default().to_string_lossy();
            self.name.set_text(&label);
            self.name
                .set_tooltip_text(Some(&frame.path.to_string_lossy()));
            let paint = ActualPaintable::transformed(
                &frame.texture,
                self.transform.clone(),
                self.mode(),
                Rc::default(),
            );
            self.picture.set_paintable(Some(&paint));
        } else {
            self.name.set_text("");
            self.name.set_tooltip_text(None);
            self.picture.set_paintable(None::<&gdk::Paintable>);
        }
    }

    fn redraw(&self) {
        if let Some(paintable) = self.picture.paintable() {
            paintable.invalidate_contents();
        }
    }
}

struct Compare {
    stack: glib::WeakRef<gtk::Stack>,
    panes: [Pane; 2],
    linked: gtk::CheckButton,
    choose: gio::SimpleAction,
    selected: gio::SimpleAction,
    close: gio::SimpleAction,
    pending: RefCell<Option<gio::Cancellable>>,
    permit: RefCell<Option<DecodePermit>>,
    active: Cell<bool>,
    paused: glib::WeakRef<gtk::MediaFile>,
    pair_available: Cell<bool>,
    available: Cell<bool>,
}

impl Compare {
    fn stop(&self) {
        let pending = self.pending.borrow_mut().take();
        if let Some(pending) = pending {
            pending.cancel();
        }
        self.active.set(false);
        if let Some(media) = self.paused.upgrade() {
            self.paused.set(None);
            if media.is_prepared() && media.error().is_none() && !media.is_ended() {
                media.play();
            }
        }
        if let Some(stack) = self.stack.upgrade() {
            stack.set_visible_child_name("viewer");
        }
        for pane in &self.panes {
            pane.frame.borrow_mut().take();
            pane.transform.borrow_mut().reset();
            pane.show();
        }
        self.permit.borrow_mut().take();
        self.refresh();
    }

    fn refresh(&self) {
        let busy = self.active.get() || self.pending.borrow().is_some();
        self.choose.set_enabled(self.available.get() && !busy);
        self.selected
            .set_enabled(self.pair_available.get() && !busy);
        self.close.set_enabled(busy);
    }

    fn reset(&self) {
        for pane in &self.panes {
            pane.transform.borrow_mut().reset();
            pane.mode.set_selected(0);
            pane.show();
        }
    }

    fn sync(&self, from: usize) {
        let [source, target] = [&self.panes[from], &self.panes[1 - from]];
        if self.linked.is_active()
            && let (Some((image, viewport)), Some((other, bounds))) =
                (source.dimensions(), target.dimensions())
        {
            let mut transform = source.transform.borrow_mut();
            if let Some(geometry) = transform.geometry(image, viewport, source.mode())
                && let Some(base) = source.mode().geometry(image, viewport)
            {
                target.transform.borrow_mut().follow_normalized(
                    geometry,
                    base,
                    viewport,
                    other,
                    bounds,
                    target.mode(),
                );
                target.redraw();
            }
        }
    }

    fn start(
        self: &Rc<Self>,
        paths: [PathBuf; 2],
        cancel: gio::Cancellable,
        view: &ImageView,
        undo: &Rc<Undo>,
    ) {
        if paths[0] == paths[1] {
            self.stop();
            undo.notify("Choose two different photos");
            return;
        }
        if COMPARING.swap(true, Ordering::AcqRel) {
            self.stop();
            undo.notify("Another image comparison is open or loading");
            return;
        }
        let permit = DecodePermit;
        let view = view.clone();
        let ready = paths.each_ref().map(|path| view.ready_texture(path));
        let weak = Rc::downgrade(self);
        let undo = Rc::downgrade(undo);
        let token = cancel.clone();
        let worker = gio::spawn_blocking(move || {
            let context = glib::MainContext::new();
            context
                .with_thread_default(|| {
                    context.block_on(async {
                        let mut frames = Vec::with_capacity(2);
                        for (path, ready) in paths.into_iter().zip(ready) {
                            if token.is_cancelled() {
                                return Err("loading cancelled".into());
                            }
                            if !candidate(&path) || image_view::animated_source(&path) {
                                return Err("Only supported still photos can be compared".into());
                            }
                            let source = match ready {
                                Some(texture) => {
                                    if pivra::image_cache::texture_cost(
                                        texture.width() as u32,
                                        texture.height() as u32,
                                    )
                                    .is_none_or(|cost| cost > TEXTURE_BUDGET / 2)
                                    {
                                        return Err(
                                            "Image exceeds the comparison texture budget".into()
                                        );
                                    }
                                    FrameSource::Ready(texture)
                                }
                                None => FrameSource::Decoded(
                                    image_loader::preload(&path, &token, TEXTURE_BUDGET / 2)
                                        .await?,
                                ),
                            };
                            frames.push((path, source));
                        }
                        Ok::<_, String>(frames)
                    })
                })
                .map_err(|error| error.to_string())?
        });
        glib::MainContext::default().spawn_local(async move {
            // Retain the single decode permit until the worker actually exits,
            // even after an obsolete comparison is cancelled.
            let result = worker.await;
            let Some(compare) = weak.upgrade() else {
                return;
            };
            if cancel.is_cancelled() || compare.pending.borrow().as_ref() != Some(&cancel) {
                return;
            }
            compare.pending.borrow_mut().take();
            match result {
                Ok(Ok(frames)) => {
                    for (pane, (path, source)) in compare.panes.iter().zip(frames) {
                        let texture = match source {
                            FrameSource::Ready(texture) => texture,
                            FrameSource::Decoded(frame) => frame.texture(),
                        };
                        pane.frame.replace(Some(Frame { path, texture }));
                    }
                    compare.permit.replace(Some(permit));
                    if let Some(media) = view.current_video().filter(|media| media.is_playing()) {
                        compare.paused.set(Some(&media));
                        media.pause();
                    }
                    compare.linked.set_active(true);
                    compare.reset();
                    if let Some(stack) = compare.stack.upgrade() {
                        stack.set_visible_child_name("comparison");
                    }
                    compare.active.set(true);
                    compare.refresh();
                }
                result => {
                    compare.refresh();
                    if let Some(undo) = undo.upgrade() {
                        let message = result
                            .ok()
                            .and_then(Result::err)
                            .unwrap_or_else(|| "image worker failed".into());
                        undo.notify(&format!("Could not compare photos: {message}"));
                    }
                }
            }
        });
    }
}

fn current_photo(window: &adw::ApplicationWindow) -> Option<PathBuf> {
    window
        .lookup_action("context-media")?
        .state()?
        .str()
        .and_then(|uri| gio::File::for_uri(uri).path())
        .filter(|path| candidate(path))
}

fn selected_pair(view: &ImageView) -> Option<[PathBuf; 2]> {
    let paths: [PathBuf; 2] = view.marked_thumbnails().try_into().ok()?;
    paths.iter().all(|path| candidate(path)).then_some(paths)
}

pub(super) fn install(
    window: &adw::ApplicationWindow,
    stack: &gtk::Stack,
    view: &ImageView,
    undo: Rc<Undo>,
) {
    let (left, left_box) = Pane::new();
    let (right, right_box) = Pane::new();
    left.picture.set_widget_name("comparison-left");
    right.picture.set_widget_name("comparison-right");
    let body = gtk::Box::new(gtk::Orientation::Horizontal, 1);
    body.set_vexpand(true);
    body.set_homogeneous(true);
    body.append(&left_box);
    body.append(&right_box);
    let split = gtk::Overlay::new();
    split.set_child(Some(&body));
    let divider = gtk::Separator::new(gtk::Orientation::Vertical);
    divider.set_halign(gtk::Align::Center);
    divider.set_width_request(1);
    divider.set_can_target(false);
    split.add_overlay(&divider);
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.set_widget_name("image-comparison");
    let tools = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    tools.set_margin_top(6);
    tools.set_margin_start(8);
    tools.set_margin_end(8);
    let linked = gtk::CheckButton::with_label("Link views");
    linked.set_active(true);
    linked.set_widget_name("comparison-linked");
    let reset = gtk::Button::with_label("Reset");
    let swap = gtk::Button::with_label("Swap");
    let close_button = gtk::Button::with_label("Close Comparison");
    close_button.set_action_name(Some("win.close-comparison"));
    for widget in [
        linked.upcast_ref::<gtk::Widget>(),
        reset.upcast_ref(),
        swap.upcast_ref(),
        close_button.upcast_ref(),
    ] {
        tools.append(widget);
    }
    root.append(&tools);
    root.append(&split);
    stack.add_named(&root, Some("comparison"));

    let choose = gio::SimpleAction::new(OPEN, None);
    let close = gio::SimpleAction::new(CLOSE, None);
    let selected = gio::SimpleAction::new(SELECTED, None);
    let compare = Rc::new(Compare {
        stack: stack.downgrade(),
        panes: [left, right],
        linked,
        choose: choose.clone(),
        selected: selected.clone(),
        close: close.clone(),
        pending: RefCell::default(),
        permit: RefCell::default(),
        active: Cell::new(false),
        paused: glib::WeakRef::new(),
        pair_available: Cell::new(false),
        available: Cell::new(false),
    });
    let update_view = view.clone();
    let update = move |compare: &Compare, window: &adw::ApplicationWindow| {
        let ordinary = !slideshow::active(window);
        compare
            .available
            .set(ordinary && current_photo(window).is_some());
        compare
            .pair_available
            .set(ordinary && selected_pair(&update_view).is_some());
        compare.refresh();
    };
    // The catalog identity signal already closes stale popup menus. It also
    // invalidates a chooser or comparison when an ordinary media action wins.
    for name in ["context-media", "slideshow", "clear-thumbnail-selection"] {
        let weak = Rc::downgrade(&compare);
        let owner = window.downgrade();
        let update = update.clone();
        window
            .lookup_action(name)
            .unwrap()
            .connect_notify_local(Some("state"), move |_, _| {
                if let (Some(compare), Some(window)) = (weak.upgrade(), owner.upgrade()) {
                    if name != "clear-thumbnail-selection" {
                        compare.stop();
                    }
                    update(&compare, &window);
                }
            });
    }
    update(&compare, window);
    let weak = Rc::downgrade(&compare);
    close.connect_activate(move |_, _| {
        if let Some(compare) = weak.upgrade() {
            compare.stop();
        }
    });
    window.add_action(&close);

    let weak = window.downgrade();
    let state = compare.clone();
    let selection_view = view.clone();
    let selection_undo = undo.clone();
    selected.connect_activate(move |_, _| {
        let Some(window) = weak.upgrade().filter(|window| !slideshow::active(window)) else {
            return;
        };
        let Some(paths) = selected_pair(&selection_view) else {
            return;
        };
        context_menu::dismiss(&window);
        let token = gio::Cancellable::new();
        state.pending.replace(Some(token.clone()));
        state.refresh();
        state.start(paths, token, &selection_view, &selection_undo);
    });
    window.add_action(&selected);
    let weak = window.downgrade();
    let state = compare.clone();
    let view = view.clone();
    choose.connect_activate(move |_, _| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let Some(path) = current_photo(&window) else {
            return;
        };
        if state.pending.borrow().is_some() {
            return;
        }
        context_menu::dismiss(&window);
        let chooser = file_chooser::media_dialog("Compare Images");
        chooser.set_initial_folder(path.parent().map(gio::File::for_path).as_ref());
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Still images"));
        for extension in [
            "jpg", "jpeg", "png", "webp", "tif", "tiff", "bmp", "heic", "heif", "avif",
        ] {
            filter.add_suffix(extension);
        }
        chooser.set_filters(Some(&gio::ListStore::from_iter([filter])));
        let cancel = gio::Cancellable::new();
        state.pending.replace(Some(cancel.clone()));
        state.refresh();
        let state = Rc::downgrade(&state);
        let undo = undo.clone();
        let view = view.clone();
        let token = cancel.clone();
        chooser.open(Some(&window), Some(&cancel), move |result| {
            let Some(state) = state.upgrade() else {
                return;
            };
            if token.is_cancelled() || state.pending.borrow().as_ref() != Some(&token) {
                return;
            }
            match result {
                Ok(file) if let Some(second) = file.path() => {
                    state.start([path, second], token, &view, &undo)
                }
                Err(error)
                    if error.matches(gtk::DialogError::Dismissed)
                        || error.matches(gtk::DialogError::Cancelled) =>
                {
                    state.stop()
                }
                _ => {
                    state.stop();
                    undo.notify("Could not open a local comparison photo");
                }
            }
        });
    });
    window.add_action(&choose);
    let weak = Rc::downgrade(&compare);
    reset.connect_clicked(move |_| {
        if let Some(compare) = weak.upgrade() {
            compare.reset();
        }
    });
    let weak = Rc::downgrade(&compare);
    swap.connect_clicked(move |_| {
        if let Some(compare) = weak.upgrade() {
            let [left, right] = &compare.panes;
            left.frame.swap(&right.frame);
            left.transform.swap(&right.transform);
            left.show();
            right.show();
        }
    });
    let weak = Rc::downgrade(&compare);
    compare.linked.connect_toggled(move |_| {
        if let Some(compare) = weak.upgrade() {
            compare.sync(0);
        }
    });
    for side in 0..2 {
        let pane = &compare.panes[side];
        let weak = Rc::downgrade(&compare);
        pane.mode.connect_selected_notify(move |mode| {
            if let Some(compare) = weak.upgrade() {
                let pane = &compare.panes[side];
                pane.transform.borrow_mut().reset();
                pane.show();
                let other = &compare.panes[1 - side].mode;
                if compare.linked.is_active() && mode.selected() != other.selected() {
                    other.set_selected(mode.selected());
                }
            }
        });
        install_gestures(&compare, side);
    }
    window.connect_close_request(move |_| {
        compare.stop();
        glib::Propagation::Proceed
    });
}

fn install_gestures(compare: &Rc<Compare>, side: usize) {
    let weak = Rc::downgrade(compare);
    let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
    scroll.connect_scroll(move |_, _, dy| {
        if let Some(compare) = weak.upgrade() {
            let pane = &compare.panes[side];
            if let Some((image, viewport)) = pane.dimensions() {
                // Center zoom is independent of surface/widget coordinates.
                if pane.transform.borrow_mut().zoom(
                    image,
                    viewport,
                    pane.mode(),
                    (viewport.0 / 2., viewport.1 / 2.),
                    dy,
                ) {
                    pane.redraw();
                    compare.sync(side);
                }
            }
        }
        glib::Propagation::Stop
    });
    compare.panes[side].picture.add_controller(scroll);
    let drag = gtk::GestureDrag::new();
    drag.set_button(1);
    let last = Rc::new(Cell::new((0., 0.)));
    let start = last.clone();
    drag.connect_drag_begin(move |_, _, _| start.set((0., 0.)));
    let weak = Rc::downgrade(compare);
    drag.connect_drag_update(move |_, x, y| {
        let (last_x, last_y) = last.replace((x, y));
        if let Some(compare) = weak.upgrade() {
            let pane = &compare.panes[side];
            if let Some((image, viewport)) = pane.dimensions()
                && pane.transform.borrow_mut().pan(
                    image,
                    viewport,
                    pane.mode(),
                    (x - last_x, y - last_y),
                )
            {
                pane.redraw();
                compare.sync(side);
            }
        }
    });
    compare.panes[side].picture.add_controller(drag);
}

pub(super) fn handle_key(
    window: &adw::ApplicationWindow,
    key: gdk::Key,
    modifiers: gdk::ModifierType,
) -> bool {
    let relevant = modifiers
        & (gdk::ModifierType::CONTROL_MASK
            | gdk::ModifierType::SHIFT_MASK
            | gdk::ModifierType::ALT_MASK
            | gdk::ModifierType::SUPER_MASK
            | gdk::ModifierType::META_MASK);
    if key.to_lower() == gdk::Key::c
        && relevant == (gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::SHIFT_MASK)
        && let Some(action) = window
            .lookup_action(OPEN)
            .filter(|action| action.is_enabled())
    {
        action.activate(None);
        return true;
    }
    let comparison = window
        .lookup_action(CLOSE)
        .filter(|action| action.is_enabled());
    comparison.is_some()
        && matches!(
            key,
            gdk::Key::Left
                | gdk::Key::Right
                | gdk::Key::F5
                | gdk::Key::F6
                | gdk::Key::F7
                | gdk::Key::F8
                | gdk::Key::F2
                | gdk::Key::Delete
        )
}
