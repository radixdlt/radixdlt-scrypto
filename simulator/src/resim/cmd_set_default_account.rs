use clap::Parser;
use scrypto::engine::types::*;

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
        let new_configs = match get_configs()? {
            Some(mut configs) => {
                configs.default_account = self.component_address;
                configs.default_private_key = hex::decode(&self.private_key).unwrap();
                configs
            }
            None => Configs {
                default_account: self.component_address,
                default_private_key: hex::decode(&self.private_key).unwrap(),
                nonce: 0,
            },
        };
        set_configs(&new_configs)?;

        writeln!(out, "Default account updated!").map_err(Error::IOError)?;
        Ok(())
    }
}
