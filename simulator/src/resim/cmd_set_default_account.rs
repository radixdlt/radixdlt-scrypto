use clap::Parser;
use scrypto::types::*;

use crate::resim::*;

/// Set default account
#[derive(Parser, Debug)]
pub struct SetDefaultAccount {
    /// The account component address
    address: Address,

    /// The public key for accessing the account
    public_key: Address,
}

impl SetDefaultAccount {
    pub fn run(&self) -> Result<(), Error> {
        let mut configs = get_configs()?;
        configs.default_account = Some((self.address, self.public_key));
        set_configs(configs)?;

        println!("Default account set!");
        Ok(())
    }
}
