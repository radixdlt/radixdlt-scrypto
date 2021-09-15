use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

#[derive(Debug)]
pub enum CargoExecutionError {
    NotCargoPackage,

    InvalidCargoToml(cargo_toml::Error),

    MissingPackageName,

    FailedToRunCargo(io::Error),

    FailedToBuild(ExitStatus),

    FailedToTest(ExitStatus),
}

pub fn build_package<P: AsRef<Path>>(path: P) -> Result<PathBuf, CargoExecutionError> {
    let mut cargo = path.as_ref().to_owned();
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
            .map_err(CargoExecutionError::FailedToRunCargo)?;
        if !status.success() {
            return Err(CargoExecutionError::FailedToBuild(status));
        }

        let manifest = cargo_toml::Manifest::from_path(cargo)
            .map_err(CargoExecutionError::InvalidCargoToml)?;

        let mut bin = path.as_ref().to_owned();
        bin.push("target");
        bin.push("wasm32-unknown-unknown");
        bin.push("release");
        bin.push(
            manifest
                .package
                .ok_or(CargoExecutionError::MissingPackageName)?
                .name,
        );
        Ok(bin.with_extension("wasm"))
    } else {
        Err(CargoExecutionError::NotCargoPackage)
    }
}

pub fn test_package<P: AsRef<Path>, I, S>(path: P, args: I) -> Result<(), CargoExecutionError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    build_package(&path)?;

    let mut cargo = path.as_ref().to_owned();
    cargo.push("Cargo.toml");
    if cargo.exists() {
        let status = Command::new("cargo")
            .arg("test")
            .arg("--manifest-path")
            .arg(cargo.canonicalize().unwrap().to_str().unwrap())
            .args(args)
            .status()
            .map_err(CargoExecutionError::FailedToRunCargo)?;
        if !status.success() {
            return Err(CargoExecutionError::FailedToTest(status));
        }
        Ok(())
    } else {
        Err(CargoExecutionError::NotCargoPackage)
    }
}
