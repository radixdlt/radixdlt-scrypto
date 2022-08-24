use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::{hash, EcdsaPublicKey, EcdsaSignature, Hash};

use crate::model::{
    ExecutableInstruction, ExecutableTransaction, NotarizedTransaction, SignedTransactionIntent,
    TransactionIntent,
};

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

    fn transaction_payload_size(&self) -> u32 {
        // TODO: update the estimation after transaction specs are finalized

        // Using a mocked notarized transaction of expected size
        // to include the sbor overhead in the payload size estimation
        let fake_signature = EcdsaSignature([0; EcdsaSignature::LENGTH]);
        let fake_notarized_transaction = NotarizedTransaction {
            signed_intent: SignedTransactionIntent {
                intent: self.preview_intent.intent.clone(),
                intent_signatures: self
                    .preview_intent
                    .signer_public_keys
                    .clone()
                    .into_iter()
                    .map(|pub_key| (pub_key, fake_signature.clone()))
                    .collect(),
            },
            notary_signature: fake_signature,
        };

        fake_notarized_transaction.to_bytes().len() as u32
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
}
