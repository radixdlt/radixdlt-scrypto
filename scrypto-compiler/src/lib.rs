
pub struct ScryptoCompiler {
    profile: Profile,
    set_environment_variables: Vec<(String, String)>,
    unset_environment_variables: Vec<String>,
    features: Vec<String>,
    package: Option<String>,
    target_directory: Option<String>,
}

impl ScryptoCompiler {
    pub fn new() -> ScryptoCompilerBuilder {
        ScryptoCompilerBuilder::default()
    }

    pub fn compile(&mut self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Default, Clone)]
enum Profile {
    Release,
    #[default]
    Debug,
}

#[derive(Default)]
pub struct ScryptoCompilerBuilder {
    profile: Profile,
    set_environment_variables: Vec<(String, String)>,
    unset_environment_variables: Vec<String>,
    features: Vec<String>,
    package: Option<String>,
    target_directory: Option<String>,
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

    pub fn compile(&mut self) -> Result<(), String> {
        let mut compiler = ScryptoCompiler {
            profile: self.profile.clone(),
            set_environment_variables: self.set_environment_variables.to_owned(),
            unset_environment_variables: self.unset_environment_variables.to_owned(),
            features: self.features.to_owned(),
            package: self.package.clone(),
            target_directory: self.target_directory.clone(),
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
