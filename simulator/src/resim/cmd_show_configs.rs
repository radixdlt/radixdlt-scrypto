use clap::Parser;
use colored::*;

use crate::resim::*;

/// Show simulator configurations
#[derive(Parser, Debug)]
pub struct ShowConfigs {}

impl ShowConfigs {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let configs = get_configs()?;
        writeln!(
            out,
            "{}: {:?}",
            "Default Account".green().bold(),
            configs.default_account
        )
        .map_err(Error::IOError)?;
        writeln!(
            out,
            "{}: {:?}",
            "Current Nonce".green().bold(),
            configs.nonce
        )
        .map_err(Error::IOError)?;
        Ok(())
    }
}
