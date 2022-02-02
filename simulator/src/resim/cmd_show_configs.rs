use clap::Parser;
use colored::*;

use crate::resim::*;

/// Show simulator configurations
#[derive(Parser, Debug)]
pub struct ShowConfigs {}

impl ShowConfigs {
    pub fn run(&self) -> Result<(), Error> {
        let configs = get_configs()?;

        println!(
            "{}: {:?}",
            "Default Account".green().bold(),
            configs.default_account
        );
        println!(
            "{}: {}",
            "Current Epoch".green().bold(),
            configs.current_epoch
        );
        println!("{}: {}", "Nonce".green().bold(), configs.nonce);

        Ok(())
    }
}
