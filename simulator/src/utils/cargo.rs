use std::ffi::OsStr;
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

    BuildArtifactsEmpty,

    WorkspaceNotSupported,
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
    _coverage: bool,
    features: &[String],
    env_variables: &[String],
    packages: &[String],
) -> Result<Vec<(PathBuf, PathBuf)>, BuildError> {
    let env_variables_decoded: Vec<Vec<&str>> = env_variables
        .iter()
        .map(|env| env.split('=').collect::<Vec<&str>>())
        .collect();

    let mut compiler_builder = ScryptoCompiler::builder();
    compiler_builder
        .manifest_path(base_path.as_ref())
        .log_level(log_level);
    if trace {
        compiler_builder.scrypto_macro_trace();
    }
    if force_local_target {
        compiler_builder.target_directory("./");
    }
    features.iter().for_each(|f| {
        compiler_builder.feature(f);
    });
    packages.iter().for_each(|p| {
        compiler_builder.package(p);
    });
    env_variables_decoded.iter().for_each(|v| {
        if v.len() == 1 {
            compiler_builder.env(v[0], EnvironmentVariableAction::Set("".into()));
        } else if v.len() == 2 {
            compiler_builder.env(v[0], EnvironmentVariableAction::Set(v[1].into()));
        }
    });

    // Optimizes the built wasm using Binaryen's wasm-opt tool. The code that follows is equivalent
    // to running the following commands in the CLI:
    // wasm-opt -0z --strip-debug --strip-dwarf --strip-procedures $some_path $some_path
    if !disable_wasm_opt {
        compiler_builder.optimize_with_wasm_opt(
            wasm_opt::OptimizationOptions::new_optimize_for_size_aggressively()
                .add_pass(wasm_opt::Pass::StripDebug)
                .add_pass(wasm_opt::Pass::StripDwarf)
                .add_pass(wasm_opt::Pass::StripProducers),
        );
    }

    let build_results = compiler_builder
        .compile()
        .map_err(|e| BuildError::ScryptoCompilerError(e))?;

    Ok(build_results
        .iter()
        .map(|item| {
            (
                item.wasm.path.to_owned(),
                item.package_definition.path.to_owned(),
            )
        })
        .collect())
}

/// Runs tests within a package.
pub fn test_package<P: AsRef<Path>, I, S>(path: P, args: I, coverage: bool) -> Result<(), TestError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    if !coverage {
        build_package(
            &path,
            false,
            false,
            false,
            Level::Trace,
            false,
            &vec![],
            &vec![],
            &vec![],
        )
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
