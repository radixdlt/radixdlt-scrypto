use crate::sbor::rust::prelude::*;
use crate::types::Level;
use radix_engine_common::crypto::Hash;

pub trait ClientTransactionRuntimeApi<E> {
    fn get_transaction_hash(&mut self) -> Result<Hash, E>;

    fn generate_ruid(&mut self) -> Result<[u8; 32], E>;

    fn emit_log(&mut self, level: Level, message: String) -> Result<(), E>;

    fn emit_event(&mut self, event_name: String, event_data: Vec<u8>) -> Result<(), E>;

    fn panic(&mut self, message: String) -> Result<(), E>;
}
