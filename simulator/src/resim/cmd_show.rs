use clap::Parser;
use scrypto::engine::types::*;
use std::str::FromStr;

use crate::ledger::*;
use crate::resim::*;

/// Show an entity in the ledger state
#[derive(Parser, Debug)]
pub struct Show {
    /// A package, component or resource definition ref
    reference: String,
}

impl Show {
    pub fn run(&self) -> Result<(), Error> {
        let ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);

        if let Ok(package_ref) = PackageRef::from_str(&self.reference) {
            dump_package(package_ref, &ledger).map_err(Error::LedgerDumpError)
        } else if let Ok(component_ref) = ComponentRef::from_str(&self.reference) {
            dump_component(component_ref, &ledger).map_err(Error::LedgerDumpError)
        } else if let Ok(resource_def_ref) = ResourceDefRef::from_str(&self.reference) {
            dump_resource_def(resource_def_ref, &ledger).map_err(Error::LedgerDumpError)
        } else {
            Err(Error::InvalidReference(self.reference.clone()))
        }
    }
}
