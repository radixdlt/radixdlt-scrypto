use crate::prelude::*;

define_wrapped_hash!(IntentHash);

pub trait HasIntentHash {
    fn intent_hash(&self) -> IntentHash;
}
