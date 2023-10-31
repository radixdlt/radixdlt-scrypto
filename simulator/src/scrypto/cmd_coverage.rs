use clap::Parser;
use radix_engine_interface::types::Level;
use regex::Regex;
use std::env;
use std::env::current_dir;
use std::fs;
use std::io::Read;
use std::io::Write;
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

    /// The package directory
    #[clap(long)]
    path: Option<PathBuf>,
}

impl Coverage {
    fn check_command_availability(command: String) -> Result<(), Error> {
        if Command::new(&command).arg("--version").output().is_err() {
            println!("Missing command: {}. Please install LLVM version matching rustc LLVM version, which is {}.", 
                command, command.split('-').last().unwrap_or("Unknown"));
            println!("For more information, check https://apt.llvm.org/");
            return Err(Error::CoverageError(CoverageError::MissingLLVM));
        }
        Ok(())
    }

    pub fn run(&self) -> Result<(), Error> {
        let output = Command::new("rustc")
            .arg("--version")
            .arg("--verbose")
            .output()
            .expect("Failed to execute rustc command");

        let output_str = String::from_utf8(output.stdout).expect("Failed to read rustc output");
        let is_nightly = output_str.contains("nightly");
        let llvm_major_version = Regex::new(r"LLVM version: (\d+)")
            .unwrap()
            .captures(&output_str)
            .and_then(|cap| cap.get(1).map(|m| m.as_str()))
            .expect("Failed to read LLVM version of rustc");

        if !is_nightly {
            println!("Coverage tool requries nightly version of rust toolchain");
            println!("You can install it by using the following commands:");
            println!("rustup default nightly");
            println!("rustup target add wasm32-unknown-unknown --toolchain=nightly");
            return Err(Error::CoverageError(CoverageError::IncorrectRustVersion));
        }

        // Verify that all llvm tools required to generate coverage report are installed
        Self::check_command_availability(format!("clang-{}", llvm_major_version))?;
        Self::check_command_availability(format!("llvm-cov-{}", llvm_major_version))?;
        Self::check_command_availability(format!("llvm-profdata-{}", llvm_major_version))?;

        // Build package
        let path = self.path.clone().unwrap_or(current_dir().unwrap());
        let (wasm_path, _) = build_package(&path, false, false, true, Level::Trace, true)
            .map_err(Error::BuildError)?;
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
        env::set_var("COVERAGE_DIRECTORY", data_path.to_str().unwrap());

        // Run tests
        test_package(path, self.arguments.clone(), true)
            .map(|_| ())
            .map_err(Error::TestError)?;

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
            println!("No .profraw files found in the coverage/data directory");
            return Err(Error::CoverageError(CoverageError::NoProfrawFiles));
        }

        let profdata_path = data_path.join("coverage.profdata");
        let output = Command::new(format!("llvm-profdata-{}", llvm_major_version))
            .args(&["merge", "-sparse"])
            .args(profraw_files)
            .arg("-o")
            .arg(profdata_path.to_str().unwrap())
            .output()
            .expect("Failed to execute llvm-profdata command");
        if !output.status.success() {
            eprintln!(
                "llvm-profdata failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(Error::CoverageError(CoverageError::ProfdataMergeFailed));
        }

        // Generate object file from intermediate representation (.ll) file
        let ir_path = wasm_path.with_extension("ll");
        let ir_path = ir_path
            .parent()
            .unwrap()
            .join("deps")
            .join(ir_path.file_name().unwrap());

        let mut ir_contents = String::new();
        fs::File::open(&ir_path)
            .expect(&format!("Failed to open IR file {ir_path:?}"))
            .read_to_string(&mut ir_contents)
            .expect("Failed to read IR file");

        let modified_ir_contents = Regex::new(r"(?ms)^(define[^\n]*\n).*?^}\s*$")
            .unwrap()
            .replace_all(&ir_contents, "${1}start:\n  unreachable\n}\n")
            .to_string();

        let new_ir_path = data_path.join(ir_path.file_name().unwrap());
        let mut new_ir_file =
            fs::File::create(&new_ir_path).expect("Failed to create modified IR file");
        new_ir_file
            .write_all(modified_ir_contents.as_bytes())
            .expect("Failed to write modified IR file");

        // Generate Object File from IR File
        let object_file_path = new_ir_path.with_extension("o");
        let output = Command::new(format!("clang-{}", llvm_major_version))
            .args(&[
                new_ir_path.to_str().unwrap(),
                "-Wno-override-module",
                "-c",
                "-o",
            ])
            .arg(object_file_path.to_str().unwrap())
            .output()
            .expect("Failed to execute clang command");

        if !output.status.success() {
            eprintln!("clang failed: {}", String::from_utf8_lossy(&output.stderr));
            return Err(Error::CoverageError(CoverageError::ClangFailed));
        }

        // Generate Coverage Report
        let coverage_report_path = coverage_path.join("report");
        if coverage_report_path.exists() {
            fs::remove_dir_all(&coverage_report_path).unwrap();
        }

        let output = Command::new(format!("llvm-cov-{}", llvm_major_version))
            .args(&["show", "--instr-profile=coverage/data/coverage.profdata"])
            .arg(object_file_path.to_str().unwrap())
            .args(&[
                "--show-instantiations=false",
                "--format=html",
                "--output-dir",
            ])
            .arg(coverage_report_path.to_str().unwrap())
            .output()
            .expect("Failed to execute llvm-cov command");

        if !output.status.success() {
            eprintln!(
                "llvm-cov failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(Error::CoverageError(CoverageError::LlvmCovFailed));
        }

        println!("Coverage report was succesfully generated, it is available in {coverage_report_path:?} directory.");

        Ok(())
    }
}
