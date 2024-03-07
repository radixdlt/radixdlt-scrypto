use cargo_toml::Manifest;
use radix_engine_interface::types::Level;
use std::process::{Command, ExitStatus};
use std::{env, ffi::OsStr, io, path::Path, path::PathBuf};

const MANIFEST_FILE: &str = "Cargo.toml";
const BUILD_TARGET: &str = "wasm32-unknown-unknown";

#[derive(Debug)]
pub enum ScryptoCompilerError {
    IOError(io::Error),
    CargoBuildFailure(ExitStatus),
    CargoMetadataFailure(String, ExitStatus),
    CargoTargetDirectoryResolutionError,
    CargoManifestLoadFailure(String),
    InvalidParam(ScryptoCompilerInvalidParam),
    WasmOptimizationError(wasm_opt::OptimizationError),
}

#[derive(Debug)]
pub enum ScryptoCompilerInvalidParam {
    CoverageDiscardsTargetDirectory,
    CoverageDiscardsForceLocalTarget,
    ForceLocalTargetDiscardsTargetDirectory,
    CargoTomlInManifestDirectory,
    EnvironmentVariableSetAndUnset(String),
}

pub struct ScryptoCompiler {
    profile: Profile,
    set_environment_variables: Vec<(String, String)>,
    unset_environment_variables: Vec<String>,
    features: Vec<String>,
    package: Option<String>,
    target_directory: Option<String>,
    manifest_directory: Option<String>,
    trace: bool,
    log_level: Level,
    no_schema: bool,
    coverage: bool,
    force_local_target: bool,
    wasm_optimization: Option<wasm_opt::OptimizationOptions>,

    internal_target_directory: PathBuf,
    internal_manifest_path: PathBuf,
    internal_binary_target_directory: PathBuf,
    internal_binary_path: PathBuf,
}

impl ScryptoCompiler {
    pub fn new() -> ScryptoCompilerBuilder {
        ScryptoCompilerBuilder::default()
    }

    fn validate_input_parameters(&self) -> Result<(), ScryptoCompilerInvalidParam> {
        if self.coverage && self.force_local_target {
            return Err(ScryptoCompilerInvalidParam::CoverageDiscardsForceLocalTarget);
        }
        if self.coverage && self.target_directory.is_some() {
            return Err(ScryptoCompilerInvalidParam::CoverageDiscardsTargetDirectory);
        }
        if self.force_local_target && self.target_directory.is_some() {
            return Err(ScryptoCompilerInvalidParam::ForceLocalTargetDiscardsTargetDirectory);
        }
        if self
            .manifest_directory
            .as_ref()
            .is_some_and(|v| PathBuf::from(v).ends_with(MANIFEST_FILE))
        {
            return Err(ScryptoCompilerInvalidParam::CargoTomlInManifestDirectory);
        }
        if let Some(env) = self.set_environment_variables.iter().find_map(|v| {
            if self.unset_environment_variables.contains(&v.0) {
                Some(v.0.clone())
            } else {
                None
            }
        }) {
            return Err(ScryptoCompilerInvalidParam::EnvironmentVariableSetAndUnset(
                env,
            ));
        }
        Ok(())
    }

    fn prepare_features(&self) -> String {
        // firstly apply user features
        let mut features = self.features.join(",");

        // now apply scrypto features
        if self.trace {
            features.push_str(",scrypto/trace");
        }
        if self.no_schema {
            features.push_str(",scrypto/no-schema");
        }
        if Level::Error <= self.log_level {
            features.push_str(",scrypto/log-error");
        }
        if Level::Warn <= self.log_level {
            features.push_str(",scrypto/log-warn");
        }
        if Level::Info <= self.log_level {
            features.push_str(",scrypto/log-info");
        }
        if Level::Debug <= self.log_level {
            features.push_str(",scrypto/log-debug");
        }
        if Level::Trace <= self.log_level {
            features.push_str(",scrypto/log-trace");
        }
        if self.coverage {
            features.push_str(",scrypto/coverage");
        }

        if features.starts_with(',') {
            features.remove(0);
        }

        features
    }

    fn prepare_rust_flags(&self) -> String {
        if self.coverage {
            "-Clto=off\x1f-Cinstrument-coverage\x1f-Zno-profiler-runtime\x1f--emit=llvm-ir"
                .to_owned()
        } else {
            env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default()
        }
    }

    fn get_default_target_directory(
        manifest_path: impl AsRef<OsStr>,
    ) -> Result<String, ScryptoCompilerError> {
        let output = Command::new("cargo")
            .arg("metadata")
            .arg("--manifest-path")
            .arg(manifest_path.as_ref())
            .arg("--format-version")
            .arg("1")
            .arg("--no-deps")
            .output()
            .map_err(ScryptoCompilerError::IOError)?;
        if output.status.success() {
            let parsed = serde_json::from_slice::<serde_json::Value>(&output.stdout)
                .map_err(|_| ScryptoCompilerError::CargoTargetDirectoryResolutionError)?;
            let target_directory = parsed
                .as_object()
                .and_then(|o| o.get("target_directory"))
                .and_then(|o| o.as_str())
                .ok_or(ScryptoCompilerError::CargoTargetDirectoryResolutionError)?;
            Ok(target_directory.to_owned())
        } else {
            Err(ScryptoCompilerError::CargoMetadataFailure(
                manifest_path.as_ref().to_str().unwrap().to_string(),
                output.status,
            ))
        }
    }

    fn get_binary_name(
        manifest_path: impl AsRef<Path>,
        binary_target_directory: impl AsRef<Path>,
    ) -> Result<PathBuf, ScryptoCompilerError> {
        // Find the binary paths
        let manifest = Manifest::from_path(&manifest_path).map_err(|_| {
            ScryptoCompilerError::CargoManifestLoadFailure(
                manifest_path.as_ref().to_str().unwrap().to_string(),
            )
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
        let mut bin_path = PathBuf::new();
        bin_path.push(binary_target_directory);
        bin_path.push(
            wasm_name.ok_or(ScryptoCompilerError::CargoManifestLoadFailure(
                manifest_path.as_ref().to_str().unwrap().to_string(),
            ))?,
        );
        bin_path.with_extension("wasm");

        Ok(bin_path)
    }

    fn prepare_paths(&mut self) -> Result<(PathBuf, PathBuf), ScryptoCompilerError> {
        let manifest_directory = self
            .manifest_directory
            .as_ref()
            .map_or(env::current_dir().unwrap(), |v| PathBuf::from(v));
        let mut manifest_path = manifest_directory.clone();
        manifest_path.push(MANIFEST_FILE);

        let target_directory = if self.coverage {
            let mut target_path = manifest_directory.clone();
            target_path.push("coverage");
            target_path
        } else if self.force_local_target {
            let mut target_path = manifest_directory;
            target_path.push("target");
            target_path
        } else if let Some(directory) = &self.target_directory {
            PathBuf::from(directory)
        } else {
            PathBuf::from(&Self::get_default_target_directory(&manifest_path)?)
        };

        self.internal_manifest_path = manifest_path.clone();
        self.internal_target_directory = target_directory.clone();

        self.internal_binary_target_directory = self.internal_target_directory.clone();
        self.internal_binary_target_directory.push(BUILD_TARGET);
        self.internal_binary_target_directory
            .push(self.profile.as_directory_name());

        self.internal_binary_path = Self::get_binary_name(&manifest_path, &target_directory)?;

        Ok((manifest_path, target_directory))
    }

    fn prepare_command(&mut self, command: &mut Command) -> Result<(), ScryptoCompilerError> {
        let (manifest_path, target_directory) = self.prepare_paths()?;

        let features_list = self.prepare_features();
        let features = (!features_list.is_empty())
            .then_some(vec!["--features", &features_list])
            .unwrap_or_default();

        let rustflags = self.prepare_rust_flags();

        let package = self
            .package
            .as_ref()
            .and_then(|p| Some(vec!["--package", &p]))
            .unwrap_or_default();

        command
            .arg("build")
            .arg("--target")
            .arg(BUILD_TARGET)
            .arg(self.profile.as_string())
            .arg("--target-dir")
            .arg(target_directory)
            .arg("--manifest-path")
            .arg(manifest_path)
            .args(package)
            .args(features)
            .env("CARGO_ENCODED_RUSTFLAGS", rustflags)
            .envs(self.set_environment_variables.clone());

        self.unset_environment_variables.iter().for_each(|e| {
            command.env_remove(e);
        });

        Ok(())
    }

    fn wasm_optimize(&mut self) -> Result<(), ScryptoCompilerError> {
        if let Some(wasm_opt_config) = &self.wasm_optimization {
            wasm_opt_config
                .run(&self.internal_binary_path, &self.internal_binary_path)
                .map_err(ScryptoCompilerError::WasmOptimizationError)
        } else {
            Ok(())
        }
    }

    pub fn compile(&mut self) -> Result<(), ScryptoCompilerError> {
        // Verify if passed builder parameters are valid
        self.validate_input_parameters()
            .map_err(|e| ScryptoCompilerError::InvalidParam(e))?;

        // Create compilation command
        let mut command = Command::new("cargo");
        self.prepare_command(&mut command)?;

        // Execute command
        let status = command.status().map_err(ScryptoCompilerError::IOError)?;

        status
            .success()
            .then_some(())
            .ok_or(ScryptoCompilerError::CargoBuildFailure(status))?;

        self.wasm_optimize()
    }
}

#[derive(Default, Clone)]
pub enum Profile {
    #[default]
    Release,
    Debug,
}

impl Profile {
    fn as_string(&self) -> String {
        match self {
            Profile::Release => String::from("--release"),
            Profile::Debug => String::new(),
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
    profile: Profile,
    set_environment_variables: Vec<(String, String)>,
    unset_environment_variables: Vec<String>,
    features: Vec<String>,
    package: Option<String>,
    target_directory: Option<String>,
    manifest_directory: Option<String>,
    trace: bool,
    log_level: Level,
    no_schema: bool,
    coverage: bool,
    force_local_target: bool,
    wasm_optimization: Option<wasm_opt::OptimizationOptions>,
}

impl ScryptoCompilerBuilder {
    pub fn profile(&mut self, profile: Profile) -> &mut Self {
        self.profile = profile;
        self
    }

    pub fn env(&mut self, name: &str, value: &str) -> &mut Self {
        self.set_environment_variables
            .push((name.to_string(), value.to_string()));
        self
    }

    pub fn unset_env(&mut self, name: &str) -> &mut Self {
        self.unset_environment_variables.push(name.to_string());
        self
    }

    pub fn feature(&mut self, name: &str) -> &mut Self {
        self.features.push(name.to_string());
        self
    }

    pub fn package(&mut self, name: &str) -> &mut Self {
        self.package = Some(name.to_string());
        self
    }

    pub fn target_directory(&mut self, directory: &str) -> &mut Self {
        self.target_directory = Some(directory.to_string());
        self
    }

    pub fn manifest_directory(&mut self, directory: &str) -> &mut Self {
        self.manifest_directory = Some(directory.to_string());
        self
    }

    pub fn trace(&mut self, trace: bool) -> &mut Self {
        self.trace = trace;
        self
    }

    pub fn log_level(&mut self, log_level: Level) -> &mut Self {
        self.log_level = log_level;
        self
    }

    pub fn no_schema(&mut self, no_schema: bool) -> &mut Self {
        self.no_schema = no_schema;
        self
    }

    pub fn coverage(&mut self, coverage: bool) -> &mut Self {
        self.coverage = coverage;
        self
    }

    pub fn force_local_target(&mut self, local_target: bool) -> &mut Self {
        self.force_local_target = local_target;
        self
    }

    pub fn optimize_with_wasm_opt(&mut self, options: wasm_opt::OptimizationOptions) -> &mut Self {
        self.wasm_optimization = Some(options);
        self
    }

    pub fn compile(&mut self) -> Result<(), ScryptoCompilerError> {
        let mut compiler = ScryptoCompiler {
            profile: self.profile.clone(),
            set_environment_variables: self.set_environment_variables.to_owned(),
            unset_environment_variables: self.unset_environment_variables.to_owned(),
            features: self.features.to_owned(),
            package: self.package.clone(),
            target_directory: self.target_directory.clone(),
            manifest_directory: self.manifest_directory.clone(),
            trace: self.trace,
            log_level: self.log_level,
            no_schema: self.no_schema,
            coverage: self.coverage,
            force_local_target: self.force_local_target,
            wasm_optimization: self.wasm_optimization.to_owned(),
            internal_target_directory: PathBuf::new(),
            internal_manifest_path: PathBuf::new(),
            internal_binary_target_directory: PathBuf::new(),
            internal_binary_path: PathBuf::new(),
        };
        compiler.compile()
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
    fn test_compilation_faucet() {
        // Arrange
        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "../assets/blueprints/faucet";
        cargo_clean(manifest_dir);
        std::env::set_current_dir(cur_dir.clone()).unwrap();

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .compile();

        if status.is_err() {
            println!("{:?}", status);
        }

        // Assert
        assert!(status.is_ok());

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_compilation_current_dir_faucet() {
        // Arrange
        let cur_dir = std::env::current_dir().unwrap();
        let manifest_dir = "../assets/blueprints/faucet";

        println!("CUR DIR: {}", std::env::current_dir().unwrap().display());

        // change current directory to fauce blueprint
        std::env::set_current_dir(manifest_dir).unwrap();

        cargo_clean("./");
        println!("CUR DIR: {}", std::env::current_dir().unwrap().display());

        // Act
        // Compile project in current directory without specyfing manifest path
        let status = ScryptoCompiler::new().compile();

        // Assert
        assert!(status.is_ok());

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
                ScryptoCompilerInvalidParam::CoverageDiscardsTargetDirectory
            ))
        ));

        assert!(matches!(
            ScryptoCompiler::new()
                .coverage(true)
                .force_local_target(true)
                .compile(),
            Err(ScryptoCompilerError::InvalidParam(
                ScryptoCompilerInvalidParam::CoverageDiscardsForceLocalTarget
            ))
        ));

        assert!(matches!(
            ScryptoCompiler::new()
                .target_directory("./out")
                .force_local_target(true)
                .compile(),
            Err(ScryptoCompilerError::InvalidParam(
                ScryptoCompilerInvalidParam::ForceLocalTargetDiscardsTargetDirectory
            ))
        ));

        assert!(matches!(
            ScryptoCompiler::new()
                .manifest_directory("./Cargo.toml")
                .compile(),
            Err(ScryptoCompilerError::InvalidParam(
                ScryptoCompilerInvalidParam::CargoTomlInManifestDirectory
            ))
        ));

        let name = String::from("TEST");
        let result = ScryptoCompiler::new()
            .env(&name, "none")
            .unset_env(&name)
            .compile();
        assert!(match result {
            Err(ScryptoCompilerError::InvalidParam(
                ScryptoCompilerInvalidParam::EnvironmentVariableSetAndUnset(error_name),
            )) => error_name == name,
            _ => false,
        });
    }
}
