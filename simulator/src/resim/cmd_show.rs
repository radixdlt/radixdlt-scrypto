use crate::{ledger::*, resim::*};
use clap::Parser;
use radix_engine::types::*;
use radix_engine_stores::rocks_db::RocksdbSubstateStore;

/// Show an entity in the ledger state
#[derive(Parser, Debug)]
pub struct Show {
    /// The address of a package, component or resource manager
    pub address: String,
}

impl Show {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
        let mut substate_db = RocksdbSubstateStore::standard(get_data_dir()?);
        bootstrap(&mut substate_db, &scrypto_interpreter);

        if let Ok(a) = SimulatorPackageAddress::from_str(&self.address) {
            dump_package(a.0, &substate_db, out).map_err(Error::LedgerDumpError)
        } else if let Ok(a) = SimulatorComponentAddress::from_str(&self.address) {
            dump_component(a.0, &substate_db, out).map_err(Error::LedgerDumpError)
        } else if let Ok(a) = SimulatorResourceAddress::from_str(&self.address) {
            dump_resource_manager(a.0, &substate_db, out).map_err(Error::LedgerDumpError)
        } else {
            Err(Error::InvalidId(self.address.clone()))
        }
    }
}
