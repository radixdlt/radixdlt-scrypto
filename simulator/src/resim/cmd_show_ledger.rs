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
        for (last, package_id) in ledger.list_packages().iter().identify_last() {
            println!("{} {}", list_item_prefix(last), package_id);
        }

        println!("{}:", "Components".green().bold());
        for (last, component_id) in ledger.list_components().iter().identify_last() {
            println!("{} {}", list_item_prefix(last), component_id);
        }

        println!("{}:", "Resource Definitions".green().bold());
        for (last, resource_def_id) in ledger.list_resource_defs().iter().identify_last() {
            println!("{} {}", list_item_prefix(last), resource_def_id);
        }

        println!("{}: {}", "Nonce".green().bold(), ledger.get_nonce());
        Ok(())
    }
}
