use crate::crypto::PublicKey;
use crate::types::*;
use sbor::rust::prelude::*;

pub struct AuthAddresses;

impl AuthAddresses {
    pub fn signer_set(signer_public_keys: &[PublicKey]) -> BTreeSet<NonFungibleGlobalId> {
        signer_public_keys
            .iter()
            .map(NonFungibleGlobalId::from_public_key)
            .collect()
    }
}
