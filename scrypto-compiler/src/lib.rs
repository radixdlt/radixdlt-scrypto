use cargo_toml::Manifest;
use radix_engine::utils::{extract_definition, ExtractSchemaError};
use radix_engine_interface::{blueprints::package::PackageDefinition, types::Level};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::{env, io};
use utils::prelude::{IndexMap, IndexSet};

const MANIFEST_FILE: &str = "Cargo.toml";
const BUILD_TARGET: &str = "wasm32-unknown-unknown";
const SCRYPTO_NO_SCHEMA: &str = "scrypto/no-schema";

#[derive(Debug)]
pub enum ScryptoCompilerError {
    /// Returns IO Error which occurred during compilation and if possible some context information
    IOError(io::Error, Option<String>),
    /// Returns process exit status in case of 'cargo build' fail
    CargoBuildFailure(ExitStatus),
    /// Returns path to Cargo.toml for which cargo metadata command failed and process exit status
    CargoMetadataFailure(String, ExitStatus),
    /// Returns path to Cargo.toml for which results of cargo metadata command is not not valid json or target directory field is missing
    CargoTargetDirectoryResolutionError(String),
    /// Returns path to Cargo.toml which was failed to load
    CargoManifestLoadFailure(String),
    /// Returns path to Cargo.toml which cannot be found
    CargoManifestFileNotFound(String),
    /// Returns WASM Optimization error
    WasmOptimizationError(wasm_opt::OptimizationError),
    /// Returns error occured during schema extraction
    ExtractSchema(ExtractSchemaError),
    /// Specified manifest is a workspace, use 'compile_workspace' function
    CargoManifestIsWorkspace(String),
    /// Specified manifest which is not a workspace
    CargoManifestNoWorkspace(String),
}

#[derive(Clone, Default)]
pub struct ScryptoCompilerInputParams {
    /// Path to Cargo.toml file, if not specified current directory will be used.
    pub manifest_path: Option<PathBuf>,
    /// Path to directory where compilation artifacts are stored, if not specified default location will by used.
    pub target_directory: Option<PathBuf>,
    /// Compilation profile. If not specified default profile: Release will be used.
    pub profile: Profile,
    /// List of environment variables to set or unest during compilation. Optional field.
    pub environment_variables: IndexMap<String, EnvironmentVariableAction>,
    /// List of features, used for 'cargo build --features'. Optional field.
    pub features: IndexSet<String>,
    /// If set to true then '--no-default-features' option is passed to 'cargo build'. Defult value is false.
    pub no_default_features: bool,
    /// If set to true then '--all-features' option is passed to 'cargo build'. Defult value is false.
    pub all_features: bool,
    /// List of packages to compile, used for 'cargo build --package'. Optional field.
    pub package: IndexSet<String>,
    /// If optimizations are specified they will by applied after compilation.
    pub wasm_optimization: Option<wasm_opt::OptimizationOptions>,
    /// List of custom options, passed as 'cargo build' arguments without any modifications. Optional field.
    /// Add each option as separate entry (for instance: '-j 1' must be added as two entires: '-j' and '1' one by one).
    pub custom_options: IndexSet<String>,
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

#[derive(Clone)]
pub enum EnvironmentVariableAction {
    Set(String),
    Unset,
}

#[derive(Debug)]
pub struct BuildArtifacts {
    pub wasm: BuildArtifact<Vec<u8>>,
    pub package_definition: BuildArtifact<PackageDefinition>,
}

#[derive(Debug)]
pub struct BuildArtifact<T> {
    pub path: PathBuf,
    pub content: T,
}

#[derive(Clone)]
pub struct ScryptoCompiler {
    /// Scrypto compiler input parameters.
    input_params: ScryptoCompilerInputParams,
    /// Path to Cargo.toml file. If specified in input_params it has the same value, otherwise it is generated.
    manifest_path: PathBuf,
    /// Path to directory where compilation artifacts are stored. If specified in input_params it has the same value,
    /// otherwise it is generated.
    target_directory: PathBuf,
    /// Path to target binary WASM file.
    target_binary_wasm_path: PathBuf,
    /// Path to target binary RPD file.
    target_binary_rpd_path: PathBuf,
}

impl ScryptoCompiler {
    pub fn builder() -> ScryptoCompilerBuilder {
        ScryptoCompilerBuilder::default()
    }

    // Internal constructor
    fn from_input_params(
        input_params: &ScryptoCompilerInputParams,
    ) -> Result<Self, ScryptoCompilerError> {
        // Firstly validate input parameters
        ScryptoCompiler::validate_input_parameters(input_params)?;
        // Secondly prepare internally used path basing on input parameters
        let (manifest_path, target_directory, target_binary_wasm_path, target_binary_rpd_path) =
            ScryptoCompiler::prepare_paths(input_params)?;
        // Lastly create ScryptoCompiler object
        Ok(Self {
            input_params: input_params.to_owned(),
            manifest_path,
            target_directory,
            target_binary_wasm_path,
            target_binary_rpd_path,
        })
    }

    fn validate_input_parameters(
        _input_params: &ScryptoCompilerInputParams,
    ) -> Result<(), ScryptoCompilerError> {
        Ok(())
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
                ScryptoCompilerError::IOError(
                    e,
                    Some(format!(
                        "Cargo metadata for manifest failed: {}",
                        manifest_path.display().to_string()
                    )),
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
                manifest_path.display().to_string(),
                output.status,
            ))
        }
    }

    // fn get_workspace_members(manifest_path: &Path) -> Result<Vec<String>, ScryptoCompilerError> {
    //     let manifest = Manifest::from_path(&manifest_path).map_err(|_| {
    //         ScryptoCompilerError::CargoManifestLoadFailure(manifest_path.display().to_string())
    //     })?;
    //     if let Some(workspace) = manifest.workspace {
    //         Ok(workspace.members)
    //     } else {
    //         Err(ScryptoCompilerError::CargoManifestNoWorkspace(
    //             manifest_path.display().to_string(),
    //         ))
    //     }
    // }

    // Returns path to Cargo.toml (including the file)
    fn get_manifest_path(
        input_params: &ScryptoCompilerInputParams,
    ) -> Result<PathBuf, ScryptoCompilerError> {
        let manifest_path = match input_params.manifest_path.clone() {
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
                        Some(String::from("Getting current directory failed")),
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

    fn get_target_binary_name(
        manifest_path: &Path,
        //binary_target_directory: &Path,
    ) -> Result<String, ScryptoCompilerError> {
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
        // Merge the name with binary target directory
        // let mut bin_path: PathBuf = binary_target_directory.into();
        // bin_path.push(
        //     wasm_name.ok_or(ScryptoCompilerError::CargoManifestLoadFailure(
        //         manifest_path.display().to_string(),
        //     ))?,
        // );
        // bin_path.set_extension("wasm");

        Ok(wasm_name.unwrap()) //todo
    }

    // Returns manifest path, target directory, target binary WASM path and target binary PRD path
    fn prepare_paths(
        input_params: &ScryptoCompilerInputParams,
    ) -> Result<(PathBuf, PathBuf, PathBuf, PathBuf), ScryptoCompilerError> {
        // Generate manifest path (manifest directory + "/Cargo.toml")
        let manifest_path = Self::get_manifest_path(input_params)?;

        // Generate target directory
        let target_directory = if let Some(directory) = &input_params.target_directory {
            // If target directory is explicitly specified as compiler parameter then use it as is
            PathBuf::from(directory)
        } else {
            // If target directory is not specified as compiler parameter then get default
            // target directory basing on manifest file
            PathBuf::from(&Self::get_default_target_directory(&manifest_path)?)
        };

        let target_binary_name = Self::get_target_binary_name(&manifest_path)?;

        let mut target_binary_wasm_path = target_directory.clone();
        target_binary_wasm_path.push(BUILD_TARGET);
        target_binary_wasm_path.push(input_params.profile.as_target_directory_name());
        target_binary_wasm_path.push(target_binary_name.clone());
        target_binary_wasm_path.set_extension("wasm");

        let mut target_binary_rpd_path = target_directory.clone();
        target_binary_rpd_path.push(BUILD_TARGET);
        target_binary_rpd_path.push(Profile::Release.as_target_directory_name());
        target_binary_rpd_path.push(target_binary_name);
        target_binary_rpd_path.set_extension("rpd");

        Ok((
            manifest_path,
            target_directory,
            target_binary_wasm_path,
            target_binary_rpd_path,
        ))
    }

    fn prepare_command(
        &mut self,
        command: &mut Command,
        for_package_extract: bool,
    ) -> Result<(), ScryptoCompilerError> {
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
            if !for_package_extract {
                features.remove(idx);
            }
        } else if for_package_extract {
            features.push(["--features", SCRYPTO_NO_SCHEMA]);
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
            .arg(&self.target_directory)
            .arg("--manifest-path")
            .arg(&self.manifest_path)
            .args(package)
            .args(features);

        if for_package_extract {
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

        self.input_params
            .environment_variables
            .iter()
            .for_each(|(name, action)| {
                match action {
                    EnvironmentVariableAction::Set(value) => command.env(name, value),
                    EnvironmentVariableAction::Unset => command.env_remove(name),
                };
            });

        command.args(self.input_params.custom_options.iter());

        Ok(())
    }

    fn wasm_optimize(&mut self, wasm_path: &Path) -> Result<(), ScryptoCompilerError> {
        if let Some(wasm_opt_config) = &self.input_params.wasm_optimization {
            wasm_opt_config
                .run(wasm_path, wasm_path)
                .map_err(ScryptoCompilerError::WasmOptimizationError)
        } else {
            Ok(())
        }
    }

    pub fn compile_with_stdio<T: Into<Stdio>>(
        &mut self,
        stdin: Option<T>,
        stdout: Option<T>,
        stderr: Option<T>,
    ) -> Result<BuildArtifacts, ScryptoCompilerError> {
        let package_definition = self.compile_internal_phase_1()?;

        let mut command = Command::new("cargo");
        if let Some(s) = stdin {
            command.stdin(s);
        }
        if let Some(s) = stdout {
            command.stdout(s);
        }
        if let Some(s) = stderr {
            command.stderr(s);
        }
        let wasm = self.compile_internal_phase_2(&mut command)?;

        Ok(BuildArtifacts {
            wasm,
            package_definition,
        })
    }

    pub fn compile(&mut self) -> Result<BuildArtifacts, ScryptoCompilerError> {
        let package_definition = self.compile_internal_phase_1()?;

        let mut command = Command::new("cargo");
        let wasm = self.compile_internal_phase_2(&mut command)?;

        Ok(BuildArtifacts {
            wasm,
            package_definition,
        })
    }

    // 1st compilation phase: compile with schema and extract schema to .rpd file
    fn compile_internal_phase_1(
        &mut self,
    ) -> Result<BuildArtifact<PackageDefinition>, ScryptoCompilerError> {
        let mut command = Command::new("cargo");

        self.prepare_command(&mut command, true)?; // build with schema and release profile
        self.cargo_command_call(&mut command)?;

        let path = self.target_binary_rpd_path.with_extension("wasm");
        let code = std::fs::read(&path).map_err(|e| {
            ScryptoCompilerError::IOError(
                e,
                Some(format!(
                    "Read WASM file for RPD extract failed: {}",
                    path.display().to_string()
                )),
            )
        })?;

        let package_definition =
            extract_definition(&code).map_err(|e| ScryptoCompilerError::ExtractSchema(e))?;

        Ok(BuildArtifact {
            path: self.target_binary_rpd_path.clone(),
            content: package_definition,
        })
    }

    // 2nd compilation phase: compile without schema and with optional wasm optimisations - this is the final .wasm file
    fn compile_internal_phase_2(
        &mut self,
        command: &mut Command,
    ) -> Result<BuildArtifact<Vec<u8>>, ScryptoCompilerError> {
        self.prepare_command(command, false)?; // build without schema and user choosen profile
        self.cargo_command_call(command)?;

        self.wasm_optimize(&self.target_binary_wasm_path.clone())?;

        let code = std::fs::read(&self.target_binary_wasm_path).map_err(|e| {
            ScryptoCompilerError::IOError(
                e,
                Some(format!(
                    "Read WASM file failed: {}",
                    self.target_binary_wasm_path.display().to_string()
                )),
            )
        })?;

        Ok(BuildArtifact {
            path: self.target_binary_wasm_path.clone(),
            content: code,
        })
    }

    fn cargo_command_call(&mut self, command: &mut Command) -> Result<(), ScryptoCompilerError> {
        let status = command.status().map_err(|e| {
            ScryptoCompilerError::IOError(e, Some(String::from("Cargo build command failed")))
        })?;
        status
            .success()
            .then_some(())
            .ok_or(ScryptoCompilerError::CargoBuildFailure(status))
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

    pub fn no_schema(&mut self) -> &mut Self {
        self.input_params
            .features
            .insert(String::from(SCRYPTO_NO_SCHEMA));
        self
    }

    pub fn coverage(&mut self) -> &mut Self {
        self.input_params
            .features
            .insert(String::from("scrypto/coverage"));
        self
    }

    pub fn optimize_with_wasm_opt(&mut self, options: &wasm_opt::OptimizationOptions) -> &mut Self {
        self.input_params.wasm_optimization = Some(options.to_owned());
        self
    }

    pub fn custom_options(&mut self, options: &[&str]) -> &mut Self {
        self.input_params
            .custom_options
            .extend(options.iter().map(|item| item.to_string()));
        self
    }

    pub fn build(&mut self) -> Result<ScryptoCompiler, ScryptoCompilerError> {
        ScryptoCompiler::from_input_params(&self.input_params)
    }

    // Returns output wasm file path
    pub fn compile(&mut self) -> Result<BuildArtifacts, ScryptoCompilerError> {
        self.build()?.compile()
    }

    // Returns output wasm file path
    pub fn compile_with_stdio<T: Into<Stdio>>(
        &mut self,
        stdin: Option<T>,
        stdout: Option<T>,
        stderr: Option<T>,
    ) -> Result<BuildArtifacts, ScryptoCompilerError> {
        self.build()?.compile_with_stdio(stdin, stdout, stderr)
    }

    // pub fn compile_workspace(&mut self) -> Result<Vec<PathBuf>, ScryptoCompilerError> {
    //     let manifest_path = ScryptoCompiler::get_manifest_path(&self.input_params)?;

    //     let members = ScryptoCompiler::get_workspace_members(&manifest_path)?;

    //     let mut result: Vec<PathBuf> = Vec::new();
    //     for member in members {
    //         let mut new_input_params = self.input_params.clone();
    //         if let Some(md) = new_input_params.manifest_path.as_mut() {
    //             md.push(member);
    //         } else {
    //             new_input_params.manifest_path = Some(member.into());
    //         }
    //         result.push(ScryptoCompiler::from_input_params(&new_input_params)?.compile()?);
    //     }
    //     Ok(result)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    fn prepare() -> (PathBuf, TempDir) {
        let mut test_assets_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_assets_path.extend(["tests", "assets", "blueprint", "Cargo.toml"]);
        (
            test_assets_path,
            TempDir::new("scrypto-compiler-test").unwrap(),
        )
    }

    #[test]
    fn test_compilation() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();
        println!(
            "{:?}\n{:?}",
            target_directory.path(),
            blueprint_manifest_path
        );
        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_in_current_dir() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        let mut package_directory = blueprint_manifest_path.clone();
        package_directory.pop(); // Remove Cargo.toml from path
        std::env::set_current_dir(package_directory).unwrap();

        // Act
        let status = ScryptoCompiler::builder()
            .target_directory(target_directory.path())
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_env_var() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .env("TEST", EnvironmentVariableAction::Set(String::from("1 1")))
            .env("OTHER", EnvironmentVariableAction::Unset)
            .env(
                "RUSTFLAGS",
                EnvironmentVariableAction::Set(String::from("-C opt-level=0")),
            )
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_with_feature() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .feature("feature-1")
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_with_feature_and_loglevel() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .feature("feature-1")
            .log_level(Level::Warn)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_fails_with_non_existing_feature() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .feature("feature-2")
            .compile();

        // Assert
        assert!(match status {
            Err(ScryptoCompilerError::CargoBuildFailure(exit_status)) =>
                exit_status.code().unwrap() == 101,
            _ => false,
        });
    }

    #[test]
    fn test_compilation_workspace() {
        // Arrange
        /*let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

        let cur_dir = std::env::current_dir().unwrap();
        let manifest_path = "./tests/assets";

        cargo_clean(manifest_path);

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(manifest_path)
            .compile_workspace();

        // Assert
        assert!(status.is_ok(), "{:?}", status);

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();*/
    }

    #[test]
    fn test_compilation_workspace_in_current_dir() {
        // Arrange
        /*let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

        let cur_dir = std::env::current_dir().unwrap();
        let manifest_path = "./tests/assets";

        cargo_clean(manifest_path);
        std::env::set_current_dir(manifest_path).unwrap();

        // Act
        let status = ScryptoCompiler::builder().compile_workspace();

        // Assert
        assert!(status.is_ok(), "{:?}", status);

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();*/
    }

    #[test]
    fn test_compilation_workspace_fail_on_wrong_method() {
        // Arrange
        /*let _shared = SERIAL_COMPILE_MUTEX.lock().unwrap();

        let cur_dir = std::env::current_dir().unwrap();
        let manifest_path = "./tests/assets";

        cargo_clean(manifest_path);
        std::env::set_current_dir(manifest_path).unwrap();

        // Act
        let status = ScryptoCompiler::builder().compile();

        // Assert
        assert!(matches!(
            status,
            Err(ScryptoCompilerError::CargoManifestIsWorkspace(..))
        ));

        // Restore current directory
        std::env::set_current_dir(cur_dir).unwrap();*/
    }

    #[test]
    fn test_compilation_profile_release() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .profile(Profile::Release)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_profile_debug() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .profile(Profile::Debug)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_profile_test() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .profile(Profile::Test)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_profile_bench() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .profile(Profile::Bench)
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_profile_custom() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .profile(Profile::Custom(String::from("custom")))
            .compile();

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_compilation_with_stdio() {
        // Arrange
        let (blueprint_manifest_path, target_directory) = prepare();

        // Act
        let status = ScryptoCompiler::builder()
            .manifest_path(blueprint_manifest_path)
            .target_directory(target_directory.path())
            .compile_with_stdio(Some(Stdio::piped()), Some(Stdio::null()), None);

        // Assert
        assert!(status.is_ok(), "{:?}", status);
    }

    #[test]
    fn test_target_binary_path() {
        let output_path =
            PathBuf::from("tests/assets/target/wasm32-unknown-unknown/release/test_blueprint.wasm");
        let package_dir = "./tests/assets/blueprint";
        let compiler = ScryptoCompiler::builder()
            .manifest_path(package_dir)
            .build()
            .unwrap();

        let absolute_path = compiler.target_binary_wasm_path;
        let skip_count = absolute_path.iter().count() - output_path.iter().count();
        let relative_path: PathBuf = absolute_path.iter().skip(skip_count).collect();

        assert_eq!(relative_path, output_path);
    }

    #[test]
    fn test_target_binary_path_target() {
        let target_dir = "./tests/target";
        let compiler = ScryptoCompiler::builder()
            .manifest_path("./tests/assets/blueprint")
            .target_directory(target_dir)
            .custom_options(&["-j", "1"])
            .build()
            .unwrap();

        assert_eq!(
            "./tests/target/wasm32-unknown-unknown/release/test_blueprint.wasm",
            compiler.target_binary_wasm_path.display().to_string()
        );
    }
}
