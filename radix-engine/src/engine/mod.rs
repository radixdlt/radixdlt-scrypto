use scrypto::buffer::*;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::utils::*;

use crate::execution::*;
use crate::ledger::*;

/// A in-memory version of Radix Engine.
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

    pub fn start_runtime(&mut self) -> Runtime<InMemoryLedger> {
        let tx_hash = sha256(self.nonce.to_string());
        self.nonce += 1;
        Runtime::new(tx_hash, &mut self.ledger)
    }
}

/// Decodes call return data into a Rust type.
pub fn decode_return<T: sbor::Decode>(data: Vec<u8>) -> Result<T, RuntimeError> {
    scrypto_decode(&data).map_err(RuntimeError::InvalidData)
}
