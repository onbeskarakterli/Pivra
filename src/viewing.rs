//! Logical viewport geometry. Sizes already include Glycin's EXIF transform.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Fit,
    Fill,
    Actual,
}
impl ViewMode {
    pub const MANUAL_LABEL: &'static str = "Zoom";
    pub const HELP: &'static str =
        "Image size: Fit (F6), Fill (F7), 100% (F8). Scroll to zoom; drag to pan.";
    pub const ALL: [Self; 3] = [Self::Fit, Self::Fill, Self::Actual];
    pub fn id(self) -> &'static str {
        match self {
            Self::Fit => "fit",
            Self::Fill => "fill",
            Self::Actual => "actual",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Fit => "Fit",
            Self::Fill => "Fill",
            Self::Actual => "100%",
        }
    }
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.id() == id)
    }
    pub fn geometry(self, image: (f64, f64), viewport: (f64, f64)) -> Option<Geometry> {
        let (iw, ih) = image;
        let (vw, vh) = viewport;
        if [iw, ih, vw, vh].iter().any(|n| !n.is_finite() || *n <= 0.) {
            return None;
        }
        let scale = match self {
            Self::Fit => (vw / iw).min(vh / ih),
            Self::Fill => (vw / iw).max(vh / ih),
            Self::Actual => 1.,
        };
        let width = iw * scale;
        let height = ih * scale;
        Some(Geometry {
            x: (vw - width) / 2.,
            y: (vh - height) / 2.,
            width,
            height,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aspect_center_bounds_and_actual_scale_across_orientations_and_resize() {
        for image in [(1600., 1200.), (1200., 1600.), (32., 64.), (4., 2.)] {
            for viewport in [(800., 600.), (300., 900.), (1920., 1080.), (1., 1.)] {
                for mode in ViewMode::ALL {
                    let g = mode.geometry(image, viewport).unwrap();
                    assert!((g.width / g.height - image.0 / image.1).abs() < 1e-9);
                    assert!((g.x * 2. + g.width - viewport.0).abs() < 1e-9);
                    assert!((g.y * 2. + g.height - viewport.1).abs() < 1e-9);
                    match mode {
                        ViewMode::Fit => {
                            assert!(g.width <= viewport.0 + 1e-9 && g.height <= viewport.1 + 1e-9);
                            assert!((g.width - viewport.0).abs() < 1e-9 || (g.height - viewport.1).abs() < 1e-9);
                        }
                        ViewMode::Fill => {
                            assert!(g.width + 1e-9 >= viewport.0 && g.height + 1e-9 >= viewport.1);
                        }
                        ViewMode::Actual => assert_eq!((g.width, g.height), image),
                    }
                }
            }
        }
    }
    #[test]
    fn fit_upscales_small_images_and_tracks_viewport_without_cropping() {
        let image = (320., 200.);
        for viewport in [(640., 400.), (1200., 600.), (300., 300.)] {
            let g = ViewMode::Fit.geometry(image, viewport).unwrap();
            let scale = (viewport.0 / image.0).min(viewport.1 / image.1);
            assert!((g.width - image.0 * scale).abs() < 1e-9);
            assert!((g.height - image.1 * scale).abs() < 1e-9);
            assert!(g.x >= -1e-9 && g.y >= -1e-9);
            assert!(g.x + g.width <= viewport.0 + 1e-9);
            assert!(g.y + g.height <= viewport.1 + 1e-9);
        }
    }
    #[test]
    fn invalid_dimensions_and_unknown_modes_are_rejected() {
        for n in [0., -1., f64::NAN, f64::INFINITY] {
            for mode in ViewMode::ALL {
                assert!(mode.geometry((n, 1.), (100., 100.)).is_none());
                assert!(mode.geometry((1., 1.), (n, 100.)).is_none());
            }
        }
        assert_eq!(ViewMode::from_id("stretch"), None);
        for mode in ViewMode::ALL {
            assert_eq!(ViewMode::from_id(mode.id()), Some(mode));
        }
    }
}

/// Manual scale is absolute in source pixels per GTK logical unit.
#[derive(Clone, Debug, Default)]
pub struct Transform {
    manual: Option<Geometry>,
    viewport: (f64, f64),
    generation: u64,
}
impl Transform {
    pub const MIN_SCALE: f64 = 0.1;
    pub const MAX_SCALE: f64 = 32.;
    pub fn reset(&mut self) {
        let generation = self.generation.wrapping_add(1);
        *self = Self::default();
        self.generation = generation;
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn geometry(
        &mut self,
        image: (f64, f64),
        viewport: (f64, f64),
        mode: ViewMode,
    ) -> Option<Geometry> {
        let base = mode.geometry(image, viewport)?;
        let Some(mut g) = self.manual else {
            self.viewport = viewport;
            return Some(base);
        };
        g.x += (viewport.0 - self.viewport.0) / 2.;
        g.y += (viewport.1 - self.viewport.1) / 2.;
        self.viewport = viewport;
        Self::constrain(&mut g, viewport);
        self.manual = Some(g);
        Some(g)
    }
    fn constrain(g: &mut Geometry, viewport: (f64, f64)) {
        g.x = if g.width <= viewport.0 {
            (viewport.0 - g.width) / 2.
        } else {
            g.x.clamp(viewport.0 - g.width, 0.)
        };
        g.y = if g.height <= viewport.1 {
            (viewport.1 - g.height) / 2.
        } else {
            g.y.clamp(viewport.1 - g.height, 0.)
        };
    }
    pub fn zoom(
        &mut self,
        image: (f64, f64),
        viewport: (f64, f64),
        mode: ViewMode,
        pointer: (f64, f64),
        dy: f64,
    ) -> bool {
        if ![pointer.0, pointer.1, dy].iter().all(|n| n.is_finite()) || dy == 0. {
            return false;
        }
        let Some(mut g) = self.geometry(image, viewport, mode) else {
            return false;
        };
        let old = g.width / image.0;
        let scale =
            (old * 1.2_f64.powf(-dy.clamp(-100., 100.))).clamp(Self::MIN_SCALE, Self::MAX_SCALE);
        // Preset scales may lie outside manual limits. Never reverse wheel direction.
        if (dy > 0. && scale >= old) || (dy < 0. && scale <= old) {
            return false;
        }
        let x = pointer.0.clamp(0., viewport.0);
        let y = pointer.1.clamp(0., viewport.1);
        let u = ((x - g.x) / g.width).clamp(0., 1.);
        let v = ((y - g.y) / g.height).clamp(0., 1.);
        g.width = image.0 * scale;
        g.height = image.1 * scale;
        g.x = x - u * g.width;
        g.y = y - v * g.height;
        Self::constrain(&mut g, viewport);
        self.manual = Some(g);
        true
    }
    pub fn pan(
        &mut self,
        image: (f64, f64),
        viewport: (f64, f64),
        mode: ViewMode,
        delta: (f64, f64),
    ) -> bool {
        if !delta.0.is_finite() || !delta.1.is_finite() {
            return false;
        }
        let Some(mut g) = self.geometry(image, viewport, mode) else {
            return false;
        };
        if g.width <= viewport.0 && g.height <= viewport.1 {
            return false;
        }
        g.x += delta.0;
        g.y += delta.1;
        Self::constrain(&mut g, viewport);
        self.manual = Some(g);
        true
    }
}
#[cfg(test)]
mod transform_tests {
    use super::*;
    #[test]
    fn zoom_preserves_source_point_unless_bounds_require_clamping() {
        let mut t = Transform::default();
        let image = (1600., 1200.);
        let vp = (800., 600.);
        let p = (300., 200.);
        let a = t.geometry(image, vp, ViewMode::Actual).unwrap();
        assert!(t.zoom(image, vp, ViewMode::Actual, p, -1.));
        let b = t.geometry(image, vp, ViewMode::Actual).unwrap();
        assert!(((p.0 - a.x) / a.width - (p.0 - b.x) / b.width).abs() < 1e-9);
        assert!(((p.1 - a.y) / a.height - (p.1 - b.y) / b.height).abs() < 1e-9);
        t.zoom(image, vp, ViewMode::Actual, p, 1.);
        let c = t.geometry(image, vp, ViewMode::Actual).unwrap();
        assert!((c.x - a.x).abs() < 1e-9);
    }
    #[test]
    fn rapid_input_limits_edges_resize_and_reset_remain_finite() {
        for image in [(1600., 1200.), (1200., 1600.), (32., 64.)] {
            let mut t = Transform::default();
            let vp = (800., 600.);
            for dy in [-1., 1.] {
                for _ in 0..1000 {
                    t.zoom(image, vp, ViewMode::Fit, (400., 300.), dy);
                }
                let g = t.geometry(image, vp, ViewMode::Fit).unwrap();
                let s = g.width / image.0;
                assert!((Transform::MIN_SCALE..=Transform::MAX_SCALE).contains(&s));
            }
            t.reset();
            assert!(!t.pan((32., 64.), vp, ViewMode::Actual, (100., 100.)));
            for delta in [(-1e9, -1e9), (1e9, 1e9)] {
                t.pan(image, vp, ViewMode::Actual, delta);
                let g = t.geometry(image, (3000., 3000.), ViewMode::Actual).unwrap();
                assert!(g.x >= 0. && g.y >= 0.);
            }
            assert!(!t.zoom(image, vp, ViewMode::Fit, (f64::NAN, 0.), 1.));
            assert!(!t.pan(image, vp, ViewMode::Fit, (f64::INFINITY, 0.)));
            t.reset();
            assert_eq!(
                t.geometry(image, vp, ViewMode::Fit),
                ViewMode::Fit.geometry(image, vp)
            );
        }
    }
}

#[cfg(test)]
mod pan_boundary_tests {
    use super::*;
    #[test]
    fn all_four_corners_are_reachable_and_resize_keeps_center_source_point() {
        let image = (1600., 1200.);
        let vp = (800., 600.);
        for (delta, expected) in [
            ((1e6, 1e6), (0., 0.)),
            ((-1e6, 1e6), (-800., 0.)),
            ((1e6, -1e6), (0., -600.)),
            ((-1e6, -1e6), (-800., -600.)),
        ] {
            let mut t = Transform::default();
            t.pan(image, vp, ViewMode::Actual, delta);
            let g = t.geometry(image, vp, ViewMode::Actual).unwrap();
            assert_eq!((g.x, g.y), expected);
        }
        let mut t = Transform::default();
        t.pan(image, vp, ViewMode::Actual, (30., 20.));
        let before = t.geometry(image, vp, ViewMode::Actual).unwrap();
        let after = t.geometry(image, (900., 700.), ViewMode::Actual).unwrap();
        assert_eq!(
            (400. - before.x, 300. - before.y),
            (450. - after.x, 350. - after.y)
        );
        let epoch = t.generation();
        t.reset();
        assert_ne!(epoch, t.generation());
        assert_eq!(
            t.geometry(image, vp, ViewMode::Fill),
            ViewMode::Fill.geometry(image, vp)
        );
    }
}

#[cfg(test)]
mod preset_transition_tests {
    use super::*;
    #[test]
    fn wheel_direction_never_reverses_when_preset_is_outside_manual_range() {
        let mut t = Transform::default();
        assert!(!t.zoom((32768., 1.), (200., 200.), ViewMode::Fit, (100., 100.), 1.));
        assert!(t.zoom((32768., 1.), (200., 200.), ViewMode::Fit, (100., 100.), -1.));
        t.reset();
        assert!(!t.zoom((1., 1.), (800., 600.), ViewMode::Fill, (400., 300.), -1.));
        assert!(t.zoom((1., 1.), (800., 600.), ViewMode::Fill, (400., 300.), 1.));
    }
}

/// Window-local preference; independent of geometry and decoder lifetime.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Resampling {
    #[default]
    Smooth,
    Nearest,
}
impl Resampling {
    pub const MENU: &'static str = "Resampling";
    pub const ALL: [Self; 2] = [Self::Smooth, Self::Nearest];
    pub fn id(self) -> &'static str {
        match self {
            Self::Smooth => "smooth",
            Self::Nearest => "nearest",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Smooth => "Smooth",
            Self::Nearest => "Nearest",
        }
    }
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|filter| filter.id() == id)
    }
}
#[cfg(test)]
mod resampling_tests {
    use super::*;
    #[test]
    fn preference_defaults_and_ids_are_explicit() {
        assert_eq!(Resampling::default(), Resampling::Smooth);
        for f in Resampling::ALL {
            assert_eq!(Resampling::from_id(f.id()), Some(f));
        }
        assert_eq!(Resampling::from_id("auto"), None);
    }
}
