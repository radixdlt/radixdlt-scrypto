use clap::Parser;
use colored::*;

use crate::resim::*;

/// Show simulator configurations
#[derive(Parser, Debug)]
pub struct ShowConfigs {}

impl ShowConfigs {

    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        if let Some(configs) = get_configs()? {
            writeln!(out,
                "{}: {}",
                "Default Account".green().bold(),
                configs.default_account
            ).map_err(Error::IOError)?;
            writeln!(out,
                "{}: {}",
                "Default Public Key".green().bold(),
                configs.default_public_key
            ).map_err(Error::IOError)?;
            writeln!(out,
                "{}: {}",
                "Default Private Key".green().bold(),
                hex::encode(configs.default_private_key)
            ).map_err(Error::IOError)?;
        } else {
            writeln!(out,"No configuration found").map_err(Error::IOError)?;
        }
        Ok(())
    }
}
