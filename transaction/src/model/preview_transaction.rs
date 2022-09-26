use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::{hash, Hash, PublicKey};

use crate::model::TransactionIntent;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct PreviewFlags {
    pub unlimited_loan: bool,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
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
