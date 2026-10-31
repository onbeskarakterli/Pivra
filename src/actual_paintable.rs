//! Native logical-pixel presentation without changing Picture's minimum size.
use gtk::{gdk, glib, prelude::*, subclass::prelude::*};
use std::{
    cell::{Cell, OnceCell, RefCell},
    rc::Rc,
};
mod imp {
    use super::*;
    #[derive(Default)]
    pub struct ActualPaintable {
        pub texture: OnceCell<gdk::Texture>,
        pub media: OnceCell<gdk::Paintable>,
        pub transform: OnceCell<Rc<RefCell<pivra::viewing::Transform>>>,
        pub mode: std::cell::Cell<pivra::viewing::ViewMode>,
        pub filter: OnceCell<Rc<Cell<pivra::viewing::Resampling>>>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for ActualPaintable {
        const NAME: &'static str = "PivraActualPaintable";
        type Type = super::ActualPaintable;
        type Interfaces = (gdk::Paintable,);
    }
    impl ObjectImpl for ActualPaintable {}
    impl gdk::subclass::prelude::PaintableImpl for ActualPaintable {
        fn intrinsic_width(&self) -> i32 {
            self.texture.get().map_or_else(
                || self.media.get().map_or(0, |m| m.intrinsic_width()),
                |t| t.width(),
            )
        }
        fn intrinsic_height(&self) -> i32 {
            self.texture.get().map_or_else(
                || self.media.get().map_or(0, |m| m.intrinsic_height()),
                |t| t.height(),
            )
        }
        fn snapshot(&self, snapshot: &gdk::Snapshot, width: f64, height: f64) {
            let image = if let Some(texture) = self.texture.get() {
                (texture.width() as f64, texture.height() as f64)
            } else if let Some(media) = self.media.get() {
                (media.intrinsic_width() as f64, media.intrinsic_height() as f64)
            } else {
                return;
            };
            let geometry = if let Some(transform) = self.transform.get() {
                transform
                    .borrow_mut()
                    .geometry(image, (width, height), self.mode.get())
            } else {
                pivra::viewing::ViewMode::Actual.geometry(image, (width, height))
            };
            let Some(g) = geometry else {
                return;
            };
            let snapshot = snapshot
                .downcast_ref::<gtk::Snapshot>()
                .expect("GTK snapshot");
            snapshot.push_clip(&gtk::graphene::Rect::new(
                0.,
                0.,
                width as f32,
                height as f32,
            ));
            let filter = self
                .filter
                .get()
                .map_or(pivra::viewing::Resampling::default(), |f| f.get());
            if let Some(texture) = self.texture.get() {
                snapshot.append_scaled_texture(
                    texture,
                    match filter {
                        pivra::viewing::Resampling::Smooth => gtk::gsk::ScalingFilter::Linear,
                        pivra::viewing::Resampling::Nearest => gtk::gsk::ScalingFilter::Nearest,
                    },
                    &gtk::graphene::Rect::new(g.x as f32, g.y as f32, g.width as f32, g.height as f32),
                );
            } else if let Some(media) = self.media.get() {
                snapshot.save();
                snapshot.translate(&gtk::graphene::Point::new(g.x as f32, g.y as f32));
                snapshot.scale((g.width / image.0) as f32, (g.height / image.1) as f32);
                media.snapshot(snapshot.upcast_ref(), image.0, image.1);
                snapshot.restore();
            }
            snapshot.pop();
        }
    }
}
glib::wrapper! { pub struct ActualPaintable(ObjectSubclass<imp::ActualPaintable>) @implements gdk::Paintable; }
impl ActualPaintable {
    pub fn video(
        media: &gtk::MediaFile,
        transform: Rc<RefCell<pivra::viewing::Transform>>,
        mode: pivra::viewing::ViewMode,
    ) -> Self {
        let object: Self = glib::Object::new();
        object.imp().media.set(media.clone().upcast()).expect("new media paintable");
        object.imp().transform.set(transform).expect("new transform");
        object.imp().mode.set(mode);
        let weak = object.downgrade();
        media.connect_invalidate_contents(move |_| {
            if let Some(paintable) = weak.upgrade() {
                paintable.invalidate_contents();
            }
        });
        let weak = object.downgrade();
        media.connect_invalidate_size(move |_| {
            if let Some(paintable) = weak.upgrade() {
                paintable.invalidate_size();
            }
        });
        object
    }
    pub fn set_mode(&self, mode: pivra::viewing::ViewMode) {
        self.imp().mode.set(mode);
    }
    pub fn has_texture(&self, texture: &gdk::Texture) -> bool {
        self.imp()
            .texture
            .get()
            .is_some_and(|current| current == texture)
    }
    pub fn has_transform(&self, transform: &Rc<RefCell<pivra::viewing::Transform>>) -> bool {
        self.imp()
            .transform
            .get()
            .is_some_and(|current| Rc::ptr_eq(current, transform))
    }
    pub fn transformed(
        texture: &gdk::Texture,
        transform: Rc<RefCell<pivra::viewing::Transform>>,
        mode: pivra::viewing::ViewMode,
        filter: Rc<Cell<pivra::viewing::Resampling>>,
    ) -> Self {
        let object = Self::new(texture);
        object
            .imp()
            .transform
            .set(transform)
            .expect("new transform");
        object.imp().mode.set(mode);
        object.imp().filter.set(filter).expect("new filter");
        object
    }
    pub fn new(texture: &gdk::Texture) -> Self {
        let object: Self = glib::Object::new();
        object
            .imp()
            .texture
            .set(texture.clone())
            .expect("new paintable");
        object
    }
}
