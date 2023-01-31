use clap::Parser;
use colored::*;
use utils::ContextualDisplay;

use crate::resim::*;

/// Show simulator configurations
#[derive(Parser, Debug)]
pub struct ShowConfigs {}

impl ShowConfigs {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let configs = get_configs()?;
        writeln!(
            out,
            "{}: {}",
            "Account Address".green().bold(),
            match configs.default_account {
                Some(component) =>
                    format!("{}", component.display(&Bech32Encoder::for_simulator()),),
                None => "None".to_owned(),
            }
        )
        .map_err(Error::IOError)?;
        writeln!(
            out,
            "{}: {}",
            "Account Private Key".green().bold(),
            match configs.default_private_key {
                Some(private_key) => private_key,
                None => "None".to_owned(),
            }
        )
        .map_err(Error::IOError)?;
        writeln!(
            out,
            "{}: {}",
            "Account Owner Badge".green().bold(),
            match configs.default_owner_badge {
                Some(owner_badge) =>
                    format!("{}", owner_badge.display(&Bech32Encoder::for_simulator())),
                None => "None".to_owned(),
            }
        )
        .map_err(Error::IOError)?;
        writeln!(
            out,
            "{}: {}",
            "Next Transaction Nonce".green().bold(),
            configs.nonce
        )
        .map_err(Error::IOError)?;
        Ok(())
    }
}
