use crate::sbor::rust::prelude::*;
use crate::types::Level;
use radix_engine_common::crypto::Hash;
use radix_engine_common::types::GlobalAddress;

pub trait ClientTransactionRuntimeApi<E> {
    fn bech32_encode_address(&mut self, address: GlobalAddress) -> Result<String, E>;

    fn get_transaction_hash(&mut self) -> Result<Hash, E>;

    fn generate_ruid(&mut self) -> Result<[u8; 32], E>;

    fn emit_log(&mut self, level: Level, message: String) -> Result<(), E>;

    fn panic(&mut self, message: String) -> Result<(), E>;
}
