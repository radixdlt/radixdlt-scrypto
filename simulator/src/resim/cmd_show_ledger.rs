use clap::Parser;
use colored::*;

use crate::ledger::*;
use crate::resim::*;
use crate::utils::*;

/// Show entries in the ledger state
#[derive(Parser, Debug)]
pub struct ShowLedger {}

impl ShowLedger {
    pub fn run(&self) -> Result<(), Error> {
        let ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);

        println!("{}:", "Packages".green().bold());
        for (last, package_address) in ledger.list_packages().iter().identify_last() {
            println!("{} {}", list_item_prefix(last), package_address);
        }

        println!("{}:", "Components".green().bold());
        for (last, component_address) in ledger.list_components().iter().identify_last() {
            println!("{} {}", list_item_prefix(last), component_address);
        }

        println!("{}:", "Resource Managers".green().bold());
        for (last, resource_address) in ledger.list_resource_managers().iter().identify_last() {
            println!("{} {}", list_item_prefix(last), resource_address);
        }

        println!("{}: {}", "Nonce".green().bold(), ledger.get_nonce());
        Ok(())
    }
}
