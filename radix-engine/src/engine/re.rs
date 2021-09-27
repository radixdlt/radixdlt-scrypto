use scrypto::rust::string::ToString;
use scrypto::utils::*;

use crate::engine::*;
use crate::ledger::*;

/// A Radix Engine which is based on an in-memory ledger.
pub struct InMemoryRadixEngine {
    ledger: InMemoryLedger,
    nonce: u32,
}

impl InMemoryRadixEngine {
    /// Creates a radix engine instance.
    pub fn new() -> Self {
        Self {
            ledger: InMemoryLedger::new(),
            nonce: 0,
        }
    }

    /// Starts a new transaction.
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
