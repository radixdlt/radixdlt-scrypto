use crate::prelude::*;

define_wrapped_hash!(SignedIntentHash);

pub trait HasSignedIntentHash {
    fn signed_intent_hash(&self) -> SignedIntentHash;
}
