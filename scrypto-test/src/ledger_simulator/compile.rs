use crate::prelude::*;
use scrypto_compiler::*;
use std::path::Path;

pub enum CompileProfile {
    /// Uses default compilation options - same as `scrypto build`. Should be used in all cases which requires
    /// compilation results to be as close to production as possible (for instance costing related tests).
    Standard,
    /// Same as Standard with enabled all logs from error to trace level.
    StandardWithTraceLogs,
    /// Disables WASM optimization to speed-up compilation process, by default used by SDK PackageFactory.
    Fast,
    /// Disables WASM optimization and enables all logs from error to trace level, by default used by Ledger Simulator.
    FastWithTraceLogs,
}

pub struct Compile;

impl Compile {
    pub fn compile<P: AsRef<Path>>(
        package_dir: P,
        compile_profile: CompileProfile,
    ) -> (Vec<u8>, PackageDefinition) {
        Self::compile_with_env_vars(
            package_dir,
            btreemap! {
                "RUSTFLAGS".to_owned() => "".to_owned(),
                "CARGO_ENCODED_RUSTFLAGS".to_owned() => "".to_owned(),
            },
            compile_profile,
            true,
        )
    }

    // required for compile-blueprints-at-build-time feature in radix-engine-tests
    pub fn compile_with_env_vars<P: AsRef<Path>>(
        package_dir: P,
        env_vars: sbor::rust::collections::BTreeMap<String, String>,
        compile_profile: CompileProfile,
        _use_coverage: bool,
    ) -> (Vec<u8>, PackageDefinition) {
        // Initialize compiler
        let mut compiler_builder = ScryptoCompiler::builder();
        compiler_builder.manifest_path(package_dir.as_ref());

        match compile_profile {
            CompileProfile::Standard => (),
            CompileProfile::StandardWithTraceLogs => {
                compiler_builder.log_level(Level::Trace); // all logs from error to trace
            }
            CompileProfile::Fast => {
                compiler_builder.optimize_with_wasm_opt(None);
            }
            CompileProfile::FastWithTraceLogs => {
                compiler_builder.optimize_with_wasm_opt(None);
                compiler_builder.log_level(Level::Trace); // all logs from error to trace
            }
        }

        env_vars.iter().for_each(|(name, value)| {
            compiler_builder.env(name, EnvironmentVariableAction::Set(value.clone()));
        });

        #[cfg(feature = "coverage")]
        if _use_coverage {
            compiler_builder.coverage();

            let mut coverage_dir = std::path::PathBuf::from(package_dir.as_ref());
            coverage_dir.push("coverage");
            compiler_builder.target_directory(coverage_dir);
        }

        let mut compiler = compiler_builder
            .build()
            .unwrap_or_else(|err| panic!("Failed to initialize Scrypto Compiler {:?}", err));

        #[cfg(feature = "coverage")]
        // Check if binary exists in coverage directory, if it doesn't only then build it
        if _use_coverage {
            let manifest = compiler.get_main_manifest_definition();
            if manifest.target_output_binary_rpd_path.exists()
                && manifest.target_phase_2_build_wasm_output_path.exists()
            {
                let code = std::fs::read(&manifest.target_phase_2_build_wasm_output_path)
                    .unwrap_or_else(|err| {
                        panic!(
                            "Failed to read built WASM from path {:?} - {:?}",
                            &manifest.target_phase_2_build_wasm_output_path, err
                        )
                    });
                let definition = std::fs::read(&manifest.target_output_binary_rpd_path)
                    .unwrap_or_else(|err| {
                        panic!(
                            "Failed to read package definition from path {:?} - {:?}",
                            &manifest.target_output_binary_rpd_path, err
                        )
                    });
                let definition = manifest_decode(&definition).unwrap_or_else(|err| {
                    panic!(
                        "Failed to parse package definition from path {:?} - {:?}",
                        &manifest.target_output_binary_rpd_path, err
                    )
                });
                return (code, definition);
            }
        }

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

#[cfg(test)]
mod tests {
    use super::{Compile, CompileProfile};
    use std::process::Command;

    fn compile_blueprint(additional_args: &[&str]) -> Vec<u8> {
        // Build `scrypto` cli
        Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--bin")
            .arg("scrypto")
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../radix-clis"))
            .output()
            .inspect_err(|e| println!("Scrypto cli build failed: {}", e))
            .unwrap();

        // Run `scrypto build` for example blueprit
        Command::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../radix-clis/target/release/scrypto"
        ))
        .arg("build")
        .args(additional_args)
        .current_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/blueprints/tuple-return"
        ))
        .output()
        .inspect_err(|e| println!("Blueprint compilation falied: {}", e))
        .unwrap();

        let output_file = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/blueprints/target/wasm32-unknown-unknown/release/tuple_return.wasm"
        );
        std::fs::read(output_file)
            .inspect_err(|e| println!("Failed to load file: {}, error: {}", output_file, e))
            .unwrap()
    }

    #[test]
    fn validate_compile_profile_standard() {
        // Compile blueprint using `scrypto compile` command
        let output_file_content = compile_blueprint(&[]);

        // Compile same blueprint using Compile object
        let (bin, _) = Compile::compile(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/blueprints/tuple-return"),
            CompileProfile::Standard,
        );

        // Assert
        assert_eq!(
            output_file_content.len(),
            bin.len(),
            "Wasm files should have same size."
        );
        assert_eq!(
            output_file_content, bin,
            "Wasm files should have same content."
        )
    }

    #[test]
    fn validate_compile_profile_standard_with_logs() {
        // Compile blueprint using `scrypto compile` command
        let output_file_content = compile_blueprint(&[]);

        // Compile same blueprint using Compile object
        let (bin, _) = Compile::compile(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/blueprints/tuple-return"),
            CompileProfile::StandardWithTraceLogs,
        );

        // Assert
        assert!(
            output_file_content.len() < bin.len(),
            "Size of Wasm file compiled by `scrypto build` command should be smaller."
        );
    }

    #[test]
    fn validate_compile_profile_fast() {
        // Compile blueprint using `scrypto compile` command
        let output_file_content = compile_blueprint(&[]);

        // Compile same blueprint using Compile object
        let (bin, _) = Compile::compile(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/blueprints/tuple-return"),
            CompileProfile::Fast,
        );

        // Assert
        assert!(
            output_file_content.len() < bin.len(),
            "Size of Wasm file compiled by `scrypto build` command should be smaller."
        );
    }

    #[test]
    fn validate_compile_profile_fast_with_logs() {
        // Compile blueprint using `scrypto compile` command
        let output_file_content = compile_blueprint(&[]);

        // Compile same blueprint using Compile object
        let (bin, _) = Compile::compile(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/blueprints/tuple-return"),
            CompileProfile::FastWithTraceLogs,
        );

        // Assert
        assert!(
            output_file_content.len() < bin.len(),
            "Size of Wasm file compiled by `scrypto build` command should be smaller."
        );
    }

    #[test]
    fn verify_scrypto_build_with_args_for_compile_profile_standard_with_logs() {
        // Compile blueprint using `scrypto compile` command
        let output_file_content = compile_blueprint(&["--log-level", "TRACE"]);

        // Compile same blueprint using Compile object
        let (bin, _) = Compile::compile(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/blueprints/tuple-return"),
            CompileProfile::StandardWithTraceLogs,
        );

        // Assert
        assert_eq!(
            output_file_content.len(),
            bin.len(),
            "Wasm files should have same size."
        );
        assert_eq!(
            output_file_content, bin,
            "Wasm files should have same content."
        )
    }

    #[test]
    fn verify_scrypto_build_with_args_for_compile_profile_fast() {
        // Compile blueprint using `scrypto compile` command
        let output_file_content = compile_blueprint(&["--disable-wasm-opt"]);

        // Compile same blueprint using Compile object
        let (bin, _) = Compile::compile(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/blueprints/tuple-return"),
            CompileProfile::Fast,
        );

        // Assert
        assert_eq!(
            output_file_content.len(),
            bin.len(),
            "Wasm files should have same size."
        );
        assert_eq!(
            output_file_content, bin,
            "Wasm files should have same content."
        )
    }

    #[test]
    fn verify_scrypto_build_with_args_for_compile_profile_fast_with_logs() {
        // Compile blueprint using `scrypto compile` command
        let output_file_content =
            compile_blueprint(&["--disable-wasm-opt", "--log-level", "TRACE"]);

        // Compile same blueprint using Compile object
        let (bin, _) = Compile::compile(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/blueprints/tuple-return"),
            CompileProfile::FastWithTraceLogs,
        );

        // Assert
        assert_eq!(
            output_file_content.len(),
            bin.len(),
            "Wasm files should have same size."
        );
        assert_eq!(
            output_file_content, bin,
            "Wasm files should have same content."
        )
    }
}
