use radix_engine_lib::resource::{NonFungibleAddress, NonFungibleId};
use scrypto::constants::{ECDSA_SECP256K1_TOKEN, EDDSA_ED25519_TOKEN, SYSTEM_TOKEN};
use scrypto::crypto::PublicKey;
use utils::crypto::hash;

pub struct AuthModule;

// TODO: Integrate with AuthModule in radix-engine
impl AuthModule {
    pub fn system_role_nf_address() -> NonFungibleAddress {
        NonFungibleAddress::new(SYSTEM_TOKEN, NonFungibleId::from_u32(1))
    }

    pub fn validator_role_nf_address() -> NonFungibleAddress {
        NonFungibleAddress::new(SYSTEM_TOKEN, NonFungibleId::from_u32(0))
    }

    pub fn nf_address_from_public_key<P: Into<PublicKey> + Clone>(public_key: &P) -> NonFungibleAddress {
        let public_key: PublicKey = public_key.clone().into();
        match public_key {
            PublicKey::EcdsaSecp256k1(public_key) => NonFungibleAddress::new(
                ECDSA_SECP256K1_TOKEN,
                NonFungibleId::from_bytes(hash(public_key.to_vec()).lower_26_bytes().into()),
            ),
            PublicKey::EddsaEd25519(public_key) => NonFungibleAddress::new(
                EDDSA_ED25519_TOKEN,
                NonFungibleId::from_bytes(hash(public_key.to_vec()).lower_26_bytes().into()),
            ),
        }
    }

    pub fn pk_non_fungibles(signer_public_keys: &[PublicKey]) -> Vec<NonFungibleAddress> {
        signer_public_keys
            .iter()
            .map(Self::nf_address_from_public_key)
            .collect()
    }
}
