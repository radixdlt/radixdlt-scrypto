use cargo_toml::Manifest;
use radix_engine::utils::{extract_definition, ExtractSchemaError};
use radix_engine_interface::{blueprints::package::PackageDefinition, types::Level};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::{env, io};
use utils::prelude::IndexMap;

const MANIFEST_FILE: &str = "Cargo.toml";
const BUILD_TARGET: &str = "wasm32-unknown-unknown";

#[derive(Debug)]
pub enum ScryptoCompilerError {
    /// Returns IO Error which occured during compilation
    IOError(io::Error),
    /// Returns output from stderr and process exit status
    CargoBuildFailure(String, ExitStatus),
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
    /// Returns error occured during schema extraction
    ExtractSchema(ExtractSchemaError),
    /// Specified manifest is a workspace, use 'compile_workspace' function
    CargoManifestIsWorkspace(String),
    /// Specified manifest which is not a workspace
    CargoManifestNoWorkspace(String),
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
    environment_variables: IndexMap<String, EnvironmentVariableAction>,
    features: Vec<String>,
    package: Vec<String>,
    target_directory: Option<PathBuf>,
    manifest_directory: Option<PathBuf>,
    trace: bool,
    log_level: Level,
    no_schema: bool,
    wasm_optimization: Option<wasm_opt::OptimizationOptions>,
}

#[derive(Default, Clone)]
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
    fn as_directory_name(&self) -> String {
        match self {
            Profile::Release => String::from("release"),
            Profile::Debug => String::from("debug"),
            Profile::Test => String::from("test"),
            Profile::Bench => String::from("bench"),
            Profile::Custom(name) => name.clone(),
        }
    }
}

#[derive(Clone)]
pub enum EnvironmentVariableAction {
    Set(String),
    Unset,
}

#[derive(Clone)]
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
        if input_params
            .manifest_directory
            .as_ref()
            .is_some_and(|v| PathBuf::from(v).ends_with(MANIFEST_FILE))
        {
            return Err(ScryptoCompilerInvalidInputParam::CargoTomlInManifestDirectory);
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
        // if self.input_params.coverage {
        //     features.push_str(",scrypto/coverage");
        // }

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
        env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default()
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

    fn get_workspace_members(manifest_path: &Path) -> Result<Vec<String>, ScryptoCompilerError> {
        let manifest = Manifest::from_path(&manifest_path).map_err(|_| {
            ScryptoCompilerError::CargoManifestLoadFailure(manifest_path.display().to_string())
        })?;
        if let Some(workspace) = manifest.workspace {
            Ok(workspace.members)
        } else {
            Err(ScryptoCompilerError::CargoManifestNoWorkspace(
                manifest_path.display().to_string(),
            ))
        }
    }

    fn get_manifest_path(
        input_params: &ScryptoCompilerInputParams,
    ) -> Result<PathBuf, ScryptoCompilerError> {
        let manifest_directory = input_params
            .manifest_directory
            .as_ref()
            .map_or(env::current_dir().unwrap(), |v| PathBuf::from(v));
        let mut manifest_path = manifest_directory.clone();
        manifest_path.push(MANIFEST_FILE);

        if !manifest_path.exists() {
            Err(ScryptoCompilerError::CargoManifestFileNotFound(
                manifest_path.display().to_string(),
            ))
        } else {
            Ok(manifest_path)
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
        if manifest.workspace.is_some() && !manifest.workspace.unwrap().members.is_empty() {
            return Err(ScryptoCompilerError::CargoManifestIsWorkspace(
                manifest_path.display().to_string(),
            ));
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
        let manifest_path = Self::get_manifest_path(input_params)?;
        let mut manifest_directory = manifest_path.clone();
        manifest_directory.pop();

        // Generate target directory
        let target_directory = if let Some(directory) = &input_params.target_directory {
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

        let package: Vec<&str> = self
            .input_params
            .package
            .iter()
            .map(|p| vec!["--package", p])
            .flatten()
            .collect();

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
            .env("CARGO_ENCODED_RUSTFLAGS", rustflags);

        self.input_params
            .environment_variables
            .iter()
            .for_each(|(name, action)| {
                match action {
                    EnvironmentVariableAction::Set(value) => command.env(name, value),
                    EnvironmentVariableAction::Unset => command.env_remove(name),
                };
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
                String::from_utf8(output.stderr.clone()).unwrap_or(format!("{:?}", output.stderr)),
                output.status,
            ))?;

        self.wasm_optimize()?;

        Ok(self.target_binary_path.clone())
    }

    pub fn target_binary_path(&self) -> PathBuf {
        self.target_binary_path.clone()
    }

    pub fn extract_schema_from_wasm(
        &self,
    ) -> Result<(Vec<u8>, PackageDefinition), ScryptoCompilerError> {
        let code = std::fs::read(&self.target_binary_path)
            .map_err(|e| ScryptoCompilerError::IOError(e))?;
        let definition =
            extract_definition(&code).map_err(|e| ScryptoCompilerError::ExtractSchema(e))?;
        Ok((code, definition))
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

    pub fn env(&mut self, name: &str, action: EnvironmentVariableAction) -> &mut Self {
        self.input_params
            .environment_variables
            .insert(name.to_string(), action);
        self
    }

    pub fn feature(&mut self, name: &str) -> &mut Self {
        self.input_params.features.push(name.to_string());
        self
    }

    pub fn package(&mut self, name: &str) -> &mut Self {
        self.input_params.package.push(name.to_string());
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

    pub fn coverage(&mut self) -> &mut Self {
        self.input_params
            .features
            .push(String::from("scrypto/coverage"));
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

    pub fn compile_workspace(&mut self) -> Result<Vec<PathBuf>, ScryptoCompilerError> {
        let manifest_path = ScryptoCompiler::get_manifest_path(&self.input_params)?;

        let members = ScryptoCompiler::get_workspace_members(&manifest_path)?;

        let mut result: Vec<PathBuf> = Vec::new();
        for member in members {
            let mut new_input_params = self.input_params.clone();
            if let Some(md) = new_input_params.manifest_directory.as_mut() {
                md.push(member);
            } else {
                new_input_params.manifest_directory = Some(member.into());
            }
            result.push(ScryptoCompiler::from_input_params(&new_input_params)?.compile()?);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    static SERIAL_COMPILE_MUTEX: Lazy<Mutex<()>> = Lazy::new(Mutex::default);

    fn cargo_clean(manifest_dir: &str) {
        Command::new("cargo")
            .arg("clean")
            .arg("--manifest-path")
            .arg(manifest_dir.to_owned() + "/Cargo.toml")
            .output()
            .unwrap();
    }

    #[test]
    fn test_compilation() {
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

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
    fn test_compilation_in_current_dir() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

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
    fn test_compilation_env_var() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";

        cargo_clean(manifest_dir);
        std::env::set_current_dir(cur_dir.clone()).unwrap();

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .env("TEST", EnvironmentVariableAction::Set(String::from("1")))
            .env("OTHER", EnvironmentVariableAction::Unset)
            .compile();

        // Assert
        assert!(status.is_ok());

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    #[ignore]
    fn test_compilation_coverage() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";

        cargo_clean(manifest_dir);
        std::env::set_current_dir(cur_dir.clone()).unwrap();

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .coverage()
            .compile();

        // Assert
        assert!(status.is_ok());

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    fn test_compilation_with_feature() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

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
    fn test_compilation_with_feature_and_loglevel() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

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
    fn test_compilation_fails_with_non_existing_feature() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

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
    fn test_compilation_workspace() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets";

        cargo_clean(manifest_dir);

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .compile_workspace();

        // Assert
        assert!(status.is_ok());

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    fn test_compilation_workspace_in_current_dir() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets";

        cargo_clean(manifest_dir);
        std::env::set_current_dir(manifest_dir).unwrap();

        // Act
        let status = ScryptoCompiler::new().compile_workspace();

        // Assert
        assert!(status.is_ok());

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    fn test_compilation_workspace_fail_on_wrong_method() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets";

        cargo_clean(manifest_dir);
        std::env::set_current_dir(manifest_dir).unwrap();

        // Act
        let status = ScryptoCompiler::new().compile();

        // Assert
        assert!(matches!(
            status,
            Err(ScryptoCompilerError::CargoManifestIsWorkspace(..))
        ));

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    fn test_compilation_profile_release() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";

        cargo_clean(manifest_dir);

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .profile(Profile::Release)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    fn test_compilation_profile_debug() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";

        cargo_clean(manifest_dir);

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .profile(Profile::Debug)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    fn test_compilation_profile_test() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";

        cargo_clean(manifest_dir);

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .profile(Profile::Test)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    fn test_compilation_profile_bench() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";

        cargo_clean(manifest_dir);

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .profile(Profile::Bench)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    fn test_compilation_profile_custom() {
        // Arrange
        let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "./tests/assets/blueprint";

        cargo_clean(manifest_dir);

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .profile(Profile::Custom(String::from("custom")))
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    fn test_invalid_param() {
        assert!(matches!(
            ScryptoCompiler::new()
                .manifest_directory("./Cargo.toml")
                .compile(),
            Err(ScryptoCompilerError::InvalidParam(
                ScryptoCompilerInvalidInputParam::CargoTomlInManifestDirectory
            ))
        ));
    }

    #[test]
    fn test_target_binary_path() {
        let output_path =
            PathBuf::from("tests/assets/target/wasm32-unknown-unknown/release/test_blueprint.wasm");
        let package_dir = "./tests/assets/blueprint";
        let compiler = ScryptoCompiler::new()
            .manifest_directory(package_dir)
            .build()
            .unwrap();

        let absolute_path = compiler.target_binary_path();
        let skip_count = absolute_path.iter().count() - output_path.iter().count();
        let relative_path: PathBuf = absolute_path.iter().skip(skip_count).collect();

        assert_eq!(relative_path, output_path);
    }

    #[test]
    fn test_target_binary_path_target() {
        let target_dir = "./tests/target";
        let compiler = ScryptoCompiler::new()
            .manifest_directory("./tests/assets/blueprint")
            .target_directory(target_dir)
            .build()
            .unwrap();

        assert_eq!(
            "./tests/target/wasm32-unknown-unknown/release/test_blueprint.wasm",
            compiler.target_binary_path().display().to_string()
        );
    }
}
