use cargo_toml::Manifest;
use fslock::{LockFile, ToOsStr};
use radix_common::prelude::*;
use radix_engine::utils::{extract_definition, ExtractSchemaError};
use radix_engine_interface::{blueprints::package::PackageDefinition, types::Level};
use radix_rust::prelude::{IndexMap, IndexSet};
use std::cmp::Ordering;
use std::error::Error;
use std::iter;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::{env, io};

const MANIFEST_FILE: &str = "Cargo.toml";
const BUILD_TARGET: &str = "wasm32-unknown-unknown";
const SCRYPTO_NO_SCHEMA: &str = "scrypto/no-schema";
const SCRYPTO_COVERAGE: &str = "scrypto/coverage";

// Radix Engine supports WASM MVP + proposals: mmutable-globals and sign-extension-ops
// (see radix-engine/src/vm/wasm.prepare.rs)
// More on CFLAGS for WASM:  https://clang.llvm.org/docs/ClangCommandLineReference.html#webassembly
const TARGET_CLAGS_FOR_WASM: &str = "-mcpu=mvp -mmutable-globals -msign-ext";

#[derive(Debug)]
pub enum ScryptoCompilerError {
    /// Returns IO Error which occurred during compilation and optional context information.
    IOError(io::Error, Option<String>),
    /// Returns IO Error which occurred during compilation, path of a file related to that fail and
    /// optional context information.
    IOErrorWithPath(io::Error, PathBuf, Option<String>),
    /// Returns process exit status in case of 'cargo build' fail.
    CargoBuildFailure(ExitStatus),
    /// Returns `cargo metadata` command stderr output, path to Cargo.toml for which cargo metadata
    /// command failed and process exit status.
    CargoMetadataFailure(String, PathBuf, ExitStatus),
    /// Returns path to Cargo.toml for which results of cargo metadata command is not not valid json
    /// or target directory field is missing.
    CargoTargetDirectoryResolutionError(String),
    /// Compiler is unable to generate target binary file name.
    CargoTargetBinaryResolutionError,
    /// Returns path to Cargo.toml which was failed to load.
    CargoManifestLoadFailure(String),
    /// Returns path to Cargo.toml which cannot be found.
    CargoManifestFileNotFound(String),
    /// Provided package ID is not a member of the workspace.
    CargoWrongPackageId(String),
    /// Returns WASM Optimization error.
    WasmOptimizationError(wasm_opt::OptimizationError),
    /// Returns error occured during schema extraction.
    SchemaExtractionError(ExtractSchemaError),
    /// Returns error occured during schema encoding.
    SchemaEncodeError(EncodeError),
    /// Returns error occured during schema decoding.
    SchemaDecodeError(DecodeError),
    /// Returned when trying to compile workspace without any scrypto packages.
    NothingToCompile,
}

#[derive(Debug, Clone)]
pub struct ScryptoCompilerInputParams {
    /// Path to Cargo.toml file, if not specified current directory will be used.
    pub manifest_path: Option<PathBuf>,
    /// Path to directory where compilation artifacts are stored, if not specified default location will by used.
    pub target_directory: Option<PathBuf>,
    /// Compilation profile. If not specified default profile: Release will be used.
    pub profile: Profile,
    /// List of environment variables to set or unset during compilation.
    /// By default it includes compilation flags for C libraries to configure WASM with the same
    /// features as Radix Engine.
    /// TARGET_CFLAGS="-mcpu=mvp -mmutable-globals -msign-ext"
    pub environment_variables: IndexMap<String, EnvironmentVariableAction>,
    /// List of features, used for 'cargo build --features'. Optional field.
    pub features: IndexSet<String>,
    /// If set to true then '--no-default-features' option is passed to 'cargo build'. The default value is false.
    pub no_default_features: bool,
    /// If set to true then '--all-features' option is passed to 'cargo build'. The default value is false.
    pub all_features: bool,
    /// List of packages to compile, used for 'cargo build --package'. Optional field.
    pub package: IndexSet<String>,
    /// If set to true then '--locked' option is passed to 'cargo build', which enforces using the `Cargo.lock` file without changes. The default value is false.
    pub locked: bool,
    /// If set, the `SCRYPTO_CARGO_LOCKED` environment variable is ignored.
    /// This is useful for unit tests in this repo, which need to run successfully independent of this setting.
    /// Defaults to false.
    pub ignore_locked_env_var: bool,
    /// List of custom options, passed as 'cargo build' arguments without any modifications. Optional field.
    /// Add each option as separate entry (for instance: '-j 1' must be added as two entires: '-j' and '1' one by one).
    pub custom_options: IndexSet<String>,
    /// If specified optimizes the built wasm using Binaryen's wasm-opt tool.
    /// Default configuration is equivalent to running the following commands in the CLI:
    /// wasm-opt -0z --strip-debug --strip-dwarf --strip-producers --dce $some_path $some_path
    pub wasm_optimization: Option<wasm_opt::OptimizationOptions>,
    /// If set to true then compiler informs about the compilation progress
    pub verbose: bool,
}
impl Default for ScryptoCompilerInputParams {
    /// Definition of default `ScryptoCompiler` configuration.
    fn default() -> Self {
        let wasm_optimization = Some(
            wasm_opt::OptimizationOptions::new_optimize_for_size_aggressively()
                .add_pass(wasm_opt::Pass::StripDebug)
                .add_pass(wasm_opt::Pass::StripDwarf)
                .add_pass(wasm_opt::Pass::StripProducers)
                .add_pass(wasm_opt::Pass::Dce)
                .to_owned(),
        );
        let mut ret = Self {
            manifest_path: None,
            target_directory: None,
            profile: Profile::Release,
            environment_variables: indexmap!(
                "TARGET_CFLAGS".to_string() =>
                EnvironmentVariableAction::Set(
                    TARGET_CLAGS_FOR_WASM.to_string()
                )
            ),
            features: indexset!(),
            no_default_features: false,
            all_features: false,
            package: indexset!(),
            custom_options: indexset!(),
            ignore_locked_env_var: false,
            locked: false,
            wasm_optimization,
            verbose: false,
        };
        // Apply default log level features
        ret.features
            .extend(Self::log_level_to_scrypto_features(Level::default()).into_iter());
        ret
    }
}
impl ScryptoCompilerInputParams {
    pub fn log_level_to_scrypto_features(log_level: Level) -> Vec<String> {
        let mut ret = Vec::new();
        if Level::Error <= log_level {
            ret.push(String::from("scrypto/log-error"));
        }
        if Level::Warn <= log_level {
            ret.push(String::from("scrypto/log-warn"));
        }
        if Level::Info <= log_level {
            ret.push(String::from("scrypto/log-info"));
        }
        if Level::Debug <= log_level {
            ret.push(String::from("scrypto/log-debug"));
        }
        if Level::Trace <= log_level {
            ret.push(String::from("scrypto/log-trace"));
        }
        ret
    }
}

#[derive(Debug, Default, Clone)]
pub enum Profile {
    #[default]
    Release,
    Debug,
    Test,
    Bench,
    Custom(String),
}
impl Profile {
    fn as_command_args(&self) -> Vec<String> {
        vec![
            String::from("--profile"),
            match self {
                Profile::Release => String::from("release"),
                Profile::Debug => String::from("dev"),
                Profile::Test => String::from("test"),
                Profile::Bench => String::from("bench"),
                Profile::Custom(name) => name.clone(),
            },
        ]
    }
    fn as_target_directory_name(&self) -> String {
        match self {
            Profile::Release => String::from("release"),
            Profile::Debug => String::from("debug"),
            Profile::Test => String::from("debug"),
            Profile::Bench => String::from("release"),
            Profile::Custom(name) => name.clone(),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ParseProfileError;
impl fmt::Display for ParseProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}
impl Error for ParseProfileError {}

impl FromStr for Profile {
    type Err = ParseProfileError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "release" => Ok(Profile::Release),
            "debug" => Ok(Profile::Debug),
            "test" => Ok(Profile::Test),
            "bench" => Ok(Profile::Bench),
            other => {
                if other.contains(' ') {
                    Err(ParseProfileError)
                } else {
                    Ok(Profile::Custom(other.to_string()))
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum EnvironmentVariableAction {
    Set(String),
    Unset,
}

#[derive(Debug, Clone)]
pub struct BuildArtifacts {
    pub wasm: BuildArtifact<Vec<u8>>,
    pub package_definition: BuildArtifact<PackageDefinition>,
}

#[derive(Debug, Clone)]
pub struct BuildArtifact<T> {
    pub path: PathBuf,
    pub content: T,
}

#[derive(Debug, Clone)]
pub struct CompilerManifestDefinition {
    /// Path to Cargo.toml file.
    pub manifest_path: PathBuf,
    /// Path to directory where compilation artifacts are stored.
    pub target_directory: PathBuf,
    /// Target binary name
    pub target_binary_name: String,
    /// Path to target binary WASM file from phase 1.
    pub target_phase_1_build_wasm_output_path: PathBuf,
    /// Path to target binary WASM file from phase 2.
    pub target_phase_2_build_wasm_output_path: PathBuf,
    /// Path to target binary RPD file.
    pub target_output_binary_rpd_path: PathBuf,
    /// Path to target binary WASM file with schema.
    pub target_copied_wasm_with_schema_path: PathBuf,
}

// Helper enum to unify different iterator types
enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Iterator for Either<L, R>
where
    L: Iterator,
    R: Iterator<Item = L::Item>,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(iter) => iter.next(),
            Either::Right(iter) => iter.next(),
        }
    }
}

/// Programmatic implementation of Scrypto compiler which is a wrapper around rust cargo tool.
/// To create an instance of `ScryptoCompiler` use `builder()` constructor which implements builder pattern,
/// provide any required parameter @see `ScryptoCompilerInputParams` and finally call `compile()` function.
/// `ScryptoCompiler` supports worspace compilation by providing workspace manifest as `manifest_path` parameter of
/// running compiler from directory containg workspace Cargo.toml file. Only packages with defined metadata group:
/// [package.metadata.scrypto] will be used during workspace compilation (so workspace manifest can contain also non
/// Scrypto packages). Alternativelly packages for workspace compilation can be provided in `package` input parameter,
/// metadata is not validated in that case.
/// Compilation results consists of list of `BuildArtifacts` which contains generated WASM file path and its content
/// and path to RPD file with package definition and `PackageDefinition` struct.
pub struct ScryptoCompiler {
    /// Scrypto compiler input parameters.
    input_params: ScryptoCompilerInputParams,
    /// Manifest definition used in 'cargo build' command calls. For workspace compilation this is a workspace manifest,
    /// for non-workspace compilation it is particular project manifest.
    /// 'cargo build' command will automatically build all workspace members for workspace compilation.
    main_manifest: CompilerManifestDefinition,
    /// List of manifest definitions in workspace compilation.
    manifests: Vec<CompilerManifestDefinition>,
}

#[derive(Debug)]
struct PackageLock {
    pub path: PathBuf,
    pub lock: LockFile,
}

impl PartialEq for PackageLock {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for PackageLock {}

impl Ord for PackageLock {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialOrd for PackageLock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PackageLock {
    fn new(path: PathBuf) -> Result<Self, ScryptoCompilerError> {
        let os_path = path.to_os_str().map_err(|err| {
            ScryptoCompilerError::IOErrorWithPath(
                err,
                path.clone(),
                Some(String::from("Convert lock file path to &str failed")),
            )
        })?;

        let lock = LockFile::open(&os_path).map_err(|err| {
            ScryptoCompilerError::IOErrorWithPath(
                err,
                path.clone(),
                Some(String::from("Open file for locking failed")),
            )
        })?;

        Ok(Self { path, lock })
    }

    fn is_locked(&self) -> bool {
        self.lock.owns_lock()
    }

    fn try_lock(&mut self) -> Result<bool, ScryptoCompilerError> {
        self.lock.try_lock().map_err(|err| {
            ScryptoCompilerError::IOErrorWithPath(
                err,
                self.path.clone(),
                Some(String::from("Lock file failed")),
            )
        })
    }

    fn unlock(&mut self) -> Result<(), ScryptoCompilerError> {
        self.lock.unlock().map_err(|err| {
            ScryptoCompilerError::IOErrorWithPath(
                err,
                self.path.clone(),
                Some(String::from("Unlock file failed")),
            )
        })
    }
}

impl ScryptoCompiler {
    pub fn builder() -> ScryptoCompilerBuilder {
        ScryptoCompilerBuilder::default()
    }

    // Internal constructor
    fn from_input_params(
        input_params: &mut ScryptoCompilerInputParams,
    ) -> Result<Self, ScryptoCompilerError> {
        let manifest_path = Self::get_manifest_path(&input_params.manifest_path)?;

        // If compiling workspace use only packages which defines [package.metadata.scrypto]
        // or only specified packages with --package parameter
        if let Some(workspace_members) = ScryptoCompiler::is_manifest_workspace(&manifest_path)? {
            // Verify if provided package names belongs to this workspace
            if !input_params.package.is_empty() {
                let wrong_packages: Vec<_> = input_params
                    .package
                    .iter()
                    .filter(|package| {
                        workspace_members
                            .iter()
                            .find(|(_, member_package_name, _)| &member_package_name == package)
                            .is_none()
                    })
                    .collect();
                if let Some(package) = wrong_packages.first() {
                    return Err(ScryptoCompilerError::CargoWrongPackageId(
                        package.to_string(),
                    ));
                }
            } else {
                input_params.package = workspace_members
                    .iter()
                    .filter_map(|(_, package, scrypto_metadata)| {
                        if scrypto_metadata.is_some() {
                            Some(package.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                if input_params.package.is_empty() {
                    return Err(ScryptoCompilerError::NothingToCompile);
                }
            }

            let manifests = workspace_members
                .into_iter()
                .filter_map(|(member_manifest_input_path, package, _)| {
                    if input_params.package.contains(&package) {
                        Some(
                            match ScryptoCompiler::get_manifest_path(&Some(
                                member_manifest_input_path,
                            )) {
                                Ok(member_manifest_path) => ScryptoCompiler::prepare_manifest_def(
                                    input_params,
                                    &member_manifest_path,
                                ),
                                Err(x) => Err(x),
                            },
                        )
                    } else {
                        None
                    }
                })
                .collect::<Result<Vec<CompilerManifestDefinition>, ScryptoCompilerError>>()?;

            Ok(Self {
                input_params: input_params.to_owned(),
                main_manifest: ScryptoCompiler::prepare_manifest_def(input_params, &manifest_path)?,
                manifests,
            })
        } else {
            Ok(Self {
                input_params: input_params.to_owned(),
                main_manifest: ScryptoCompiler::prepare_manifest_def(input_params, &manifest_path)?,
                manifests: Vec::new(),
            })
        }
    }

    // Generates target paths basing on manifest path
    fn prepare_manifest_def(
        input_params: &ScryptoCompilerInputParams,
        manifest_path: &Path,
    ) -> Result<CompilerManifestDefinition, ScryptoCompilerError> {
        ScryptoCompiler::prepare_paths_for_manifest(input_params, manifest_path)
    }

    fn get_default_target_directory(manifest_path: &Path) -> Result<String, ScryptoCompilerError> {
        let output = Command::new("cargo")
            .arg("metadata")
            .arg("--manifest-path")
            .arg(manifest_path)
            .arg("--format-version")
            .arg("1")
            .arg("--no-deps")
            .output()
            .map_err(|e| {
                ScryptoCompilerError::IOErrorWithPath(
                    e,
                    manifest_path.to_path_buf(),
                    Some(String::from("Cargo metadata for manifest failed.")),
                )
            })?;
        if output.status.success() {
            let parsed =
                serde_json::from_slice::<serde_json::Value>(&output.stdout).map_err(|_| {
                    ScryptoCompilerError::CargoTargetDirectoryResolutionError(
                        manifest_path.display().to_string(),
                    )
                })?;
            let target_directory = parsed
                .as_object()
                .and_then(|o| o.get("target_directory"))
                .and_then(|o| o.as_str())
                .ok_or(ScryptoCompilerError::CargoTargetDirectoryResolutionError(
                    manifest_path.display().to_string(),
                ))?;
            Ok(target_directory.to_owned())
        } else {
            Err(ScryptoCompilerError::CargoMetadataFailure(
                String::from_utf8_lossy(&output.stderr).to_string(),
                manifest_path.to_path_buf(),
                output.status,
            ))
        }
    }

    // Returns path to Cargo.toml (including the file)
    fn get_manifest_path(
        input_manifest_path: &Option<PathBuf>,
    ) -> Result<PathBuf, ScryptoCompilerError> {
        let manifest_path = match input_manifest_path.clone() {
            Some(mut path) => {
                if !path.ends_with(MANIFEST_FILE) {
                    path.push(MANIFEST_FILE);
                }
                path
            }
            None => {
                let mut path = env::current_dir().map_err(|e| {
                    ScryptoCompilerError::IOError(
                        e,
                        Some(String::from("Getting current directory failed.")),
                    )
                })?;
                path.push(MANIFEST_FILE);
                path
            }
        };

        if !manifest_path.exists() {
            Err(ScryptoCompilerError::CargoManifestFileNotFound(
                manifest_path.display().to_string(),
            ))
        } else {
            Ok(manifest_path)
        }
    }

    // If manifest is a workspace this function returns non-empty vector of tuple with workspace members (path),
    // package name and package scrypto metadata (content of section from Cargo.toml [package.metadata.scrypto]).
    fn is_manifest_workspace(
        manifest_path: &Path,
    ) -> Result<Option<Vec<(PathBuf, String, Option<cargo_toml::Value>)>>, ScryptoCompilerError>
    {
        let manifest = Manifest::from_path(&manifest_path).map_err(|_| {
            ScryptoCompilerError::CargoManifestLoadFailure(manifest_path.display().to_string())
        })?;
        if let Some(workspace) = manifest.workspace {
            if workspace.members.is_empty() {
                Ok(None)
            } else {
                Ok(Some(
                    workspace
                        .members
                        .iter()
                        .map(|i| {
                            let mut member_manifest_input_path = manifest_path.to_path_buf();
                            member_manifest_input_path.pop(); // Workspace Cargo.toml file
                            member_manifest_input_path.push(PathBuf::from(i));
                            member_manifest_input_path.push(MANIFEST_FILE); // Manifest Cargo.toml file

                            match Manifest::from_path(&member_manifest_input_path) {
                                Ok(manifest) => {
                                    let metadata = match &manifest.package().metadata {
                                        Some(cargo_toml::Value::Table(map)) => {
                                            map.get("scrypto").cloned()
                                        }
                                        _ => None,
                                    };
                                    Ok((
                                        member_manifest_input_path,
                                        manifest.package().name().to_string(),
                                        metadata,
                                    ))
                                }
                                Err(_) => Err(ScryptoCompilerError::CargoManifestLoadFailure(
                                    member_manifest_input_path.display().to_string(),
                                )),
                            }
                        })
                        .collect::<Result<Vec<_>, ScryptoCompilerError>>()?,
                ))
            }
        } else {
            Ok(None)
        }
    }

    fn get_target_binary_name(
        manifest_path: &Path,
    ) -> Result<Option<String>, ScryptoCompilerError> {
        // Find the binary name
        let manifest = Manifest::from_path(&manifest_path).map_err(|_| {
            ScryptoCompilerError::CargoManifestLoadFailure(manifest_path.display().to_string())
        })?;
        if let Some(w) = manifest.workspace {
            if !w.members.is_empty() {
                // For workspace compilation there is no binary file for the main manifest
                return Ok(None);
            }
        }
        let mut wasm_name = None;
        if let Some(lib) = manifest.lib {
            wasm_name = lib.name.clone();
        }
        if wasm_name.is_none() {
            if let Some(pkg) = manifest.package {
                wasm_name = Some(pkg.name.replace("-", "_"));
            }
        }
        Ok(Some(wasm_name.ok_or(
            ScryptoCompilerError::CargoTargetBinaryResolutionError,
        )?))
    }

    // Basing on manifest path returns target directory, target binary WASM path and target binary PRD path
    fn prepare_paths_for_manifest(
        input_params: &ScryptoCompilerInputParams,
        manifest_path: &Path,
    ) -> Result<CompilerManifestDefinition, ScryptoCompilerError> {
        // Generate target directory
        let target_directory = if let Some(directory) = &input_params.target_directory {
            // If target directory is explicitly specified as compiler parameter then use it as is
            PathBuf::from(directory)
        } else {
            // If target directory is not specified as compiler parameter then get default
            // target directory basing on manifest file
            PathBuf::from(&Self::get_default_target_directory(&manifest_path)?)
        };

        let definition = if let Some(target_binary_name) =
            Self::get_target_binary_name(&manifest_path)?
        {
            // First in phase 1, we build the package with schema extract facilities
            // This has to be built in the release profile
            let mut target_phase_1_build_wasm_output_path = target_directory.clone();
            target_phase_1_build_wasm_output_path.push(BUILD_TARGET);
            target_phase_1_build_wasm_output_path.push(Profile::Release.as_target_directory_name());
            target_phase_1_build_wasm_output_path.push(target_binary_name.clone());
            target_phase_1_build_wasm_output_path.set_extension("wasm");

            let mut target_copied_wasm_with_schema_path = target_directory.clone();
            target_copied_wasm_with_schema_path.push(BUILD_TARGET);
            target_copied_wasm_with_schema_path.push(Profile::Release.as_target_directory_name());
            target_copied_wasm_with_schema_path
                .push(format!("{}_with_schema", target_binary_name.clone()));
            target_copied_wasm_with_schema_path.set_extension("wasm");

            // In phase 2, we build the package in the requested profile
            let mut target_phase_2_build_wasm_output_path = target_directory.clone();
            target_phase_2_build_wasm_output_path.push(BUILD_TARGET);
            target_phase_2_build_wasm_output_path
                .push(input_params.profile.as_target_directory_name());
            target_phase_2_build_wasm_output_path.push(target_binary_name.clone());
            target_phase_2_build_wasm_output_path.set_extension("wasm");

            // We output the rpd in the target profile
            let mut target_output_binary_rpd_path = target_directory.clone();
            target_output_binary_rpd_path.push(BUILD_TARGET);
            target_output_binary_rpd_path.push(input_params.profile.as_target_directory_name());
            target_output_binary_rpd_path.push(target_binary_name.clone());
            target_output_binary_rpd_path.set_extension("rpd");

            CompilerManifestDefinition {
                manifest_path: manifest_path.to_path_buf(),
                target_directory,
                target_binary_name,
                target_phase_1_build_wasm_output_path,
                target_phase_2_build_wasm_output_path,
                target_output_binary_rpd_path,
                target_copied_wasm_with_schema_path,
            }
        } else {
            CompilerManifestDefinition {
                manifest_path: manifest_path.to_path_buf(),
                target_directory,
                // for workspace compilation these paths are empty
                target_binary_name: String::new(),
                target_phase_1_build_wasm_output_path: PathBuf::new(),
                target_phase_2_build_wasm_output_path: PathBuf::new(),
                target_output_binary_rpd_path: PathBuf::new(),
                target_copied_wasm_with_schema_path: PathBuf::new(),
            }
        };

        Ok(definition)
    }

    // Prepares OS command arguments
    fn prepare_command(&mut self, command: &mut Command, for_package_extract: bool) {
        let mut features: Vec<[&str; 2]> = self
            .input_params
            .features
            .iter()
            .map(|f| ["--features", f])
            .collect();
        if let Some(idx) = features
            .iter()
            .position(|[_tag, value]| *value == SCRYPTO_NO_SCHEMA)
        {
            if for_package_extract {
                features.remove(idx);
            }
        } else if !for_package_extract {
            features.push(["--features", SCRYPTO_NO_SCHEMA]);
        }

        let mut remove_cargo_rustflags_env = false;
        if for_package_extract {
            if let Some(idx) = features
                .iter()
                .position(|[_tag, value]| *value == SCRYPTO_COVERAGE)
            {
                // for schema extract 'scrypto/coverage' flag must be removed
                features.remove(idx);
                remove_cargo_rustflags_env = true;
            }
        }

        let features: Vec<&str> = features.into_iter().flatten().collect();

        let package: Vec<&str> = self
            .input_params
            .package
            .iter()
            .map(|p| ["--package", p])
            .flatten()
            .collect();

        command
            .arg("build")
            .arg("--target")
            .arg(BUILD_TARGET)
            .arg("--target-dir")
            .arg(&self.main_manifest.target_directory)
            .arg("--manifest-path")
            .arg(&self.main_manifest.manifest_path)
            .args(package)
            .args(features);

        if for_package_extract {
            // At package extract time, we have to use release mode, else we get an error
            // when running the WASM:
            // Err(SchemaExtractionError(InvalidWasm(TooManyFunctionLocals { max: 256, actual: 257 })))
            command.arg("--release");
        } else {
            command.args(self.input_params.profile.as_command_args());
        }

        if self.input_params.no_default_features {
            command.arg("--no-default-features");
        }
        if self.input_params.all_features {
            command.arg("--all_features");
        }

        // We support an environment variable to make it easy to turn `--locked` mode
        // on in CI, without having to rewrite all the code/plumbing.
        let force_locked =
            !self.input_params.ignore_locked_env_var && is_scrypto_cargo_locked_env_var_active();
        if force_locked || self.input_params.locked {
            command.arg("--locked");
        }

        self.input_params
            .environment_variables
            .iter()
            .for_each(|(name, action)| {
                match action {
                    EnvironmentVariableAction::Set(value) => {
                        // CARGO_ENCODED_RUSTFLAGS for coverage build must be removed for 1st phase compilation
                        if !(remove_cargo_rustflags_env && name == "CARGO_ENCODED_RUSTFLAGS") {
                            command.env(name, value);
                        }
                    }
                    EnvironmentVariableAction::Unset => {
                        command.env_remove(name);
                    }
                };
            });

        command.args(self.input_params.custom_options.iter());
    }

    fn wasm_optimize(&self, wasm_path: &Path) -> Result<(), ScryptoCompilerError> {
        if let Some(wasm_opt_config) = &self.input_params.wasm_optimization {
            if self.input_params.verbose {
                println!("Optimizing WASM {:?}", wasm_opt_config);
            }
            wasm_opt_config
                .run(wasm_path, wasm_path)
                .map_err(ScryptoCompilerError::WasmOptimizationError)
        } else {
            Ok(())
        }
    }

    // Create scrypto build lock file for each compiled package to protect compilation in case it is invoked multiple times in parallel.
    fn lock_packages(&self) -> Result<Vec<PackageLock>, ScryptoCompilerError> {
        let mut package_locks: Vec<PackageLock> = vec![];
        // Create target folder if it doesn't exist
        std::fs::create_dir_all(&self.main_manifest.target_directory).map_err(|err| {
            ScryptoCompilerError::IOErrorWithPath(
                err,
                self.main_manifest.target_directory.clone(),
                Some(String::from("Create target folder failed")),
            )
        })?;

        // Collect packages to be locked
        for package in self
            .iter_manifests()
            .map(|manifest| &manifest.target_binary_name)
        {
            let lock_file_path = self
                .main_manifest
                .target_directory
                .join(format!("{}.lock", package));
            let package_lock = PackageLock::new(lock_file_path)?;
            package_locks.push(package_lock);
        }
        package_locks.sort();

        let mut all_locked = false;
        // Attempt to lock all compiled packages.
        while !all_locked {
            all_locked = true;
            for package_lock in package_locks.iter_mut() {
                if !package_lock.is_locked() {
                    if !package_lock.try_lock()? {
                        all_locked = false;
                    }
                }
            }

            // Unlock if not all packages locked.
            // We need all packages to be locked at once to make sure
            // no other thread locked some package in the meantime.
            if !all_locked {
                for package_lock in package_locks.iter_mut() {
                    if package_lock.is_locked() {
                        package_lock.unlock()?;
                    }
                }
            }

            // Give CPU some rest - sleep for 10ms
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        Ok(package_locks)
    }

    // Unlock packages
    fn unlock_packages(&self, package_locks: Vec<PackageLock>) -> Result<(), ScryptoCompilerError> {
        for mut package_lock in package_locks {
            package_lock.unlock()?;
        }
        Ok(())
    }

    fn iter_manifests<'a>(&self) -> impl Iterator<Item = &CompilerManifestDefinition> {
        if self.manifests.is_empty() {
            Either::Left(iter::once(&self.main_manifest))
        } else {
            Either::Right(self.manifests.iter())
        }
    }

    // Scrypto compilation flow:
    //  - Compile with schema (without "scrypto/no-schema" feature) and release profile.
    //    Rename WASM files from '*.wasm' to '*_with_schema.wasm'
    //  - Try to get the remaining build artifacts (optimized WASM without schema '*.wasm' and '*.rpd' files) from Scrypto cache.
    //    It is done by calculating hash of the '*_with_schema.wasm' and searching its
    //  - If no files in Scrypto cache then:
    //    - Extract schema from '*_with_schema.wasm' into '*.rpd' files
    //    - Compile (with "scrypto/no-schema" feature) and optionally optimize WASM files '*.wasm'
    //    - Store '*.wasm' and '*.rpd' in Scrypto cache
    pub fn compile_with_stdio<T: Into<Stdio>>(
        &mut self,
        stdin: Option<T>,
        stdout: Option<T>,
        stderr: Option<T>,
    ) -> Result<Vec<BuildArtifacts>, ScryptoCompilerError> {
        let package_locks = self.lock_packages()?;

        let mut command = Command::new("cargo");
        // Stdio streams used only for 1st phase compilation due to lack of Copy trait.
        if let Some(s) = stdin {
            command.stdin(s);
        }
        if let Some(s) = stdout {
            command.stdout(s);
        }
        if let Some(s) = stderr {
            command.stderr(s);
        }

        self.compile_phase_1(&mut command)?;

        // For simplicity, do not use cache if coverage enabled
        let artifacts = if self.input_params.features.get(SCRYPTO_COVERAGE).is_none() {
            self.get_artifacts_from_cache()?
        } else {
            vec![]
        };

        let artifacts = if artifacts.is_empty() {
            let mut command = Command::new("cargo");
            self.compile_phase_2(&mut command)?
        } else {
            artifacts
        };

        self.unlock_packages(package_locks)?;
        Ok(artifacts)
    }

    pub fn compile(&mut self) -> Result<Vec<BuildArtifacts>, ScryptoCompilerError> {
        self.compile_with_stdio::<Stdio>(None, None, None)
    }

    // Compile with schema
    fn compile_phase_1(&mut self, command: &mut Command) -> Result<(), ScryptoCompilerError> {
        self.prepare_command_phase_1(command);
        self.cargo_command_call(command)?;

        for manifest in self.iter_manifests() {
            self.compile_phase_1_postprocess(&manifest)?;
        }

        Ok(())
    }

    // Rename WASM files from '*.wasm' to '*_with_schema.wasm'
    fn compile_phase_1_postprocess(
        &self,
        manifest_def: &CompilerManifestDefinition,
    ) -> Result<(), ScryptoCompilerError> {
        // The best would be to directly produce wasm file with schema by overriding Cargo.toml
        // values from command line.
        // Possibly it could be done by replacing 'cargo build' with 'cargo rustc' command,
        // which allows to customize settings on lower level. It is very likely it would implicate
        // more changes. And we don't want to complicate things more. So lets just rename the file.
        std::fs::rename(
            &manifest_def.target_phase_1_build_wasm_output_path,
            &manifest_def.target_copied_wasm_with_schema_path,
        )
        .map_err(|err| {
            ScryptoCompilerError::IOErrorWithPath(
                err,
                manifest_def.target_phase_1_build_wasm_output_path.clone(),
                Some(String::from("Rename WASM file failed.")),
            )
        })?;
        Ok(())
    }

    // used for unit tests
    fn prepare_command_phase_2(&mut self, command: &mut Command) {
        self.prepare_command(command, false); // build without schema and with userchoosen profile
    }

    // Compile without schema and with optional wasm optimisations - this is the final .wasm file
    fn compile_phase_2(
        &mut self,
        command: &mut Command,
    ) -> Result<Vec<BuildArtifacts>, ScryptoCompilerError> {
        self.prepare_command_phase_2(command);
        self.cargo_command_call(command)?;

        Ok(self
            .iter_manifests()
            .map(|manifest| self.compile_phase_2_postprocess(&manifest))
            .collect::<Result<Vec<_>, ScryptoCompilerError>>()?)
    }

    // Extract schema, optionally optimize WASM, store artifacts in cache
    fn compile_phase_2_postprocess(
        &self,
        manifest_def: &CompilerManifestDefinition,
    ) -> Result<BuildArtifacts, ScryptoCompilerError> {
        // TODO: code was already read to calculate hash. Optimize it.
        let code =
            std::fs::read(&manifest_def.target_copied_wasm_with_schema_path).map_err(|e| {
                ScryptoCompilerError::IOErrorWithPath(
                    e,
                    manifest_def.target_copied_wasm_with_schema_path.clone(),
                    Some(String::from("Read WASM file for RPD extract failed.")),
                )
            })?;
        let code_hash = hash(&code);

        let package_definition =
            extract_definition(&code).map_err(ScryptoCompilerError::SchemaExtractionError)?;

        std::fs::write(
            &manifest_def.target_output_binary_rpd_path,
            manifest_encode(&package_definition)
                .map_err(ScryptoCompilerError::SchemaEncodeError)?,
        )
        .map_err(|err| {
            ScryptoCompilerError::IOErrorWithPath(
                err,
                manifest_def.target_output_binary_rpd_path.clone(),
                Some(String::from("RPD file write failed.")),
            )
        })?;

        self.wasm_optimize(&manifest_def.target_phase_2_build_wasm_output_path.clone())?;

        let code =
            std::fs::read(&manifest_def.target_phase_2_build_wasm_output_path).map_err(|e| {
                ScryptoCompilerError::IOErrorWithPath(
                    e,
                    manifest_def.target_phase_2_build_wasm_output_path.clone(),
                    Some(String::from("Read optimized WASM file failed.")),
                )
            })?;

        let package_definition = BuildArtifact {
            path: manifest_def.target_output_binary_rpd_path.clone(),
            content: package_definition,
        };
        let wasm = BuildArtifact {
            path: manifest_def.target_phase_2_build_wasm_output_path.clone(),
            content: code,
        };
        let artifacts = BuildArtifacts {
            wasm,
            package_definition,
        };

        self.store_artifacts_in_cache(manifest_def, code_hash, &artifacts)?;

        Ok(artifacts)
    }

    fn cargo_command_call(&mut self, command: &mut Command) -> Result<(), ScryptoCompilerError> {
        if self.input_params.verbose {
            println!("Executing command: {}", cmd_to_string(command));
        }
        let status = command.status().map_err(|e| {
            ScryptoCompilerError::IOError(e, Some(String::from("Cargo build command failed.")))
        })?;
        status
            .success()
            .then_some(())
            .ok_or(ScryptoCompilerError::CargoBuildFailure(status))
    }

    // Return paths to the Scrypto cache for given manifest deifinition and code hash
    fn get_scrypto_cache_paths(
        &self,
        manifest_def: &CompilerManifestDefinition,
        code_hash: Hash,
        create_if_not_exists: bool,
    ) -> Result<(PathBuf, PathBuf), ScryptoCompilerError> {
        // WASM optimizations are optional and might be configured on different ways.
        // They are applied in 2nd compilation, which means one can receive different WASMs
        // for the same WASM files from 1st compilation.
        let options = format!(
            "{:?}/{:?}/{:?}",
            code_hash,
            self.input_params.profile.as_target_directory_name(),
            self.input_params.wasm_optimization
        );
        let hash_dir = hash(options);

        let cache_path = manifest_def
            .target_directory
            .join("scrypto_cache")
            .join(hash_dir.to_string());

        if create_if_not_exists {
            // Create target folder if it doesn't exist
            std::fs::create_dir_all(&cache_path).map_err(|err| {
                ScryptoCompilerError::IOErrorWithPath(
                    err,
                    cache_path.clone(),
                    Some(String::from("Create cache folder failed")),
                )
            })?;
        }

        let mut rpd_cache_path = cache_path
            .clone()
            .join(manifest_def.target_binary_name.clone());
        rpd_cache_path.set_extension("rpd");

        let mut wasm_cache_path = cache_path.join(manifest_def.target_binary_name.clone());
        wasm_cache_path.set_extension("wasm");
        Ok((rpd_cache_path, wasm_cache_path))
    }

    // Store build artifacts in Scrypto cache.
    // Override existing entries.
    fn store_artifacts_in_cache(
        &self,
        manifest_def: &CompilerManifestDefinition,
        code_hash: Hash,
        artifacts: &BuildArtifacts,
    ) -> Result<(), ScryptoCompilerError> {
        let (rpd_cache_path, wasm_cache_path) =
            self.get_scrypto_cache_paths(manifest_def, code_hash, true)?;

        std::fs::copy(&artifacts.package_definition.path, &rpd_cache_path).map_err(|err| {
            ScryptoCompilerError::IOErrorWithPath(
                err,
                artifacts.package_definition.path.clone(),
                Some(String::from("Copy RPD into cache folder failed")),
            )
        })?;

        std::fs::copy(&artifacts.wasm.path, &wasm_cache_path).map_err(|err| {
            ScryptoCompilerError::IOErrorWithPath(
                err,
                artifacts.wasm.path.clone(),
                Some(String::from("Copy WASM file into cache folder failed")),
            )
        })?;

        Ok(())
    }

    // Collect build artifacts from Scrypto cache.
    fn get_artifacts_from_cache(&mut self) -> Result<Vec<BuildArtifacts>, ScryptoCompilerError> {
        // compilation post-processing for all manifests
        let mut artifacts = vec![];
        for manifest in self.iter_manifests() {
            let artifact = self.get_artifact_from_cache_for_manifest(manifest)?;

            // If artifact for any manifest is missing then assume no artifacts in cache at all
            if let Some(artifact) = artifact {
                artifacts.push(artifact);
            } else {
                return Ok(vec![]);
            }
        }

        Ok(artifacts)
    }

    // Collect build artifacts from Scrypto cache for given manifest definition.
    fn get_artifact_from_cache_for_manifest(
        &self,
        manifest_def: &CompilerManifestDefinition,
    ) -> Result<Option<BuildArtifacts>, ScryptoCompilerError> {
        let code =
            std::fs::read(&manifest_def.target_copied_wasm_with_schema_path).map_err(|e| {
                ScryptoCompilerError::IOErrorWithPath(
                    e,
                    manifest_def.target_copied_wasm_with_schema_path.clone(),
                    Some(String::from("Read WASM with schema file failed.")),
                )
            })?;
        let code_hash = hash(&code);

        let (rpd_cache_path, wasm_cache_path) =
            self.get_scrypto_cache_paths(manifest_def, code_hash, false)?;

        // Get WASM and RPD files only if they both exist
        if std::fs::metadata(&rpd_cache_path).is_ok() && std::fs::metadata(&wasm_cache_path).is_ok()
        {
            let rpd = std::fs::read(&rpd_cache_path).map_err(|e| {
                ScryptoCompilerError::IOErrorWithPath(
                    e,
                    rpd_cache_path.clone(),
                    Some(String::from("Read RPD from cache failed.")),
                )
            })?;

            let package_definition: PackageDefinition =
                manifest_decode(&rpd).map_err(ScryptoCompilerError::SchemaDecodeError)?;

            let wasm = std::fs::read(&wasm_cache_path).map_err(|e| {
                ScryptoCompilerError::IOErrorWithPath(
                    e,
                    wasm_cache_path.clone(),
                    Some(String::from("Read WASM from cache failed.")),
                )
            })?;

            // Store artifacts into release folder
            let rpd_output_parent = manifest_def.target_output_binary_rpd_path.parent().unwrap();
            std::fs::create_dir_all(rpd_output_parent).map_err(|e| {
                ScryptoCompilerError::IOErrorWithPath(
                    e,
                    rpd_output_parent.to_path_buf(),
                    Some(String::from(
                        "Error creating the RPD file's parent folder if it doesn't exist.",
                    )),
                )
            })?;
            std::fs::write(&manifest_def.target_output_binary_rpd_path, rpd).map_err(|e| {
                ScryptoCompilerError::IOErrorWithPath(
                    e,
                    manifest_def.target_output_binary_rpd_path.clone(),
                    Some(String::from("Write RPD file failed.")),
                )
            })?;

            // On filesystems with hard-linking support `target_binary_wasm_path` might be a hard-link
            // (rust caching for incremental builds)
            // pointing to `./<target-dir>/wasm32-unknown-unknown/release/deps/<wasm_binary>`,
            // which would be also modified if we would directly wrote below data.
            // Which in turn would be reused in the next recompilation resulting with a
            // `target_binary_wasm_with_schema_path` not including the schema.
            // So if `target_binary_wasm_path` exists just remove it assuming it is a hard-link.
            let wasm_output_parent = manifest_def
                .target_phase_2_build_wasm_output_path
                .parent()
                .unwrap();
            std::fs::create_dir_all(wasm_output_parent).map_err(|e| {
                ScryptoCompilerError::IOErrorWithPath(
                    e,
                    wasm_output_parent.to_path_buf(),
                    Some(String::from(
                        "Error creating the WASM file's parent folder if it doesn't exist.",
                    )),
                )
            })?;
            if std::fs::metadata(&manifest_def.target_phase_2_build_wasm_output_path).is_ok() {
                std::fs::remove_file(&manifest_def.target_phase_2_build_wasm_output_path).map_err(
                    |e| {
                        ScryptoCompilerError::IOErrorWithPath(
                            e,
                            manifest_def.target_phase_2_build_wasm_output_path.clone(),
                            Some(String::from("Remove WASM file failed.")),
                        )
                    },
                )?;
            }
            std::fs::write(
                &manifest_def.target_phase_2_build_wasm_output_path,
                wasm.clone(),
            )
            .map_err(|e| {
                ScryptoCompilerError::IOErrorWithPath(
                    e,
                    manifest_def.target_phase_2_build_wasm_output_path.clone(),
                    Some(String::from("Write WASM file failed.")),
                )
            })?;

            let wasm = BuildArtifact {
                path: manifest_def.target_phase_2_build_wasm_output_path.clone(),
                content: wasm,
            };
            let package_definition = BuildArtifact {
                path: manifest_def.target_output_binary_rpd_path.clone(),
                content: package_definition,
            };

            Ok(Some(BuildArtifacts {
                wasm,
                package_definition,
            }))
        } else {
            Ok(None)
        }
    }

    // used for unit tests
    fn prepare_command_phase_1(&mut self, command: &mut Command) {
        self.prepare_command(command, true); // build with schema and release profile
    }

    /// Returns information about the main manifest
    pub fn get_main_manifest_definition(&self) -> CompilerManifestDefinition {
        self.main_manifest.clone()
    }
}

#[derive(Default)]
pub struct ScryptoCompilerBuilder {
    input_params: ScryptoCompilerInputParams,
}

impl ScryptoCompilerBuilder {
    pub fn manifest_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.input_params.manifest_path = Some(path.into());
        self
    }

    pub fn target_directory(&mut self, directory: impl Into<PathBuf>) -> &mut Self {
        self.input_params.target_directory = Some(directory.into());

        self
    }

    pub fn profile(&mut self, profile: Profile) -> &mut Self {
        self.input_params.profile = profile;
        self
    }

    pub fn env(&mut self, name: &str, action: EnvironmentVariableAction) -> &mut Self {
        self.input_params
            .environment_variables
            .insert(name.to_string(), action);
        self
    }

    pub fn feature(&mut self, name: &str) -> &mut Self {
        self.input_params.features.insert(name.to_string());
        self
    }

    pub fn no_default_features(&mut self) -> &mut Self {
        self.input_params.no_default_features = true;
        self
    }

    pub fn all_features(&mut self) -> &mut Self {
        self.input_params.all_features = true;
        self
    }

    pub fn locked(&mut self) -> &mut Self {
        self.input_params.locked = true;
        self
    }

    pub fn ignore_locked_env_var(&mut self) -> &mut Self {
        self.input_params.ignore_locked_env_var = true;
        self
    }

    pub fn package(&mut self, name: &str) -> &mut Self {
        self.input_params.package.insert(name.to_string());
        self
    }

    pub fn scrypto_macro_trace(&mut self) -> &mut Self {
        self.input_params
            .features
            .insert(String::from("scrypto/trace"));
        self
    }

    pub fn log_level(&mut self, log_level: Level) -> &mut Self {
        // Firstly clear any log level previously set
        let all_features = ScryptoCompilerInputParams::log_level_to_scrypto_features(Level::Trace);
        all_features.iter().for_each(|log_level| {
            self.input_params.features.swap_remove(log_level);
        });

        // Now set log level provided by the user
        if Level::Error <= log_level {
            self.input_params
                .features
                .insert(String::from("scrypto/log-error"));
        }
        if Level::Warn <= log_level {
            self.input_params
                .features
                .insert(String::from("scrypto/log-warn"));
        }
        if Level::Info <= log_level {
            self.input_params
                .features
                .insert(String::from("scrypto/log-info"));
        }
        if Level::Debug <= log_level {
            self.input_params
                .features
                .insert(String::from("scrypto/log-debug"));
        }
        if Level::Trace <= log_level {
            self.input_params
                .features
                .insert(String::from("scrypto/log-trace"));
        }
        self
    }

    pub fn coverage(&mut self) -> &mut Self {
        self.input_params
            .features
            .insert(String::from(SCRYPTO_COVERAGE));
        self
    }

    pub fn optimize_with_wasm_opt(
        &mut self,
        options: Option<wasm_opt::OptimizationOptions>,
    ) -> &mut Self {
        self.input_params.wasm_optimization = options;
        self
    }

    pub fn custom_options(&mut self, options: &[&str]) -> &mut Self {
        self.input_params
            .custom_options
            .extend(options.iter().map(|item| item.to_string()));
        self
    }

    pub fn debug(&mut self, verbose: bool) -> &mut Self {
        self.input_params.verbose = verbose;
        self
    }

    pub fn build(&mut self) -> Result<ScryptoCompiler, ScryptoCompilerError> {
        ScryptoCompiler::from_input_params(&mut self.input_params)
    }

    pub fn compile(&mut self) -> Result<Vec<BuildArtifacts>, ScryptoCompilerError> {
        self.build()?.compile()
    }

    pub fn compile_with_stdio<T: Into<Stdio>>(
        &mut self,
        stdin: Option<T>,
        stdout: Option<T>,
        stderr: Option<T>,
    ) -> Result<Vec<BuildArtifacts>, ScryptoCompilerError> {
        self.build()?.compile_with_stdio(stdin, stdout, stderr)
    }
}

#[cfg(feature = "std")]
pub fn is_scrypto_cargo_locked_env_var_active() -> bool {
    std::env::var("SCRYPTO_CARGO_LOCKED").is_ok_and(|val| {
        let normalized = val.to_lowercase();
        &normalized == "true" || &normalized == "1"
    })
}

#[cfg(not(feature = "std"))]
pub fn is_scrypto_cargo_locked_env_var_active() -> bool {
    false
}

// helper function
fn cmd_to_string(cmd: &Command) -> String {
    let args = cmd
        .get_args()
        .into_iter()
        .map(|arg| arg.to_str().unwrap())
        .collect::<Vec<_>>()
        .join(" ");
    let envs = cmd
        .get_envs()
        .into_iter()
        .map(|(name, value)| {
            if let Some(value) = value {
                format!("{}='{}'", name.to_str().unwrap(), value.to_str().unwrap())
            } else {
                format!("{}", name.to_str().unwrap())
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    let mut ret = envs;
    if !ret.is_empty() {
        ret.push(' ');
    }
    ret.push_str(cmd.get_program().to_str().unwrap());
    ret.push(' ');
    ret.push_str(&args);
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_binary_path_target() {
        let target_dir = "./tests/target";
        let compiler = ScryptoCompiler::builder()
            .manifest_path("./tests/assets/scenario_1/blueprint")
            .target_directory(target_dir)
            .custom_options(&["-j", "1"])
            .build()
            .unwrap();

        assert_eq!(
            "./tests/target/wasm32-unknown-unknown/release/test_blueprint.wasm",
            compiler
                .main_manifest
                .target_phase_1_build_wasm_output_path
                .display()
                .to_string()
        );
    }

    #[test]
    fn test_command_output_default() {
        // Arrange
        let mut manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut default_target_path = manifest_path.clone();
        manifest_path.push("Cargo.toml");
        default_target_path.pop(); // ScryptoCompiler dir
        default_target_path.push("target");
        let mut cmd_phase_1 = Command::new("cargo");
        let mut cmd_phase_2 = Command::new("cargo");

        // Act
        ScryptoCompiler::builder()
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_1(&mut cmd_phase_1);
        ScryptoCompiler::builder()
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_2(&mut cmd_phase_2);

        // Assert
        assert_eq!(cmd_to_string(&cmd_phase_1),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --release", default_target_path.display(), manifest_path.display()));
        assert_eq!(cmd_to_string(&cmd_phase_2),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --features scrypto/no-schema --profile release", default_target_path.display(), manifest_path.display()));
    }

    #[test]
    fn test_command_output_with_manifest_path() {
        // Arrange
        let mut manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut default_target_path = manifest_path.clone();
        manifest_path.push("tests/assets/scenario_1/blueprint/Cargo.toml");
        default_target_path.push("tests/assets/scenario_1/target");
        let mut cmd_phase_1 = Command::new("cargo");
        let mut cmd_phase_2 = Command::new("cargo");

        // Act
        ScryptoCompiler::builder()
            .manifest_path(&manifest_path)
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_1(&mut cmd_phase_1);
        ScryptoCompiler::builder()
            .manifest_path(&manifest_path)
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_2(&mut cmd_phase_2);

        // Assert
        assert_eq!(cmd_to_string(&cmd_phase_1),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --release", default_target_path.display(), manifest_path.display()));
        assert_eq!(cmd_to_string(&cmd_phase_2),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --features scrypto/no-schema --profile release", default_target_path.display(), manifest_path.display()));
    }

    #[test]
    fn test_command_output_target_directory() {
        // Arrange
        let mut manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_path.push("Cargo.toml");
        let target_path = PathBuf::from("/tmp/build");
        let mut cmd_phase_1 = Command::new("cargo");
        let mut cmd_phase_2 = Command::new("cargo");

        // Act
        ScryptoCompiler::builder()
            .target_directory(&target_path)
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_1(&mut cmd_phase_1);
        ScryptoCompiler::builder()
            .target_directory(&target_path)
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_2(&mut cmd_phase_2);

        // Assert
        assert_eq!(cmd_to_string(&cmd_phase_1),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --release", target_path.display(), manifest_path.display()));
        assert_eq!(cmd_to_string(&cmd_phase_2),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --features scrypto/no-schema --profile release", target_path.display(), manifest_path.display()));
    }

    #[test]
    fn test_command_output_features() {
        // Arrange
        let mut manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut default_target_path = manifest_path.clone();
        manifest_path.push("Cargo.toml");
        default_target_path.pop(); // ScryptoCompiler dir
        default_target_path.push("target");
        let mut cmd_phase_1 = Command::new("cargo");
        let mut cmd_phase_2 = Command::new("cargo");

        // Act
        ScryptoCompiler::builder()
            .log_level(Level::Trace)
            .feature("feature_1")
            .no_default_features()
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_1(&mut cmd_phase_1);
        ScryptoCompiler::builder()
            .log_level(Level::Trace)
            .feature("feature_1")
            .no_default_features()
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_2(&mut cmd_phase_2);

        // Assert
        assert_eq!(cmd_to_string(&cmd_phase_1),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --features scrypto/log-debug --features scrypto/log-trace --features feature_1 --release --no-default-features", default_target_path.display(), manifest_path.display()));
        assert_eq!(cmd_to_string(&cmd_phase_2),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --features scrypto/log-debug --features scrypto/log-trace --features feature_1 --features scrypto/no-schema --profile release --no-default-features", default_target_path.display(), manifest_path.display()));
    }

    #[test]
    fn test_command_output_lower_log_level_than_default() {
        // Arrange
        let mut manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut default_target_path = manifest_path.clone();
        manifest_path.push("Cargo.toml");
        default_target_path.pop(); // ScryptoCompiler dir
        default_target_path.push("target");
        let mut cmd_phase_1 = Command::new("cargo");
        let mut cmd_phase_2 = Command::new("cargo");

        // Act
        ScryptoCompiler::builder()
            .log_level(Level::Error)
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_1(&mut cmd_phase_1);
        ScryptoCompiler::builder()
            .log_level(Level::Error)
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_2(&mut cmd_phase_2);

        // Assert
        assert_eq!(cmd_to_string(&cmd_phase_1),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --release", default_target_path.display(), manifest_path.display()));
        assert_eq!(cmd_to_string(&cmd_phase_2),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/no-schema --profile release", default_target_path.display(), manifest_path.display()));
    }
    #[test]
    fn test_command_output_workspace() {
        // Arrange
        let mut manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut default_target_path = manifest_path.clone();
        manifest_path.push("tests/assets/scenario_1/Cargo.toml");
        default_target_path.push("tests/assets/scenario_1/target");
        let mut cmd_phase_1 = Command::new("cargo");
        let mut cmd_phase_2 = Command::new("cargo");

        // Act
        ScryptoCompiler::builder()
            .manifest_path(&manifest_path)
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_1(&mut cmd_phase_1);
        ScryptoCompiler::builder()
            .manifest_path(&manifest_path)
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_2(&mut cmd_phase_2);

        // Assert
        assert_eq!(cmd_to_string(&cmd_phase_1),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --package test_blueprint --package test_blueprint_2 --package test_blueprint_3 --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --release", default_target_path.display(), manifest_path.display()));
        assert_eq!(cmd_to_string(&cmd_phase_2),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --package test_blueprint --package test_blueprint_2 --package test_blueprint_3 --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --features scrypto/no-schema --profile release", default_target_path.display(), manifest_path.display()));
    }

    #[test]
    fn test_command_output_workspace_with_packages() {
        // Arrange
        let mut manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut default_target_path = manifest_path.clone();
        manifest_path.push("tests/assets/scenario_1/Cargo.toml");
        default_target_path.push("tests/assets/scenario_1/target");
        let mut cmd_phase_1 = Command::new("cargo");
        let mut cmd_phase_2 = Command::new("cargo");

        // Act
        ScryptoCompiler::builder()
            .manifest_path(&manifest_path)
            .package("test_blueprint")
            .package("test_blueprint_3")
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_1(&mut cmd_phase_1);
        ScryptoCompiler::builder()
            .manifest_path(&manifest_path)
            .package("test_blueprint")
            .package("test_blueprint_3")
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_2(&mut cmd_phase_2);

        // Assert
        assert_eq!(cmd_to_string(&cmd_phase_1),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --package test_blueprint --package test_blueprint_3 --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --release", default_target_path.display(), manifest_path.display()));
        assert_eq!(cmd_to_string(&cmd_phase_2),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --package test_blueprint --package test_blueprint_3 --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --features scrypto/no-schema --profile release", default_target_path.display(), manifest_path.display()));
    }

    #[test]
    fn test_command_output_profiles() {
        // Arrange
        let mut manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut default_target_path = manifest_path.clone();
        manifest_path.push("Cargo.toml");
        default_target_path.pop(); // ScryptoCompiler dir
        default_target_path.push("target");
        let mut cmd_phase_1 = Command::new("cargo");
        let mut cmd_phase_2 = Command::new("cargo");

        // Act
        ScryptoCompiler::builder()
            .profile(Profile::Debug)
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_1(&mut cmd_phase_1);
        ScryptoCompiler::builder()
            .profile(Profile::Debug)
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_2(&mut cmd_phase_2);

        // Assert
        assert_eq!(cmd_to_string(&cmd_phase_1),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --release", default_target_path.display(), manifest_path.display()));
        assert_eq!(cmd_to_string(&cmd_phase_2),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --features scrypto/no-schema --profile dev", default_target_path.display(), manifest_path.display()));
    }

    #[test]
    fn test_command_output_no_schema_check() {
        // Arrange
        let mut manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut default_target_path = manifest_path.clone();
        manifest_path.push("Cargo.toml");
        default_target_path.pop(); // ScryptoCompiler dir
        default_target_path.push("target");
        let mut cmd_phase_1 = Command::new("cargo");
        let mut cmd_phase_2 = Command::new("cargo");

        // Act
        // Ensure that no-schema is properly used across both phase compilation, even if specified explicitly by the user.
        ScryptoCompiler::builder()
            .feature(SCRYPTO_NO_SCHEMA)
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_1(&mut cmd_phase_1);
        ScryptoCompiler::builder()
            .feature(SCRYPTO_NO_SCHEMA)
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_2(&mut cmd_phase_2);

        // Assert
        assert_eq!(cmd_to_string(&cmd_phase_1),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --release", default_target_path.display(), manifest_path.display()));
        assert_eq!(cmd_to_string(&cmd_phase_2),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --features scrypto/no-schema --profile release", default_target_path.display(), manifest_path.display()));
    }

    #[test]
    fn test_command_coverage() {
        // Arrange
        let mut manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut target_path = manifest_path.clone();
        manifest_path.push("Cargo.toml");
        target_path.pop(); // ScryptoCompiler dir
        target_path.push("coverage");
        let mut cmd_phase_1 = Command::new("cargo");
        let mut cmd_phase_2 = Command::new("cargo");

        // Act
        ScryptoCompiler::builder()
            .coverage()
            .target_directory(target_path.clone())
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_1(&mut cmd_phase_1);
        ScryptoCompiler::builder()
            .coverage()
            .target_directory(target_path.clone())
            .ignore_locked_env_var()
            .build()
            .unwrap()
            .prepare_command_phase_2(&mut cmd_phase_2);

        // Assert
        assert_eq!(cmd_to_string(&cmd_phase_1),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --release", target_path.display(), manifest_path.display()));
        assert_eq!(cmd_to_string(&cmd_phase_2),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --features scrypto/coverage --features scrypto/no-schema --profile release", target_path.display(), manifest_path.display()));
    }

    #[test]
    fn test_command_coverage_with_env() {
        // Arrange
        let mut manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut target_path = manifest_path.clone();
        manifest_path.push("Cargo.toml");
        target_path.pop(); // ScryptoCompiler dir
        target_path.push("coverage");
        let action = EnvironmentVariableAction::Set(String::from(
            "-Clto=off\x1f-Cinstrument-coverage\x1f-Zno-profiler-runtime\x1f--emit=llvm-ir",
        ));
        let mut cmd_phase_1 = Command::new("cargo");
        let mut cmd_phase_2 = Command::new("cargo");

        // Act
        ScryptoCompiler::builder()
            .coverage()
            .target_directory(target_path.clone())
            .ignore_locked_env_var()
            .env("CARGO_ENCODED_RUSTFLAGS", action.clone()) // CARGO_ENCODED_RUSTFLAGS must be removed for 1st phase
            .build()
            .unwrap()
            .prepare_command_phase_1(&mut cmd_phase_1);
        ScryptoCompiler::builder()
            .coverage()
            .target_directory(target_path.clone())
            .ignore_locked_env_var()
            .env("CARGO_ENCODED_RUSTFLAGS", action.clone())
            .build()
            .unwrap()
            .prepare_command_phase_2(&mut cmd_phase_2);

        // Assert
        assert_eq!(cmd_to_string(&cmd_phase_1),
            format!("TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --release", target_path.display(), manifest_path.display()));
        assert_eq!(cmd_to_string(&cmd_phase_2),
            format!("CARGO_ENCODED_RUSTFLAGS='-Clto=off\x1f-Cinstrument-coverage\x1f-Zno-profiler-runtime\x1f--emit=llvm-ir' TARGET_CFLAGS='-mcpu=mvp -mmutable-globals -msign-ext' cargo build --target wasm32-unknown-unknown --target-dir {} --manifest-path {} --features scrypto/log-error --features scrypto/log-warn --features scrypto/log-info --features scrypto/coverage --features scrypto/no-schema --profile release", target_path.display(), manifest_path.display()));
    }

    #[test]
    fn test_parallel_compilation() {
        use rayon::iter::{IntoParallelIterator, ParallelIterator};

        fn artifacts_hash(artifacts: Vec<BuildArtifacts>) -> Hash {
            let mut artifacts = artifacts.clone();

            artifacts.sort_by(|a, b| a.wasm.path.cmp(&b.wasm.path));

            let wasms: Vec<u8> = artifacts
                .iter()
                .map(|item| item.wasm.content.clone())
                .flatten()
                .collect();
            hash(wasms)
        }

        // Arrange
        let mut manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_path.push("tests/assets/scenario_1/Cargo.toml");

        let mut compiler = ScryptoCompiler::builder()
            .manifest_path(&manifest_path)
            .package("test_blueprint")
            .package("test_blueprint_2")
            .build()
            .unwrap();

        let artifacts = compiler.compile().unwrap();
        let reference_wasms_hash = artifacts_hash(artifacts);

        // Act
        // Run couple of compilations in parallel and compare hash of the build artifacts
        // with the reference hash.
        let found = (0u64..20u64).into_par_iter().find_map_any(|_| {
            let mut compiler = ScryptoCompiler::builder()
                .manifest_path(&manifest_path)
                .package("test_blueprint")
                .package("test_blueprint_2")
                .build()
                .unwrap();

            let artifacts = compiler.compile().unwrap();
            if reference_wasms_hash != artifacts_hash(artifacts) {
                Some(())
            } else {
                None
            }
        });

        assert!(found.is_none());
    }
}
