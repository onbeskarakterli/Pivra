//! TH-01 metadata/cache only; never displays/decodes a thumbnail or requests generation.
use gtk::{gio, prelude::*};
use pivra::thumbnail::{Outcome, Provider};
use std::path::PathBuf;
fn main() {
    let mut args = std::env::args_os().skip(1);
    let factory = std::env::var_os("PIVRA_THUMBNAIL_FACTORY_PROBE").map(PathBuf::from);
    let provider = Provider::discover(factory);
    println!(
        "initial capability={:?}; GNOME only; no GTK/pixel decode/generation",
        provider.capability()
    );
    let path = args.next().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images/wide.png")
    });
    assert!(
        args.next().is_none(),
        "one source per invocation; no directory scan"
    );
    let token = gio::Cancellable::new();
    // Real provider work is off the caller/UI thread, not GLib event polling.
    let result=std::thread::spawn(move || {
        let outcome=provider.query(&path,&token);
        println!("capability={:?}",provider.capability());
        match outcome {
            Outcome::Hit(candidate) => {
                println!("HIT metadata/container valid uri={} cache={:?} bytes={} current={} secure pixel decode still required",candidate.uri,candidate.thumbnail.path,candidate.png.len(),candidate.still_current(&token));
                assert!(candidate.still_current(&token));
            }
            other=>println!("OUTCOME {other:?}; no thumbnail pixel PASS claimed"),
        }
    }).join();
    assert!(result.is_ok(), "thumbnail worker panicked");
    // Cancellation is a controlled query result, not generation process cancellation.
    let token = gio::Cancellable::new();
    token.cancel();
    let provider = Provider::new(true, None);
    assert!(matches!(
        provider.query(std::path::Path::new("unused"), &token),
        Outcome::Cancelled
    ));
}
