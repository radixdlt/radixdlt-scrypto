use crate::crypto::*;
use crate::engine::{api::*, types::*, utils::*};

// TODO: remove

/// The transaction context at runtime.
#[derive(Debug)]
pub struct Transaction {}

impl Transaction {
    /// Returns the transaction hash.
    pub fn transaction_hash() -> Hash {
         Runtime::transaction_hash();
    }

    /// Returns the current epoch number.
    pub fn current_epoch() -> u64 {
         Runtime::current_epoch()
    }
}
