//! Real GDK memory textures, synthetic already-decoded pixels; no decode/VRAM claim.
#[path = "../src/thumbnail_pixels.rs"]
mod thumbnail_pixels;
use gtk::{gdk, glib, prelude::*};
use std::{hint::black_box, time::Instant};
fn rss() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap()
        .lines()
        .find_map(|l| {
            l.strip_prefix("VmRSS:")
                .map(|v| v.split_whitespace().next().unwrap().parse::<u64>().unwrap() * 1024)
        })
        .unwrap()
}
fn main() {
    const WIDTH: i32 = 192;
    const HEIGHT: i32 = 160;
    let sources: Vec<_> = (0..64u8)
        .map(|n| {
            let pixels = [n, 128, 255 - n, n].repeat((WIDTH * HEIGHT) as usize);
            glib::Bytes::from_owned(pixels)
        })
        .collect();
    let before = rss();
    let mut samples = vec![];
    let mut outputs = vec![];
    for _ in 0..11 {
        outputs.clear();
        let started = Instant::now();
        for bytes in &sources {
            outputs.push(
                thumbnail_pixels::from_rgba(
                    bytes,
                    WIDTH,
                    HEIGHT,
                    WIDTH as usize * 4,
                    (WIDTH, HEIGHT),
                )
                .unwrap(),
            );
        }
        samples.push(started.elapsed().as_micros());
        black_box(&outputs);
    }
    let after = rss();
    for (texture, pixels) in outputs.iter().zip(&sources) {
        assert_eq!((texture.width(), texture.height()), (WIDTH, HEIGHT));
        assert_eq!(texture.format(), gdk::MemoryFormat::R8g8b8a8);
        let mut downloader = gdk::TextureDownloader::new(texture);
        downloader.set_format(gdk::MemoryFormat::R8g8b8a8);
        let (bytes, stride) = downloader.download_bytes();
        assert_eq!(stride, WIDTH as usize * 4);
        assert_eq!(bytes.as_ref(), pixels.as_ref(), "pixel/alpha preservation");
    }
    samples.sort_unstable();
    println!(
        "{{\"scenario\":\"64-packed-rgba-thumbnails\",\"synthetic_pixels\":true,\"samples\":11,\"median_us\":{},\"p95_us\":{},\"rss_before_bytes\":{},\"rss_retained_bytes\":{},\"rss_delta_bytes\":{},\"source_bytes\":{},\"vram_measured\":false}}",
        samples[5],
        samples[10],
        before,
        after,
        after.saturating_sub(before),
        64 * WIDTH * HEIGHT * 4
    );
}
