use crate::prelude::*;

define_wrapped_hash!(NotarizedTransactionHash);

pub trait HasNotarizedTransactionHash {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash;
}
