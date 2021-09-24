use scrypto::buffer::*;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::utils::*;

use crate::execution::*;
use crate::ledger::*;

/// A Radix Engine which employs an in-memory ledger.
pub struct InMemoryRadixEngine {
    ledger: InMemoryLedger,
    nonce: u32,
}

impl InMemoryRadixEngine {
    /// Creates a new in-memory radix engine.
    pub fn new() -> Self {
        Self {
            ledger: InMemoryLedger::new(),
            nonce: 0,
        }
    }

    pub fn start_transaction(&mut self) -> Runtime<InMemoryLedger> {
        let tx_hash = sha256(self.nonce.to_string());
        self.nonce += 1;
        Runtime::new(tx_hash, &mut self.ledger)
    }
}

impl Default for InMemoryRadixEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Decodes call return data into an instance of `T`.
pub fn decode_return<T: sbor::Decode>(data: Vec<u8>) -> Result<T, RuntimeError> {
    scrypto_decode(&data).map_err(RuntimeError::InvalidData)
}
