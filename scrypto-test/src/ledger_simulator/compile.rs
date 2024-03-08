use crate::prelude::*;
use scrypto_compiler::*;
use std::path::Path;

pub struct Compile;

impl Compile {
    pub fn compile<P: AsRef<Path>>(package_dir: P) -> (Vec<u8>, PackageDefinition) {
        let coverage = false;
        #[cfg(feature = "coverage")]
        let coverage = true;

        let mut compiler = match ScryptoCompiler::new()
            .manifest_directory(package_dir.as_ref())
            .env("RUSTFLAGS", "")
            .env("CARGO_ENCODED_RUSTFLAGS", "")
            .coverage(coverage)
            .log_level(Level::Trace) // all logs from error to trace
            .build()
        {
            Ok(compiler) => compiler,
            Err(error) => panic!(
                "Failed to compile package: {:?}, error: {:?}",
                package_dir.as_ref(),
                error
            ),
        };

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

        let wasm_path = match compiler.compile() {
            Ok(wasm_path) => wasm_path,
            Err(error) => {
                match &error {
                    ScryptoCompilerError::CargoBuildFailure(stderr, _) => eprintln!(
                        "Package compilation error:\n{}",
                        std::str::from_utf8(&stderr).unwrap()
                    ),
                    _ => (),
                }

                panic!(
                    "Failed to compile package: {:?}, error: {:?}",
                    package_dir.as_ref(),
                    error
                );
            }
        };

        // Extract schema
        let code = std::fs::read(&wasm_path).unwrap_or_else(|err| {
            panic!(
                "Failed to read built WASM from path {:?} - {:?}",
                &wasm_path, err
            )
        });
        let definition = extract_definition(&code).unwrap();

        (code, definition)
    }
}
