use clap::Parser;
use scrypto::engine::types::*;

use crate::resim::*;

/// Set default account
#[derive(Parser, Debug)]
pub struct SetDefaultAccount {
    /// The account component address
    component_ref: ComponentRef,

    /// The public key for accessing the account
    public_key: EcdsaPublicKey,
}

impl SetDefaultAccount {
    pub fn run(&self) -> Result<(), Error> {
        set_configs(&Configs {
            default_account: self.component_ref,
            default_signers: vec![self.public_key],
        })?;

        println!("Default account updated!");
        Ok(())
    }
}
