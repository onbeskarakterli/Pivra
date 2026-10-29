//! Worker-only native benchmark entry. No cache access or policy override.
use gtk::{gio, prelude::*};
use pivra::{
    freedesktop_generator::generate,
    freedesktop_thumbnail::SizeClass,
    image_cache::FileKey,
    thumbnail_provider::{Error, Request},
};
use std::{
    io::{self, Write},
    path::PathBuf,
    time::{Duration, Instant},
};
fn main() {
    let mut args = std::env::args().skip(1);
    let first = args.next().expect("source");
    if first == "--fixture-jpeg" {
        let w: u32 = args.next().unwrap().parse().unwrap();
        let h: u32 = args.next().unwrap().parse().unwrap();
        let path = PathBuf::from(args.next().unwrap());
        let pixels = u64::from(w) * u64::from(h);
        assert!(w > 0 && h > 0 && pixels <= 50_000_000);
        let len = usize::try_from(pixels * 3).unwrap();
        if pivra::freedesktop_generator::memory_headroom().unwrap() < len + 512 * 1024 * 1024 {
            eprintln!("NOT RUN fixture memory headroom");
            std::process::exit(77);
        }
        let mut data = vec![0; len];
        for p in data.chunks_exact_mut(3) {
            p.copy_from_slice(&[17, 83, 191]);
        }
        let bytes = gtk::glib::Bytes::from_owned(data);
        let pixbuf = gtk::gdk::gdk_pixbuf::Pixbuf::from_bytes(
            &bytes,
            gtk::gdk::gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            w as i32,
            h as i32,
            w as i32 * 3,
        );
        pixbuf
            .savev(&path, "jpeg", &[("quality", "90")])
            .expect("trusted TEST RGB JPEG encoding");
        return;
    }
    let path = PathBuf::from(first);
    let cancel_active = args.any(|a| a == "--cancel-active");
    let key = FileKey::read(&path).unwrap();
    let token = gio::Cancellable::new();
    let input = if cancel_active {
        let token = token.clone();
        Some(std::thread::spawn(move || {
            let mut text = String::new();
            if io::stdin().read_line(&mut text).unwrap() > 0 && text.trim() == "cancel" {
                token.cancel();
                println!("CANCEL_ISSUED");
                io::stdout().flush().unwrap();
            }
        }))
    } else {
        None
    };
    let result = generate(
        &Request {
            source: &key,
            cancel: &token,
            deadline: Instant::now() + Duration::from_secs(90),
        },
        SizeClass::Normal,
    );
    if let Some(input) = input {
        input.join().unwrap();
    }
    match result {
        Ok(t) => {
            if cancel_active {
                eprintln!("NOT RUN decode completed before active cancellation checkpoint");
                std::process::exit(77);
            }
            assert_eq!(FileKey::read(&path).unwrap(), key);
            println!(
                "PASS decoded={}x{} frame_bytes={} download_bytes={} output_bytes={} decode_us={} resize_us={}",
                t.metrics.decoded_width,
                t.metrics.decoded_height,
                t.metrics.frame_bytes,
                t.metrics.download_bytes,
                t.pixels.len(),
                t.metrics.decode_time.as_micros(),
                t.metrics.resize_time.as_micros()
            );
        }
        Err(Error::Cancelled) if cancel_active => {
            assert_eq!(
                FileKey::read(&path).ok().as_ref(),
                Some(&key),
                "source changed during cancellation"
            );
            println!("PASS CANCELLED future_completed source_unchanged=true");
        }
        Err(Error::GenerationFailed(e))
            if ["bwrap", "Loader process exited early", "No loader"]
                .iter()
                .any(|s| e.contains(s)) =>
        {
            eprintln!("NOT RUN secure prerequisite: {e}");
            std::process::exit(77);
        }
        Err(Error::LimitExceeded(e)) => {
            eprintln!("LIMIT_EXCEEDED: {e}");
            std::process::exit(77);
        }
        Err(e) => {
            eprintln!("FAIL {e:?}");
            std::process::exit(1);
        }
    }
}
