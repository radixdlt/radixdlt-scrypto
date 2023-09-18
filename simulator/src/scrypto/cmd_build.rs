use crate::scrypto::*;
use crate::utils::*;
use clap::Parser;
use radix_engine_interface::prelude::Level;
use std::env::current_dir;
use std::path::PathBuf;

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

    /// The min log level, such as ERROR, WARN, INFO, DEBUG and TRACE.
    /// The default is INFO.
    #[clap(long)]
    min_log_level: Option<Level>,
}

impl Build {
    pub fn run(&self) -> Result<(), Error> {
        build_package(
            self.path.clone().unwrap_or(current_dir().unwrap()),
            self.trace,
            false,
            self.disable_wasm_opt,
            self.min_log_level.unwrap_or(Level::default()),
        )
        .map(|_| ())
        .map_err(Error::BuildError)
    }
}
