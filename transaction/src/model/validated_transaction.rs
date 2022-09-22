use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::*;
use scrypto::resource::NonFungibleAddress;

use crate::model::*;
use auth_module::AuthModule;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum ValidatedTransactionActor {
    User(Vec<PublicKey>),
    Supervisor,
}

/// Represents a validated transaction
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub struct ValidatedTransaction {
    pub transaction: NotarizedTransaction,
    pub transaction_hash: Hash,
    pub instructions: Vec<ExecutableInstruction>,
    pub actor: ValidatedTransactionActor,
}

impl ExecutableTransaction for ValidatedTransaction {
    fn transaction_hash(&self) -> Hash {
        self.transaction_hash
    }

    fn manifest_instructions_size(&self) -> u32 {
        scrypto_encode(&self.transaction.signed_intent.intent.manifest.instructions).len() as u32
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

    fn initial_proofs(&self) -> Vec<NonFungibleAddress> {
        match &self.actor {
            ValidatedTransactionActor::User(signer_public_keys) => {
                AuthModule::signer_keys_to_non_fungibles(signer_public_keys)
            }
            ValidatedTransactionActor::Supervisor => vec![AuthModule::supervisor_address()],
        }
    }

    fn blobs(&self) -> &[Vec<u8>] {
        &self.transaction.signed_intent.intent.manifest.blobs
    }
}
