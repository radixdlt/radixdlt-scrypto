use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

use cargo_toml::Manifest;
use radix_engine::types::*;
use radix_engine::utils::*;

#[derive(Debug)]
pub enum BuildError {
    NotCargoPackage(PathBuf),

    MissingPackageName,

    IOError(io::Error),

    IOErrorAtPath(io::Error, PathBuf),

    CargoTargetDirectoryResolutionError,

    CargoFailure(ExitStatus),

    SchemaExtractionError(ExtractSchemaError),

    SchemaEncodeError(sbor::EncodeError),

    InvalidManifestFile(PathBuf),
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

fn run_cargo_build(
    manifest_path: impl AsRef<OsStr>,
    target_path: impl AsRef<OsStr>,
    trace: bool,
    no_schema_gen: bool,
) -> Result<(), BuildError> {
    let mut features = Vec::<String>::new();
    if trace {
        features.push("scrypto/trace".to_owned());
    }
    if no_schema_gen {
        features.push("scrypto/no-schema".to_owned());
    }
    if !features.is_empty() {
        features.insert(0, "--features".to_owned());
    }

    let status = Command::new("cargo")
        .arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("--release")
        .arg("--target-dir")
        .arg(target_path.as_ref())
        .arg("--manifest-path")
        .arg(manifest_path.as_ref())
        .args(features)
        .status()
        .map_err(BuildError::IOError)?;
    if status.success() {
        Ok(())
    } else {
        Err(BuildError::CargoFailure(status))
    }
}

/// Gets the default cargo directory for the given crate.
/// This respects whether the crate is in a workspace.
pub fn get_default_target_directory(
    manifest_path: impl AsRef<OsStr>,
) -> Result<String, BuildError> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--manifest-path")
        .arg(manifest_path.as_ref())
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .output()
        .map_err(BuildError::IOError)?;
    if output.status.success() {
        let parsed = serde_json::from_slice::<serde_json::Value>(&output.stdout)
            .map_err(|_| BuildError::CargoTargetDirectoryResolutionError)?;
        let target_directory = parsed
            .as_object()
            .and_then(|o| o.get("target_directory"))
            .and_then(|o| o.as_str())
            .ok_or(BuildError::CargoTargetDirectoryResolutionError)?;
        Ok(target_directory.to_owned())
    } else {
        Err(BuildError::CargoFailure(output.status))
    }
}

/// Builds a package.
pub fn build_package<P: AsRef<Path>>(
    base_path: P,
    trace: bool,
    force_local_target: bool,
) -> Result<(PathBuf, PathBuf), BuildError> {
    let base_path = base_path.as_ref().to_owned();

    let mut manifest_path = base_path.clone();
    manifest_path.push("Cargo.toml");

    if !manifest_path.exists() {
        return Err(BuildError::NotCargoPackage(manifest_path));
    }

    // Use the scrypto directory as a target, even if the scrypto crate is part of a workspace
    // This allows us to find where the WASM and SCHEMA ends up deterministically.
    let target_path = if force_local_target {
        let mut target_path = base_path.clone();
        target_path.push("target");
        target_path
    } else {
        PathBuf::from_str(&get_default_target_directory(&manifest_path)?).unwrap()
        // Infallible
    };

    let mut out_path = target_path.clone();
    out_path.push("wasm32-unknown-unknown");
    out_path.push("release");

    // Build with SCHEMA
    run_cargo_build(&manifest_path, &target_path, trace, false)?;

    // Find the binary paths
    let manifest = Manifest::from_path(&manifest_path)
        .map_err(|_| BuildError::InvalidManifestFile(manifest_path.clone()))?;
    let mut wasm_name = None;
    if let Some(lib) = manifest.lib {
        wasm_name = lib.name.clone();
    }
    if wasm_name == None {
        if let Some(pkg) = manifest.package {
            wasm_name = Some(pkg.name.replace("-", "_"));
        }
    }
    let mut bin_path = out_path.clone();
    bin_path.push(wasm_name.ok_or(BuildError::InvalidManifestFile(manifest_path.clone()))?);

    let wasm_path = bin_path.with_extension("wasm");
    let schema_path = bin_path.with_extension("schema");

    // Extract SCHEMA
    let wasm =
        fs::read(&wasm_path).map_err(|err| BuildError::IOErrorAtPath(err, wasm_path.clone()))?;
    let schema = extract_schema(&wasm).map_err(BuildError::SchemaExtractionError)?;
    fs::write(
        &schema_path,
        scrypto_encode(&schema).map_err(BuildError::SchemaEncodeError)?,
    )
    .map_err(|err| BuildError::IOErrorAtPath(err, schema_path.clone()))?;

    // Build without SCHEMA
    run_cargo_build(&manifest_path, &target_path, trace, true)?;

    Ok((wasm_path, schema_path))
}

/// Runs tests within a package.
pub fn test_package<P: AsRef<Path>, I, S>(path: P, args: I) -> Result<(), TestError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    build_package(&path, false, false).map_err(TestError::BuildError)?;

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

        if status.success() {
            Ok(())
        } else {
            Err(FormatError::CargoFailure(status))
        }
    } else {
        Ok(())
    }
}
