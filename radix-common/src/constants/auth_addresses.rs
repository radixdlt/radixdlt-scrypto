use crate::constants::SYSTEM_TRANSACTION_BADGE;
use crate::crypto::PublicKey;
use crate::data::scrypto::model::NonFungibleLocalId;
use crate::types::*;
use radix_rust::rust::collections::*;
use radix_rust::rust::prelude::*;

pub struct AuthAddresses;

impl AuthAddresses {
    pub fn system_role() -> NonFungibleGlobalId {
        NonFungibleGlobalId::new(SYSTEM_TRANSACTION_BADGE, NonFungibleLocalId::integer(0))
    }

    pub fn validator_role() -> NonFungibleGlobalId {
        NonFungibleGlobalId::new(SYSTEM_TRANSACTION_BADGE, NonFungibleLocalId::integer(1))
    }

    pub fn signer_set(signer_public_keys: &[PublicKey]) -> BTreeSet<NonFungibleGlobalId> {
        signer_public_keys
            .iter()
            .map(NonFungibleGlobalId::from_public_key)
            .collect()
    }
}
