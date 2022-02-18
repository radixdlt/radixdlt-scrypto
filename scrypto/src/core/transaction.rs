use crate::crypto::*;
use crate::engine::{api::*, call_engine};

/// The transaction context at runtime.
#[derive(Debug)]
pub struct Transaction {}

impl Transaction {
    /// Returns the transaction hash.
    pub fn transaction_hash() -> Hash {
        let input = GetTransactionHashInput {};
        let output: GetTransactionHashOutput = call_engine(GET_TRANSACTION_HASH, input);
        output.transaction_hash
    }

    /// Returns the current epoch number.
    pub fn current_epoch() -> u64 {
        let input = GetCurrentEpochInput {};
        let output: GetCurrentEpochOutput = call_engine(GET_CURRENT_EPOCH, input);
        output.current_epoch
    }
}
