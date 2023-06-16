use crate::*;
use radix_engine_common::crypto::*;
use radix_engine_common::data::scrypto::{scrypto_decode, ScryptoDecode};
use radix_engine_common::prelude::Epoch;
use sbor::rust::prelude::*;

pub const TRANSACTION_PROCESSOR_BLUEPRINT: &str = "TransactionProcessor";

pub const TRANSACTION_PROCESSOR_RUN_IDENT: &str = "run";

// TransactionProcessorInput in the engine

pub type TransactionProcessorRunOutput = Vec<InstructionOutput>;

#[derive(Debug, Clone, Sbor, Eq, PartialEq)]
pub enum InstructionOutput {
    CallReturn(Vec<u8>),
    None,
}

impl InstructionOutput {
    pub fn expect_return_value<V: ScryptoDecode + Eq + Debug>(&self, expected: &V) {
        match self {
            Self::CallReturn(buf) => {
                let actual: V = scrypto_decode(buf).expect("Value does not decode to type");
                if !expected.eq(&actual) {
                    panic!("Expected: {:?} but was: {:?}", expected, actual)
                }
            }
            Self::None => {
                panic!("Expected: {:?} but was None", expected);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum RuntimeValidation {
    /// To ensure we don't commit a duplicate intent hash
    CheckIntentHash {
        intent_hash: Hash,
        expiry_epoch: Epoch,
    },
    /// For preview - still do the look-ups to give equivalent cost unit spend, but ignore the result
    CheckEpochRange {
        start_epoch_inclusive: Epoch,
        end_epoch_exclusive: Epoch,
    },
}
