use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::core::Network;
use scrypto::crypto::{hash, EcdsaPublicKey, EcdsaSignature, Hash};

use crate::model::{ExecutableInstruction, ExecutableTransaction, TransactionIntent};

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct PreviewFlags {
    // Empty for now
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct PreviewIntent {
    pub intent: TransactionIntent,
    pub signer_public_keys: Vec<EcdsaPublicKey>,
    pub flags: PreviewFlags,
}

impl PreviewIntent {
    pub fn hash(&self) -> Hash {
        hash(self.to_bytes())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        scrypto_encode(self)
    }
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct ValidatedPreviewTransaction {
    pub preview_intent: PreviewIntent,
    pub transaction_hash: Hash,
    pub instructions: Vec<ExecutableInstruction>,
}

impl ValidatedPreviewTransaction {
    pub fn signer_public_keys(&self) -> &[EcdsaPublicKey] {
        &self.preview_intent.signer_public_keys
    }
}

impl ExecutableTransaction for ValidatedPreviewTransaction {
    fn transaction_hash(&self) -> Hash {
        self.transaction_hash
    }

    fn transaction_network(&self) -> Network {
        self.preview_intent.intent.header.network.clone()
    }

    fn transaction_payload_size(&self) -> u32 {
        // TODO: update the estimation after transaction specs are finalized
        let intent_size = self.preview_intent.intent.to_bytes().len();
        let num_expected_signatures =
            self.signer_public_keys().len() /* Intent signatures */  + 1 /* Notary signature */;
        let signatures_size = num_expected_signatures * EcdsaSignature::LENGTH;
        let estimated_notarized_transaction_size = intent_size + signatures_size;
        return estimated_notarized_transaction_size as u32;
    }

    fn instructions(&self) -> &[ExecutableInstruction] {
        &self.instructions
    }

    fn signer_public_keys(&self) -> &[EcdsaPublicKey] {
        &self.signer_public_keys()
    }

    fn cost_unit_limit(&self) -> u32 {
        self.preview_intent.intent.header.cost_unit_limit
    }
}
