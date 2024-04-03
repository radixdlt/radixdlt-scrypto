use crate::prelude::*;
use scrypto_compiler::*;
use std::path::Path;

pub struct Compile;

impl Compile {
    pub fn compile<P: AsRef<Path>>(package_dir: P) -> (Vec<u8>, PackageDefinition) {
        Self::compile_with_env_vars(
            package_dir,
            btreemap! {
                "RUSTFLAGS".to_owned() => "".to_owned(),
                "CARGO_ENCODED_RUSTFLAGS".to_owned() => "".to_owned(),
            },
        )
    }

    // required for compile-blueprints-at-build-time feature in radix-engine-tests
    pub fn compile_with_env_vars<P: AsRef<Path>>(
        package_dir: P,
        env_vars: sbor::rust::collections::BTreeMap<String, String>,
    ) -> (Vec<u8>, PackageDefinition) {
        // Initialize compiler
        let mut compiler_builder = ScryptoCompiler::builder();
        compiler_builder
            .manifest_path(package_dir.as_ref())
            .env("RUSTFLAGS", EnvironmentVariableAction::Set("".into()))
            .env(
                "CARGO_ENCODED_RUSTFLAGS",
                EnvironmentVariableAction::Set("".into()),
            )
            .optimize_with_wasm_opt(None)
            .log_level(Level::Trace); // all logs from error to trace

        env_vars.iter().for_each(|(name, value)| {
            if value.is_empty() {
                compiler_builder.env(name, EnvironmentVariableAction::Unset);
            } else {
                compiler_builder.env(name, EnvironmentVariableAction::Set(value.clone()));
            }
        });

        #[cfg(feature = "coverage")]
        {
            compiler_builder.coverage();
        }

        let mut compiler = compiler_builder
            .build()
            .unwrap_or_else(|err| panic!("Failed to initialize Scrypto Compiler {:?}", err));

        #[cfg(feature = "coverage")]
        // Check if binary exists in coverage directory, if it doesn't only then build it
        {
            let mut coverage_path = compiler.target_binary_path();
            if coverage_path.is_file() {
                let code = fs::read(&coverage_path).unwrap_or_else(|err| {
                    panic!(
                        "Failed to read built WASM from path {:?} - {:?}",
                        &path, err
                    )
                });
                coverage_path.set_extension("rpd");
                let definition = fs::read(&coverage_path).unwrap_or_else(|err| {
                    panic!(
                        "Failed to read package definition from path {:?} - {:?}",
                        &coverage_path, err
                    )
                });
                let definition = manifest_decode(&definition).unwrap_or_else(|err| {
                    panic!(
                        "Failed to parse package definition from path {:?} - {:?}",
                        &coverage_path, err
                    )
                });
                return (code, definition);
            }
        };

        // Build
        let mut build_artifacts = compiler.compile().unwrap_or_else(|error| {
            match &error {
                ScryptoCompilerError::CargoBuildFailure(exit_code) => {
                    eprintln!("Package compilation error:\n{:?}", exit_code)
                }
                _ => (),
            }

            panic!(
                "Failed to compile package: {:?}, error: {:?}",
                package_dir.as_ref(),
                error
            );
        });

        if !build_artifacts.is_empty() {
            let build_artifact = build_artifacts.remove(0); // take first element
            (
                build_artifact.wasm.content,
                build_artifact.package_definition.content,
            )
        } else {
            panic!("Build artifacts list is empty: {:?}", package_dir.as_ref(),);
        }
    }
}
