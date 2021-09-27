use scrypto::rust::string::ToString;
use scrypto::utils::*;

use crate::engine::*;
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

    pub fn start_transaction(&mut self) -> Track<InMemoryLedger> {
        let tx_hash = sha256(self.nonce.to_string());
        self.nonce += 1;
        Track::new(tx_hash, &mut self.ledger)
    }
}

impl Default for InMemoryRadixEngine {
    fn default() -> Self {
        Self::new()
    }
}
