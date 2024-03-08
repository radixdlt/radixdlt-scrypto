use cargo_toml::Manifest;
use radix_engine_interface::types::Level;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::{env, io};

const MANIFEST_FILE: &str = "Cargo.toml";
const BUILD_TARGET: &str = "wasm32-unknown-unknown";

#[derive(Debug)]
pub enum ScryptoCompilerError {
    /// Returns IO Error which occured during compilation
    IOError(io::Error),
    /// Returns output from stderr and process exit status
    CargoBuildFailure(Vec<u8>, ExitStatus),
    /// Returns path to Cargo.toml for which cargo metadata command failed and process exit status
    CargoMetadataFailure(String, ExitStatus),
    /// Returns path to Cargo.toml for which results of cargo metadata command is not not valid json or target directory field is missing
    CargoTargetDirectoryResolutionError(String),
    /// Returns path to Cargo.toml which was failed to load
    CargoManifestLoadFailure(String),
    /// Returns path to Cargo.toml which cannot be found
    CargoManifestFileNotFound(String),
    /// Returns information about invalid input compiler parameter
    InvalidParam(ScryptoCompilerInvalidInputParam),
    /// Returns WASM Optimization error
    WasmOptimizationError(wasm_opt::OptimizationError),
}

#[derive(Debug)]
pub enum ScryptoCompilerInvalidInputParam {
    /// Both parameters were specified: 'coverage' and 'target directory'
    CoverageDiscardsTargetDirectory,
    /// Both parameters were specified: 'coverage' and 'force local target'
    CoverageDiscardsForceLocalTarget,
    /// Both parameters were specified: 'force local target' and 'target directory'
    ForceLocalTargetDiscardsTargetDirectory,
    /// Remove 'Cargo.toml' from 'manifest directory' parameter
    CargoTomlInManifestDirectory,
    /// Same variable were set and unset
    EnvironmentVariableSetAndUnset(String),
}

#[derive(Clone, Default)]
pub struct ScryptoCompilerInputParams {
    profile: Profile,
    set_environment_variables: Vec<(String, String)>,
    unset_environment_variables: Vec<String>,
    features: Vec<String>,
    package: Option<String>,
    target_directory: Option<PathBuf>,
    manifest_directory: Option<PathBuf>,
    trace: bool,
    log_level: Level,
    no_schema: bool,
    coverage: bool,
    force_local_target: bool,
    wasm_optimization: Option<wasm_opt::OptimizationOptions>,
}

pub struct ScryptoCompiler {
    input_params: ScryptoCompilerInputParams,

    target_directory: PathBuf,
    manifest_path: PathBuf,
    target_binary_path: PathBuf,
}

impl ScryptoCompiler {
    pub fn new() -> ScryptoCompilerBuilder {
        ScryptoCompilerBuilder::default()
    }

    // Internal constructor
    fn from_input_params(
        input_params: &ScryptoCompilerInputParams,
    ) -> Result<Self, ScryptoCompilerError> {
        // Firstly validate input parameters
        ScryptoCompiler::validate_input_parameters(input_params)
            .map_err(|e| ScryptoCompilerError::InvalidParam(e))?;
        // Secondly prepare internally used path basing on input parameters
        let (manifest_path, target_directory, target_binary_path) =
            ScryptoCompiler::prepare_paths(input_params)?;
        // Lastly create ScryptoCompiler object
        Ok(Self {
            input_params: input_params.to_owned(),
            manifest_path,
            target_directory,
            target_binary_path,
        })
    }

    fn validate_input_parameters(
        input_params: &ScryptoCompilerInputParams,
    ) -> Result<(), ScryptoCompilerInvalidInputParam> {
        if input_params.coverage && input_params.force_local_target {
            return Err(ScryptoCompilerInvalidInputParam::CoverageDiscardsForceLocalTarget);
        }
        if input_params.coverage && input_params.target_directory.is_some() {
            return Err(ScryptoCompilerInvalidInputParam::CoverageDiscardsTargetDirectory);
        }
        if input_params.force_local_target && input_params.target_directory.is_some() {
            return Err(ScryptoCompilerInvalidInputParam::ForceLocalTargetDiscardsTargetDirectory);
        }
        if input_params
            .manifest_directory
            .as_ref()
            .is_some_and(|v| PathBuf::from(v).ends_with(MANIFEST_FILE))
        {
            return Err(ScryptoCompilerInvalidInputParam::CargoTomlInManifestDirectory);
        }
        if let Some(env) = input_params.set_environment_variables.iter().find_map(|v| {
            if input_params.unset_environment_variables.contains(&v.0) {
                Some(v.0.clone())
            } else {
                None
            }
        }) {
            return Err(ScryptoCompilerInvalidInputParam::EnvironmentVariableSetAndUnset(env));
        }
        Ok(())
    }

    fn prepare_features(&self) -> String {
        let mut features = String::new();

        // Firstly apply scrypto features
        if self.input_params.trace {
            features.push_str(",scrypto/trace");
        }
        if self.input_params.no_schema {
            features.push_str(",scrypto/no-schema");
        }
        if Level::Error <= self.input_params.log_level {
            features.push_str(",scrypto/log-error");
        }
        if Level::Warn <= self.input_params.log_level {
            features.push_str(",scrypto/log-warn");
        }
        if Level::Info <= self.input_params.log_level {
            features.push_str(",scrypto/log-info");
        }
        if Level::Debug <= self.input_params.log_level {
            features.push_str(",scrypto/log-debug");
        }
        if Level::Trace <= self.input_params.log_level {
            features.push_str(",scrypto/log-trace");
        }
        if self.input_params.coverage {
            features.push_str(",scrypto/coverage");
        }

        // Then apply user features
        if !self.input_params.features.is_empty() {
            features.push(',');
            features.push_str(&self.input_params.features.join(","));
        }

        if features.starts_with(',') {
            features.remove(0);
        }

        features
    }

    fn prepare_rust_flags(&self) -> String {
        if self.input_params.coverage {
            "-Clto=off\x1f-Cinstrument-coverage\x1f-Zno-profiler-runtime\x1f--emit=llvm-ir"
                .to_owned()
        } else {
            env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default()
        }
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
            .map_err(ScryptoCompilerError::IOError)?;
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
                manifest_path.display().to_string(),
                output.status,
            ))
        }
    }

    fn get_target_binary_path(
        manifest_path: &Path,
        binary_target_directory: &Path,
    ) -> Result<PathBuf, ScryptoCompilerError> {
        // Find the binary name
        let manifest = Manifest::from_path(&manifest_path).map_err(|_| {
            ScryptoCompilerError::CargoManifestLoadFailure(manifest_path.display().to_string())
        })?;
        let mut wasm_name = None;
        if let Some(lib) = manifest.lib {
            wasm_name = lib.name.clone();
        }
        if wasm_name.is_none() {
            if let Some(pkg) = manifest.package {
                wasm_name = Some(pkg.name.replace("-", "_"));
            }
        }
        // Merge the name with binary tearget directory
        let mut bin_path: PathBuf = binary_target_directory.into();
        bin_path.push(
            wasm_name.ok_or(ScryptoCompilerError::CargoManifestLoadFailure(
                manifest_path.display().to_string(),
            ))?,
        );
        bin_path.set_extension("wasm");

        Ok(bin_path)
    }

    // Returns manifest path, target directory, target binary path
    fn prepare_paths(
        input_params: &ScryptoCompilerInputParams,
    ) -> Result<(PathBuf, PathBuf, PathBuf), ScryptoCompilerError> {
        // Generate manifest path (manifest directory + "/Cargo.toml")
        let manifest_directory = input_params
            .manifest_directory
            .as_ref()
            .map_or(env::current_dir().unwrap(), |v| PathBuf::from(v));
        let mut manifest_path = manifest_directory.clone();
        manifest_path.push(MANIFEST_FILE);

        if !manifest_path.exists() {
            return Err(ScryptoCompilerError::CargoManifestFileNotFound(
                manifest_path.display().to_string(),
            ));
        }

        // Generate target directory
        let target_directory = if input_params.coverage {
            // If coverate compiler parameter is set to true then set target directory as
            // manifest directory + "/coverage"
            let mut target_path = manifest_directory.clone();
            target_path.push("coverage");
            target_path
        } else if input_params.force_local_target {
            // If force local target compiler parameter is set to true then set target directory as
            // manifest directory + "/target"
            let mut target_path = manifest_directory;
            target_path.push("target");
            target_path
        } else if let Some(directory) = &input_params.target_directory {
            // If target directory is explicitly specified as compiler parameter then use it as is
            PathBuf::from(directory)
        } else {
            // If target directory is not specified as compiler parameter then get default
            // target directory basing on manifest file
            PathBuf::from(&Self::get_default_target_directory(&manifest_path)?)
        };

        let mut target_binary_directory = target_directory.clone();
        target_binary_directory.push(BUILD_TARGET);
        target_binary_directory.push(input_params.profile.as_directory_name());

        let target_binary_path =
            Self::get_target_binary_path(&manifest_path, &target_binary_directory)?;

        Ok((manifest_path, target_directory, target_binary_path))
    }

    fn prepare_command(&mut self, command: &mut Command) -> Result<(), ScryptoCompilerError> {
        let features_list = self.prepare_features();
        let features = (!features_list.is_empty())
            .then_some(vec!["--features", &features_list])
            .unwrap_or_default();

        let rustflags = self.prepare_rust_flags();

        let package = self
            .input_params
            .package
            .as_ref()
            .and_then(|p| Some(vec!["--package", &p]))
            .unwrap_or_default();

        if self.input_params.coverage {
            // coverage uses '-Z' flag which requires use of nightly toolchain
            command.arg("+nightly");
        }

        command
            .arg("build")
            .arg("--target")
            .arg(BUILD_TARGET)
            .args(self.input_params.profile.as_command_args())
            .arg("--target-dir")
            .arg(&self.target_directory)
            .arg("--manifest-path")
            .arg(&self.manifest_path)
            .args(package)
            .args(features)
            .env("CARGO_ENCODED_RUSTFLAGS", rustflags)
            .envs(self.input_params.set_environment_variables.clone());

        self.input_params
            .unset_environment_variables
            .iter()
            .for_each(|e| {
                command.env_remove(e);
            });

        Ok(())
    }

    fn wasm_optimize(&mut self) -> Result<(), ScryptoCompilerError> {
        if let Some(wasm_opt_config) = &self.input_params.wasm_optimization {
            wasm_opt_config
                .run(&self.target_binary_path, &self.target_binary_path)
                .map_err(ScryptoCompilerError::WasmOptimizationError)
        } else {
            Ok(())
        }
    }

    // Returns output wasm file path
    pub fn compile(&mut self) -> Result<PathBuf, ScryptoCompilerError> {
        // Create compilation command
        let mut command = Command::new("cargo");
        self.prepare_command(&mut command)?;

        // Execute command
        let output = command.output().map_err(ScryptoCompilerError::IOError)?;

        output
            .status
            .success()
            .then_some(())
            .ok_or(ScryptoCompilerError::CargoBuildFailure(
                output.stderr,
                output.status,
            ))?;

        self.wasm_optimize()?;

        Ok(self.target_binary_path.clone())
    }

    pub fn target_binary_path(&self) -> PathBuf {
        self.target_binary_path.clone()
    }
}

#[derive(Default, Clone)]
pub enum Profile {
    #[default]
    Release,
    Debug,
}

impl Profile {
    fn as_command_args(&self) -> Vec<String> {
        match self {
            Profile::Release => vec![String::from("--release")],
            Profile::Debug => vec![],
        }
    }
    fn as_directory_name(&self) -> String {
        match self {
            Profile::Release => String::from("release"),
            Profile::Debug => String::from("debug"),
        }
    }
}

#[derive(Default)]
pub struct ScryptoCompilerBuilder {
    input_params: ScryptoCompilerInputParams,
}

impl ScryptoCompilerBuilder {
    pub fn profile(&mut self, profile: Profile) -> &mut Self {
        self.input_params.profile = profile;
        self
    }

    pub fn env(&mut self, name: &str, value: &str) -> &mut Self {
        self.input_params
            .set_environment_variables
            .push((name.to_string(), value.to_string()));
        self
    }

    pub fn unset_env(&mut self, name: &str) -> &mut Self {
        self.input_params
            .unset_environment_variables
            .push(name.to_string());
        self
    }

    pub fn feature(&mut self, name: &str) -> &mut Self {
        self.input_params.features.push(name.to_string());
        self
    }

    pub fn package(&mut self, name: &str) -> &mut Self {
        self.input_params.package = Some(name.to_string());
        self
    }

    pub fn target_directory(&mut self, directory: impl Into<PathBuf>) -> &mut Self {
        self.input_params.target_directory = Some(directory.into());

        self
    }

    pub fn manifest_directory(&mut self, directory: impl Into<PathBuf>) -> &mut Self {
        self.input_params.manifest_directory = Some(directory.into());
        self
    }

    pub fn trace(&mut self, trace: bool) -> &mut Self {
        self.input_params.trace = trace;
        self
    }

    pub fn log_level(&mut self, log_level: Level) -> &mut Self {
        self.input_params.log_level = log_level;
        self
    }

    pub fn no_schema(&mut self, no_schema: bool) -> &mut Self {
        self.input_params.no_schema = no_schema;
        self
    }

    pub fn coverage(&mut self, coverage: bool) -> &mut Self {
        self.input_params.coverage = coverage;
        self
    }

    pub fn force_local_target(&mut self, local_target: bool) -> &mut Self {
        self.input_params.force_local_target = local_target;
        self
    }

    pub fn optimize_with_wasm_opt(&mut self, options: &wasm_opt::OptimizationOptions) -> &mut Self {
        self.input_params.wasm_optimization = Some(options.to_owned());
        self
    }

    pub fn build(&mut self) -> Result<ScryptoCompiler, ScryptoCompilerError> {
        ScryptoCompiler::from_input_params(&self.input_params)
    }

    // Returns output wasm file path
    pub fn compile(&mut self) -> Result<PathBuf, ScryptoCompilerError> {
        self.build()?.compile()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn cargo_clean(manifest_dir: &str) {
        Command::new("cargo")
            .arg("clean")
            .arg("--manifest-path")
            .arg(manifest_dir.to_owned() + "/Cargo.toml")
            .output()
            .unwrap();
    }

    #[test]
    #[serial]
    fn test_compilation() {
        // Arrange
        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";
        cargo_clean(manifest_dir);
        std::env::set_current_dir(cur_dir.clone()).unwrap();

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .compile();

        // Assert
        assert!(status.is_ok());

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_compilation_in_current_dit() {
        // Arrange
        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";
        std::env::set_current_dir(manifest_dir).unwrap();

        cargo_clean("./");

        // Act
        let status = ScryptoCompiler::new().compile();

        // Assert
        assert!(status.is_ok());

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_compilation_debug() {
        // Arrange
        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";
        cargo_clean(manifest_dir);
        std::env::set_current_dir(cur_dir.clone()).unwrap();

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .profile(Profile::Debug)
            .compile();

        // Assert
        assert!(status.is_ok());

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_compilation_coverage() {
        // Arrange
        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";
        cargo_clean(manifest_dir);
        std::env::set_current_dir(cur_dir.clone()).unwrap();

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .coverage(true)
            .compile();

        // Assert
        assert!(status.is_ok());

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_compilation_with_feature() {
        // Arrange
        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";
        cargo_clean(manifest_dir);
        std::env::set_current_dir(cur_dir.clone()).unwrap();

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .feature("feature-1")
            .compile();

        // Assert
        assert!(status.is_ok());

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_compilation_with_feature_and_loglevel() {
        // Arrange
        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";
        cargo_clean(manifest_dir);
        std::env::set_current_dir(cur_dir.clone()).unwrap();

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .feature("feature-1")
            .log_level(Level::Warn)
            .compile();

        // Assert
        assert!(status.is_ok());

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_compilation_fails_with_non_existing_feature() {
        // Arrange
        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";
        cargo_clean(manifest_dir);
        std::env::set_current_dir(cur_dir.clone()).unwrap();

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .feature("feature-2")
            .compile();

        // Assert
        assert!(match status {
            Err(ScryptoCompilerError::CargoBuildFailure(_stderr, exit_status)) =>
                exit_status.code().unwrap() == 101,
            _ => false,
        });

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    fn test_invalid_param() {
        assert!(matches!(
            ScryptoCompiler::new()
                .coverage(true)
                .target_directory("./out")
                .compile(),
            Err(ScryptoCompilerError::InvalidParam(
                ScryptoCompilerInvalidInputParam::CoverageDiscardsTargetDirectory
            ))
        ));

        assert!(matches!(
            ScryptoCompiler::new()
                .coverage(true)
                .force_local_target(true)
                .compile(),
            Err(ScryptoCompilerError::InvalidParam(
                ScryptoCompilerInvalidInputParam::CoverageDiscardsForceLocalTarget
            ))
        ));

        assert!(matches!(
            ScryptoCompiler::new()
                .target_directory("./out")
                .force_local_target(true)
                .compile(),
            Err(ScryptoCompilerError::InvalidParam(
                ScryptoCompilerInvalidInputParam::ForceLocalTargetDiscardsTargetDirectory
            ))
        ));

        assert!(matches!(
            ScryptoCompiler::new()
                .manifest_directory("./Cargo.toml")
                .compile(),
            Err(ScryptoCompilerError::InvalidParam(
                ScryptoCompilerInvalidInputParam::CargoTomlInManifestDirectory
            ))
        ));

        let name = String::from("TEST");
        let result = ScryptoCompiler::new()
            .env(&name, "none")
            .unset_env(&name)
            .compile();
        assert!(match result {
            Err(ScryptoCompilerError::InvalidParam(
                ScryptoCompilerInvalidInputParam::EnvironmentVariableSetAndUnset(error_name),
            )) => error_name == name,
            _ => false,
        });
    }
}
