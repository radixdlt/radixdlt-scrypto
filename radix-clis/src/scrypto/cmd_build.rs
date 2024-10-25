use crate::scrypto::*;
use crate::utils::*;
use clap::Parser;
use radix_engine_interface::prelude::Level;
use scrypto_compiler::*;
use std::path::PathBuf;

/// Build a Scrypto package
#[derive(Parser, Debug, Default)]
pub struct Build {
    /// The package directory. If not specified current directory will be used.
    #[clap(long)]
    path: Option<PathBuf>,

    /// The terget directory. If not specified default target directory for project will be used.
    #[clap(long)]
    target_dir: Option<PathBuf>,

    /// Turn on tracing.
    #[clap(short, long)]
    trace: bool,

    /// Disables wasm-opt from running on the built wasm.
    #[clap(long)]
    disable_wasm_opt: bool,

    /// The max log level, such as ERROR, WARN, INFO, DEBUG and TRACE.
    /// The default is INFO.
    #[clap(long)]
    log_level: Option<Level>,

    /// Comma separated list of features to activate.
    #[clap(short = 'F', long)]
    features: Option<String>,

    /// Environment variables to define. Specify as NAME=VALUE or NAME.
    /// Scrypto compiler internally sets some compilation flags `TARGET_CFLAGS` for C libraries
    /// to configure WASM with the same features as Radix Engine.
    /// If you want to override it, then you can use this option.
    /// If you want to remove TARGET_CFLAGS, then use `--unset-env` option
    #[clap(short, long)]
    env: Option<Vec<String>>,

    /// Environment variables to unset by name.
    #[clap(long)]
    unset_env: Option<Vec<String>>,

    /// If provided for workspace compilation only these packages will be compiled.
    /// For workspace compilation all Scrypto packages must define in their manifest files
    /// Scrypto metadata section: [package.metadata.scrypto].
    #[clap(short, long)]
    package: Option<Vec<String>>,

    /// Project profile to use. The default is Release.
    #[clap(long)]
    profile: Option<Profile>,

    /// Do not activate the `default` feature.
    #[clap(long)]
    no_default_features: bool,

    /// Activate all available features.
    #[clap(long)]
    all_features: bool,

    /// Ensures the Cargo.lock file is used as-is. Equivalent to `cargo build --locked`.
    /// Alternatively, the `SCRYPTO_CARGO_LOCKED` environment variable can be used,
    /// which makes it easy to set universally in CI.
    #[clap(long)]
    locked: bool,

    /// Pass any additional option to `cargo build` call.
    #[clap(long)]
    custom_option: Option<Vec<String>>,

    /// Prints compilation steps.
    #[clap(short, long)]
    verbose: bool,
}

impl Build {
    pub fn run(&self) -> Result<(), String> {
        let mut compiler_builder = ScryptoCompiler::builder();

        if let Some(manifest_path) = &self.path {
            compiler_builder.manifest_path(manifest_path);
        }
        if let Some(target_dir) = &self.target_dir {
            compiler_builder.target_directory(target_dir);
        }
        if let Some(log_level) = self.log_level {
            compiler_builder.log_level(log_level);
        }
        if let Some(profile) = &self.profile {
            compiler_builder.profile(profile.clone());
        }
        if self.trace {
            compiler_builder.scrypto_macro_trace();
        }
        if self.disable_wasm_opt {
            compiler_builder.optimize_with_wasm_opt(None);
        }
        if self.no_default_features {
            compiler_builder.no_default_features();
        }
        if self.all_features {
            compiler_builder.all_features();
        }
        if self.locked {
            compiler_builder.locked();
        }
        if let Some(features) = &self.features {
            features.split(',').for_each(|f| {
                compiler_builder.feature(f);
            });
        }
        if let Some(packages) = &self.package {
            packages.iter().for_each(|p| {
                compiler_builder.package(p);
            });
        }
        compiler_builder.debug(self.verbose);

        if let Some(env) = &self.env {
            let env_variables_decoded: Vec<Vec<&str>> = env
                .iter()
                .map(|e|
                    // Split string on the first '=' occurence.
                    // This is to cover cases like this:
                    //   ENV_NAME=foo=bar
                    match e.split_once('=') {
                        Some((key, val)) => vec![key, val],
                        None => vec![e.as_str()],
                })
                .collect();
            for v in env_variables_decoded {
                if v.len() == 1 {
                    compiler_builder.env(v[0], EnvironmentVariableAction::Set("".into()));
                } else if v.len() == 2 {
                    compiler_builder.env(v[0], EnvironmentVariableAction::Set(v[1].into()));
                } else {
                    return Err(Error::BuildError(BuildError::EnvParsingError).into());
                }
            }
        }
        if let Some(unset_env) = &self.unset_env {
            unset_env.iter().for_each(|v| {
                compiler_builder.env(v, EnvironmentVariableAction::Unset);
            });
        }

        if let Some(options) = &self.custom_option {
            compiler_builder.custom_options(
                options
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>()
                    .as_slice(),
            );
        }

        compiler_builder
            .compile()
            .map(|_| ())
            .map_err(|e| Error::BuildError(BuildError::ScryptoCompilerError(e)).into())
    }
}
