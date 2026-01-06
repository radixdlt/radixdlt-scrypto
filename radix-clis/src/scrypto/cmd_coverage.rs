//! Core assumptions made in this file:
//!
//! 1. That all of the WASM files compiled for coverage are built using the `nightly` toolchain.
//! 2. That the user doesn't have control over which `nightly` toolchain to use and we will just use
//!    the `nightly` toolchain available on the system.
//! 3. That we're always building the packages for the `wasm32-unknown-unknown` target.
//! 4. That we're always using the `release` profile for all of the coverage builds.
//! 5. That the user already has `clang`, `llvm-cov`, and `llvm-profdata` installed on their local
//!    machine and available in the path.

use cargo_metadata::MetadataCommand;
use cargo_metadata::Package;
use clap::Parser;
use radix_engine_interface::types::Level;
use regex::Regex;
use sbor::prelude::*;
use scrypto_compiler::is_scrypto_cargo_locked_env_var_active;
use scrypto_compiler::RustFlags;
use scrypto_compiler::ScryptoCompiler;
use scrypto_compiler::DEFAULT_ENVIRONMENT_VARIABLES;
use std::env::current_dir;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::string::FromUtf8Error;
use std::sync::LazyLock;
use walkdir::WalkDir;

use crate::utils::*;

/// Run Scrypto tests and generate code coverage report
#[derive(Parser, Debug)]
pub struct Coverage {
    /// The arguments to be passed to the test executable.
    ///
    /// Note that these arguments will not be passed to the compilation process, only to the test
    /// executable.
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
    pub fn run(self) -> Result<(), CoverageError> {
        static LLVM_IR_CORRECTIONS_REGEX: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(?ms)^(define[^\n]*\n).*?^}\s*$").unwrap());

        // Constructing the paths to all of the required files based on the provided package path.
        let paths = Paths::new(self.path)?;

        // Initializing the llvm-toolchain that will be used to match the current version used for
        // the nightly compiler.
        let llvm_toolchain = LLVMToolchain::new()?;

        // Building the package with WASM instrumentation.
        let build_environment_variables = construct_build_environment_variables();
        ScryptoCompiler::builder()
            .manifest_path(paths.manifest_path.as_path())
            .log_level(Level::Trace)
            .optimize_with_wasm_opt(None)
            .target_directory(paths.coverage_dir_path.as_path())
            .envs(build_environment_variables)
            .coverage()
            .compile()
            .map_err(BuildError::ScryptoCompilerError)
            .map_err(CoverageError::BuildError)?;

        // Reinitializing the directories that require reinitialization.
        paths.reinitialize_required_directories()?;

        // Running the tests on the package, this will generate the profraw files for us.
        test_package(
            paths.package_directory_path.as_path(),
            self.arguments.clone(),
            true,
            is_scrypto_cargo_locked_env_var_active() || self.locked,
            indexmap! {
                "COVERAGE_DIRECTORY" => paths.coverage_data_dir_path.as_path()
            },
        )
        .map_err(CoverageError::TestError)?;

        // Reading the LLVM-IR file of the compiled package and applying the necessary corrections
        // to it.
        let llvm_ir_file_pre_correction_contents =
            std::fs::read_to_string(paths.llvm_ir_pre_corrections_file_path.as_path())?;
        let llvm_ir_file_post_correction_contents = LLVM_IR_CORRECTIONS_REGEX
            .replace_all(
                llvm_ir_file_pre_correction_contents.as_str(),
                "${1}start:\n  unreachable\n}\n",
            )
            .to_string();
        std::fs::write(
            paths.llvm_ir_post_corrections_file_path.as_path(),
            llvm_ir_file_post_correction_contents,
        )?;

        // Converting the corrected LLVM-IR into an object file through clang.
        let object_file_conversion_output = llvm_toolchain
            .new_clang_command()
            .arg(paths.llvm_ir_post_corrections_file_path.as_path())
            .arg("-Wno-override-module")
            .arg("-c")
            .arg("-o")
            .arg(paths.object_file_path.as_path())
            .arg("--target=aarch64-unknown-linux-gnu")
            .output()
            .map_err(CoverageError::CommandFailedToRun)?;
        if !object_file_conversion_output.status.success() {
            let error = String::from_utf8_lossy(&object_file_conversion_output.stderr);
            eprintln!("clang failed: {}", error);
            return Err(CoverageError::ClangFailed(error.to_string()));
        }

        // Merging all of the profraw files into a profdata file.
        let profraw_files_iterator = WalkDir::new(paths.coverage_data_dir_path.as_path())
            .into_iter()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.into_path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("profraw"))
            });
        let profraw_merge_output = llvm_toolchain
            .new_llvm_profdata_command()
            .arg("merge")
            .arg("-sparse")
            .args(profraw_files_iterator)
            .arg("-o")
            .arg(paths.profdata_file_path.as_path())
            .output()
            .map_err(CoverageError::CommandFailedToRun)?;
        if !profraw_merge_output.status.success() {
            let error = String::from_utf8_lossy(&profraw_merge_output.stderr);
            eprintln!("clang failed: {}", error);
            return Err(CoverageError::LlvmProfdataFailed(error.to_string()));
        }

        // Generating the final report based on the object file and the profdata that was generated.
        let report_generation_output = llvm_toolchain
            .new_llvm_cov_command()
            .arg("show")
            .arg("--instr-profile")
            .arg(paths.profdata_file_path.as_path())
            .arg(paths.object_file_path.as_path())
            .arg("--show-instantiations=false")
            .arg("--format=html")
            .arg("--output-dir")
            .arg(paths.report_dir_path.as_path())
            .arg("-sources")
            .arg(paths.package_directory_path.as_path())
            .output()
            .map_err(CoverageError::CommandFailedToRun)?;
        if !report_generation_output.status.success() {
            let error = String::from_utf8_lossy(&report_generation_output.stderr);
            eprintln!("clang failed: {}", error);
            return Err(CoverageError::LlvmCovFailed(error.to_string()));
        }

        Ok(())
    }
}

/// A struct that contains all of the paths, filenames, and information on the build artifacts. Note
/// that all of the paths contained in this struct are canonicalized and do not require the user of
/// the struct to do it again.
///
/// # Note
///
/// You should never construct this struct yourself. You should always construct it through the
/// [`Paths::new`] function which performs the required checks to ensure that all paths are correct.
///
/// This struct makes the assumption (when appropriate) that the nightly compiler is used since we
/// always make use of it for coverage and it also assumes a release profile. There is no reason for
/// us to turn those into arguments since this struct is EXCLUSIVELY used in this coverage module
/// and we can safely make assumptions about how other parts of the code will act.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Paths {
    /// The path of the package that we're doing a coverage report for.
    pub package_directory_path: PathBuf,
    /// The path of the `Cargo.toml` manifest of package that we're doing a coverage report for.
    pub manifest_path: PathBuf,
    /// The name of the package that we're doing a coverage report for. We're disallowing workspaces
    /// at this moment of time and assuming that there will only be a single compiled WASM.
    pub package_name: String,

    /// The path of the target directory used for the compilations of all of the artifacts.
    pub target_dir_path: PathBuf,
    /// The path of the coverage directory within the target directory that contains everything
    /// related to coverage.
    pub coverage_dir_path: PathBuf,
    /// The path of the coverage data directory within the target directory that contains everything
    /// related to coverage.
    pub coverage_data_dir_path: PathBuf,
    /// The path of the directory that contains the final generated coverage report.
    pub report_dir_path: PathBuf,
    /// The path of the output directory that contains all of the build artifacts.
    pub build_artifacts_dir_path: PathBuf,

    /// The file name that's used for all of the artifacts built from this package. This is the same
    /// name as the package but with the `-` replaced with `_`.
    pub file_name: String,

    /// The file name of the final WASM file that doesn't contain the schema.
    pub wasm_file_name: String,
    /// The file path of the final WASM file that doesn't contain the schema.
    pub wasm_file_path: PathBuf,

    /// The file name of the final WASM file that does contain the schema.
    pub wasm_with_schema_file_name: String,
    /// The file path of the final WASM file that does contain the schema.
    pub wasm_with_schema_file_path: PathBuf,

    /// The file name of the package definition file.
    pub rpd_file_name: String,
    /// The file path of the package definition file.
    pub rpd_file_path: PathBuf,

    /// The file name of the LLVM-IR file.
    pub llvm_ir_file_name: String,
    /// The file path of the LLVM-IR file before applying the corrections.
    pub llvm_ir_pre_corrections_file_path: PathBuf,
    /// The file path of the LLVM-IR file after applying the corrections.
    pub llvm_ir_post_corrections_file_path: PathBuf,

    /// The file name of the object file.
    pub object_file_name: String,
    /// The file path of the object file.
    pub object_file_path: PathBuf,

    /// The file name of the profdata file.
    pub profdata_file_name: String,
    /// The file path of the profdata file.
    pub profdata_file_path: PathBuf,
}

impl Paths {
    pub fn new(user_provided_path: Option<PathBuf>) -> Result<Self, CoverageError> {
        // We either use the user provided path or the current directory path. Error out of neither
        // path exists.
        let package_directory_path = user_provided_path
            .or(current_dir().ok())
            .ok_or(CoverageError::FailedToResolvePackagePath)?
            .canonicalize()
            .map_err(|_| CoverageError::FailedToResolvePackagePath)?;
        let manifest_path = assert_path_exists(package_directory_path.join("Cargo.toml"))?;

        // Getting the package name through the `cargo metadata` command.
        let metadata = MetadataCommand::new()
            .manifest_path(manifest_path.as_path())
            .no_deps()
            .exec()
            .map_err(CoverageError::CargoMetadataError)?;
        let package_name = match metadata.packages.as_slice() {
            [Package { name, .. }] => Ok(name.as_str()),
            [] => Err(CoverageError::NoPackagesFound),
            [..] => Err(CoverageError::WorkspacesNotPermitted),
        }?
        .to_owned();
        let file_name = package_name.replace('-', "_");

        // Creating the paths of the target directory
        let target_dir_path = package_directory_path.join("target");
        let coverage_dir_path = target_dir_path.join("coverage");
        let report_dir_path = coverage_dir_path.join("report");
        let coverage_data_dir_path = coverage_dir_path.join("data");
        let build_artifacts_dir_path = coverage_dir_path
            .join("wasm32-unknown-unknown")
            .join("release");

        let wasm_file_name = format!("{file_name}.wasm");
        let wasm_file_path = build_artifacts_dir_path.join(wasm_file_name.clone());

        let wasm_with_schema_file_name = format!("{file_name}_with_schema.wasm");
        let wasm_with_schema_file_path =
            build_artifacts_dir_path.join(wasm_with_schema_file_name.clone());

        let rpd_file_name = format!("{file_name}.rpd");
        let rpd_file_path = build_artifacts_dir_path.join(rpd_file_name.clone());

        let llvm_ir_file_name = format!("{file_name}.ll");
        let llvm_ir_pre_corrections_file_path = build_artifacts_dir_path
            .join("deps")
            .join(llvm_ir_file_name.clone());
        let llvm_ir_post_corrections_file_path =
            build_artifacts_dir_path.join(llvm_ir_file_name.clone());

        let object_file_name = format!("{file_name}.o");
        let object_file_path = build_artifacts_dir_path.join(object_file_name.clone());

        let profdata_file_name = format!("{file_name}.profdata");
        let profdata_file_path = coverage_data_dir_path.join(profdata_file_name.clone());

        Ok(Self {
            package_directory_path,
            manifest_path,
            package_name,
            target_dir_path,
            coverage_dir_path,
            coverage_data_dir_path,
            report_dir_path,
            build_artifacts_dir_path,
            file_name,
            wasm_file_name,
            wasm_file_path,
            wasm_with_schema_file_name,
            wasm_with_schema_file_path,
            rpd_file_name,
            rpd_file_path,
            llvm_ir_file_name,
            llvm_ir_pre_corrections_file_path,
            llvm_ir_post_corrections_file_path,
            object_file_name,
            object_file_path,
            profdata_file_name,
            profdata_file_path,
        })
    }

    /// Reinitializes any directory that requires re-initialization.
    pub fn reinitialize_required_directories(&self) -> Result<(), CoverageError> {
        let directory_path = self.coverage_data_dir_path.as_path();
        let _ = std::fs::remove_dir_all(directory_path);
        std::fs::create_dir(directory_path)?;
        Ok(())
    }
}

/// A struct that contains the paths and helper methods for the tools from the LLVM Toolchain that
/// we will be using.
///
/// # Note
///
/// This struct makes the same set of assumptions made in this module.
///
/// Do not manually construct this struct and only construct it through the [`new`] function on the
/// struct to perform all of the required checks.
///
/// [`new`]: LLVMToolchain::new
struct LLVMToolchain {
    /// The path of the `clang` binary used by the selected version of the rust compiler.
    clang_path: PathBuf,
    /// The path of the `llvm-profdata` binary used by the selected version of the rust compiler.
    llvm_profdata_path: PathBuf,
    /// The path of the `llvm-cov` binary used by the selected version of the rust compiler.
    llvm_cov_path: PathBuf,
}

impl LLVMToolchain {
    pub fn new() -> Result<Self, CoverageError> {
        static VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"(?m)^LLVM version: (?<major>\d+)\.(?<minor>\d+)\.(?<patch>\d+)$").unwrap()
        });

        // We need to use the same version of LLVM that `rustc` is configured to use.
        let output = new_nightly_command("rustc")
            .arg("-vV")
            .stdout(Stdio::piped())
            .spawn()
            .map_err(CoverageError::CommandFailedToRun)?
            .wait_with_output()
            .map_err(CoverageError::CommandFailedToRun)?;
        let stdout_string = String::from_utf8(output.stdout)?;

        // Intentional unwraps: we rely on the `rustc -vV` to output the version in a specific way
        // and we don't have a way to recover if it doesn't produce the versions in the way that we
        // expect.
        let llvm_major_version = VERSION_REGEX
            .captures(&stdout_string)
            .expect("Can't fail")
            .name("major")
            .expect("Can't fail")
            .as_str()
            .parse::<usize>()
            .expect("Can't fail");

        Ok(Self {
            clang_path: select_llvm_command(
                ["clang".to_string(), format!("clang-{llvm_major_version}")],
                llvm_major_version,
            )?
            .into(),
            llvm_profdata_path: select_llvm_command(
                [
                    "llvm-profdata".to_string(),
                    format!("llvm-profdata-{llvm_major_version}"),
                ],
                llvm_major_version,
            )?
            .into(),
            llvm_cov_path: select_llvm_command(
                [
                    "llvm-cov".to_string(),
                    format!("llvm-cov-{llvm_major_version}"),
                ],
                llvm_major_version,
            )?
            .into(),
        })
    }

    /// Creates a new [`Command`] that calls `clang` at the path configured in this struct.
    pub fn new_clang_command(&self) -> Command {
        let mut cmd = Command::new(self.clang_path.as_path());
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        cmd
    }

    /// Creates a new [`Command`] that calls `llvm-profdata` at the path configured in this struct.
    pub fn new_llvm_profdata_command(&self) -> Command {
        let mut cmd = Command::new(self.llvm_profdata_path.as_path());
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        cmd
    }

    /// Creates a new [`Command`] that calls `llvm-cov` at the path configured in this struct.
    pub fn new_llvm_cov_command(&self) -> Command {
        let mut cmd = Command::new(self.llvm_cov_path.as_path());
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        cmd
    }
}

/// The error type used in coverage.
#[derive(Debug, thiserror::Error)]
pub enum CoverageError {
    /// We've been unable to resolve the path of the package. It could be due to the user providing
    /// an invalid path or due to us not being able to get the current working directory or due to
    /// us failing to canonicalize the path of the package.
    #[error("Resolution of the package path failed.")]
    FailedToResolvePackagePath,

    /// One of the paths that are required for coverage was checked for and it doesn't exist.
    #[error("This path doesn't exist but it must exist for the coverage tool to work: {0:?}")]
    PathDoesntExist(PathBuf),

    /// An error occurred when we tried to get the `cargo metadata` of the package.
    #[error("Encountered an error when trying to get the cargo metadata for the package: {0}")]
    CargoMetadataError(#[from] cargo_metadata::Error),

    /// Found multiple packages when we got the metadata leading us to know that this is a workspace
    /// which is currently not permitted.
    #[error("The provided package is a workspace which we don't currently support")]
    WorkspacesNotPermitted,

    /// Could not find any packages when we got the `cargo metadata` and therefore there is nothing
    /// that we can perform.
    #[error("The provided directory doesn't contain any packages")]
    NoPackagesFound,

    /// We ran a command but it failed to run or failed during waiting.
    #[error("Command failed to run: {0:?}")]
    CommandFailedToRun(std::io::Error),

    /// One of the commands produced stdout output that wasn't valid utf-8
    #[error(
        "The data the the command produced on stdout is not a valid utf-8, decoding failed: {0:?}"
    )]
    StdoutIsNotValidUtf8(#[from] FromUtf8Error),

    /// One of the commands that we look for were not found in the system.
    #[error("A command with the following permitted aliases was not found in the system. Is it available in $PATH?")]
    CommandNotFound(Vec<String>),

    /// An error was encountered when trying to build the package.
    #[error("An error was encountered when trying to build the package: {0:?}")]
    BuildError(BuildError),

    /// An error was encountered when trying to test the package.
    #[error("An error was encountered when trying to test the package: {0:?}")]
    TestError(TestError),

    /// A generic IO error.
    #[error("An IO error was encountered: {0:?}")]
    IoError(#[from] std::io::Error),

    /// An error was encountered when running the clang command
    #[error("An error was encountered when running the clang command: {0:?}")]
    ClangFailed(String),

    /// An error was encountered when running the llvm-profdata command
    #[error("An error was encountered when running the llvm-profdata command: {0:?}")]
    LlvmProfdataFailed(String),

    /// An error was encountered when running the llvm-cov command
    #[error("An error was encountered when running the llvm-cov command: {0:?}")]
    LlvmCovFailed(String),
}

/// Check if a path exists or not. If it does then it's returned, otherwise, an error is returned.
fn assert_path_exists<P: AsRef<Path>>(path: P) -> Result<P, CoverageError> {
    if path.as_ref().exists() {
        Ok(path)
    } else {
        Err(CoverageError::PathDoesntExist(path.as_ref().to_path_buf()))
    }
}

/// Creates a new [`Command`] that uses the nightly compiler by setting the `RUSTUP_TOOLCHAIN`
/// environment variable. This should be used for all of the commands that we run to ensure that we
/// are always making use of the same compiler.
fn new_nightly_command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    command.env("RUSTUP_TOOLCHAIN", "nightly");
    command
}

/// A helper method that goes through a list of llvm commands and finds the first one with
/// the same LLVM major version. This is used to allow us to accept `clang` or `clang-21`
/// and not force the user to have the postfixed commands installed.
fn select_llvm_command<P: AsRef<OsStr>>(
    commands: impl IntoIterator<Item = P> + Clone,
    llvm_major_version: usize,
) -> Result<P, CoverageError> {
    let match_string = format!("version {llvm_major_version}");
    for command in commands.clone() {
        let Ok(output) = new_nightly_command(command.as_ref())
            .arg("--version")
            .stdout(Stdio::piped())
            .spawn()
            .map_err(CoverageError::CommandFailedToRun)
            .and_then(|child| {
                child
                    .wait_with_output()
                    .map_err(CoverageError::CommandFailedToRun)
            })
        else {
            continue;
        };
        let Ok(stdout_string) = String::from_utf8(output.stdout) else {
            continue;
        };
        if stdout_string.contains(match_string.as_str()) {
            return Ok(command);
        }
    }
    Err(CoverageError::CommandNotFound(
        commands
            .into_iter()
            .map(|os_str| os_str.as_ref().to_string_lossy().to_string())
            .collect(),
    ))
}

/// Constructs a map of the environment variables that will be used to build the package with
/// instrumentation.
fn construct_build_environment_variables() -> IndexMap<String, String> {
    let mut environment_variables = DEFAULT_ENVIRONMENT_VARIABLES
        .clone()
        .into_iter()
        .flat_map(|(k, v)| v.into_set().map(|v| (k, v)))
        .collect::<IndexMap<_, _>>();
    let rust_flags = RustFlags::for_scrypto_compilation()
        .with_flag("-Clto=off")
        .with_flag("-Cinstrument-coverage")
        .with_flag("-Zno-profiler-runtime")
        .with_flag("--emit=llvm-ir")
        .with_flag("-Zlocation-detail=none");
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
    environment_variables.insert("RUSTUP_TOOLCHAIN".to_owned(), "nightly".to_owned());
    environment_variables
}
