use crate::resim::*;
use clap::Parser;
use radix_engine::types::*;

/// Show an entity in the ledger state
#[derive(Parser, Debug)]
pub struct Show {
    /// The address of a package, component or resource manager, if no
    /// address is provided, then we default to `show <DEFAULT_ACCOUNT_ADDRESS>`.
    pub address: Option<String>,
}

impl Show {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let SimulatorEnvironment { db, .. } = SimulatorEnvironment::new()?;

        match &self.address {
            Some(address) => {
                if let Ok(a) = SimulatorPackageAddress::from_str(address) {
                    dump_package(a.0, &db, out).map_err(Error::LedgerDumpError)
                } else if let Ok(a) = SimulatorComponentAddress::from_str(address) {
                    dump_component(a.0, &db, out).map_err(Error::LedgerDumpError)
                } else if let Ok(a) = SimulatorResourceAddress::from_str(address) {
                    dump_resource_manager(a.0, &db, out).map_err(Error::LedgerDumpError)
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
                .and_then(|x| dump_component(x, &db, out).map_err(Error::LedgerDumpError)),
        }
    }
}
