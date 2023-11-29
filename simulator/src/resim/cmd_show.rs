use crate::resim::*;
use clap::Parser;
use radix_engine::types::*;
use radix_engine_stores::rocks_db::RocksdbSubstateStore;

/// Show an entity in the ledger state
#[derive(Parser, Debug)]
pub struct Show {
    /// The address of a package, component or resource manager, if no
    /// address is provided, then we default to `show <DEFAULT_ACCOUNT_ADDRESS>`.
    pub address: Option<String>,
}

impl Show {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
        let native_vm = DefaultNativeVm::new();
        let vm = Vm::new(&scrypto_vm, native_vm);
        let mut substate_db = RocksdbSubstateStore::standard(get_data_dir()?);
        Bootstrapper::new(NetworkDefinition::simulator(), &mut substate_db, vm, false)
            .bootstrap_test_default();

        match &self.address {
            Some(address) => {
                if let Ok(a) = SimulatorPackageAddress::from_str(address) {
                    dump_package(a.0, &substate_db, out).map_err(Error::LedgerDumpError)
                } else if let Ok(a) = SimulatorComponentAddress::from_str(address) {
                    dump_component(a.0, &substate_db, out).map_err(Error::LedgerDumpError)
                } else if let Ok(a) = SimulatorResourceAddress::from_str(address) {
                    dump_resource_manager(a.0, &substate_db, out).map_err(Error::LedgerDumpError)
                } else {
                    Err(Error::InvalidId(address.clone()))
                }
            }
            None => get_configs()
                .and_then(|c| {
                    c.default_account.ok_or(Error::LedgerDumpError(
                        EntityDumpError::NoAddressProvidedAndNotDefaultAccountSet,
                    ))
                })
                .and_then(|x| dump_component(x, &substate_db, out).map_err(Error::LedgerDumpError)),
        }
    }
}

#[cfg(test)]
#[test]
fn test_no_value() {
    let mut out = std::io::stdout();
    let new_account = NewAccount {
        network: None,
        manifest: None,
        trace: false,
    };
    assert!(new_account.run(&mut out).is_ok());
    let cmd = Show { address: None };
    assert!(cmd.run(&mut out).is_ok());
}
