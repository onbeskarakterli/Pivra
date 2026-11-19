//! Normalize sandbox-decoded thumbnail pixels. No decoder or source-file I/O.
use gtk::{gdk, glib, prelude::*};

pub(crate) fn from_rgba(
    bytes: &glib::Bytes,
    width: i32,
    height: i32,
    stride: usize,
    size: (i32, i32),
) -> Result<gdk::Texture, String> {
    let row = usize::try_from(width)
        .ok()
        .and_then(|w| w.checked_mul(4))
        .ok_or("Invalid thumbnail width")?;
    let length = usize::try_from(height)
        .ok()
        .and_then(|h| h.checked_mul(stride))
        .ok_or("Invalid thumbnail height")?;
    if width <= 0 || height <= 0 || stride < row || bytes.len() != length {
        return Err("Invalid thumbnail RGBA layout".into());
    }
    let texture =
        gdk::MemoryTexture::new(width, height, gdk::MemoryFormat::R8g8b8a8, bytes, stride).upcast();
    // Generated, packed RGBA8 is already normalized and sRGB. Preserve the same
    // GBytes/texture instead of creating a downloader and a second texture.
    // Other layouts and resized images retain the existing normalization path.
    if size == (width, height) && stride == row {
        return Ok(texture);
    }
    prepare(&texture, size)
}

pub(crate) fn prepare(texture: &gdk::Texture, size: (i32, i32)) -> Result<gdk::Texture, String> {
    let (width, height) = size;
    let mut download = gdk::TextureDownloader::new(texture);
    download.set_format(gdk::MemoryFormat::R8g8b8a8);
    let (bytes, stride) = download.download_bytes();
    if (width, height) == (texture.width(), texture.height()) {
        return Ok(gdk::MemoryTexture::new(
            width,
            height,
            gdk::MemoryFormat::R8g8b8a8,
            &bytes,
            stride,
        )
        .upcast());
    }
    let stride = i32::try_from(stride).map_err(|_| "Thumbnail stride exceeds limit")?;
    let pixels = gdk::gdk_pixbuf::Pixbuf::from_bytes(
        &bytes,
        gdk::gdk_pixbuf::Colorspace::Rgb,
        true,
        8,
        texture.width(),
        texture.height(),
        stride,
    );
    let small = pixels
        .scale_simple(width, height, gdk::gdk_pixbuf::InterpType::Bilinear)
        .ok_or("Cannot scale decoded thumbnail pixels")?;
    Ok(gdk::Texture::for_pixbuf(&small))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rgba(texture: &gdk::Texture) -> (glib::Bytes, usize) {
        let mut download = gdk::TextureDownloader::new(texture);
        download.set_format(gdk::MemoryFormat::R8g8b8a8);
        download.download_bytes()
    }
    #[test]
    fn packed_pixels_alpha_and_color_are_unchanged() {
        let bytes = glib::Bytes::from_owned([31, 99, 201, 12, 77, 63, 55, 255].repeat(8));
        let texture = from_rgba(&bytes, 4, 4, 16, (4, 4)).unwrap();
        assert_eq!(texture.color_state(), gdk::ColorState::srgb());
        assert_eq!(texture.format(), gdk::MemoryFormat::R8g8b8a8);
        let (actual, stride) = rgba(&texture);
        assert_eq!(stride, 16);
        assert_eq!(actual.as_ref(), bytes.as_ref());
    }
    #[test]
    fn padded_and_scaled_inputs_keep_normalization() {
        let bytes = glib::Bytes::from_owned([31, 99, 201, 12].repeat(20));
        let padded = from_rgba(&bytes, 4, 4, 20, (4, 4)).unwrap();
        let (pixels, stride) = rgba(&padded);
        assert!(stride >= 16);
        for row in pixels.chunks_exact(stride) {
            assert_eq!(&row[..16], [31, 99, 201, 12].repeat(4));
        }
        let source: gdk::Texture =
            gdk::MemoryTexture::new(4, 4, gdk::MemoryFormat::R8g8b8a8, &bytes, 20).upcast();
        let expected = prepare(&source, (2, 2)).unwrap();
        let scaled = from_rgba(&bytes, 4, 4, 20, (2, 2)).unwrap();
        assert_eq!((scaled.width(), scaled.height()), (2, 2));
        assert_eq!(rgba(&scaled).0, rgba(&expected).0);
    }
    #[test]
    fn malformed_layouts_are_rejected_before_texture_construction() {
        let bytes = glib::Bytes::from_owned([0u8; 16]);
        for (w, h, stride) in [
            (0, 1, 4),
            (-1, 1, 4),
            (4, 4, 15),
            (4, 4, 16),
            (1, 4, usize::MAX),
        ] {
            assert!(from_rgba(&bytes, w, h, stride, (1, 1)).is_err());
        }
    }
}
