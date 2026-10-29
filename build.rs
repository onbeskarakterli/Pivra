// Optional native GNOME factory sidecar. Compile against ONE real public API;
// absence leaves the viewer/cache reader functional, never selects another decoder.
use std::{env, path::PathBuf, process::Command};
fn main() {
    println!("cargo:rerun-if-changed=native/thumbnail_factory.c");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_SYSROOT_DIR");
    println!("cargo:rerun-if-env-changed=PIVRA_DISTRIBUTION_BUILD");
    // Packages compile/install the mandatory sidecar separately. Never bake
    // their temporary OUT_DIR into the installed viewer's runtime fallback.
    if env::var_os("PIVRA_DISTRIBUTION_BUILD").is_some_and(|value| value == "1") {
        println!("cargo:rustc-env=PIVRA_BUILT_THUMBNAIL_FACTORY=");
        println!("cargo::metadata=gnome-helper-status=runtime-sibling");
        return;
    }
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("pivra-thumbnail-factory");
    for module in ["gnome-desktop-4", "gnome-desktop-3.0"] {
        let Ok(flags) = Command::new("pkg-config")
            .args([
                "--cflags",
                "--libs",
                module,
                "gio-2.0",
                "gobject-2.0",
                "gdk-pixbuf-2.0",
            ])
            .output()
        else {
            continue;
        };
        if !flags.status.success() {
            continue;
        }
        let flags = String::from_utf8_lossy(&flags.stdout);
        let result = Command::new("cc")
            .args([
                "-O2",
                "-Wall",
                "-Wextra",
                "-Werror",
                "native/thumbnail_factory.c",
                "-o",
            ])
            .arg(&output)
            .args(flags.split_whitespace())
            .output();
        if let Ok(result) = result {
            if result.status.success() {
                println!(
                    "cargo:rustc-env=PIVRA_BUILT_THUMBNAIL_FACTORY={}",
                    output.display()
                );
                println!("cargo::metadata=gnome-helper-status=built:{module}");
                return;
            }
            println!(
                "cargo:warning=GNOME helper API/link check failed for {module}: {}",
                String::from_utf8_lossy(&result.stderr).replace('\n', " ")
            );
        }
    }
    println!("cargo:rustc-env=PIVRA_BUILT_THUMBNAIL_FACTORY=");
    // Missing optional development headers is an expected capability, not a
    // compiler warning. Cargo -vv retains this status; real link failures above
    // still emit warnings, and runtime discovery/cache-only behavior is unchanged.
    println!("cargo::metadata=gnome-helper-status=cache-only:no-development-api");
}
