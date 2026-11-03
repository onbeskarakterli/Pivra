use std::{env, path::PathBuf, process::Command};
pub fn build() {
    println!("cargo:rustc-check-cfg=cfg(pivra_gpu_backend)");
    println!("cargo:rerun-if-changed=native/video");
    for name in [
        "CPPFLAGS",
        "LDFLAGS",
        "CC",
        "AR",
        "PKG_CONFIG_LIBDIR",
        "PIVRA_DISABLE_GPU_BUILD",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    if env::var_os("PIVRA_DISABLE_GPU_BUILD").is_some_and(|v| v == "1") {
        return;
    }
    if !Command::new("pkg-config")
        .args(["--atleast-version=1.24.0", "gstreamer-gl-1.0"])
        .status()
        .is_ok_and(|s| s.success())
    {
        println!("cargo::metadata=video-gpu-status=unavailable:requires-native-gstreamer-1.24");
        return;
    }
    let packages = [
        "gtk4",
        "gstreamer-video-1.0",
        "gstreamer-gl-1.0",
        "gstreamer-allocators-1.0",
        "gstreamer-pbutils-1.0",
    ];
    let flags = |option| {
        Command::new("pkg-config")
            .arg(option)
            .args(packages)
            .output()
    };
    let (Ok(cflags), Ok(libs)) = (flags("--cflags"), flags("--libs")) else {
        return;
    };
    if !cflags.status.success() || !libs.status.success() {
        println!("cargo::metadata=video-gpu-status=unavailable:missing-native-development-api");
        return;
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut objects = Vec::new();
    for name in ["gtkgstmediafile", "gtkgstpaintable", "gtkgstsink", "bridge"] {
        let object = out.join(format!("video-{name}.o"));
        let status = Command::new(env::var_os("CC").unwrap_or_else(|| "cc".into()))
            .args([
                "-c",
                "-O2",
                "-fPIC",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-Wno-unused-parameter",
                "-DGTK_COMPILATION",
                "-Inative/video",
            ])
            .args(env::var("CPPFLAGS").unwrap_or_default().split_whitespace())
            .args(String::from_utf8_lossy(&cflags.stdout).split_whitespace())
            .arg(format!("native/video/{name}.c"))
            .arg("-o")
            .arg(&object)
            .status()
            .unwrap();
        assert!(status.success(), "GPU video compilation failed: {name}");
        objects.push(object);
    }
    assert!(
        Command::new(env::var_os("AR").unwrap_or_else(|| "ar".into()))
            .arg("crs")
            .arg(out.join("libpivra_video_gpu.a"))
            .args(objects)
            .status()
            .unwrap()
            .success()
    );
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=pivra_video_gpu");
    for flag in String::from_utf8_lossy(&libs.stdout)
        .split_whitespace()
        .chain(env::var("LDFLAGS").unwrap_or_default().split_whitespace())
    {
        if let Some(path) = flag.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={path}");
        } else if let Some(lib) = flag.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={lib}");
        } else {
            println!("cargo:rustc-link-arg={flag}");
        }
    }
    println!("cargo:rustc-cfg=pivra_gpu_backend");
    println!("cargo::metadata=video-gpu-status=built:static-adapter");
}
