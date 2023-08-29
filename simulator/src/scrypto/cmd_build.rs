use clap::Parser;
use std::env::current_dir;
use std::path::PathBuf;

use crate::scrypto::*;
use crate::utils::*;

/// Build a Scrypto package
#[derive(Parser, Debug)]
pub struct Build {
    /// The package directory
    #[clap(long)]
    path: Option<PathBuf>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,

    /// When passed, this argument disables wasm-opt from running on the built wasm.
    #[clap(long)]
    disable_wasm_opt: bool,
}

impl Build {
    pub fn run(&self) -> Result<(), Error> {
        build_package(
            self.path.clone().unwrap_or(current_dir().unwrap()),
            self.trace,
            false,
            self.disable_wasm_opt,
        )
        .map(|_| ())
        .map_err(Error::BuildError)
    }
}
