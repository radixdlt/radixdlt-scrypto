use crate::internal_prelude::*;
use radix_common::data::scrypto::{scrypto_decode, ScryptoDecode};
use sbor::rust::prelude::*;

pub const TRANSACTION_PROCESSOR_BLUEPRINT: &str = "TransactionProcessor";

pub const TRANSACTION_PROCESSOR_RUN_IDENT: &str = "run";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct TransactionProcessorRunInput {
    pub manifest_encoded_instructions: Vec<u8>,
    pub global_address_reservations: Vec<GlobalAddressReservation>,
    pub references: Vec<Reference>, // Required so that the kernel passes the references to the processor frame
    pub blobs: IndexMap<Hash, Vec<u8>>,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct TransactionProcessorRunManifestInput {
    pub manifest_encoded_instructions: Vec<u8>,
    pub global_address_reservations: Vec<ManifestAddressReservation>,
    pub references: Vec<GlobalAddress>, // Required so that the kernel passes the references to the processor frame
    pub blobs: IndexMap<Hash, Vec<u8>>,
}

// This needs to match the above, but is easily encodable to avoid cloning from the transaction payload to encode
#[derive(Debug, Eq, PartialEq, ScryptoEncode)]
pub struct TransactionProcessorRunInputEfficientEncodable<'a> {
    pub manifest_encoded_instructions: &'a [u8],
    pub global_address_reservations: &'a [GlobalAddressReservation],
    pub references: &'a IndexSet<Reference>,
    pub blobs: &'a IndexMap<Hash, Vec<u8>>,
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
