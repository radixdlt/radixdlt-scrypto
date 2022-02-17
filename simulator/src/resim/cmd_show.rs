use clap::Parser;
use scrypto::engine::types::*;
use std::str::FromStr;

use crate::ledger::*;
use crate::resim::*;

/// Show an entity in the ledger state
#[derive(Parser, Debug)]
pub struct Show {
    /// The ID of a package, component or resource definition
    id: String,
}

impl Show {
    pub fn run(&self) -> Result<(), Error> {
        let ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);

        if let Ok(package_id) = PackageId::from_str(&self.id) {
            dump_package(package_id, &ledger).map_err(Error::LedgerDumpError)
        } else if let Ok(component_id) = ComponentId::from_str(&self.id) {
            dump_component(component_id, &ledger).map_err(Error::LedgerDumpError)
        } else if let Ok(resource_def_id) = ResourceDefId::from_str(&self.id) {
            dump_resource_def(resource_def_id, &ledger).map_err(Error::LedgerDumpError)
        } else {
            Err(Error::InvalidId(self.id.clone()))
        }
    }
}
