use clap::Parser;
use std::env::current_dir;
use std::path::PathBuf;

use crate::scrypto::*;
use crate::utils::*;

/// Run Scrypto tests
#[derive(Parser, Debug)]
pub struct Test {
    /// The arguments to be passed to the test executable
    arguments: Vec<String>,

    /// The package directory
    #[clap(long)]
    path: Option<PathBuf>,
}

impl Test {
    pub fn run(&self) -> Result<(), Error> {
        test_package(
            self.path.clone().unwrap_or(current_dir().unwrap()),
            self.arguments.clone(),
        )
        .map(|_| ())
        .map_err(Error::TestError)
    }
}
