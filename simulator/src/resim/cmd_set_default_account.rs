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

#[cfg(test)]
#[test]
fn test_validation() {
    let mut out = std::io::stdout();
    let private_key = Secp256k1PrivateKey::from_hex(
        "6847c11e2d602548dbf38789e0a1f4543c1e7719e4f591d4aa6e5684f5c13d9c",
    )
    .unwrap();
    let public_key = private_key.public_key().to_string();

    let make_cmd = |key_string: String| {
        return SetDefaultAccount {
            component_address: SimulatorComponentAddress::from_str(
                "account_sim1c9yeaya6pehau0fn7vgavuggeev64gahsh05dauae2uu25njk224xz",
            )
            .unwrap(),
            private_key: key_string,
            owner_badge: SimulatorNonFungibleGlobalId::from_str(
                "resource_sim1ngvrads4uj3rgq2v9s78fzhvry05dw95wzf3p9r8skhqusf44dlvmr:#1#",
            )
            .unwrap(),
        };
    };

    assert!(make_cmd(private_key.to_hex()).run(&mut out).is_ok());
    assert!(make_cmd(public_key.to_string()).run(&mut out).is_err());
}
