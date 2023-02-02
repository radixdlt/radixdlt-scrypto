use clap::Parser;
use radix_engine::types::*;

use crate::resim::*;

/// Set default account
#[derive(Parser, Debug)]
pub struct SetDefaultAccount {
    /// The account component address
    component_address: SimulatorComponentAddress,

    /// The private key for accessing the account
    private_key: String,

    /// The owner badge.
    owner_badge: SimulatorNonFungibleGlobalId,
}

impl SetDefaultAccount {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let mut configs = get_configs()?;
        configs.default_account = Some(self.component_address.0);
        configs.default_private_key = Some(self.private_key.clone());
        configs.default_owner_badge = Some(self.owner_badge.clone().0);
        set_configs(&configs)?;

        writeln!(out, "Default account updated!").map_err(Error::IOError)?;
        Ok(())
    }
}
