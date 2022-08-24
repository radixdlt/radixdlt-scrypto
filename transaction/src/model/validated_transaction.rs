use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::crypto::*;

use crate::model::*;

/// Represents a validated transaction
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub struct ValidatedTransaction {
    pub transaction: NotarizedTransaction,
    pub transaction_hash: Hash,
    pub instructions: Vec<ExecutableInstruction>,
    pub signer_public_keys: Vec<EcdsaPublicKey>,
}

impl ExecutableTransaction for ValidatedTransaction {
    fn transaction_hash(&self) -> Hash {
        self.transaction_hash
    }

    fn transaction_payload_size(&self) -> u32 {
        self.transaction.to_bytes().len() as u32
    }

    fn cost_unit_limit(&self) -> u32 {
        self.transaction.signed_intent.intent.header.cost_unit_limit
    }

    fn tip_percentage(&self) -> u32 {
        self.transaction.signed_intent.intent.header.tip_percentage
    }

    fn instructions(&self) -> &[ExecutableInstruction] {
        &self.instructions
    }

    fn signer_public_keys(&self) -> &[EcdsaPublicKey] {
        &self.signer_public_keys
    }
}
