//! Logical viewport geometry. Sizes already include Glycin's EXIF transform.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Fit,
    Fill,
    Actual,
}
impl ViewMode {
    pub const HELP: &'static str = "Image size: Fit (F6), Fill (F7), 100% (F8)";
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
            Self::Fit => (vw / iw).min(vh / ih).min(1.),
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
                            assert!(g.width <= image.0 + 1e-9);
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
