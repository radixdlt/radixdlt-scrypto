use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

use radix_engine::utils::*;
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

    EnvParsingError,
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
    disable_wasm_opt: bool,
    log_level: Level,
    coverage: bool,
    env: &[(String, String)],
) -> Result<Vec<(PathBuf, PathBuf)>, BuildError> {
    let mut compiler_builder = ScryptoCompiler::builder();
    compiler_builder
        .manifest_path(base_path.as_ref())
        .log_level(log_level);

    if disable_wasm_opt {
        compiler_builder.optimize_with_wasm_opt(None);
    }
    if coverage {
        compiler_builder.coverage();

        let mut target_path = PathBuf::from(base_path.as_ref());
        target_path.push("coverage");
        compiler_builder.target_directory(target_path);
    }
    env.iter().for_each(|(name, value)| {
        compiler_builder.env(name, EnvironmentVariableAction::Set(value.clone()));
    });

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
pub fn test_package<P: AsRef<Path>, I, S>(
    path: P,
    args: I,
    coverage: bool,
    locked: bool,
) -> Result<(), TestError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    if !coverage {
        build_package(&path, false, Level::Trace, false, &[]).map_err(TestError::BuildError)?;
    }

    let mut cargo = path.as_ref().to_owned();
    cargo.push("Cargo.toml");
    if cargo.exists() {
        let status = Command::new("cargo")
            .arg("test")
            .arg("--release")
            .arg("--manifest-path")
            .arg(cargo.to_str().unwrap())
            .args({
                if coverage {
                    vec!["--features", "scrypto-test/coverage"]
                } else {
                    vec![]
                }
            })
            .args({
                if locked {
                    vec!["--locked"]
                } else {
                    vec![]
                }
            })
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
