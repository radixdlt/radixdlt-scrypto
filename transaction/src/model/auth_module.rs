use scrypto::constants::{ECDSA_TOKEN, ED25519_TOKEN, SYSTEM_TOKEN};
use scrypto::crypto::PublicKey;
use scrypto::resource::{NonFungibleAddress, NonFungibleId};

pub struct AuthModule;

// TODO: Integrate with AuthModule in radix-engine
impl AuthModule {
    pub fn supervisor_address() -> NonFungibleAddress {
        NonFungibleAddress::new(SYSTEM_TOKEN, NonFungibleId::from_u32(0))
    }

    pub fn signer_keys_to_non_fungibles(
        signer_public_keys: &[PublicKey],
    ) -> Vec<NonFungibleAddress> {
        signer_public_keys
            .iter()
            .map(|k| match k {
                PublicKey::EddsaEd25519(pk) => {
                    NonFungibleAddress::new(ED25519_TOKEN, NonFungibleId::from_bytes(pk.to_vec()))
                }
                PublicKey::EcdsaSecp256k1(pk) => {
                    NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::from_bytes(pk.to_vec()))
                }
            })
            .collect()
    }
}
