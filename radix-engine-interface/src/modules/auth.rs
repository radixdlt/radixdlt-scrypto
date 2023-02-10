use crate::constants::SYSTEM_TOKEN;
use crate::crypto::PublicKey;
use crate::model::FromPublicKey;
use crate::model::*;
use sbor::rust::vec::Vec;

pub struct AuthAddresses;

impl AuthAddresses {
    pub fn system_role() -> NonFungibleGlobalId {
        NonFungibleGlobalId::new(SYSTEM_TOKEN, NonFungibleLocalId::integer(0))
    }

    pub fn validator_role() -> NonFungibleGlobalId {
        NonFungibleGlobalId::new(SYSTEM_TOKEN, NonFungibleLocalId::integer(1))
    }

    pub fn signer_set(signer_public_keys: &[PublicKey]) -> Vec<NonFungibleGlobalId> {
        signer_public_keys
            .iter()
            .map(NonFungibleGlobalId::from_public_key)
            .collect()
    }
}
