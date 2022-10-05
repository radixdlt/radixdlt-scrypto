use clap::Parser;
use colored::*;
use radix_engine_stores::rocks_db::RadixEngineDB;
use scrypto::address::Bech32Encoder;

use crate::resim::*;
use crate::utils::*;

/// Show entries in the ledger state
#[derive(Parser, Debug)]
pub struct ShowLedger {}

impl ShowLedger {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);

        let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());

        writeln!(out, "{}:", "Packages".green().bold()).map_err(Error::IOError)?;
        for (last, package_address) in ledger.list_packages().iter().identify_last() {
            writeln!(
                out,
                "{} {}",
                list_item_prefix(last),
                package_address.displayable(&bech32_encoder)
            )
            .map_err(Error::IOError)?;
        }

        writeln!(out, "{}:", "Components".green().bold()).map_err(Error::IOError)?;
        for (last, component_address) in ledger.list_components().iter().identify_last() {
            writeln!(
                out,
                "{} {}",
                list_item_prefix(last),
                component_address.displayable(&bech32_encoder)
            )
            .map_err(Error::IOError)?;
        }

        writeln!(out, "{}:", "Resource Managers".green().bold()).map_err(Error::IOError)?;
        for (last, resource_address) in ledger.list_resource_managers().iter().identify_last() {
            writeln!(
                out,
                "{} {}",
                list_item_prefix(last),
                resource_address.displayable(&bech32_encoder)
            )
            .map_err(Error::IOError)?;
        }

        Ok(())
    }
}
