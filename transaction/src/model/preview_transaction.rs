use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::{hash, Hash, PublicKey};
use scrypto::scrypto;

use crate::model::TransactionIntent;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct PreviewFlags {
    pub unlimited_loan: bool,
    pub assume_all_signature_proofs: bool,
    pub permit_duplicate_intent_hash: bool,
    pub permit_invalid_header_epoch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PreviewIntent {
    pub intent: TransactionIntent,
    pub signer_public_keys: Vec<PublicKey>,
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
