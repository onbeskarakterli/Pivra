//! Model/filesystem timings; never claims decoder or GPU success.
use std::{path::PathBuf, sync::atomic::AtomicBool, time::Instant};
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let path = PathBuf::from(&args[0]);
    for iteration in 0..11 {
        let start = Instant::now();
        let mut report = pivra::discovery::scan_directory(&path, &AtomicBool::new(false)).unwrap();
        assert!(report.issues.is_empty());
        let count = report.collection.items().len();
        println!(
            "{{\"scenario\":\"scan\",\"iteration\":{iteration},\"files\":{count},\"elapsed_us\":{}}}",
            start.elapsed().as_micros()
        );
        let start = Instant::now();
        report.collection.select(0);
        for _ in 0..1000 {
            report.collection.select_photo(true);
            report.collection.select_photo(false);
        }
        println!(
            "{{\"scenario\":\"model_navigation_2000\",\"iteration\":{iteration},\"files\":{count},\"elapsed_us\":{}}}",
            start.elapsed().as_micros()
        );
    }
    println!(
        "{{\"scenario\":\"64mp_policy\",\"estimated_charge_bytes\":{}}}",
        pivra::image_cache::texture_cost(8000, 8000).unwrap()
    );
}
