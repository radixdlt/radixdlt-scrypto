use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn compile_package<P: AsRef<Path>, S: AsRef<str>>(package_dir: P, wasm_name: S) -> Vec<u8> {
    // build
    Command::new("cargo")
        .current_dir(package_dir.as_ref())
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .status()
        .unwrap();

    // path of the wasm executable
    let mut path = PathBuf::from(package_dir.as_ref());
    path.push("target");
    path.push("wasm32-unknown-unknown");
    path.push("release");
    path.push(wasm_name.as_ref());
    path.set_extension("wasm");

    // return
    fs::read(path).unwrap()
}
