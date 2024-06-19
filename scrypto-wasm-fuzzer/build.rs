use std::{process::Command, env};

fn main() {
    let status = Command::new("make")
        .args(&["-C", "honggfuzz", "honggfuzz", "libhfuzz/libhfuzz.a", "libhfcommon/libhfcommon.a"])
        .status()
        .expect("failed to run \"make -C honggfuzz hongfuzz libhfuzz/libhfuzz.a libhfcommon/libhfcommon.a\"");
    assert!(status.success());

    let lib_dir = env::current_dir().unwrap().join("honggfuzz/libhfuzz");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    let lib_dir = env::current_dir().unwrap().join("honggfuzz/libhfcommon");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    println!("cargo:rustc-link-lib=static=hfuzz");
    println!("cargo:rustc-link-lib=static=hfcommon");
}
