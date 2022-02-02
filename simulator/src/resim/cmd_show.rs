use clap::Parser;
use scrypto::types::*;

use crate::ledger::*;
use crate::resim::*;

/// Show an entity in the ledger state
#[derive(Parser, Debug)]
pub struct Show {
    /// The address of a package, component or resource definition
    address: Address,
}

impl Show {
    pub fn run(&self) -> Result<(), Error> {
        let ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
        match self.address {
            Address::Package(_) => {
                dump_package(self.address, &ledger).map_err(Error::LedgerDumpError)
            }
            Address::Component(_) => {
                dump_component(self.address, &ledger).map_err(Error::LedgerDumpError)
            }
            Address::ResourceDef(_) => {
                dump_resource_def(self.address, &ledger).map_err(Error::LedgerDumpError)
            }
            Address::PublicKey(_) => Ok(println!("Public Key: {}", self.address)),
        }
    }
}
