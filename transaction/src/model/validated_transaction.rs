use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::crypto::*;

use crate::model::*;

/// Represents a validated transaction
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub struct ValidatedTransaction {
    pub transaction: Transaction,
    pub transaction_hash: Hash,
    pub instructions: Vec<ExecutableInstruction>,
    pub signer_public_keys: Vec<EcdsaPublicKey>,
}

impl ExecutableTransaction for ValidatedTransaction {
    fn transaction_hash(&self) -> Hash {
        self.transaction_hash
    }

    fn instructions(&self) -> &[ExecutableInstruction] {
        &self.instructions
    }

    fn signer_public_keys(&self) -> &[EcdsaPublicKey] {
        &self.signer_public_keys
    }
}
