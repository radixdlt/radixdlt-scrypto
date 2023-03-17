use radix_engine_common::crypto::Hash;

pub trait ClientTransactionRuntimeApi<E> {
    fn get_transaction_hash(&mut self) -> Result<Hash, E>;

    fn generate_uuid(&mut self) -> Result<u128, E>;
}
