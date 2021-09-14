use std::process::Command;

fn main() {
    Command::new("cargo")
        .current_dir("./tests/everything")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    println!("cargo:rerun-if-changed=tests/everything/src");
    println!("cargo:rerun-if-changed=tests/everything/Cargo.toml");
}
