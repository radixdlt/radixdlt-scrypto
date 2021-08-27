use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::cli::*;

pub fn build_cargo_package(mut path: PathBuf) -> Result<PathBuf, BuildPackageError> {
    let mut cargo = path.clone();
    cargo.push("Cargo.toml");

    if cargo.exists() {
        Command::new("cargo")
            .arg("build")
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .arg("--release")
            .arg("--manifest-path")
            .arg(cargo.canonicalize().unwrap().to_str().unwrap())
            .spawn()
            .map_err(|_| BuildPackageError::FailedToRunCargo)?
            .wait()
            .map_err(|_| BuildPackageError::FailedToWaitCargo)?;

        let toml =
            fs::read_to_string(cargo).map_err(|_| BuildPackageError::FailedToReadCargoToml)?;
        let mut wasm = None;
        for line in toml.split('\n') {
            if line.starts_with("name = \"") {
                let start = line.find("\"").unwrap();
                let end = line.rfind("\"").unwrap();
                path.push("target");
                path.push("wasm32-unknown-unknown");
                path.push("release");
                path.push(&line[start + 1..end]);
                wasm = Some(path.with_extension("wasm"));
                break;
            }
        }
        wasm.ok_or(BuildPackageError::FailedToParseCargoToml)
    } else {
        Err(BuildPackageError::NotCargoPackage)
    }
}
