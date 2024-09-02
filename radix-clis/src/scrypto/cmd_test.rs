use clap::Parser;
use scrypto_compiler::is_scrypto_cargo_locked_env_var_active;
use std::env::current_dir;
use std::path::PathBuf;

use crate::scrypto::*;
use crate::utils::*;

/// Run Scrypto tests
#[derive(Parser, Debug)]
pub struct Test {
    /// The arguments to be passed to the test executable
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

impl Test {
    pub fn run(&self) -> Result<(), String> {
        test_package(
            self.path.clone().unwrap_or(current_dir().unwrap()),
            self.arguments.clone(),
            false,
            is_scrypto_cargo_locked_env_var_active() || self.locked,
        )
        .map(|_| ())
        .map_err(|err| Error::TestError(err).into())
    }
}
