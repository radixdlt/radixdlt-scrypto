use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

use cargo_toml::Manifest;

/// Represents an error when running a cargo command.
#[derive(Debug)]
pub enum CargoExecutionError {
    NotCargoPackage,

    MissingPackageName,

    IOError(io::Error),

    FailedToRunCargo(io::Error),

    FailedToBuild(ExitStatus),

    FailedToTest(ExitStatus),

    FailedToFormat(ExitStatus),

    InvalidManifestFile,
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

        let manifest =
            Manifest::from_path(&cargo).map_err(|_| CargoExecutionError::InvalidManifestFile)?;

        // resolve lib name from manifest
        let mut wasm_name = None;
        if let Some(lib) = manifest.lib {
            wasm_name = lib.name.clone();
        }
        if wasm_name == None {
            if let Some(pkg) = manifest.package {
                wasm_name = Some(pkg.name.replace("-", "_"));
            }
        }

        let mut bin = path.as_ref().to_owned();
        bin.push("target");
        bin.push("wasm32-unknown-unknown");
        bin.push("release");
        bin.push(wasm_name.ok_or(CargoExecutionError::InvalidManifestFile)?);
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

/// Format a package.
pub fn fmt_package<P: AsRef<Path>>(
    path: P,
    check: bool,
    quiet: bool,
) -> Result<(), CargoExecutionError> {
    let mut cargo = path.as_ref().to_owned();
    cargo.push("Cargo.toml");
    if cargo.exists() {
        // replace `blueprint!` with `mod blueprint`
        let mut src = path.as_ref().to_owned();
        src.push("src");
        for entry in fs::read_dir(&src).map_err(CargoExecutionError::IOError)? {
            let p = entry.map_err(CargoExecutionError::IOError)?.path();
            if let Some(ext) = p.extension() {
                if ext.to_str() == Some("rs") {
                    let code = fs::read_to_string(&p).map_err(CargoExecutionError::IOError)?;
                    let code_transformed = code.replace("blueprint!", "mod blueprint");
                    fs::write(&p, code_transformed).map_err(CargoExecutionError::IOError)?;
                }
            }
        }

        let status = Command::new("cargo")
            .arg("fmt")
            .arg("--manifest-path")
            .arg(cargo.to_str().unwrap())
            .args({
                let mut args = Vec::new();
                if (check) {
                    args.push("--check")
                }
                if (quiet) {
                    args.push("--quiet")
                }
                args
            })
            .status()
            .map_err(CargoExecutionError::FailedToRunCargo)?;

        // replace `mod blueprint` with `blueprint!`
        for entry in fs::read_dir(&src).map_err(CargoExecutionError::IOError)? {
            let p = entry.map_err(CargoExecutionError::IOError)?.path();
            if let Some(ext) = p.extension() {
                if ext.to_str() == Some("rs") {
                    let code = fs::read_to_string(&p).map_err(CargoExecutionError::IOError)?;
                    let code_transformed = code.replace("mod blueprint", "blueprint!");
                    fs::write(&p, code_transformed).map_err(CargoExecutionError::IOError)?;
                }
            }
        }

        if status.success() {
            Ok(())
        } else {
            Err(CargoExecutionError::FailedToFormat(status))
        }
    } else {
        Err(CargoExecutionError::NotCargoPackage)
    }
}
