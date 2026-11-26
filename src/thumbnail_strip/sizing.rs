//! Display texture dimensions and memory charge; policies stay in the strip.
use super::{CELL_HEIGHT, CELL_WIDTH};

pub(super) fn display_cost(width: i32, height: i32) -> Option<usize> {
    usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?
        .checked_mul(8)
}

pub(super) fn display_size(width: i32, height: i32, scale: i32) -> (i32, i32) {
    let scale = scale.clamp(1, 2);
    let ratio = (f64::from(CELL_WIDTH * scale) / f64::from(width))
        .min(f64::from(CELL_HEIGHT * scale) / f64::from(height))
        .min(1.);
    (
        (f64::from(width) * ratio).floor().max(1.) as i32,
        (f64::from(height) * ratio).floor().max(1.) as i32,
    )
}
