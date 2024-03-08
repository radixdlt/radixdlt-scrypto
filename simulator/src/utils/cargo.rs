use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

use radix_engine::utils::*;
use radix_engine_common::prelude::*;
use radix_engine_interface::types::Level;
use scrypto_compiler::*;

#[derive(Debug)]
pub enum BuildError {
    ScryptoCompilerError(ScryptoCompilerError),

    IOErrorAtPath(io::Error, PathBuf),

    SchemaExtractionError(ExtractSchemaError),

    SchemaEncodeError(sbor::EncodeError),
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

/// Builds a package.
pub fn build_package<P: AsRef<Path>>(
    base_path: P,
    trace: bool,
    force_local_target: bool,
    disable_wasm_opt: bool,
    log_level: Level,
    coverage: bool,
) -> Result<(PathBuf, PathBuf), BuildError> {
    // Build with schema
    let wasm_path = ScryptoCompiler::new()
        .manifest_directory(base_path.as_ref())
        .force_local_target(force_local_target)
        .trace(trace)
        .log_level(log_level)
        .no_schema(false)
        .compile()
        .map_err(|e| BuildError::ScryptoCompilerError(e))?;

    let definition_path = wasm_path.with_extension("rpd");

    // Extract SCHEMA
    let wasm =
        fs::read(&wasm_path).map_err(|err| BuildError::IOErrorAtPath(err, wasm_path.clone()))?;
    let definition = extract_definition(&wasm).map_err(BuildError::SchemaExtractionError)?;
    fs::write(
        &definition_path,
        manifest_encode(&definition).map_err(BuildError::SchemaEncodeError)?,
    )
    .map_err(|err| BuildError::IOErrorAtPath(err, definition_path.clone()))?;

    // Build without SCHEMA
    let mut compiler = ScryptoCompiler::new();
    compiler
        .manifest_directory(base_path.as_ref())
        .force_local_target(force_local_target)
        .trace(trace)
        .no_schema(true)
        .log_level(log_level)
        .coverage(coverage);

    // Optimizes the built wasm using Binaryen's wasm-opt tool. The code that follows is equivalent
    // to running the following commands in the CLI:
    // wasm-opt -0z --strip-debug --strip-dwarf --strip-procedures $some_path $some_path
    if !disable_wasm_opt {
        compiler.optimize_with_wasm_opt(
            wasm_opt::OptimizationOptions::new_optimize_for_size_aggressively()
                .add_pass(wasm_opt::Pass::StripDebug)
                .add_pass(wasm_opt::Pass::StripDwarf)
                .add_pass(wasm_opt::Pass::StripProducers),
        );
    }

    let wasm_path = compiler
        .compile()
        .map_err(|e| BuildError::ScryptoCompilerError(e))?;

    Ok((wasm_path, definition_path))
}

/// Runs tests within a package.
pub fn test_package<P: AsRef<Path>, I, S>(path: P, args: I, coverage: bool) -> Result<(), TestError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    if !coverage {
        build_package(&path, false, false, false, Level::Trace, false)
            .map_err(TestError::BuildError)?;
    }

    let mut cargo = path.as_ref().to_owned();
    cargo.push("Cargo.toml");
    if cargo.exists() {
        let features = if coverage {
            vec!["--features", "scrypto-test/coverage"]
        } else {
            vec![]
        };
        let status = Command::new("cargo")
            .arg("test")
            .arg("--release")
            .arg("--manifest-path")
            .arg(cargo.to_str().unwrap())
            .args(features)
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
