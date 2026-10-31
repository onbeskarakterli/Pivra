//! Strict validation of RGB P6 frames from an isolated RAW helper.
//! No untrusted RAW decoder executes in the GTK process.
const MAX_PIXELS: u64 = 64_000_000;
const MAX_BYTES: usize = 256 * 1024 * 1024;

pub(crate) struct RgbFrame<'a> {
    pub width: u32,
    pub height: u32,
    pub pixels: &'a [u8],
}

pub(crate) fn parse(data: &[u8]) -> Result<RgbFrame<'_>, &'static str> {
    if !data.starts_with(b"P6\n") {
        return Err("invalid PPM signature");
    }
    let rest = &data[3..];
    let end_dimensions = rest.iter().position(|&b| b == b'\n').ok_or("missing dimensions")?;
    if end_dimensions > 32 {
        return Err("oversized dimensions header");
    }
    let dims = std::str::from_utf8(&rest[..end_dimensions]).map_err(|_| "non-ASCII dimensions")?;
    let mut values = dims.split(' ');
    let width = values.next().ok_or("missing width")?.parse::<u32>().map_err(|_| "invalid width")?;
    let height = values.next().ok_or("missing height")?.parse::<u32>().map_err(|_| "invalid height")?;
    if values.next().is_some() || width == 0 || height == 0 || width > 16384 || height > 16384 {
        return Err("invalid geometry");
    }
    if u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err("too many pixels");
    }
    let offset = 3 + end_dimensions + 1;
    if data.get(offset..offset + 4) != Some(&b"255\n"[..]) {
        return Err("invalid PPM maxval");
    }
    let start = offset + 4;
    let expected = usize::try_from(u64::from(width) * u64::from(height) * 3)
        .map_err(|_| "raster overflow")?;
    if expected > MAX_BYTES || data.len().checked_sub(start) != Some(expected) {
        return Err("invalid raster length");
    }
    Ok(RgbFrame { width, height, pixels: &data[start..] })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_exact_rgb_frame() {
        let f = parse(b"P6\n2 1\n255\n\x00\x01\x02\x03\x04\x05").unwrap();
        assert_eq!((f.width, f.height, f.pixels.len()), (2, 1, 6));
    }
    #[test]
    fn rejects_malformed_and_trailing_bytes() {
        for data in [
            &b"P6\\n2 1\\n255\\nabcdef"[..],
            &b"P6\n2 1\n255\nabcde"[..],
            &b"P6\n2 1\n255\nabcdefg"[..],
            &b"P6\n0 1\n255\n"[..],
            &b"P6\n999999 999999\n255\n"[..],
            &b"P6\n2 1\n65535\nabcdef"[..],
        ] {
            assert!(parse(data).is_err());
        }
    }
}
