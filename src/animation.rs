//! Animated foreground images; thumbnails stay on their static first frame.
use super::{present_filtered, ViewMode};
use gtk::{gio, glib, prelude::*};
use pivra::viewing::{Resampling, Transform};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    time::Duration,
};

pub(super) fn start(
    path: PathBuf,
    generation: Rc<Cell<u64>>,
    expected: u64,
    picture: glib::WeakRef<gtk::Picture>,
    texture: Rc<RefCell<Option<gtk::gdk::Texture>>>,
    mode_action: gio::SimpleAction,
    transform: Rc<RefCell<Transform>>,
    filter: Rc<Cell<Resampling>>,
) {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !["gif", "webp", "png", "apng"]
        .iter()
        .any(|s| ext.eq_ignore_ascii_case(s))
    {
        return;
    }
    glib::MainContext::default().spawn_local(async move {
        let cancelled = gio::Cancellable::new();
        let mut loader = glycin::Loader::new(gio::File::for_path(&path));
        loader.sandbox_selector(glycin::SandboxSelector::Bwrap);
        loader.cancellable(cancelled.clone());
        loader.apply_transformations(true);
        loader.use_expose_base_dir(false);
        let Ok(mut image) = loader.load().await else {
            return;
        };
        if generation.get() != expected {
            cancelled.cancel();
            return;
        }
        let Ok(first) = image.next_frame().await else {
            return;
        };
        let Some(mut delay) = first.delay() else {
            return;
        };
        loop {
            if generation.get() != expected {
                cancelled.cancel();
                break;
            }
            glib::timeout_future(delay.clamp(Duration::from_millis(20), Duration::from_secs(10)))
                .await;
            if generation.get() != expected {
                cancelled.cancel();
                break;
            }
            let Ok(frame) = image.next_frame().await else {
                break;
            };
            if generation.get() != expected {
                cancelled.cancel();
                break;
            }
            if crate::image_loader::validate_dimensions(frame.width(), frame.height()).is_err()
                || frame.buf_slice().len() > 512 * 1024 * 1024
            {
                break;
            }
            let Some(next_delay) = frame.delay() else {
                break;
            };
            let Some(picture) = picture.upgrade() else {
                break;
            };
            let next = frame.texture();
            let mode = mode_action
                .state()
                .and_then(|v| v.str().and_then(ViewMode::from_id))
                .unwrap_or_default();
            texture.replace(Some(next.clone()));
            present_filtered(
                &picture,
                Some(&next),
                mode.id(),
                transform.clone(),
                filter.clone(),
            );
            delay = next_delay;
        }
    });
}
