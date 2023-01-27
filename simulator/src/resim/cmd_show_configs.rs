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
                Some((component, _)) =>
                    format!("{}", component.display(&Bech32Encoder::for_simulator()),),
                None => "None".to_owned(),
            }
        )
        .map_err(Error::IOError)?;
        writeln!(
            out,
            "{}: {}",
            "Account Private Key".green().bold(),
            match configs.default_account {
                Some((_, sk)) => sk,
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
