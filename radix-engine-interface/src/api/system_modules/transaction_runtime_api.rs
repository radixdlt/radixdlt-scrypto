use radix_engine_common::crypto::Hash;
use sbor::prelude::*;

pub trait ClientTransactionRuntimeApi<E> {
    fn get_transaction_hash(&mut self) -> Result<Hash, E>;

    fn generate_uuid(&mut self) -> Result<u128, E>;

    fn panic(&mut self, message: String) -> Result<(), E>;
}
