use crate::crypto::PublicKey;
use crate::types::*;
use sbor::rust::prelude::*;

pub struct AuthAddresses;

impl AuthAddresses {
    pub fn signer_set<'a>(
        signer_public_keys: impl IntoIterator<Item = &'a PublicKey>,
    ) -> BTreeSet<NonFungibleGlobalId> {
        signer_public_keys
            .into_iter()
            .map(NonFungibleGlobalId::from_public_key)
            .collect()
    }
}
