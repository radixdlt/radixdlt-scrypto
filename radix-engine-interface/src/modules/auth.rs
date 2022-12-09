use crate::constants::SYSTEM_TOKEN;
use crate::crypto::PublicKey;
use crate::model::FromPublicKey;
use crate::model::*;
use sbor::rust::vec::Vec;

pub struct AuthAddresses;

impl AuthAddresses {
    pub fn system_role() -> NonFungibleAddress {
        NonFungibleAddress::new(SYSTEM_TOKEN, NonFungibleId::U32(0))
    }

    pub fn validator_role() -> NonFungibleAddress {
        NonFungibleAddress::new(SYSTEM_TOKEN, NonFungibleId::U32(1))
    }

    pub fn signer_set(signer_public_keys: &[PublicKey]) -> Vec<NonFungibleAddress> {
        signer_public_keys
            .iter()
            .map(NonFungibleAddress::from_public_key)
            .collect()
    }
}
