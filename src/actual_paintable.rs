//! Native logical-pixel presentation without changing Picture's minimum size.
use gtk::{gdk, glib, prelude::*, subclass::prelude::*};
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};
mod imp {
    use super::*;
    #[derive(Default)]
    pub struct ActualPaintable {
        pub texture: OnceCell<gdk::Texture>,
        pub transform: OnceCell<Rc<RefCell<pivra::viewing::Transform>>>,
        pub mode: std::cell::Cell<pivra::viewing::ViewMode>,
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
            self.texture.get().map_or(0, |t| t.width())
        }
        fn intrinsic_height(&self) -> i32 {
            self.texture.get().map_or(0, |t| t.height())
        }
        fn snapshot(&self, snapshot: &gdk::Snapshot, width: f64, height: f64) {
            let Some(texture) = self.texture.get() else {
                return;
            };
            let image = (texture.width() as f64, texture.height() as f64);
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
            snapshot.append_texture(
                texture,
                &gtk::graphene::Rect::new(g.x as f32, g.y as f32, g.width as f32, g.height as f32),
            );
            snapshot.pop();
        }
    }
}
glib::wrapper! { pub struct ActualPaintable(ObjectSubclass<imp::ActualPaintable>) @implements gdk::Paintable; }
impl ActualPaintable {
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
    ) -> Self {
        let object = Self::new(texture);
        object
            .imp()
            .transform
            .set(transform)
            .expect("new transform");
        object.imp().mode.set(mode);
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
