//! Optional sidebar. One coalesced metadata worker; no decode or pixel readback.
use adw::prelude::*;
use gtk::{gio, glib};
use pivra::media_information::Fields;
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
};
pub struct Panel {
    root: gtk::Revealer,
    rows: gtk::Box,
    path: RefCell<Option<PathBuf>>,
    file: RefCell<Fields>,
    media: RefCell<Fields>,
    epoch: Cell<u64>,
    busy: Cell<bool>,
    dirty: Cell<bool>,
    closed: Cell<bool>,
    stream: RefCell<Option<(glib::WeakRef<gtk::MediaFile>, Vec<glib::SignalHandlerId>)>>,
}
impl Panel {
    pub fn new() -> Rc<Self> {
        let root = gtk::Revealer::new();
        root.set_widget_name("media-information-panel");
        // The expanding heading is internal to the sidebar. Do not inherit it:
        // a closed revealer must not claim the viewer row's spare width.
        root.set_hexpand(false);
        root.set_transition_type(gtk::RevealerTransitionType::SlideLeft);
        root.set_transition_duration(150);
        let body = gtk::Box::new(gtk::Orientation::Vertical, 12);
        body.add_css_class("sidebar");
        body.set_width_request(260);
        body.set_margin_start(12);
        body.set_margin_end(12);
        body.set_margin_top(12);
        body.set_margin_bottom(12);
        let heading = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let title = gtk::Label::new(Some("Information"));
        title.add_css_class("heading");
        title.set_hexpand(true);
        title.set_xalign(0.);
        let close = gtk::Button::from_icon_name("window-close-symbolic");
        close.add_css_class("flat");
        close.set_tooltip_text(Some("Close information panel"));
        close.set_action_name(Some("win.media-info"));
        heading.append(&title);
        heading.append(&close);
        body.append(&heading);
        let rows = gtk::Box::new(gtk::Orientation::Vertical, 12);
        let scroll = gtk::ScrolledWindow::new();
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scroll.set_child(Some(&rows));
        scroll.set_vexpand(true);
        body.append(&scroll);
        root.set_child(Some(&body));
        Rc::new(Self {
            root,
            rows,
            path: RefCell::new(None),
            file: RefCell::default(),
            media: RefCell::default(),
            epoch: Cell::new(0),
            busy: Cell::new(false),
            dirty: Cell::new(false),
            closed: Cell::new(false),
            stream: RefCell::new(None),
        })
    }
    pub fn widget(&self) -> &gtk::Revealer {
        &self.root
    }
    pub fn show(self: &Rc<Self>, show: bool) {
        self.root.set_reveal_child(show);
        if show {
            self.refresh();
        } else {
            self.epoch.set(self.epoch.get().wrapping_add(1));
            self.dirty.set(false);
        }
    }
    pub fn select(self: &Rc<Self>, path: Option<PathBuf>) {
        self.epoch.set(self.epoch.get().wrapping_add(1));
        self.path.replace(path);
        self.file.borrow_mut().clear();
        self.media.borrow_mut().clear();
        self.refresh();
    }
    pub fn image_ready(self: &Rc<Self>, width: i32, height: i32) {
        self.media.replace(vec![
            ("Dimensions".into(), format!("{width} × {height}")),
            ("Media type".into(), "Photo".into()),
        ]);
        self.refresh();
    }
    fn render(&self) {
        if !self.root.reveals_child() || self.closed.get() {
            return;
        }
        while let Some(child) = self.rows.first_child() {
            self.rows.remove(&child);
        }
        for (title, file_group) in [("File", true), ("Media details", false)] {
            let group = adw::PreferencesGroup::new();
            group.set_title(title);
            let mut count = 0;
            for (key, value) in self.file.borrow().iter().chain(self.media.borrow().iter()) {
                let is_file = matches!(
                    key.as_str(),
                    "Filename" | "Full path" | "File size" | "Modified" | "File unavailable"
                );
                if is_file != file_group || value.is_empty() {
                    continue;
                }
                let row = adw::PreferencesRow::new();
                let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
                content.set_margin_start(12);
                content.set_margin_end(12);
                content.set_margin_top(10);
                content.set_margin_bottom(10);
                let caption = gtk::Label::new(Some(key));
                caption.set_xalign(0.);
                caption.add_css_class("dim-label");
                let label = gtk::Label::new(Some(value));
                label.set_xalign(0.);
                label.set_selectable(true);
                label.set_wrap(true);
                label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
                label.set_max_width_chars(26);
                content.append(&caption);
                content.append(&label);
                row.set_child(Some(&content));
                row.set_activatable(false);
                row.set_selectable(false);
                group.add(&row);
                count += 1;
            }
            if count > 0 {
                self.rows.append(&group);
            }
        }
        if self.path.borrow().is_none() {
            self.rows
                .append(&gtk::Label::new(Some("No media selected")));
        }
    }
    fn refresh(self: &Rc<Self>) {
        self.render();
        if self.closed.get() || !self.root.reveals_child() {
            return;
        }
        let Some(path) = self.path.borrow().clone() else {
            return;
        };
        if self.busy.replace(true) {
            self.dirty.set(true);
            return;
        }
        self.dirty.set(false);
        let epoch = self.epoch.get();
        let weak = Rc::downgrade(self);
        glib::MainContext::default().spawn_local(async move {
            let fields = gio::spawn_blocking(move || file_fields(&path))
                .await
                .unwrap_or_default();
            let Some(this) = weak.upgrade() else {
                return;
            };
            this.busy.set(false);
            if this.closed.get() {
                return;
            }
            if this.epoch.get() == epoch && !this.dirty.get() {
                this.file.replace(fields);
                this.render();
            }
            if this.dirty.replace(false) {
                this.refresh();
            }
        });
    }
    pub fn unbind_video(&self) {
        if let Some((weak, handlers)) = self.stream.borrow_mut().take() {
            if let Some(media) = weak.upgrade() {
                for id in handlers {
                    media.disconnect(id);
                }
            }
        }
    }
    fn video_ready(&self, media: &gtk::MediaFile) {
        let mut fields = vec![("Media type".into(), "Video".into())];
        let (width, height) = (media.intrinsic_width(), media.intrinsic_height());
        if width > 0 && height > 0 {
            fields.push(("Dimensions".into(), format!("{width} × {height}")));
        }
        if media.duration() > 0 {
            let seconds = media.duration() / 1_000_000;
            fields.push((
                "Duration".into(),
                format!(
                    "{}:{:02}:{:02}",
                    seconds / 3600,
                    seconds / 60 % 60,
                    seconds % 60
                ),
            ));
        }
        if media.is_prepared() {
            fields.push((
                "Audio".into(),
                if media.has_audio() { "Yes" } else { "No" }.into(),
            ));
        }
        self.media.replace(fields);
        self.render();
    }
    pub fn bind_video(self: &Rc<Self>, media: &gtk::MediaFile) {
        self.unbind_video();
        let mut handlers = vec![];
        for property in ["prepared", "duration", "has-video", "has-audio", "error"] {
            let weak = Rc::downgrade(self);
            handlers.push(media.connect_notify_local(Some(property), move |media, _| {
                if let Some(this) = weak.upgrade() {
                    if !this.closed.get() {
                        this.video_ready(media);
                    }
                }
            }));
        }
        let weak = Rc::downgrade(self);
        handlers.push(media.connect_invalidate_size(move |media| {
            if let Some(this) = weak.upgrade() {
                if !this.closed.get() {
                    this.video_ready(media);
                }
            }
        }));
        self.stream.replace(Some((media.downgrade(), handlers)));
        self.video_ready(media);
        self.refresh();
    }
    pub fn close(&self) {
        self.closed.set(true);
        self.epoch.set(self.epoch.get().wrapping_add(1));
        self.unbind_video();
    }
}
fn file_fields(path: &std::path::Path) -> Fields {
    let mut fields = vec![
        (
            "Filename".into(),
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
        ),
        ("Full path".into(), path.to_string_lossy().into_owned()),
    ];
    match std::fs::metadata(path) {
        Ok(metadata) => {
            fields.push((
                "File size".into(),
                glib::format_size(metadata.len()).to_string(),
            ));
            if let Some(date) = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .and_then(|t| i64::try_from(t.as_secs()).ok())
                .and_then(|t| glib::DateTime::from_unix_local(t).ok())
                .and_then(|date| date.format("%Y-%m-%d %H:%M:%S %Z").ok())
            {
                fields.push(("Modified".into(), date.to_string()));
            }
            fields.extend(pivra::media_information::lookup(path));
        }
        Err(error) => fields.push(("File unavailable".into(), error.to_string())),
    }
    fields
}
