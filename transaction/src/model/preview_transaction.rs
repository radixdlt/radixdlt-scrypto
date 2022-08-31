use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::{hash, EcdsaPublicKey, Hash};

use crate::model::{ExecutableInstruction, ExecutableTransaction, TransactionIntent};

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct PreviewFlags {
    pub unlimited_loan: bool,
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

    fn manifest_size(&self) -> u32 {
        scrypto_encode(&self.preview_intent.intent.manifest).len() as u32
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

    fn tip_percentage(&self) -> u32 {
        self.preview_intent.intent.header.tip_percentage
    }

    fn blobs(&self) -> &[Vec<u8>] {
        &self.preview_intent.intent.manifest.blobs
    }
}
