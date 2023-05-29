use crate::prelude::*;

define_wrapped_hash!(SystemTransactionHash);

pub trait HasSystemTransactionHash {
    fn system_transaction_hash(&self) -> SystemTransactionHash;
}
