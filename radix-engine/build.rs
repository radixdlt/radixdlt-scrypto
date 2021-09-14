use std::process::Command;

fn main() {
    let status = Command::new("cargo")
        .current_dir("./tests/everything")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .status()
        .unwrap();
    if !status.success() {
        panic!()
    }

    println!("cargo:rerun-if-changed=tests/everything/src");
    println!("cargo:rerun-if-changed=tests/everything/Cargo.toml");
}
