//! PIV-12 extension of the existing production-window probe, sharing its helpers.
use super::*;
fn status(w: &adw::ApplicationWindow) -> String {
    picture(w)
        .next_sibling()
        .unwrap()
        .downcast::<gtk::Label>()
        .unwrap()
        .text()
        .to_string()
}
fn failure(w: &adw::ApplicationWindow, native: bool) {
    wait(|| status(w).starts_with("Unable to display image."));
    assert!(picture(w).paintable().is_none());
    let message = status(w);
    if native {
        assert!(
            !message.contains("Loader process exited early"),
            "loader failure is not an invalid-file PASS: {message}"
        );
    }
    settle();
    assert_eq!(
        status(w),
        message,
        "error retried or stale callback changed it"
    );
}
fn healthy(w: &adw::ApplicationWindow, native: bool, baseline: bool) {
    pixels(w, native, baseline);
    if native {
        let frame = picture(w).paintable().unwrap();
        assert_eq!((frame.intrinsic_width(), frame.intrinsic_height()), (4, 2));
    }
}
pub(super) fn run(app: &adw::Application, native: bool) {
    let root = tempfile::tempdir().unwrap();
    let independent = tempfile::tempdir().unwrap();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images");
    let good = std::fs::read(fixtures.join("small.png")).unwrap();
    std::fs::write(root.path().join("1.png"), &good).unwrap();
    std::fs::write(independent.path().join("only.png"), &good).unwrap();
    let first = open(app, &root.path().join("1.png"));
    let second = open(app, &independent.path().join("only.png"));
    healthy(&first, native, true);
    healthy(&second, native, false);
    let survivor = picture(&second).paintable();
    let cases = [
        ("2.jpg", std::fs::read(fixtures.join("broken.jpg")).unwrap()),
        ("3.png", std::fs::read(fixtures.join("broken.png")).unwrap()),
        ("4.png", good[..20].to_vec()),
        ("5.jpg", b"this is plain text, not an image".to_vec()),
        ("6.jpg", vec![]),
        ("7.webp", b"unsupported candidate".to_vec()),
        ("8.png", good.clone()),
    ];
    for (name, bytes) in &cases {
        std::fs::write(root.path().join(name), bytes).unwrap();
    }
    wait(|| count(&first, 8));
    for name in ["2.jpg", "3.png", "4.png", "5.jpg", "6.jpg"] {
        action(&first, "next-media");
        assert!(first.title().unwrap().starts_with(name));
        failure(&first, native);
        assert_eq!(picture(&second).paintable(), survivor);
        assert!(!banner(&second).reveals_child());
    }
    // A failed selected image remains selected as its folder changes; no retry.
    let failed_message = status(&first);
    std::fs::write(root.path().join("9.mp4"), b"catalog only").unwrap();
    wait(|| count(&first, 9));
    assert!(first.title().unwrap().starts_with("6.jpg"));
    assert_eq!(status(&first), failed_message);
    // Repeated refresh then navigation must discard obsolete error callbacks.
    action(&first, "reload-media");
    action(&first, "reload-media");
    action(&first, "next-media");
    wait(|| status(&first).starts_with("This image format is not supported yet."));
    settle();
    assert!(first.title().unwrap().starts_with("7.webp"));
    std::fs::write(root.path().join("7.webp"), b"modified unsupported format").unwrap();
    wait(|| banner(&first).reveals_child());
    action(&first, "reload-media");
    assert!(status(&first).starts_with("This image format is not supported yet."));
    action(&first, "next-media");
    healthy(&first, native, false);
    assert!(first.title().unwrap().starts_with("8.png"));
    // Only this window receives a notice; repeated refresh + navigation keeps it local.
    std::fs::write(root.path().join("8.png"), &good).unwrap();
    wait(|| banner(&first).reveals_child());
    assert!(!banner(&second).reveals_child());
    action(&first, "reload-media");
    action(&first, "reload-media");
    action(&first, "next-media");
    settle();
    assert!(status(&first).starts_with("Video playback is not available yet."));
    assert!(!banner(&first).reveals_child());
    std::fs::write(root.path().join("9.mp4"), b"changed video candidate").unwrap();
    wait(|| banner(&first).reveals_child());
    action(&first, "reload-media");
    assert!(status(&first).starts_with("Video playback is not available yet."));
    assert!(!banner(&first).reveals_child());
    // Delete a selected file immediately after scheduling a real production load.
    // Scheduling is not proof the secure decoder was active at deletion time.
    action(&first, "previous-media");
    std::fs::remove_file(root.path().join("8.png")).unwrap();
    wait(|| {
        first
            .title()
            .is_some_and(|s| s.starts_with("No media selected"))
    });
    assert!(picture(&first).paintable().is_none());
    action(&first, "next-media");
    healthy(&first, native, false);
    // Close with outstanding image requests and real monitor events.
    for _ in 0..20 {
        action(&first, "next-media");
        action(&first, "previous-media");
    }
    std::fs::write(root.path().join("10.png"), &good).unwrap();
    let weak = first.downgrade();
    first.close();
    drop(first);
    wait(|| weak.upgrade().is_none());
    settle();
    assert!(second.is_visible());
    assert_eq!(picture(&second).paintable(), survivor);
    assert!(!banner(&second).reveals_child());
    assert!(count(&second, 1));
    // Survivor can still refresh through its own monitor and loader after the close.
    std::fs::write(independent.path().join("only.png"), &good).unwrap();
    wait(|| banner(&second).reveals_child());
    let windows_before = app.windows();
    let button = widgets(second.upcast_ref())
        .into_iter()
        .find_map(|widget| {
            widget
                .downcast::<gtk::Button>()
                .ok()
                .filter(|b| b.label().as_deref() == Some("Yenile"))
        })
        .unwrap();
    button.emit_clicked();
    assert_eq!(
        app.windows(),
        windows_before,
        "refresh created/replaced a window"
    );
    healthy(&second, native, false);
    if native {
        wait(|| !banner(&second).reveals_child());
    }
    assert_eq!(
        app.windows(),
        windows_before,
        "refresh replaced window asynchronously"
    );
    second.close();
    drop(second);
    for (name, bytes) in cases {
        if name != "7.webp" && name != "8.png" {
            assert_eq!(std::fs::read(root.path().join(name)).unwrap(), bytes);
        }
    }
    println!(
        "PASS PIV-12 production GTK/GIO failure states, navigation recovery, catalog/selection/monitor/notice isolation, refresh replacement and close cleanup"
    );
    if native {
        println!(
            "PASS PIV-12 secure baseline, corrupt/truncated/empty/disguised-media errors and recovery to real frames"
        );
    } else {
        println!(
            "NOT RUN real decoder classification/pixel acceptance: scoped sandbox-error paths only"
        );
    }
    println!(
        "NOT MEASURED secure decode activity at deletion/close or temporal decoder overlap; controlled-worker races are separate"
    );
}
