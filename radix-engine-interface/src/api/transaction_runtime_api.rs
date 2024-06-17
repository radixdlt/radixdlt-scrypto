use crate::internal_prelude::*;
use crate::types::Level;
use radix_common::crypto::Hash;
use radix_common::types::GlobalAddress;

pub trait SystemTransactionRuntimeApi<E> {
    /// Encode an address into Bech32. The HRP part is dependent on the network which is running.
    fn bech32_encode_address(&mut self, address: GlobalAddress) -> Result<String, E>;

    /// Retrieve the hash of the current transaction which is running.
    fn get_transaction_hash(&mut self) -> Result<Hash, E>;

    /// Generate a unique id
    fn generate_ruid(&mut self) -> Result<[u8; 32], E>;

    /// Emit a log message which will be available in the transaction receipt
    fn emit_log(&mut self, level: Level, message: String) -> Result<(), E>;

    /// End the transaction immediately with a given message to be included in the transaction receipt
    fn panic(&mut self, message: String) -> Result<(), E>;
}
