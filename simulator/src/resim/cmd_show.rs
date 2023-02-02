use clap::Parser;
use radix_engine::types::*;
use radix_engine_stores::rocks_db::RadixEngineDB;

use crate::ledger::*;
use crate::resim::*;

/// Show an entity in the ledger state
#[derive(Parser, Debug)]
pub struct Show {
    /// The address of a package, component or resource manager
    pub address: String,
}

impl Show {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
        let substate_store = RadixEngineDB::with_bootstrap(get_data_dir()?, &scrypto_interpreter);
        let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());

        if let Ok(package_address) =
            bech32_decoder.validate_and_decode_package_address(&self.address)
        {
            dump_package(package_address, &substate_store, out).map_err(Error::LedgerDumpError)
        } else if let Ok(component_address) =
            bech32_decoder.validate_and_decode_component_address(&self.address)
        {
            dump_component(component_address, &substate_store, out).map_err(Error::LedgerDumpError)
        } else if let Ok(resource_address) =
            bech32_decoder.validate_and_decode_resource_address(&self.address)
        {
            dump_resource_manager(resource_address, &substate_store, out)
                .map_err(Error::LedgerDumpError)
        } else {
            Err(Error::InvalidId(self.address.clone()))
        }
    }
}
