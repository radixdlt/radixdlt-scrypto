use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

use cargo_toml::Manifest;
use radix_engine::model::extract_abi;
use radix_engine::model::ExtractAbiError;
use radix_engine::types::*;

#[derive(Debug)]
pub enum BuildError {
    NotCargoPackage,

    MissingPackageName,

    IOError(io::Error),

    CargoFailure(ExitStatus),

    AbiExtractionError(ExtractAbiError),

    InvalidManifestFile,
}

#[derive(Debug)]
pub enum TestError {
    NotCargoPackage,

    BuildError(BuildError),

    IOError(io::Error),

    CargoFailure(ExitStatus),
}

#[derive(Debug)]
pub enum FormatError {
    BuildError(BuildError),

    IOError(io::Error),

    CargoFailure(ExitStatus),
}

fn run_cargo_build(path: &str, trace: bool, no_abi_gen: bool) -> Result<(), BuildError> {
    let mut features = Vec::<String>::new();
    if trace {
        features.push("scrypto/trace".to_owned());
    }
    if no_abi_gen {
        features.push("scrypto/no-abi-gen".to_owned());
    }
    if !features.is_empty() {
        features.insert(0, "--features".to_owned());
    }

    let status = Command::new("cargo")
        .arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("--release")
        .arg("--manifest-path")
        .arg(path)
        .args(features)
        .status()
        .map_err(BuildError::IOError)?;
    if status.success() {
        Ok(())
    } else {
        Err(BuildError::CargoFailure(status))
    }
}

/// Builds a package.
pub fn build_package<P: AsRef<Path>>(path: P, trace: bool) -> Result<PathBuf, BuildError> {
    let mut cargo = path.as_ref().to_owned();
    cargo.push("Cargo.toml");
    if cargo.exists() {
        // Build with ABI
        run_cargo_build(cargo.to_str().unwrap(), trace, false)?;

        // Find the binary paths
        let manifest = Manifest::from_path(&cargo).map_err(|_| BuildError::InvalidManifestFile)?;
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
        bin.push(wasm_name.ok_or(BuildError::InvalidManifestFile)?);
        let wasm_path = bin.with_extension("wasm");
        let abi_path = bin.with_extension("abi");

        // Extract ABI
        let wasm = fs::read(&wasm_path).map_err(BuildError::IOError)?;
        let abi = extract_abi(&wasm).map_err(BuildError::AbiExtractionError)?;
        fs::write(&abi_path, scrypto_encode(&abi)).map_err(BuildError::IOError)?;

        // Build without ABI
        run_cargo_build(cargo.to_str().unwrap(), trace, true)?;

        Ok(wasm_path)
    } else {
        Err(BuildError::NotCargoPackage)
    }
}

/// Runs tests within a package.
pub fn test_package<P: AsRef<Path>, I, S>(path: P, args: I) -> Result<(), TestError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    build_package(&path, false).map_err(TestError::BuildError)?;

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
            .map_err(TestError::IOError)?;
        if !status.success() {
            return Err(TestError::CargoFailure(status));
        }
        Ok(())
    } else {
        Err(TestError::NotCargoPackage)
    }
}

/// Format a package.
pub fn fmt_package<P: AsRef<Path>>(path: P, check: bool, quiet: bool) -> Result<(), FormatError> {
    let mut cargo = path.as_ref().to_owned();
    cargo.push("Cargo.toml");
    if cargo.exists() {
        // replace `blueprint!` with `mod blueprint`
        let mut src = path.as_ref().to_owned();
        src.push("src");
        for entry in fs::read_dir(&src).map_err(FormatError::IOError)? {
            let p = entry.map_err(FormatError::IOError)?.path();
            if let Some(ext) = p.extension() {
                if ext.to_str() == Some("rs") {
                    let code = fs::read_to_string(&p).map_err(FormatError::IOError)?;
                    let code_transformed = code
                        .replace("blueprint!", "mod blueprint")
                        // Reverts unintended replacement of `external_blueprint!` to `external_mod blueprint` by the `replace` above
                        .replace("external_mod blueprint", "external_blueprint!");
                    fs::write(&p, code_transformed).map_err(FormatError::IOError)?;
                }
            }
        }

        let status = Command::new("cargo")
            .arg("fmt")
            .arg("--manifest-path")
            .arg(cargo.to_str().unwrap())
            .args({
                let mut args = Vec::new();
                if check {
                    args.push("--check")
                }
                if quiet {
                    args.push("--quiet")
                }
                args
            })
            .status()
            .map_err(FormatError::IOError)?;

        // replace `mod blueprint` with `blueprint!`
        for entry in fs::read_dir(&src).map_err(FormatError::IOError)? {
            let p = entry.map_err(FormatError::IOError)?.path();
            if let Some(ext) = p.extension() {
                if ext.to_str() == Some("rs") {
                    let code = fs::read_to_string(&p).map_err(FormatError::IOError)?;
                    let code_transformed = code.replace("mod blueprint", "blueprint!");
                    fs::write(&p, code_transformed).map_err(FormatError::IOError)?;
                }
            }
        }

        if status.success() {
            Ok(())
        } else {
            Err(FormatError::CargoFailure(status))
        }
    } else {
        Ok(())
    }
}
