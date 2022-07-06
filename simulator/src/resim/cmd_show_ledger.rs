use clap::Parser;
use colored::*;
use scrypto::address::Bech32Addressable;

use crate::ledger::*;
use crate::resim::*;
use crate::utils::*;

/// Show entries in the ledger state
#[derive(Parser, Debug)]
pub struct ShowLedger {}

impl ShowLedger {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);

        writeln!(out, "{}:", "Packages".green().bold()).map_err(Error::IOError)?;
        for (last, package_address) in ledger.list_packages().iter().identify_last() {
            writeln!(
                out,
                "{} {}",
                list_item_prefix(last),
                package_address
                    .to_bech32_string(&Network::LocalSimulator)
                    .unwrap()
            )
            .map_err(Error::IOError)?;
        }

        writeln!(out, "{}:", "Components".green().bold()).map_err(Error::IOError)?;
        for (last, component_address) in ledger.list_components().iter().identify_last() {
            writeln!(
                out,
                "{} {}",
                list_item_prefix(last),
                component_address
                    .to_bech32_string(&Network::LocalSimulator)
                    .unwrap()
            )
            .map_err(Error::IOError)?;
        }

        writeln!(out, "{}:", "Resource Managers".green().bold()).map_err(Error::IOError)?;
        for (last, resource_address) in ledger.list_resource_managers().iter().identify_last() {
            writeln!(
                out,
                "{} {}",
                list_item_prefix(last),
                resource_address
                    .to_bech32_string(&Network::LocalSimulator)
                    .unwrap()
            )
            .map_err(Error::IOError)?;
        }

        Ok(())
    }
}
