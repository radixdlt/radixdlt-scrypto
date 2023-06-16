use crate::*;
use radix_engine_common::data::scrypto::{scrypto_decode, ScryptoDecode};
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
