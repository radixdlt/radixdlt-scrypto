use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

/// Represents an error when running a cargo command.
#[derive(Debug)]
pub enum CargoExecutionError {
    NotCargoPackage,

    MissingPackageName,

    FailedToRunCargo(io::Error),

    FailedToBuild(ExitStatus),

    FailedToTest(ExitStatus),
}

/// Builds a package.
pub fn build_package<P: AsRef<Path>>(path: P, trace: bool) -> Result<PathBuf, CargoExecutionError> {
    let mut cargo = path.as_ref().to_owned();
    cargo.push("Cargo.toml");
    if cargo.exists() {
        let status = Command::new("cargo")
            .arg("build")
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .arg("--release")
            .arg("--manifest-path")
            .arg(cargo.to_str().unwrap())
            .args(if trace {
                vec!["--features", "scrypto/trace"]
            } else {
                vec![]
            })
            .status()
            .map_err(CargoExecutionError::FailedToRunCargo)?;
        if !status.success() {
            return Err(CargoExecutionError::FailedToBuild(status));
        }

        let mut bin = path.as_ref().to_owned();
        bin.push("target");
        bin.push("wasm32-unknown-unknown");
        bin.push("release");
        bin.push("out");
        Ok(bin.with_extension("wasm"))
    } else {
        Err(CargoExecutionError::NotCargoPackage)
    }
}

/// Runs tests within a package.
pub fn test_package<P: AsRef<Path>, I, S>(path: P, args: I) -> Result<(), CargoExecutionError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    build_package(&path, false)?;

    let mut cargo = path.as_ref().to_owned();
    cargo.push("Cargo.toml");
    if cargo.exists() {
        let status = Command::new("cargo")
            .arg("test")
            .arg("--release")
            .arg("--manifest-path")
            .arg(cargo.to_str().unwrap())
            .arg("--")
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
