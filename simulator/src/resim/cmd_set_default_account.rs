use clap::Parser;
use scrypto::engine::types::*;

use crate::resim::*;

/// Set default account
#[derive(Parser, Debug)]
pub struct SetDefaultAccount {
    /// The account component address
    component_address: ComponentAddress,

    /// The public key for accessing the account
    public_key: EcdsaPublicKey,

    /// The private key for accessing the account
    private_key: EcdsaPrivateKey,
}

impl SetDefaultAccount {
    pub fn run(&self) -> Result<(), Error> {
        set_configs(&Configs {
            default_account: self.component_address,
            default_public_key: self.public_key,
            default_private_key: self.private_key.clone(),
        })?;

        println!("Default account updated!");
        Ok(())
    }
}
