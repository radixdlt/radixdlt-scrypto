use radix_engine_interface::types::Level;
use std::env;
use std::ffi::OsStr;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

const MANIFEST_FILE: &str = "Cargo.toml";

#[derive(Debug)]
pub enum ScryptoCompilerError {
    IOError(io::Error),
    CargoBuildFailure(ExitStatus),
    CargoMetadataFailure(ExitStatus),
    CargoTargetDirectoryResolutionError,
    InvalidParam(ScryptoCompilerInvalidParam),
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
            Err(ScryptoCompilerError::CargoMetadataFailure(output.status))
        }
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
            .arg("wasm32-unknown-unknown")
            .arg(self.profile.clone().as_string())
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
            .ok_or(ScryptoCompilerError::CargoBuildFailure(status))
    }
}

#[derive(Default, Clone)]
pub enum Profile {
    #[default]
    Release,
    Debug,
}

impl Profile {
    fn as_string(self) -> String {
        match self {
            Profile::Release => String::from("--release"),
            Profile::Debug => String::new(),
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

    // pub fn optimize_size_with_wasm_opt(&mut self, WasmOptConfig::default()) -> &mut Self {
    //     self
    // }

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
        };
        compiler.compile()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cargo_clean(manifest_dir: &str) {
        Command::new("cargo")
            .arg("clean")
            .arg("--manifest-path")
            .arg(manifest_dir.to_owned() + "/Cargo.toml")
            .output()
            .unwrap();
    }

    #[test]
    fn test_compilation_faucet() {
        // Arrange
        let manifest_dir = "../assets/blueprints/faucet";

        cargo_clean(manifest_dir);

        // Act
        let status = ScryptoCompiler::new()
            .manifest_directory(manifest_dir)
            .compile();

        // Assert
        assert!(status.is_ok())
    }

    #[test]
    fn test_compilation_current_dir_faucet() {
        // Arrange
        let manifest_dir = "../assets/blueprints/faucet";

        // change current directory to fauce blueprint
        std::env::set_current_dir(manifest_dir).unwrap();

        cargo_clean("./");

        // Act
        // Compile project in current directory without specyfing manifest path
        let status = ScryptoCompiler::new().compile();

        // Assert
        assert!(status.is_ok())
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
