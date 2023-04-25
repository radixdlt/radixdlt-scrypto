use crate::*;
use radix_engine_common::data::scrypto::{scrypto_decode, ScryptoDecode};
use radix_engine_common::{crypto::*, data::scrypto::model::Reference};
use sbor::rust::prelude::*;

pub const TRANSACTION_PROCESSOR_BLUEPRINT: &str = "TransactionProcessor";

pub const TRANSACTION_PROCESSOR_RUN_IDENT: &str = "run";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct TransactionProcessorRunInput {
    pub transaction_hash: Hash,
    pub runtime_validations: Vec<RuntimeValidationRequest>,
    pub instructions: Vec<u8>,
    pub blobs: BTreeMap<Hash, Vec<u8>>,
    pub references: BTreeSet<Reference>,
}

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
pub struct RuntimeValidationRequest {
    /// The validation to perform
    pub validation: RuntimeValidation,
    /// This option is intended for preview uses cases
    /// In these cases, we still want to do the look ups to give equivalent cost unit spend, but may wish to ignore the result
    pub skip_assertion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum RuntimeValidation {
    /// To ensure we don't commit a duplicate intent hash
    IntentHashUniqueness { intent_hash: Hash },
    /// For preview - still do the look-ups to give equivalent cost unit spend, but ignore the result
    WithinEpochRange {
        start_epoch_inclusive: u64,
        end_epoch_exclusive: u64,
    },
}

impl RuntimeValidation {
    pub fn enforced(self) -> RuntimeValidationRequest {
        RuntimeValidationRequest {
            validation: self,
            skip_assertion: false,
        }
    }

    pub fn with_skipped_assertion_if(self, skip_assertion: bool) -> RuntimeValidationRequest {
        RuntimeValidationRequest {
            validation: self,
            skip_assertion,
        }
    }
}
