use clap::Parser;
use scrypto::engine::types::*;
use std::str::FromStr;

use crate::ledger::*;
use crate::resim::*;

/// Show an entity in the ledger state
#[derive(Parser, Debug)]
pub struct Show {
    /// The address of a package, component or resource manager
    address: String,
}

impl Show {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let ledger = RadixEngineDB::new(get_data_dir()?);

        if let Ok(package_address) = PackageAddress::from_str(&self.address) {
            dump_package(package_address, &ledger, out).map_err(Error::LedgerDumpError)
        } else if let Ok(component_address) = ComponentAddress::from_str(&self.address) {
            dump_component(component_address, &ledger, out).map_err(Error::LedgerDumpError)
        } else if let Ok(resource_address) = ResourceAddress::from_str(&self.address) {
            dump_resource_manager(resource_address, &ledger, out).map_err(Error::LedgerDumpError)
        } else {
            Err(Error::InvalidId(self.address.clone()))
        }
    }
}
