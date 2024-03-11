use crate::prelude::*;
use scrypto_compiler::*;
use std::path::Path;

pub struct Compile;

impl Compile {
    pub fn compile<P: AsRef<Path>>(package_dir: P) -> (Vec<u8>, PackageDefinition) {
        let coverage = false;
        #[cfg(feature = "coverage")]
        let coverage = true;

        // Initialize compiler
        let mut compiler = ScryptoCompiler::new()
            .manifest_directory(package_dir.as_ref())
            .env("RUSTFLAGS", EnvironmentVariableAction::Set("".into()))
            .env(
                "CARGO_ENCODED_RUSTFLAGS",
                EnvironmentVariableAction::Set("".into()),
            )
            .coverage(coverage)
            .log_level(Level::Trace) // all logs from error to trace
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
        let wasm_path = compiler.compile().unwrap_or_else(|error| {
            match &error {
                ScryptoCompilerError::CargoBuildFailure(stderr, _) => {
                    eprintln!("Package compilation error:\n{}", stderr)
                }
                _ => (),
            }

            panic!(
                "Failed to compile package: {:?}, error: {:?}",
                package_dir.as_ref(),
                error
            );
        });

        // Extract schema
        compiler.extract_schema_from_wasm().unwrap_or_else(|err| {
            panic!(
                "Failed to extract schema from WASM from path {:?} - {:?}",
                &wasm_path, err
            )
        })
    }
}
