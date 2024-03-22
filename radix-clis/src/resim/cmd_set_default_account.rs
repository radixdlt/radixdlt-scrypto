use clap::Parser;
use radix_common::prelude::*;

use crate::resim::*;

/// Set default account
#[derive(Parser, Debug)]
pub struct SetDefaultAccount {
    /// The account component address
    pub component_address: SimulatorComponentAddress,

    /// The private key for accessing the account
    pub private_key: String,

    /// The owner badge.
    pub owner_badge: SimulatorNonFungibleGlobalId,
}

impl SetDefaultAccount {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), String> {
        let mut configs = get_configs()?;
        let private_key = parse_private_key_from_str(&self.private_key).map_err(|e| {
            if Secp256k1PublicKey::from_str(&self.private_key).is_ok() {
                Error::GotPublicKeyExpectedPrivateKey
            } else {
                e
            }
        })?;
        configs.default_account = Some(self.component_address.0);
        configs.default_private_key = Some(private_key.to_hex());
        configs.default_owner_badge = Some(self.owner_badge.clone().0);
        set_configs(&configs)?;

        writeln!(out, "Default account updated!").map_err(Error::IOError)?;
        Ok(())
    }
}
