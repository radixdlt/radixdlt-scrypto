#![allow(unused_must_use)]
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
            );
            writeln!(out,
                "{}: {}",
                "Default Public Key".green().bold(),
                configs.default_public_key
            );
            writeln!(out,
                "{}: {}",
                "Default Private Key".green().bold(),
                hex::encode(configs.default_private_key)
            );
        } else {
            writeln!(out,"No configuration found");
        }
        Ok(())
    }
}
