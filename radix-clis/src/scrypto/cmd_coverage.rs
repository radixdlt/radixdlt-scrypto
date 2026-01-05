use clap::Parser;
use radix_engine_interface::types::Level;
use regex::Regex;
use sbor::prelude::IndexMap;
use scrypto_compiler::is_scrypto_cargo_locked_env_var_active;
use scrypto_compiler::RustFlags;
use scrypto_compiler::DEFAULT_ENVIRONMENT_VARIABLES;
use std::env;
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

use crate::scrypto::*;
use crate::utils::*;

/// Run Scrypto tests and generate code coverage report
#[derive(Parser, Debug)]
pub struct Coverage {
    /// The arguments to be passed to the test executable
    arguments: Vec<String>,

    /// Ensures the Cargo.lock file is used as-is. Equivalent to `cargo test --locked`.
    /// Alternatively, the `SCRYPTO_CARGO_LOCKED` environment variable can be used,
    /// which makes it easy to set universally in CI.
    #[clap(long)]
    locked: bool,

    /// The package directory
    #[clap(long)]
    path: Option<PathBuf>,
}

impl Coverage {
    fn check_wasm_target(nightly: bool) -> Result<(), Error> {
        let output = Command::new("rustup")
            .args(&["target", "list", "--installed"])
            .output()
            .expect("Failed to execute rustup target list command");

        let output_str = String::from_utf8(output.stdout).unwrap();
        let is_wasm_target_installed = output_str.contains("wasm32-unknown-unknown");

        if !is_wasm_target_installed {
            eprintln!(
                "The {} wasm32-unknown-unknown target is not installed.",
                if nightly { "nightly" } else { "" }
            );
            eprintln!("You can install it by using the following command:");
            eprintln!(
                "rustup target add wasm32-unknown-unknown{}",
                if nightly { " --toolchain=nightly" } else { "" }
            );
            Err(Error::CoverageError(CoverageError::MissingWasm32Target))
        } else {
            Ok(())
        }
    }

    fn check_rustc_version() -> (bool, String, String) {
        let output = Command::new("rustc")
            .args(&["--version", "--verbose"])
            .output()
            .expect("Failed to execute rustc command");

        let output_str = String::from_utf8(output.stdout).expect("Failed to read rustc output");
        let is_nightly = output_str.contains("nightly");
        let llvm_major_version = Regex::new(r"LLVM version: ([0-9]+)")
            .unwrap()
            .captures(&output_str)
            .and_then(|cap| cap.get(1).map(|m| m.as_str()))
            .map(String::from)
            .unwrap();

        let host_triple = Regex::new(r"(?m)^host: ([^\n]+)$")
            .unwrap()
            .captures(&output_str)
            .and_then(|cap| cap.get(1).map(|m| m.as_str()))
            .map(String::from)
            .unwrap();

        (is_nightly, llvm_major_version, host_triple)
    }

    fn check_command_availability(command: String) -> Result<(), Error> {
        if Command::new(&command).arg("--version").output().is_err() {
            eprintln!("Missing command: {}. Please install LLVM version matching rustc LLVM version, which is {}.",
                command, command.split('-').last().unwrap_or("Unknown"));
            eprintln!("For more information, check https://apt.llvm.org/");
            Err(Error::CoverageError(CoverageError::MissingLLVM))
        } else {
            Ok(())
        }
    }

    pub fn run(&self) -> Result<(), String> {
        // Verify rust version and wasm target
        Self::check_wasm_target(false)?;

        let (mut is_nightly, mut llvm_major_version, mut host_triple) = Self::check_rustc_version();
        let mut unset_rustup_toolchain = false;
        if !is_nightly {
            // Try to use nightly toolchain automatically
            env::set_var("RUSTUP_TOOLCHAIN", "nightly");
            (is_nightly, llvm_major_version, host_triple) = Self::check_rustc_version();
            if !is_nightly {
                eprintln!("Coverage tool requries nightly version of rust toolchain");
                eprintln!("You can install it by using the following commands:");
                eprintln!("rustup target add wasm32-unknown-unknown --toolchain=nightly");
                return Err(Error::CoverageError(CoverageError::IncorrectRustVersion).into());
            }
            Self::check_wasm_target(true)?;
            unset_rustup_toolchain = true;
        }

        // Verify that all llvm tools required to generate coverage report are installed
        Self::check_command_availability(format!("clang-{}", llvm_major_version))?;
        Self::check_command_availability(format!("llvm-cov-{}", llvm_major_version))?;
        Self::check_command_availability(format!("llvm-profdata-{}", llvm_major_version))?;

        // Build package
        let mut environment_variables = DEFAULT_ENVIRONMENT_VARIABLES
            .clone()
            .into_iter()
            .flat_map(|(k, v)| match v {
                scrypto_compiler::EnvironmentVariableAction::Set(v) => Some((k, v)),
                scrypto_compiler::EnvironmentVariableAction::Unset => None,
            })
            .collect::<IndexMap<_, _>>();
        let rust_flags = RustFlags::for_scrypto_compilation()
            .with_flag("-Clto=off")
            .with_flag("-Cdebuginfo=2")
            .with_flag("-Cinstrument-coverage")
            .with_flag("-Zno-profiler-runtime")
            .with_flag("--emit=llvm-ir")
            .with_flag("-Cstrip=none");
        for (env_var, cargo_encoding) in [
            ("RUSTFLAGS", false),
            ("CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS", false),
            ("CARGO_ENCODED_RUSTFLAGS", true),
        ] {
            let encoded_rust_flags = if cargo_encoding {
                rust_flags.encode_as_cargo_encoded_rust_flags()
            } else {
                rust_flags.encode_as_rust_flags()
            };
            environment_variables.insert(env_var.to_owned(), encoded_rust_flags);
        }

        let path = self.path.clone().unwrap_or(current_dir().unwrap());
        let build_artifacts = build_package(&path, true, Level::Trace, true, environment_variables)
            .map_err(Error::BuildError)?;
        if build_artifacts.len() > 1 {
            return Err(Error::BuildError(BuildError::WorkspaceNotSupported).into());
        }
        let (wasm_path, _) = build_artifacts
            .first()
            .ok_or(Error::BuildError(BuildError::BuildArtifactsEmpty))?
            .to_owned();

        assert!(wasm_path.is_file());

        // Remove wasm32-unknown-unknown/release/file.wasm from wasm_path
        let mut coverage_path = wasm_path.clone();
        coverage_path.pop();
        coverage_path.pop();
        coverage_path.pop();
        assert!(coverage_path.ends_with("coverage"));
        assert!(coverage_path.is_dir());

        // Remove "data" directory from coverage directory if it exists, then create it
        let data_path = coverage_path.join("data");
        if data_path.exists() {
            fs::remove_dir_all(&data_path).unwrap();
        }
        fs::create_dir_all(&data_path).unwrap();

        // Set enviromental variable COVERAGE_DIRECTORY
        env::set_var(
            "COVERAGE_DIRECTORY",
            fs::canonicalize(&data_path).unwrap().to_str().unwrap(),
        );

        // Run tests
        test_package(
            path,
            self.arguments.clone(),
            true,
            is_scrypto_cargo_locked_env_var_active() || self.locked,
        )
        .map(|_| ())
        .map_err(Error::TestError)?;

        // Generate object file from intermediate representation (.ll) file and link it to a final WASM module.
        //
        // `llvm-cov` needs coverage mapping embedded in the analyzed object/binary. Having `.profraw`/`.profdata`
        // data only proves counters exist; without the mapping (`.llvm_covmap`/`__llvm_covmap`) `llvm-cov` will
        // fail with "no coverage data found".
        //
        // Also, `llvm-cov` can't load coverage from a relocatable WASM object (it contains COMDAT/relocation
        // info). We must link it into a final WASM module first.
        let ir_path = wasm_path.with_extension("ll");
        let ir_path = ir_path
            .parent()
            .unwrap()
            .join("deps")
            .join(ir_path.file_name().unwrap());

        let object_file_path = data_path
            .join(wasm_path.file_stem().unwrap())
            .with_extension("o");
        let output = Command::new(format!("clang-{}", llvm_major_version))
            .args(&[
                "--target=wasm32-unknown-unknown",
                ir_path.to_str().unwrap(),
                "-Wno-override-module",
                "-c",
                "-o",
                object_file_path.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to execute clang command");

        if !output.status.success() {
            eprintln!("clang failed: {}", String::from_utf8_lossy(&output.stderr));
            return Err(Error::CoverageError(CoverageError::ClangFailed).into());
        }

        let rustc_sysroot = Command::new("rustc")
            .args(&["--print", "sysroot"])
            .output()
            .expect("Failed to execute rustc --print sysroot");
        let rustc_sysroot = String::from_utf8(rustc_sysroot.stdout)
            .expect("Failed to read rustc sysroot output")
            .trim()
            .to_owned();

        let rust_lld = PathBuf::from(rustc_sysroot)
            .join("lib")
            .join("rustlib")
            .join(&host_triple)
            .join("bin")
            .join(if cfg!(windows) {
                "rust-lld.exe"
            } else {
                "rust-lld"
            });

        if !rust_lld.is_file() {
            eprintln!(
                "Missing rust-lld at {rust_lld:?}. Ensure the linker is installed as part of your Rust toolchain."
            );
            return Err(Error::CoverageError(CoverageError::MissingRustLld).into());
        }

        let linked_wasm_path = data_path.join(format!(
            "{}_linked.wasm",
            wasm_path.file_stem().unwrap().to_string_lossy()
        ));
        let output = Command::new(&rust_lld)
            .args(&[
                "-flavor",
                "wasm",
                object_file_path.to_str().unwrap(),
                "--no-entry",
                "--export-all",
                "--allow-undefined",
                "-o",
                linked_wasm_path.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to execute rust-lld command");

        if !output.status.success() {
            eprintln!(
                "rust-lld failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(Error::CoverageError(CoverageError::RustLldFailed).into());
        }

        // Merge profraw files into profdata file
        let profraw_files: Vec<String> = WalkDir::new(&data_path)
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "profraw" {
                    Some(path.to_str()?.to_owned())
                } else {
                    None
                }
            })
            .collect();

        if profraw_files.is_empty() {
            eprintln!("No .profraw files found in the coverage/data directory");
            return Err(Error::CoverageError(CoverageError::NoProfrawFiles).into());
        }

        let profdata_path = data_path.join("coverage.profdata");
        let output = Command::new(format!("llvm-profdata-{}", llvm_major_version))
            .args(&["merge", "-sparse"])
            .args(profraw_files)
            .args(&["-o", profdata_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute llvm-profdata command");
        if !output.status.success() {
            eprintln!(
                "llvm-profdata failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(Error::CoverageError(CoverageError::ProfdataMergeFailed).into());
        }

        // Generate Coverage Report
        let coverage_report_path = coverage_path.join("report");
        if coverage_report_path.exists() {
            fs::remove_dir_all(&coverage_report_path).unwrap();
        }

        let output = Command::new(format!("llvm-cov-{}", llvm_major_version))
            .args(&[
                "show",
                "--instr-profile",
                profdata_path.to_str().unwrap(),
                linked_wasm_path.to_str().unwrap(),
                "--show-instantiations=false",
                "--format=html",
                "--output-dir",
                coverage_report_path.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to execute llvm-cov command");

        if !output.status.success() {
            eprintln!(
                "llvm-cov failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            eprintln!(
                "Hint: this usually means the analyzed object/binary is missing LLVM coverage mapping \
                (`.llvm_covmap`/`__llvm_covmap`), or it doesn't match the build that produced the profile."
            );
            return Err(Error::CoverageError(CoverageError::LlvmCovFailed).into());
        }

        println!("Coverage report was succesfully generated, it is available in {coverage_report_path:?} directory.");

        if unset_rustup_toolchain {
            env::remove_var("RUSTUP_TOOLCHAIN");
        }

        Ok(())
    }
}
