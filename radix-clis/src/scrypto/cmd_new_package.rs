use clap::Parser;
use std::path::PathBuf;

/// Create a Scrypto package
#[derive(Parser, Debug)]
pub struct NewPackage {
    /// The package name
    package_name: String,

    /// The package directory
    #[clap(long)]
    path: Option<PathBuf>,

    /// Use local Scrypto as dependency
    #[clap(short, long)]
    local: bool,
}

impl NewPackage {
    pub fn run(&self) -> Result<(), String> {
        radix_clis_common::package::new_package(&self.package_name, self.path.clone(), self.local)
            .map_err(|error| format!("{error:#?}"))
    }
}
