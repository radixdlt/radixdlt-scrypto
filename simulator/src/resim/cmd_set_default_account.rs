use clap::Parser;
use radix_engine::types::*;

use crate::resim::*;

/// Set default account
#[derive(Parser, Debug)]
pub struct SetDefaultAccount {
    /// The account component address
    component_address: ComponentAddress,

    /// The private key for accessing the account
    private_key: String,
}

impl SetDefaultAccount {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let mut configs = get_configs()?;
        configs.default_account = Some((self.component_address, self.private_key.clone()));
        set_configs(&configs)?;

        writeln!(out, "Default account updated!").map_err(Error::IOError)?;
        Ok(())
    }
}
