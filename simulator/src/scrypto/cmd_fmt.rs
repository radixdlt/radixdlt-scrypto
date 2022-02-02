use clap::Parser;
use std::env::current_dir;
use std::path::PathBuf;

use crate::scrypto::*;
use crate::utils::*;

/// Format a Scrypto package.
#[derive(Parser, Debug)]
pub struct Fmt {
    /// The package directory
    #[clap(long)]
    path: Option<PathBuf>,
}

impl Fmt {
    pub fn run(&self) -> Result<(), Error> {
        fmt_package(self.path.clone().unwrap_or(current_dir().unwrap()))
            .map(|_| ())
            .map_err(Error::CargoError)
    }
}
