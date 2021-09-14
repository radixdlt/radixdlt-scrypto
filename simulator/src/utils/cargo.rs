use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

#[derive(Debug)]
pub enum BuildPackageError {
    NotCargoPackage,

    FailedToParseCargoToml(cargo_toml::Error),

    MissingPackageInCargoToml,

    FailedToRunCargo(io::Error),

    FailedToBuild(ExitStatus),
}

pub fn build_package(mut path: PathBuf) -> Result<PathBuf, BuildPackageError> {
    let mut cargo = path.clone();
    cargo.push("Cargo.toml");

    if cargo.exists() {
        let status = Command::new("cargo")
            .arg("build")
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .arg("--release")
            .arg("--manifest-path")
            .arg(cargo.canonicalize().unwrap().to_str().unwrap())
            .status()
            .map_err(|e| BuildPackageError::FailedToRunCargo(e))?;
        if !status.success() {
            return Err(BuildPackageError::FailedToBuild(status));
        }

        let manifest = cargo_toml::Manifest::from_path(cargo)
            .map_err(|e| BuildPackageError::FailedToParseCargoToml(e))?;
        path.push("target");
        path.push("wasm32-unknown-unknown");
        path.push("release");
        path.push(
            manifest
                .package
                .ok_or(BuildPackageError::MissingPackageInCargoToml)?
                .name,
        );
        Ok(path.with_extension("wasm"))
    } else {
        Err(BuildPackageError::NotCargoPackage)
    }
}
