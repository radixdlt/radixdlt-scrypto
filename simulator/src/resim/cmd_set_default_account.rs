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
        set_configs(&Configs {
            default_account: self.address,
            default_signers: vec![self.public_key],
        })?;

        println!("Default account updated!");
        Ok(())
    }
}
