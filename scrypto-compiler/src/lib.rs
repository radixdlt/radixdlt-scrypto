use radix_engine_interface::types::Level;
use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

#[derive(Debug)]
pub enum ScryptoCompilerError {
    IOError(io::Error),
    CargoFailure(ExitStatus),
}

pub struct ScryptoCompiler {
    profile: Profile,
    set_environment_variables: Vec<(String, String)>,
    unset_environment_variables: Vec<String>,
    features: Vec<String>,
    package: Option<String>,
    target_directory: Option<String>,
    manifest_directory: Option<String>,
    tracing: bool,
    log_level: Level,
    no_schema: bool,
    coverage: bool,
    force_local_target: bool,
}

impl ScryptoCompiler {
    pub fn new() -> ScryptoCompilerBuilder {
        ScryptoCompilerBuilder::default()
    }

    pub fn prepare_features(&self) -> String {
        self.features.join(",")
    }

    pub fn prepare_rust_flags(&self) -> String {
        if self.coverage {
            "-Clto=off\x1f-Cinstrument-coverage\x1f-Zno-profiler-runtime\x1f--emit=llvm-ir"
                .to_owned()
        } else {
            env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default()
        }
    }

    pub fn compile(&mut self) -> Result<(), ScryptoCompilerError> {
        //manifest_path: impl AsRef<OsStr>,

        let target_directory = self
            .target_directory
            .as_ref()
            .map_or(env::current_dir().unwrap(), |v| PathBuf::from(v));
        let mut manifest_path = self
            .manifest_directory
            .as_ref()
            .map_or(env::current_dir().unwrap(), |v| PathBuf::from(v));
        manifest_path.push("Cargo.toml");
        let features = self.prepare_features();
        let rustflags = self.prepare_rust_flags();

        let status = Command::new("cargo")
            .arg("build")
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .arg(self.profile.clone().as_string())
            .arg("--target-dir")
            .arg(target_directory)
            .arg("--manifest-path")
            .arg(manifest_path)
            .args(if features.is_empty() {
                vec![]
            } else {
                vec!["--features", &features]
            })
            .env("CARGO_ENCODED_RUSTFLAGS", rustflags)
            .status()
            .map_err(ScryptoCompilerError::IOError)?;

        status
            .success()
            .then_some(())
            .ok_or(ScryptoCompilerError::CargoFailure(status))
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
    tracing: bool,
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

    pub fn tracing(&mut self, tracing: bool) -> &mut Self {
        self.tracing = tracing;
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
            tracing: self.tracing,
            log_level: self.log_level,
            no_schema: self.no_schema,
            coverage: self.coverage,
            force_local_target: self.force_local_target,
        };
        compiler.compile()
    }
}

#[test]
fn test_builder() {
    ScryptoCompiler::new()
        .env("test", "value")
        .feature("feature_1")
        .target_directory("./out")
        .compile()
        .unwrap();
}
