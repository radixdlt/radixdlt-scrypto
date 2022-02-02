use clap::Parser;
use colored::*;

use crate::resim::*;

/// Show simulator configurations
#[derive(Parser, Debug)]
pub struct ShowConfigs {}

impl ShowConfigs {
    pub fn run(&self) -> Result<(), Error> {
        if let Some(configs) = get_configs()? {
            println!(
                "{}: {:?}",
                "Default Account".green().bold(),
                configs.default_account
            );
            println!(
                "{}: {:?}",
                "Default Signers".green().bold(),
                configs.default_signers
            );
        } else {
            println!("No configuration found");
        }
        Ok(())
    }
}
